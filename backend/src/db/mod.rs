use rocket::{
	Build,
	Rocket,
};
use rocket_db_pools::{
	diesel::{
		prelude::*,
		MysqlPool,
	},
	Database,
};
use serde::{
	Deserialize,
	Serialize,
};

pub mod activity;
pub mod client;
pub mod project;
pub mod tracking;
pub mod tracking_to_activity;
pub mod user;

/// Result of a `.paginate` function
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationResult<T> {
	/// Resulting items that are from the current page
	pub items: Vec<T>,
	/// The count of total items there are
	pub total_items: i64,
	/// Current page, 0-based index
	pub page: i64,
	/// Size of a page
	pub page_size: i64,
	/// Number of total possible pages, given the `page_size` and `total_items`
	pub num_pages: i64,
}

#[derive(Database)]
#[database("zeiterfassung")]
pub struct DB(MysqlPool);

diesel::sql_function!(fn last_insert_id() -> Integer);

pub async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
	use diesel_migrations::{
		embed_migrations,
		EmbeddedMigrations,
		MigrationHarness,
	};

	trace!("Running migrations");
	const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");

	let db_url: String = rocket
		.figment()
		.extract_inner("databases.zeiterfassung.url")
		.expect("DB not configured");

	rocket::tokio::task::spawn_blocking(move || {
		diesel::MysqlConnection::establish(&db_url)
			.expect("No database")
			.run_pending_migrations(MIGRATIONS)
			.expect("Invalid migrations");
	})
	.await
	.expect("tokio doesn't work");

	rocket
}
