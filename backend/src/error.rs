use axum::{
	http::StatusCode,
	response::{
		IntoResponse,
		Response,
	},
	Json,
};
use serde::Serialize;
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
	Serde(#[from] serde_json::Error),
	#[error("Database Primary Key Error: {0}")]
	TryFromInt(#[from] std::num::TryFromIntError),
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
	#[error("Validation Error")]
	Validation(Vec<FieldError>),
}

impl From<argon2::password_hash::Error> for Error {
	fn from(e: argon2::password_hash::Error) -> Self {
		Error::Argon2PasswordHash(e)
	}
}

#[derive(Serialize, Debug, Clone)]
pub struct FieldError {
	pub field: String,
	pub reason: String,
}

#[derive(Serialize, Debug)]
pub struct ErrorBody {
	pub code: &'static str,
	pub message: String,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub details: Vec<FieldError>,
}

impl IntoResponse for Error {
	fn into_response(self) -> Response {
		let (status, code, details) = match &self {
			Error::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", vec![]),
			Error::UnauthenticatedUser | Error::WrongCredentials | Error::Unauthorized => {
				(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", vec![])
			}
			Error::ForbiddenAccess => (StatusCode::FORBIDDEN, "FORBIDDEN", vec![]),
			Error::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", vec![]),
			Error::Validation(field_errors) => (
				StatusCode::UNPROCESSABLE_ENTITY,
				"VALIDATION_ERROR",
				field_errors.clone(),
			),
			_ => {
				tracing::error!(error = ?self, "Internal server error");
				(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", vec![])
			}
		};

		let message = match &self {
			Error::Database(_)
			| Error::Argon2PasswordHash(_)
			| Error::Serde(_)
			| Error::TryFromInt(_)
			| Error::JWTSign(_)
			| Error::JWTVerify(_)
			| Error::JWT(_)
			| Error::ZeiterfassungInsert
			| Error::Unknown
			| Error::Internal => "Internal server error".to_string(),
			_ => self.to_string(),
		};

		let body = ErrorBody {
			code,
			message,
			details,
		};

		(status, Json(body)).into_response()
	}
}
