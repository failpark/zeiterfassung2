use std::sync::OnceLock;

use rocket::{
	http::Header,
	local::blocking::{
		Client,
		LocalRequest,
	},
	serde::json::to_string,
};

use super::{
	db::{
		create_admin,
		create_user,
	},
	generate_user,
};
use crate::{
	auth::Tokenizer,
	routes::login::{
		Login,
		Token,
	},
};

pub trait AuthHeader<'a> {
	fn add_auth_header(self, token: &'_ str) -> Self;
}

impl AuthHeader<'_> for LocalRequest<'_> {
	fn add_auth_header(mut self, token: &str) -> Self {
		self.add_header(Header::new("Authorization", format!("Bearer {}", token)));
		self
	}
}

pub fn get_admin_token(client: &Client, password: Option<String>) -> [String; 2] {
	let [admin_email, admin_password] = create_admin(client, password).unwrap();
	println!("Admin email: {admin_email}");
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

/// Get a token for a test user
pub fn get_token_user(client: &Client) -> &'static str {
	static TOKEN: OnceLock<String> = OnceLock::new();
	TOKEN.get_or_init(|| {
		let mut user = generate_user();
		let password = user.password;
		user.password = Tokenizer::hash_password(password.as_bytes()).unwrap();
		create_user(client, user.clone()).expect("Creating test user failed");
		get_token(client, &user.email, &password)
	})
}

pub fn get_token_admin(client: &Client) -> &'static str {
	static TOKEN: OnceLock<String> = OnceLock::new();
	TOKEN.get_or_init(|| {
		let [token, _email] = get_admin_token(client, None);
		token
	})
}

pub fn get_token(client: &Client, email: &str, password: &str) -> String {
	let token = client
		.post("/login")
		.body(to_string(&Login::new(email, password)).expect("Could not serialize Login"))
		.dispatch()
		.into_json::<Token>()
		.unwrap();
	token.token
}
