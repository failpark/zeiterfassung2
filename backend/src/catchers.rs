use rocket::{
	http::{
		Status,
		StatusClass,
	},
	Request,
};

use crate::error::ErrorJson;

#[catch(default)]
/// always return json
pub fn default_catcher(status: Status, request: &Request) -> String {
	let error = ErrorJson::new(status.code, status.reason_lossy());
	let json = rocket::serde::json::to_string(&error).expect("Could not serialize json");
	let debug = format!(
		"Default Catcher with: {:?} for request_uri: {} and method: {}",
		json,
		request.uri(),
		request.method()
	);
	match status.class() {
		StatusClass::Success | StatusClass::Informational => {
			debug!("{}", debug);
		}
		StatusClass::Redirection => {
			info!("{}", debug);
		}
		StatusClass::ClientError | StatusClass::ServerError | StatusClass::Unknown => {
			error!("{}", debug);
		}
	}

	json
}
