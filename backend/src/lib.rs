// TODO: Port to Axum in Phase 3+
// mod auth;
// mod catchers;
// mod db;
// mod guard;
// mod schema;
// #[cfg(test)] mod test;

pub mod config;
pub mod error;
mod routes;
pub mod state;
pub mod tracing;

pub use error::{
	Error,
	Result,
};
pub use state::AppState;
