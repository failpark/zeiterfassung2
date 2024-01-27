use rocket::{
	fairing::AdHoc,
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
	create_user.hash = Tokenizer::hash_password(create_user.hash.as_bytes())?;
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

#[delete("/", data = "<user_id>")]
async fn delete(user: User, mut db: Connection<DB>, user_id: Json<i32>) -> Result<(), Error> {
	if user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	User::delete(&mut db, *user_id).await?;
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
		local::blocking::{
			Client,
			LocalResponse,
		},
		serde::json::to_string,
	};

	use crate::{
		db::{
			user::{
				CreateUser,
				UpdateUser,
				User,
			},
			PaginationResult,
		},
		error::ErrorJson,
		rocket,
		test::{
			db::cleanup_admin_user,
			generate_user,
			token::{
				get_admin_token,
				get_token,
				AuthHeader,
			},
		},
	};

	#[test]
	fn user() {
		let client = Client::tracked(rocket()).unwrap();
		let mut user = generate_user();

		// Test unauthorized
		let response = client
			.post("/user")
			.body(to_string(&user).expect("Could not serialize CreateUser"))
			.dispatch();
		assert_eq!(response.status(), Status::Unauthorized);

		let [token, admin_email] = get_admin_token(&client, None);
		// Create new user
		let response = create_user(&client, user.clone(), &token);
		let inserted_user = response.into_json::<User>().unwrap();
		assert_eq!(inserted_user, user);
		let user_id = inserted_user.id;

		// Check duplicate user insert
		let response = create_user(&client, user.clone(), &token);
		assert_eq!(response.status(), Status::BadRequest);
		assert_eq!(
			response.into_string(),
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
		let response = client
			.patch(format!("/user/{user_id}"))
			.body(to_string(&update_user).expect("Could not serialize CreateUser"))
			.add_auth_header(&token)
			.dispatch();
		assert_eq!(response.status(), Status::Ok);
		let updated_user = response.into_json::<User>().unwrap();
		assert_eq!(updated_user.username, new_username);
		assert_eq!(updated_user.firstname, user.firstname);
		// They should differ because the password is hashed and updated_at is updated and username is changed
		assert_ne!(updated_user, inserted_user);

		let response = client
			.get(format!("/user/{}", user_id))
			.add_auth_header(&token)
			.dispatch();
		assert_eq!(response.status(), Status::Ok);
		let response = response.into_json::<User>().unwrap();
		assert_eq!(response, user);
		assert_eq!(response, updated_user);

		// Test inserting as normal user -> should fail
		let user_token = get_token(&client, &user.email, &user.hash);
		let response = create_user(&client, generate_user(), &user_token);
		assert_eq!(response.status(), Status::Forbidden);

		// test deleting user
		let response = client
			.delete("/user")
			.body(user_id.to_string())
			.add_auth_header(&token)
			.dispatch();
		assert_eq!(response.status(), Status::Ok);

		cleanup_admin_user(&client, admin_email);
	}

	#[derive(Default)]
	struct UserList {
		users: Vec<User>,
		create_users: Vec<CreateUser>,
	}

	#[test]
	fn users() {
		let client = Client::tracked(rocket()).unwrap();
		let [token, admin_email] = get_admin_token(&client, None);
		let mut user_list = UserList::default();
		for _ in 0..10 {
			let user = generate_user();
			user_list.create_users.push(user.clone());
			let response = create_user(&client, user, &token);
			assert_eq!(response.status(), Status::Ok);
			user_list.users.push(response.into_json::<User>().unwrap());
		}
		println!("Users: {:#?}", user_list.users);
		let user_token = get_token(
			&client,
			&user_list.users[0].email,
			&user_list.create_users[0].hash,
		);

		// get single user from id
		let response = client
			.get(format!("/user/{}", user_list.users[0].id))
			.dispatch();
		assert_eq!(response.status(), Status::Unauthorized);
		let response = client
			.get(format!("/user/{}", user_list.users[0].id))
			.add_auth_header(&user_token)
			.dispatch();
		assert_eq!(response.status(), Status::Ok);
		assert_eq!(response.into_json::<User>().unwrap(), user_list.users[0]);

		// get first page of users with page_size 5
		let response = client
			.get("/user/page/5/0")
			.add_auth_header(&user_token)
			.dispatch();
		assert_eq!(response.status(), Status::Ok);
		let users = response
			.into_json::<PaginationResult<User>>()
			.unwrap()
			.items;
		assert!(users.len() == 5);

		// get next page of users with page_size 5
		let response = client
			.get("/user/page/5/1")
			.add_auth_header(&user_token)
			.dispatch();
		assert_eq!(response.status(), Status::Ok);
		let pagination = response.into_json::<PaginationResult<User>>().unwrap();
		let next_users = pagination.items;
		assert!(next_users.len() == 5);
		assert_ne!(users, next_users);

		let response = client
			.get(format!("/user/page/5/{}", pagination.num_pages - 1))
			.add_auth_header(&user_token)
			.dispatch();
		let pagination = response.into_json::<PaginationResult<User>>().unwrap();
		let last_page_items: usize = (pagination.total_items - (pagination.num_pages - 1) * 5)
			.try_into()
			.unwrap();
		// reverse to get items from the bottom
		let last_page_items = 10 - last_page_items;
		assert_eq!(pagination.items, user_list.users[last_page_items..]);

		let response = client
			.get("/user/page/5/last")
			.add_auth_header(&user_token)
			.dispatch();
		let last_page = response.into_json::<PaginationResult<User>>().unwrap();
		assert_eq!(last_page, pagination);

		// cleanup users
		for user in user_list.users {
			let response = client
				.delete("/user")
				.body(user.id.to_string())
				.add_auth_header(&token)
				.dispatch();
			assert_eq!(response.status(), Status::Ok);
		}

		cleanup_admin_user(&client, admin_email);
	}

	fn create_user<'a>(client: &'a Client, item: CreateUser, token: &str) -> LocalResponse<'a> {
		client
			.post("/user")
			.body(to_string(&item).expect("Could not serialize CreateUser"))
			.add_auth_header(token)
			.dispatch()
	}
}
