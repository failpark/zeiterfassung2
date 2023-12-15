// use crate::Error;
use jwt_simple::prelude::*;

use crate::models::user::User;

/// This struct is a Singelton that is initialized on startup
/// ```rust
/// rocket::build()
///   .manage(auth::Tokenizer::new(std::time::Duration::new(
///      5 * 24 * 60 * 60,
///      0,
///   )));
/// ```
/// It uses the Ed25519 algorithm to sign & encrypt the Mitarbeiter struct
/// it is the EdDSA signature scheme using SHA-512 and Curve25519
///
/// On every startup the key is newly generated.
/// THIS IS BY DESIGN
/// We want tokens that expire after `Duration` _OR_ after the server is restarted
pub struct Tokenizer {
	key_pair: Ed25519KeyPair,
	pub_key: Ed25519PublicKey,
	exp: Duration,
}

impl Tokenizer {
	/// Returns `Self` aftre generating a new EdDSA keypair
	pub fn new(exp: impl Into<Duration>) -> Self {
		let key_pair = Ed25519KeyPair::generate();
		let pub_key: Ed25519PublicKey = key_pair.public_key();
		Self {
			key_pair,
			pub_key,
			exp: exp.into(),
		}
	}

	/// Signs the Claim and returns `Ok` with the Token if the Token could be Signed
	/// Returns `Err` with own JWTSign Error if an error occured while signing
	/// `Error::JWTSign` is only thrown here. We don't have to care what exactly
	/// is thrown here since JWTSign points here
	pub fn generate(&self, user: User) -> anyhow::Result<String> {
		let claims = Claims::with_custom_claims(user, self.exp);

		match self.key_pair.sign(claims) {
			Ok(signed_token) => Ok(signed_token),
			Err(err) => Err(err),
		}
	}

	/// Verify the Token
	/// Returns `Ok` if the Token could be verified
	/// `Err` if Token is invalid
	pub fn verify(&self, token: &str) -> anyhow::Result<User> {
		Ok(self.pub_key.verify_token::<User>(token, None)?.custom)
	}
}
