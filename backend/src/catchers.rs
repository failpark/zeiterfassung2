use rocket::{
	catch,
	http::{
		Status,
		StatusClass,
	},
	Request,
};
use tracing::{
	debug,
	error,
	info,
};

use crate::error::ErrorJson;

#[catch(default)]
/// always return json
pub fn default_catcher(status: Status, request: &Request) -> String {
	let error = ErrorJson::new(status.code, status.reason_lossy());
	let json = rocket::serde::json::to_string(&error).expect("Could not serialize json");
	let debug_msg = format!(
		"Default Catcher: status: {} with reason: {} for request_uri: {} and method: {}",
		status.code,
		status.reason_lossy(),
		request.uri(),
		request.method()
	);
	match status.class() {
		StatusClass::Success | StatusClass::Informational => {
			debug!(debug_msg);
		}
		StatusClass::Redirection => {
			info!(debug_msg);
		}
		StatusClass::ClientError | StatusClass::ServerError | StatusClass::Unknown => {
			match status.code {
				401 => {
					info!(debug_msg);
				}
				_ => {
					error!(debug_msg);
				}
			}
		}
	}

	json
}
