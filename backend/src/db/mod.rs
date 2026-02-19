use diesel::MysqlConnection;
use diesel_migrations::{
	embed_migrations,
	EmbeddedMigrations,
	MigrationHarness,
};
use serde::{
	Deserialize,
	Serialize,
};

// TODO: Port from Rocket to Axum/deadpool-diesel in Phase 3+
// pub mod activity;
pub mod client;
// pub mod helper;
// pub mod project;
// pub mod tracking;
pub mod user;

/// Result of a `.paginate` function
#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

diesel::sql_function!(fn last_insert_id() -> Integer);

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");

pub fn run_migrations(conn: &mut MysqlConnection) {
	conn.run_pending_migrations(MIGRATIONS)
		.expect("Failed to run database migrations");
	::tracing::info!("Database migrations applied successfully");
}
