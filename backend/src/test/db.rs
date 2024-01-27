use std::sync::OnceLock;

use diesel::{
	prelude::*,
	r2d2::{
		ConnectionManager,
		Pool,
		PooledConnection,
	},
};
use rocket::local::blocking::Client;

use super::generate_user;
use crate::{
	auth::Tokenizer,
	db::user::CreateUser,
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
