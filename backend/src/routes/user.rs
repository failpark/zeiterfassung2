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
	auth::Tokenizer,
	db::{
		user::{
			CreateUser,
			UpdateUser,
			User,
		},
		PaginationResult,
	},
	Error,
	DB,
};

#[post("/", data = "<create_user>")]
async fn create(
	user: User,
	mut db: Connection<DB>,
	mut create_user: Json<CreateUser>,
) -> Result<Json<User>, Error> {
	if user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	create_user.password = Tokenizer::hash_password(create_user.password.as_bytes())?;
	let user = User::create(&mut db, &create_user).await;
	if let Ok(user) = user {
		Ok(Json(user))
	} else {
		Err(Error::BadRequest("User already exists".to_string()))
	}
}

#[patch("/<id>", data = "<update_user>")]
async fn update(
	user: User,
	mut db: Connection<DB>,
	update_user: Json<UpdateUser>,
	id: i32,
) -> Result<Json<User>, Error> {
	if user.id != id && user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	let user = User::update(&mut db, id, &update_user).await?;
	Ok(Json(user))
}

#[get("/<id>")]
async fn get(_user: User, mut db: Connection<DB>, id: i32) -> Result<Json<User>, Error> {
	Ok(Json(User::read(&mut db, id).await?))
}

#[get("/page/<page_size>/<page>")]
async fn get_page(
	_user: User,
	mut db: Connection<DB>,
	page_size: i64,
	page: i64,
) -> Result<Json<PaginationResult<User>>, Error> {
	Ok(Json(User::paginate(&mut db, page, page_size).await?))
}

#[get("/page/<page_size>/last", rank = 2)]
async fn get_last_page(
	_user: User,
	mut db: Connection<DB>,
	page_size: i64,
) -> Result<Json<PaginationResult<User>>, Error> {
	let last_page = User::last_page(&mut db, page_size).await?;
	Ok(Json(User::paginate(&mut db, last_page, page_size).await?))
}

#[delete("/<id>")]
async fn delete(user: User, mut db: Connection<DB>, id: i32) -> Result<(), Error> {
	if user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	User::delete(&mut db, id).await?;
	Ok(())
}

// #[delete("/", data = "<user_id>")]
// async fn soft_delete(user: User, mut db: Connection<DB>, user_id: Json<i32>) -> Result<(), Error> {
// 	if user.sys_role != "admin" {
// 		return Err(Error::ForbiddenAccess);
// 	}
// 	User::soft_delete(&mut db, *user_id).await?;
// 	Ok(())
// }

pub fn mount() -> AdHoc {
	AdHoc::on_ignite("User Routes", |rocket| async {
		rocket.mount(
			"/user",
			routes![create, get, get_page, get_last_page, update, delete],
		)
	})
}

#[cfg(test)]
mod test {
	use fake::{
		faker::internet::en::Username,
		Fake,
	};
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
			user::{
				UpdateUser,
				User,
			},
			PaginationResult,
		},
		error::ErrorJson,
		rocket,
		test::{
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

	#[test]
	fn user() {
		let client = Client::tracked(rocket()).unwrap();
		let mut user = generate_user();
		let base_url = String::from("/user");

		// Test unauthorized
		let res = post(&client, &base_url, to_string(&user).unwrap(), "");
		assert_eq!(res.status(), Status::Unauthorized);

		let token = get_token_admin(&client);
		// Create new user
		let res = post(&client, &base_url, to_string(&user).unwrap(), token);
		let inserted_user = res.into_json::<User>().unwrap();
		assert_eq!(inserted_user, user);
		let user_id = inserted_user.id;

		// Check duplicate user insert
		let res = post(&client, &base_url, to_string(&user).unwrap(), token);
		assert_eq!(res.status(), Status::BadRequest);
		assert_eq!(
			res.into_string(),
			Some(
				to_string(&ErrorJson::new(400, "User already exists"))
					.expect("Could not serialize ErrorJson")
			)
		);
		let new_username = Username().fake::<String>();
		user.username = new_username.clone();
		let update_user = UpdateUser {
			username: Some(new_username.clone()),
			..Default::default()
		};
		let url = format!("{base_url}/{user_id}");
		let res = patch(&client, &url, to_string(&update_user).unwrap(), token);
		assert_eq!(res.status(), Status::Ok);
		let updated_user = res.into_json::<User>().unwrap();
		assert_eq!(updated_user.username, new_username);
		assert_eq!(updated_user.firstname, user.firstname);
		// They should differ because the password is hashed and updated_at is updated and username is changed
		assert_ne!(updated_user, inserted_user);

		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		let res = res.into_json::<User>().unwrap();
		assert_eq!(res, user);
		assert_eq!(res, updated_user);

		// Test inserting as normal user -> should fail
		let user_token = get_token_user(&client);
		let user = generate_user();
		let res = post(&client, &base_url, to_string(&user).unwrap(), user_token);
		assert_eq!(res.status(), Status::Forbidden);

		// test deleting user
		let res = delete(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
	}

	#[test]
	fn users() {
		let client = Client::tracked(rocket()).unwrap();
		let token = get_token_admin(&client);
		let base_url = String::from("/user");
		let mut user_list = Vec::new();
		for _ in 0..10 {
			let user = generate_user();
			let res = post(&client, &base_url, to_string(&user).unwrap(), token);
			assert_eq!(res.status(), Status::Ok);
			user_list.push(res.into_json::<User>().unwrap());
		}
		let user_token = get_token_user(&client);

		// get single user from id
		let url = format!("{base_url}/{}", user_list[0].id);
		let res = get(&client, &url, "");
		assert_eq!(res.status(), Status::Unauthorized);
		let res = get(&client, &url, user_token);
		assert_eq!(res.status(), Status::Ok);
		assert_eq!(res.into_json::<User>().unwrap(), user_list[0]);

		// get first page of users with page_size 5
		let res = get(&client, "/user/page/5/0", user_token);
		assert_eq!(res.status(), Status::Ok);
		let pagination = res.into_json::<PaginationResult<User>>().unwrap();
		assert_eq!(pagination.page, 0);
		assert_eq!(pagination.page_size, 5);

		// get last page of users with page_size 5 with number
		let url = format!("{base_url}/page/5/{}", pagination.num_pages - 1);
		let res = get(&client, &url, user_token);
		assert_eq!(res.status(), Status::Ok);
		let pagination = res.into_json::<PaginationResult<User>>().unwrap();
		// the last page contains 5 OR LESS items
		let last_page_items: usize = (pagination.total_items - (pagination.num_pages - 1) * 5)
			.try_into()
			.unwrap();
		// reverse to get items from the bottom
		let last_page_items = 10 - last_page_items;
		// some race conditions could arrise here, but in prod it doesn't matter
		assert_eq!(pagination.items, user_list[last_page_items..]);

		let url = format!("{base_url}/page/5/last");
		let res = get(&client, &url, user_token);
		assert_eq!(res.status(), Status::Ok);
		let last_page = res.into_json::<PaginationResult<User>>().unwrap();
		assert_eq!(last_page, pagination);

		// delete all activitys
		for user in user_list {
			let url = format!("{base_url}/{}", user.id);
			let res = delete(&client, &url, user_token);
			assert_eq!(res.status(), Status::Unauthorized);
			let res = delete(&client, &url, token);
			assert_eq!(res.status(), Status::Ok);
		}
	}
}
