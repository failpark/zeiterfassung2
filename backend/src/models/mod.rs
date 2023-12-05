pub mod activity;
pub mod client;
pub mod project;
pub mod tracking;
pub mod tracking_to_activity;
pub mod user;

diesel::sql_function!(fn last_insert_id() -> Integer);
