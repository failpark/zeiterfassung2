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
		tracking::{
			CreateTracking,
			Tracking,
			UpdateTracking,
		},
		PaginationResult,
	},
	Error,
	Result,
	User,
	DB,
};

#[post("/", data = "<create_tracking>")]
async fn create(
	user: User,
	mut db: Connection<DB>,
	create_tracking: Json<CreateTracking>,
) -> Result<Json<Tracking>> {
	if user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	let tracking = Tracking::create(&mut db, &create_tracking).await;
	if let Ok(tracking) = tracking {
		Ok(Json(tracking))
	} else {
		Err(Error::BadRequest("Tracking already exists".to_string()))
	}
}

#[patch("/<id>", data = "<update_tracking>")]
async fn update(
	_user: User,
	mut db: Connection<DB>,
	update_tracking: Json<UpdateTracking>,
	id: i32,
) -> Result<Json<Tracking>> {
	Ok(Json(Tracking::update(&mut db, id, &update_tracking).await?))
}

#[get("/<id>")]
async fn get(_user: User, mut db: Connection<DB>, id: i32) -> Result<Json<Tracking>> {
	Ok(Json(Tracking::read(&mut db, id).await?))
}

#[get("/page/<page_size>/<page>")]
async fn get_page(
	_user: User,
	mut db: Connection<DB>,
	page_size: i64,
	page: i64,
) -> Result<Json<PaginationResult<Tracking>>> {
	Ok(Json(Tracking::paginate(&mut db, page, page_size).await?))
}

#[get("/page/<page_size>/last", rank = 2)]
async fn get_last_page(
	_user: User,
	mut db: Connection<DB>,
	page_size: i64,
) -> Result<Json<PaginationResult<Tracking>>> {
	let last_page = Tracking::last_page(&mut db, page_size).await?;
	Ok(Json(
		Tracking::paginate(&mut db, last_page, page_size).await?,
	))
}

#[delete("/<id>")]
async fn delete(user: User, mut db: Connection<DB>, id: i32) -> Result<Json<usize>> {
	if user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	Ok(Json(Tracking::delete(&mut db, id).await?))
}

pub fn mount() -> AdHoc {
	AdHoc::on_ignite("Tracking", |rocket| async {
		rocket.mount(
			"/tracking",
			routes![create, get, update, delete, get_page, get_last_page,],
		)
	})
}

#[cfg(test)]
mod test {
	use fake::{
		Fake,
		Faker,
	};
	use itertools::Itertools;
	use pretty_assertions::{
		assert_eq,
		assert_ne,
	};
	use rand::{
		rngs::StdRng,
		SeedableRng,
	};
	use rocket::{
		http::Status,
		local::blocking::Client,
		serde::json::to_string,
	};

	use crate::{
		db::{
			client::Client as ClientDB,
			project::Project,
			tracking::{
				CreateTracking,
				Tracking,
				UpdateTracking,
			},
			user::User,
			PaginationResult,
		},
		error::ErrorJson,
		rocket,
		test::{
			generate_client,
			generate_project,
			generate_user,
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

	pub fn generate_tracking_raw(client_id: i32, user_id: i32, project_id: i32) -> CreateTracking {
		let mut rng = StdRng::from_entropy();
		let activities = vec![
			rand::Rng::gen_range(&mut rng, 1..50),
			rand::Rng::gen_range(&mut rng, 1..50),
			rand::Rng::gen_range(&mut rng, 1..50),
		];
		let activities = activities.into_iter().unique().collect();
		CreateTracking {
			client_id,
			user_id,
			project_id,
			date: Faker.fake(),
			begin: Faker.fake(),
			end: Faker.fake(),
			pause: Faker.fake(),
			performed: ((Faker.fake::<f32>() * 100.0).round() / 100.0),
			billed: ((Faker.fake::<f32>() * 100.0).round() / 100.0),
			description: Faker.fake(),
			activities,
		}
	}

	fn generate_tracking<'a>(client: &'a Client, token: &'a str) -> CreateTracking {
		let (client_db, user, project) = generate_client_user_project(client, token);
		generate_tracking_raw(client_db.id, user.id, project.id)
	}

	fn generate_client_user_project<'a>(
		client: &'a Client,
		token: &'a str,
	) -> (ClientDB, User, Project) {
		let client_db = generate_client();
		let client_db = post(client, "/client", to_string(&client_db).unwrap(), token)
			.into_json::<ClientDB>()
			.unwrap();
		let user = generate_user();
		let user = post(client, "/user", to_string(&user).unwrap(), token)
			.into_json::<User>()
			.unwrap();
		let mut project = generate_project();
		project.client_id = client_db.id;
		let project = post(client, "/project", to_string(&project).unwrap(), token)
			.into_json::<Project>()
			.unwrap();
		(client_db, user, project)
	}

