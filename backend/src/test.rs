use std::sync::OnceLock;

use diesel::{
	prelude::*,
	r2d2::{
		ConnectionManager,
		Pool,
		PooledConnection,
	},
};
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

pub fn db_url(client: &Client) -> &'static str {
	static DB_URL: OnceLock<String> = OnceLock::new();
	DB_URL.get_or_init(|| {
		client
			.rocket()
			.figment()
			.extract_inner("databases.zeiterfassung.url")
			.unwrap()
	})
}

fn get_sync_connection(client: &Client) -> PooledConnection<ConnectionManager<MysqlConnection>> {
	static DB_POOL: OnceLock<Pool<ConnectionManager<MysqlConnection>>> = OnceLock::new();
	DB_POOL.get_or_init(|| {
		let db_url = db_url(client);
		let manager = ConnectionManager::<MysqlConnection>::new(db_url);
		Pool::builder()
			.build(manager)
			.expect("Could not build connection pool")
	});
	DB_POOL.get().unwrap().get().unwrap()
}

pub fn create_user(client: &Client, item: CreateUser) -> anyhow::Result<()> {
	use crate::schema::user::dsl::*;

	let mut conn = get_sync_connection(client);

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
	let mut conn = get_sync_connection(client);

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
	Faker.fake_with_rng(&mut rng)
}

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

pub fn get_token(client: &Client, email: &str, password: &str) -> String {
	let token = client
		.post("/login")
		.body(to_string(&Login::new(email, password)).expect("Could not serialize Login"))
		.dispatch()
		.into_json::<Token>()
		.unwrap();
	token.token
}
