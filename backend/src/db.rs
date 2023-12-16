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

#[derive(Database)]
#[database("zeiterfassung2")]
pub struct DB(MysqlPool);

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
		.extract_inner("databases.zeiterfassung2.url")
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
