mod middlelayer;
// use accurate file names for db but not for api
// -> this is ok since the db is a private module and the api is a public module
#[allow(clippy::module_inception)]
mod tracking;
mod tracking_to_activity;

pub use middlelayer::{
	CreateTracking,
	Tracking,
	UpdateTracking,
};
