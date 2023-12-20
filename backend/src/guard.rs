use rocket::{
	async_trait,
	http::Status,
	outcome::{
		try_outcome,
		IntoOutcome,
	},
	request::{
		FromRequest,
		Outcome,
		Request,
	},
	State,
};

use crate::{
	auth::Tokenizer,
	db::user::User,
	error::Error,
};

#[async_trait]
impl<'r> FromRequest<'r> for User {
	type Error = Error;

	async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
		let tokenizer = try_outcome!(req
			.guard::<&State<Tokenizer>>()
			.await
			.map_error(|_| { (Status::Unauthorized, Error::UnauthenticatedUser) }));

		let token: Result<User, Error> = req
			.headers()
			.get_one("Authorization")
			.map(|header| header.split("Bearer").collect::<Vec<_>>())
			.ok_or(Error::UnauthenticatedUser)
			.and_then(|bearer| {
				let token = bearer
					.as_slice()
					.get(1)
					.map(|token| token.trim())
					.unwrap_or_default();
				tokenizer.verify(token)
			});

		match token {
			Ok(user) => Outcome::Success(user),
			Err(err) => Outcome::Error((Status::Unauthorized, err)),
		}
	}
}
