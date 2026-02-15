use axum::{
	extract::Request,
	middleware::Next,
	response::Response,
};

use crate::error::Error;

/// Middleware that checks for the presence of a Bearer token in the Authorization header.
///
/// Returns 401 UNAUTHORIZED if:
/// - No Authorization header is present
/// - Authorization header does not start with "Bearer "
/// - The Bearer token is empty (after trimming)
///
/// // TODO(Phase 3): Validate JWT token and extract claims
pub async fn require_auth(request: Request, next: Next) -> Result<Response, Error> {
	let auth_header = request
		.headers()
		.get(axum::http::header::AUTHORIZATION)
		.and_then(|v| v.to_str().ok());

	match auth_header {
		None => return Err(Error::Unauthorized),
		Some(value) => {
			if !value.starts_with("Bearer ") {
				return Err(Error::Unauthorized);
			}
			let token = value["Bearer ".len()..].trim();
			if token.is_empty() {
				return Err(Error::Unauthorized);
			}
		}
	}

	Ok(next.run(request).await)
}
