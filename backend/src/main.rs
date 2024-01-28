// #[macro_use]
// extern crate rocket;
use rocket::{
	catchers,
	fairing::AdHoc,
	launch,
	Rocket,
};
use rocket_db_pools::Database;
mod auth;
mod db;
mod error;
mod guard;
mod routes;
mod schema;
#[cfg(test)]
mod test;
pub use db::{
	user::User,
	DB,
};
pub use error::{
	Error,
	Result,
};
use rocket_cors::CorsOptions;
mod catchers;
use tracing::{
	subscriber::set_global_default,
	Level,
};
use tracing_subscriber::FmtSubscriber;

#[launch]
fn rocket() -> _ {
	let subscriber = FmtSubscriber::builder()
		.with_max_level(Level::ERROR)
		.finish();
	set_global_default(subscriber).expect("setting default subscriber failed");

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
		.attach(routes::login::mount())
		.attach(routes::activity::mount())
		.attach(routes::user::mount())
		.attach(routes::client::mount())
		.attach(routes::project::mount())
		.manage(auth::Tokenizer::new(std::time::Duration::new(
			5 * 24 * 60 * 60,
			0,
		)))
		.register("/", catchers![catchers::default_catcher])
}