	#[tracing_test::traced_test]
	#[test]
	fn tracking_single() {
		let client = Client::tracked(rocket()).unwrap();
		let token = get_token_admin(&client);
		let tracking = generate_tracking(&client, token);
		let base_url = String::from("/tracking");

		// Test unauthorized
		let res = client
			.post("/tracking")
			.body(to_string(&tracking).unwrap())
			.dispatch();
		assert_eq!(res.status(), Status::Unauthorized);

		// Test empty token
		let res = post(&client, &base_url, to_string(&tracking).unwrap(), "");
		assert_eq!(res.status(), Status::Unauthorized);

		// Create new tracking
		let res = post(&client, &base_url, to_string(&tracking).unwrap(), token);
		let inserted_tracking = res.into_json::<Tracking>().unwrap();
		assert_eq!(inserted_tracking, tracking);
		let tracking_id = inserted_tracking.id;

		// Check duplicate tracking insert
		let res = post(&client, &base_url, to_string(&tracking).unwrap(), token);
		// TODO Allow duplicate tracking for now
		assert_eq!(res.status(), Status::Ok);
		assert_ne!(
			res.into_string(),
			Some(
				to_string(&ErrorJson::new(400, "Tracking already exists",))
					.expect("Could not serialize ErrorJson")
			)
		);

		// Update tracking
		let new_activities = vec![2, 5, 7];
		let update_tracking = UpdateTracking {
			activities: Some(new_activities.clone()),
			..Default::default()
		};
		let url = format!("{base_url}/{tracking_id}");
		let res = patch(&client, &url, to_string(&update_tracking).unwrap(), token);
		assert_eq!(res.status(), Status::Ok);
		let updated_tracking = res.into_json::<Tracking>().unwrap();
		assert_eq!(updated_tracking.activities, new_activities);
		assert_ne!(updated_tracking, inserted_tracking);

		// Get tracking
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		let res = res.into_json::<Tracking>().unwrap();
		assert_eq!(res, updated_tracking);

		// Test inserting as normal user
		let user_token = get_token_user(&client);
		let res = post(
			&client,
			"/tracking",
			to_string(&tracking).unwrap(),
			user_token,
		);
		assert_eq!(res.status(), Status::Forbidden);

		// delete tracking
		let res = delete(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
	}

	#[tracing_test::traced_test]
	#[test]
	fn tracking_multiple() {
		let client = Client::tracked(rocket()).unwrap();
		let token = get_token_admin(&client);
		let base_url = String::from("/tracking");
		let (client_db, user, project) = generate_client_user_project(&client, token);

		let mut tracking_list: Vec<Tracking> = Vec::new();
		// insert multiple trackings
		for _ in 0..10 {
			let tracking = generate_tracking_raw(client_db.id, user.id, project.id);
			let res = post(&client, "/tracking", to_string(&tracking).unwrap(), token);
			if res.status() != Status::Ok {
				dbg!(res.into_string());
				panic!();
			}
			tracking_list.push(res.into_json::<Tracking>().unwrap());
		}

		let url = format!("{base_url}/{}", tracking_list[0].id);
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		assert_eq!(res.into_json::<Tracking>().unwrap(), tracking_list[0]);

		// get first page of users with page_size 5
		let res = get(&client, "/tracking/page/5/0", token);
		assert_eq!(res.status(), Status::Ok);
		let pagination = res.into_json::<PaginationResult<Tracking>>().unwrap();
		assert_eq!(pagination.page, 0);
		assert_eq!(pagination.page_size, 5);

		// get last page of users with page_size 5 with number
		let url = format!("{base_url}/page/5/{}", pagination.num_pages - 1);
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		let pagination = res.into_json::<PaginationResult<Tracking>>().unwrap();
		// the last page contains 5 OR LESS items
		let last_page_items: usize = (pagination.total_items - (pagination.num_pages - 1) * 5)
			.try_into()
			.unwrap();
		// reverse to get items from the bottom
		let last_page_items = 10 - last_page_items;
		assert_eq!(
			pagination.items,
			tracking_list[last_page_items..],
			"race conditions may occur here, but in prod it doesn't matter"
		);

		let url = format!("{base_url}/page/5/last");
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		let last_page = res.into_json::<PaginationResult<Tracking>>().unwrap();
		assert_eq!(last_page, pagination);

		// delete all trackings
		for tracking in tracking_list {
			let url = format!("{base_url}/{}", tracking.id);
			let res = delete(&client, &url, token);
			assert_eq!(res.status(), Status::Ok);
		}
	}
}
