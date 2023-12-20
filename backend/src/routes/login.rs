use rocket::{
	fairing::AdHoc,
	serde::json::Json,
	State,
};
use rocket_db_pools::Connection;
use serde::{
	Deserialize,
	Serialize,
};
use tracing::debug;

use crate::{
	auth::Tokenizer,
	db::user::User,
	error::Error,
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

#[derive(Deserialize, Serialize)]
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

#[get("/hello")]
async fn hello() -> &'static str {
	trace!("Hello, world!");
	debug!("Hello, world!");
	"Hello, world!"
}

#[cfg(test)]
mod test {
	use rocket::{
		http::Status,
		local::blocking::Client,
	};

	use crate::rocket;

	#[test]
	fn test_login() {
		let client = Client::tracked(rocket()).unwrap();
		let login = super::Login {
			email: "test@example.com",
			password: "admin",
		};
		let response = client.post("/login").json(&login).dispatch();
		println!("{:?}", response);
	}
}

pub fn mount() -> AdHoc {
	AdHoc::on_ignite("Mount Login Routes", |rocket| async {
		rocket.mount("/login", routes![post_login, hello])
	})
}
