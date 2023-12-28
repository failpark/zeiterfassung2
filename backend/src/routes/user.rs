use rocket::{
	fairing::AdHoc,
	serde::json::Json,
};
use rocket_db_pools::Connection;

use crate::{
	db::user::{
		CreateUser,
		User,
	},
	Error,
	DB,
};

#[post("/", data = "<create_user>")]
async fn create(
	user: User,
	mut db: Connection<DB>,
	create_user: Json<CreateUser>,
) -> Result<Json<User>, Error> {
	if user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	let user = User::create(&mut db, &create_user).await;
	if let Ok(user) = user {
		Ok(Json(user))
	} else {
		Err(Error::BadRequest("User already exists".to_string()))
	}
	// Ok(Json(user))
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
		rocket.mount("/user", routes![create, delete])
	})
}

#[cfg(test)]
mod test {
	use pretty_assertions::assert_eq;
	use rocket::{
		http::Status,
		local::blocking::{
			Client,
			LocalResponse,
		},
		serde::json::to_string,
	};

	use crate::{
		db::user::{
			CreateUser,
			User,
		},
		error::ErrorJson,
		rocket,
		test::{
			cleanup_admin_user,
			generate_user,
			get_admin_token,
			AuthHeader,
		},
	};

	#[test]
	fn user() {
		let client = Client::tracked(rocket()).unwrap();
		let user = generate_user();

		// Test unauthorized
		let response = client
			.post("/user")
			.body(to_string(&user).expect("Could not serialize CreateUser"))
			.dispatch();
		assert_eq!(response.status(), Status::Unauthorized);

		let [token, admin_email] = get_admin_token(&client, None);
		// Create new user
		let response = create_user(&client, user.clone(), token.clone());
		let response = response.into_json::<User>().unwrap();
		assert_eq!(response.username, user.username);
		let user_id = response.id;

		// Check duplicate user insert
		let response = create_user(&client, user.clone(), token.clone());
		assert_eq!(response.status(), Status::BadRequest);
		assert_eq!(
			response.into_string(),
			Some(
				to_string(&ErrorJson::new(400, "User already exists"))
					.expect("Could not serialize ErrorJson")
			)
		);

		// test deleting user
		let response = client
			.delete("/user")
			.body(format!("{}", user_id))
			.add_auth_header(token)
			.dispatch();
		assert_eq!(response.status(), Status::Ok);

		cleanup_admin_user(&client, admin_email);
	}

	fn create_user(client: &Client, item: CreateUser, token: String) -> LocalResponse {
		client
			.post("/user")
			.body(to_string(&item).expect("Could not serialize CreateUser"))
			.add_auth_header(token)
			.dispatch()
	}
}
