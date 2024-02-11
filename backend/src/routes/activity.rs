use rocket::{
	delete,
	fairing::AdHoc,
	get,
	patch,
	post,
	routes,
	serde::json::Json,
};
use rocket_db_pools::Connection;

use crate::{
	db::{
		activity::{
			Activity,
			CreateActivity,
			UpdateActivity,
		},
		PaginationResult,
	},
	Error,
	Result,
	User,
	DB,
};

#[post("/", data = "<create_activity>")]
async fn create(
	user: User,
	mut db: Connection<DB>,
	create_activity: Json<CreateActivity>,
) -> Result<Json<Activity>> {
	if user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	let activity = Activity::create(&mut db, &create_activity).await;
	if let Ok(activity) = activity {
		Ok(Json(activity))
	} else {
		Err(Error::BadRequest("Activity already exists".to_string()))
	}
}

#[patch("/<id>", data = "<update_activity>")]
async fn update(
	_user: User,
	mut db: Connection<DB>,
	update_activity: Json<UpdateActivity>,
	id: i32,
) -> Result<Json<Activity>> {
	Ok(Json(Activity::update(&mut db, id, &update_activity).await?))
}

#[get("/<id>")]
async fn get(_user: User, mut db: Connection<DB>, id: i32) -> Result<Json<Activity>> {
	Ok(Json(Activity::read(&mut db, id).await?))
}

#[get("/page/<page_size>/<page>")]
async fn get_page(
	_user: User,
	mut db: Connection<DB>,
	page_size: i64,
	page: i64,
) -> Result<Json<PaginationResult<Activity>>> {
	Ok(Json(Activity::paginate(&mut db, page, page_size).await?))
}

#[get("/page/<page_size>/last", rank = 2)]
async fn get_last_page(
	_user: User,
	mut db: Connection<DB>,
	page_size: i64,
) -> Result<Json<PaginationResult<Activity>>> {
	let last_page = Activity::last_page(&mut db, page_size).await?;
	Ok(Json(
		Activity::paginate(&mut db, last_page, page_size).await?,
	))
}

#[delete("/<id>")]
async fn delete(user: User, mut db: Connection<DB>, id: i32) -> Result<Json<usize>> {
	if user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	Ok(Json(Activity::delete(&mut db, id).await?))
}

pub fn mount() -> AdHoc {
	AdHoc::on_ignite("Activity", |rocket| async {
		rocket.mount(
			"/activity",
			routes![create, get, update, delete, get_page, get_last_page,],
		)
	})
}

#[cfg(test)]
mod test {
	use pretty_assertions::{
		assert_eq,
		assert_ne,
	};
	use rocket::{
		http::Status,
		local::blocking::Client,
		serde::json::to_string,
	};

	use crate::{
		db::{
			activity::{
				Activity,
				UpdateActivity,
			},
			PaginationResult,
		},
		error::ErrorJson,
		rocket,
		test::{
			generate_activity,
			methods::{
				delete,
				get,
				patch,
				post,
			},
			token::{
				get_token_admin,
				get_token_user,
			},
		},
	};

	#[tracing_test::traced_test]
	#[test]
	fn activity_single() {
		let client = Client::tracked(rocket()).unwrap();
		let activity = generate_activity();
		let base_url = String::from("/activity");

		// Test unauthorized
		let res = client
			.post("/activity")
			.body(to_string(&activity).unwrap())
			.dispatch();
		assert_eq!(res.status(), Status::Unauthorized);

		// Test empty token
		let res = post(&client, &base_url, to_string(&activity).unwrap(), "");
		assert_eq!(res.status(), Status::Unauthorized);

		let token = get_token_admin(&client);
		// Create new activity
		let res = post(&client, &base_url, to_string(&activity).unwrap(), token);
		let inserted_activity = res.into_json::<Activity>().unwrap();
		assert_eq!(inserted_activity, activity);
		let activity_id = inserted_activity.id;

		// Check duplicate activity insert
		let res = post(&client, &base_url, to_string(&activity).unwrap(), token);
		assert_eq!(res.status(), Status::BadRequest);
		assert_eq!(
			res.into_string(),
			Some(
				to_string(&ErrorJson::new(400, "Activity already exists",))
					.expect("Could not serialize ErrorJson")
			)
		);

		// Update activity
		let mut new_company_name = activity.name.clone();
		// Companys name should end with GmbH on the bill or smth... 🤷
		new_company_name.push_str(" GmbH");
		let update_activity = UpdateActivity {
			name: Some(new_company_name.clone()),
			..Default::default()
		};
		let url = format!("{base_url}/{activity_id}");
		let res = patch(&client, &url, to_string(&update_activity).unwrap(), token);
		assert_eq!(res.status(), Status::Ok);
		let updated_activity = res.into_json::<Activity>().unwrap();
		assert_eq!(updated_activity.name, new_company_name);
		assert_ne!(updated_activity, inserted_activity);

		// Get activity
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		let res = res.into_json::<Activity>().unwrap();
		assert_eq!(res, updated_activity);

		// Test inserting as normal user
		let user_token = get_token_user(&client);
		let res = post(
			&client,
			"/activity",
			to_string(&activity).unwrap(),
			user_token,
		);
		assert_eq!(res.status(), Status::Forbidden);

		// delete activity
		let res = delete(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
	}

	#[tracing_test::traced_test]
	#[test]
	fn activity_multiple() {
		let client = Client::tracked(rocket()).unwrap();
		let token = get_token_admin(&client);
		let base_url = String::from("/activity");

		let mut activity_list: Vec<Activity> = Vec::new();
		// insert multiple activitys
		for _ in 0..10 {
			let activity = generate_activity();
			let res = post(&client, "/activity", to_string(&activity).unwrap(), token);
			if res.status() != Status::Ok {
				dbg!(res.into_string());
				panic!();
			}
			activity_list.push(res.into_json::<Activity>().unwrap());
		}

		let url = format!("{base_url}/{}", activity_list[0].id);
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		assert_eq!(res.into_json::<Activity>().unwrap(), activity_list[0]);

		// get first page of users with page_size 5
		let res = get(&client, "/activity/page/5/0", token);
		assert_eq!(res.status(), Status::Ok);
		let pagination = res.into_json::<PaginationResult<Activity>>().unwrap();
		assert_eq!(pagination.page, 0);
		assert_eq!(pagination.page_size, 5);

		// get last page of users with page_size 5 with number
		let url = format!("{base_url}/page/5/{}", pagination.num_pages - 1);
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		let pagination = res.into_json::<PaginationResult<Activity>>().unwrap();
		// the last page contains 5 OR LESS items
		let last_page_items: usize = (pagination.total_items - (pagination.num_pages - 1) * 5)
			.try_into()
			.unwrap();
		// reverse to get items from the bottom
		let last_page_items = 10 - last_page_items;
		assert_eq!(
			pagination.items,
			activity_list[last_page_items..],
			"race conditions may occur here, but in prod it doesn't matter"
		);

		let url = format!("{base_url}/page/5/last");
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		let last_page = res.into_json::<PaginationResult<Activity>>().unwrap();
		assert_eq!(last_page, pagination);

		// delete all activitys
		for activity in activity_list {
			let url = format!("{base_url}/{}", activity.id);
			let res = delete(&client, &url, token);
			assert_eq!(res.status(), Status::Ok);
		}
	}
}
