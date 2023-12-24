use rocket::{
	http::{
		ContentType,
		Status,
	},
	request::Request,
	response::{
		self,
		Responder,
	},
	serde::json::Json,
};
use serde::{
	ser::{
		SerializeStruct,
		Serializer,
	},
	Deserialize,
	Serialize,
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum Error {
	#[error("Database Error: {0}")]
	Database(#[from] diesel::result::Error),
	#[error("Password Error: {0}")]
	Argon2PasswordHash(argon2::password_hash::Error),
	#[error("Serde Json Error: {0}")]
	Serde(#[from] rocket::serde::json::serde_json::Error),
	#[error("Database Primary Key Error: {0}")]
	TryFromInt(#[from] std::num::TryFromIntError),
	#[error("Error setting up CORS: {0}")]
	RocketCors(#[from] rocket_cors::Error),
	#[error("Launch Failed: {0}")]
	Rocket(#[from] rocket::Error),
	#[error("Could not sign token: {0}")]
	JWTSign(#[source] jwt_simple::Error),
	#[error("Could not verify token: {0}")]
	JWTVerify(#[source] jwt_simple::Error),
	#[error("General JWT Error: {0}")]
	JWT(#[from] jwt_simple::Error),
	#[error("Not found")]
	NotFound,
	#[error("Could not insert Zeiterfassung")]
	ZeiterfassungInsert,
	#[error("Unknown Error")]
	Unknown,
	#[error("Internal error")]
	Internal,
	#[error("Unauthenticated user")]
	UnauthenticatedUser,
	#[error("User does not have access rights")]
	ForbiddenAccess,
	#[error("{0}")]
	BadRequest(String),
	#[error("Wrong Credentials")]
	WrongCredentials,
	#[error("Unauthorized")]
	Unauthorized,
}

impl Error {
	fn to_status(&self) -> Status {
		match self {
			Self::NotFound => Status::NotFound,
			Self::UnauthenticatedUser | Self::WrongCredentials | Self::Unauthorized => {
				Status::Unauthorized
			}
			Self::ForbiddenAccess => Status::Forbidden,
			Self::BadRequest(_) | Self::JWT(_) => Status::BadRequest,
			_ => Status::InternalServerError,
		}
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorJson<'a> {
	error: &'a str,
	code: u16,
}

impl<'a> ErrorJson<'a> {
	pub fn new(code: u16, error: &'a str) -> Self {
		Self { error, code }
	}
}

impl Serialize for Error {
	fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut state = serializer.serialize_struct("Error", 2)?;
		state.serialize_field("error", &self.to_string())?;
		state.serialize_field("code", &self.to_status().code)?;

		state.end()
	}
}

impl From<argon2::password_hash::Error> for Error {
	fn from(e: argon2::password_hash::Error) -> Self {
		Error::Argon2PasswordHash(e)
	}
}

impl<'r> Responder<'r, 'static> for Error {
	fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
		let status = self.to_status();
		response::Response::build_from(Json(self).respond_to(request)?)
			.status(status)
			.header(ContentType::JSON)
			.ok()
	}
}
