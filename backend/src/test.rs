use diesel::prelude::*;
use fake::{
	Fake,
	Faker,
};
use rand::{
	rngs::StdRng,
	SeedableRng,
};
use rocket::{
	http::Header,
	local::blocking::{
		Client,
		LocalRequest,
	},
	serde::json::to_string,
};

use crate::{
	auth::Tokenizer,
	db::user::CreateUser,
	routes::login::{
		Login,
		Token,
	},
};
// use std::sync::OnceLock;

// static ADMIN_USER_IDS: OnceLock<Vec<i32>> = OnceLock::new();

pub fn create_user(client: &Client, item: CreateUser) -> anyhow::Result<()> {
	use crate::schema::user::dsl::*;

	let db_url: String = client
		.rocket()
		.figment()
		.extract_inner("databases.zeiterfassung2.url")
		.unwrap();

	let mut conn = diesel::MysqlConnection::establish(&db_url).expect("No database");

	diesel::insert_into(user)
		.values(item)
		.execute(&mut conn)
		.expect("Inserting into user table failed");
	Ok(())
}

pub fn cleanup_admin_user(client: &Client, admin_email: String) {
	let cleanup = delete_user(client, admin_email.clone());
	if let Err(e) = cleanup {
		panic!("Could not cleanup for user: {admin_email}\n{e:?}");
	}
}

pub fn delete_user(client: &Client, param_email: String) -> anyhow::Result<()> {
	use crate::schema::user::dsl::*;

	let db_url: String = client
		.rocket()
		.figment()
		.extract_inner("databases.zeiterfassung2.url")
		.unwrap();

	let mut conn = diesel::MysqlConnection::establish(&db_url).expect("No database");

	diesel::delete(user.filter(email.eq(param_email)))
		.execute(&mut conn)
		.expect("Deleting from user table failed");
	Ok(())
}

pub fn create_admin(client: &Client, password: Option<String>) -> anyhow::Result<[String; 2]> {
	let mut admin = generate_user();
	let admin_password = if let Some(password) = password {
		password
	} else {
		String::from("Admin_01!")
	};
	admin.sys_role = "admin".to_string();
	admin.hash = Tokenizer::hash_password(admin_password.as_bytes()).unwrap();
	let admin_email = admin.email.clone();
	create_user(client, admin).expect("Creating admin failed");
	Ok([admin_email, admin_password])
}

pub fn generate_user() -> CreateUser {
	let mut rng = StdRng::from_entropy();
	let mut user: CreateUser = Faker.fake_with_rng(&mut rng);
	user.hash = Tokenizer::hash_password("User_01!".as_bytes()).unwrap();
	user
}

pub trait AuthHeader {
	fn add_auth_header(self, token: String) -> Self;
}

impl AuthHeader for LocalRequest<'_> {
	fn add_auth_header(mut self, token: String) -> Self {
		self.add_header(Header::new("Authorization", format!("Bearer {}", token)));
		self
	}
}

pub fn get_admin_token(client: &Client, password: Option<String>) -> [String; 2] {
	let [admin_email, admin_password] = create_admin(client, password).unwrap();
	let token = client
		.post("/login")
		.body(
			to_string(&Login::new(admin_email.as_str(), admin_password.as_str()))
				.expect("Could not serialize Login"),
		)
		.dispatch()
		.into_json::<Token>()
		.unwrap();
	[token.token, admin_email]
}
