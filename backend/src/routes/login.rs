use rocket::{
	serde::json::Json,
	State,
};
use rocket_db_pools::Connection;
use serde::Deserialize;

use crate::{
	auth::Tokenizer,
	error::Error,
	models::user::User,
	DB,
};

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
pub struct LoginResponder {
	inner: String,
}

impl LoginResponder {
	pub fn new(inner: String) -> Self {
		trace!("LoginResponder::new({:?})", inner);
		Self {
			inner: format!("{{\"token\": \"{inner}\"}}"),
		}
	}
}

#[derive(Deserialize)]
struct Login<'r> {
	email: &'r str,
	password: &'r str,
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
