#[macro_use]
extern crate rocket;
use rocket::{
	fairing::AdHoc,
	Rocket,
};
use rocket_db_pools::Database;
mod auth;
mod db;
mod error;
mod models;
mod routes;
mod schema;
#[cfg(test)]
mod tests;
pub use db::DB;
use rocket_cors::CorsOptions;

#[launch]
fn rocket() -> _ {
	let allowed_origins = rocket_cors::AllowedOrigins::some_exact(&["http://localhost:5173"]);
	Rocket::build()
		.attach(
			CorsOptions {
				allowed_origins,
				..Default::default()
			}
			.to_cors()
			.expect("error while building CORS"),
		)
		.attach(DB::init())
		.attach(AdHoc::on_ignite("Run Migrations", db::run_migrations))
		.manage(auth::Tokenizer::new(std::time::Duration::new(
			5 * 24 * 60 * 60,
			0,
		)))
}
