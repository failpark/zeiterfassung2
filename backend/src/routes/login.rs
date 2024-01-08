use rocket::{
	fairing::AdHoc,
	request::Request,
	response::{
		content,
		Responder as ResponderImpl,
	},
	serde::json::{
		to_string,
		Json,
	},
	State,
};
use rocket_db_pools::Connection;
use serde::{
	Deserialize,
	Serialize,
};

use crate::{
	auth::Tokenizer,
	Error,
	User,
	DB,
};

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
pub struct LoginResponder {
	inner: Token,
}

#[derive(Serialize, Deserialize)]
pub struct Token {
	pub token: String,
}

impl<'r> ResponderImpl<'r, 'static> for Token {
	fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
		content::RawJson(to_string(&self).expect("Could not serialize Token")).respond_to(req)
	}
}

impl LoginResponder {
	pub fn new(inner: String) -> Self {
		trace!("LoginResponder::new({:?})", inner);
		Self {
			inner: Token { token: inner },
		}
	}
}

#[derive(Deserialize, Serialize)]
pub struct Login<'r> {
	email: &'r str,
	password: &'r str,
}

/// We do not need to construct Login manually in production, so we hide the constructor behind a cfg(test)
#[cfg(test)]
impl<'a> Login<'a> {
	pub fn new(email: &'a str, password: &'a str) -> Login<'a> {
		Self { email, password }
	}
}

#[post("/", data = "<login>")]
async fn post_login<'r>(
	tokenizer: &State<Tokenizer>,
	mut db: Connection<DB>,
	login: Json<Login<'_>>,
) -> Result<LoginResponder, Error> {
	Ok(LoginResponder::new(tokenizer.generate(
		User::check_credentials(&mut db, login.email, login.password).await?,
	)?))
}

pub fn mount() -> AdHoc {
	AdHoc::on_ignite("Mount Login Routes", |rocket| async {
		rocket.mount("/login", routes![post_login])
	})
}

#[cfg(test)]
mod test {
	use pretty_assertions::assert_eq;
	use rocket::{
		http::Status,
		local::blocking::Client,
		serde::json::to_string,
	};

	use super::Token;
	use crate::{
		auth::Tokenizer,
		error::ErrorJson,
		rocket,
		test::{
			cleanup_admin_user,
			create_admin,
		},
	};

	#[test]
	fn login() {
		let client = Client::tracked(rocket()).unwrap();
		let [admin_email, admin_password] = create_admin(&client, None).unwrap();
		let login = super::Login {
			email: admin_email.as_str(),
			password: admin_password.as_str(),
		};
		let response = client
			.post("/login")
			.body(format!("email={}&password=Admin_01!", login.email))
			.dispatch();
		assert_eq!(response.status(), Status::BadRequest);
		assert_eq!(
			response.into_string(),
			Some(to_string(&ErrorJson::new(400, "Bad Request")).expect("Could not serialize ErrorJson"))
		);

		let response = client
			.post("/login")
			.body(to_string(&login).expect("Could not serialize Login"))
			.dispatch();
		assert_eq!(response.status(), Status::Ok);
		let tokenizer = client.rocket().state::<Tokenizer>().unwrap();
		let token = response.into_json::<Token>().unwrap().token;
		assert!(tokenizer.verify(&token).is_ok());
		cleanup_admin_user(&client, admin_email);
	}
}
