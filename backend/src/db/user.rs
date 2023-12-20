use argon2::password_hash::{
	rand_core::OsRng,
	PasswordHash,
	PasswordHasher,
	PasswordVerifier,
};
#[cfg(test)]
use fake::{
	faker::internet::en::*,
	faker::name::en::*,
	Dummy,
};
use rocket_db_pools::diesel::{
	insert_into,
	prelude::*,
};

use super::last_insert_id;
use crate::schema::*;

pub type ConnectionType = rocket_db_pools::Connection<crate::DB>;

/// Struct representing a row in table `user`
#[derive(
	Debug, Clone, serde::Serialize, serde::Deserialize, Queryable, Selectable, QueryableByName,
)]
#[diesel(table_name=user, primary_key(id))]
pub struct User {
	/// Field representing column `id`
	pub id: i32,
	/// Field representing column `username`
	pub username: String,
	/// Field representing column `firstname`
	pub firstname: String,
	/// Field representing column `lastname`
	pub lastname: String,
	/// Field representing column `email`
	pub email: String,
	/// Field representing column `hash`
	pub hash: String,
	/// Field representing column `sys_role`
	pub sys_role: String,
	/// Field representing column `created_at`
	pub created_at: chrono::NaiveDateTime,
	/// Field representing column `updated_at`
	pub updated_at: chrono::NaiveDateTime,
}

/// Create Struct for a row in table `user` for [`User`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Insertable)]
#[cfg_attr(test, derive(Dummy))]
#[diesel(table_name=user)]
pub struct CreateUser {
	/// Field representing column `username`
	#[cfg_attr(test, dummy(faker = "Username()"))]
	pub username: String,
	/// Field representing column `firstname`
	#[cfg_attr(test, dummy(faker = "FirstName()"))]
	pub firstname: String,
	/// Field representing column `lastname`
	#[cfg_attr(test, dummy(faker = "LastName()"))]
	pub lastname: String,
	/// Field representing column `email`
	#[cfg_attr(test, dummy(faker = "SafeEmail()"))]
	pub email: String,
	/// Field representing column `hash`
	#[cfg_attr(test, dummy(expr = "\"REPLACE ME\".into()"))]
	pub hash: String,
	/// Field representing column `sys_role`
	#[cfg_attr(test, dummy(expr = "\"user\".into()"))]
	pub sys_role: String,
}

/// Update Struct for a row in table `user` for [`User`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, AsChangeset, PartialEq, Default)]
#[diesel(table_name=user)]
pub struct UpdateUser {
	/// Field representing column `username`
	pub username: Option<String>,
	/// Field representing column `firstname`
	pub firstname: Option<String>,
	/// Field representing column `lastname`
	pub lastname: Option<String>,
	/// Field representing column `email`
	pub email: Option<String>,
	/// Field representing column `hash`
	pub hash: Option<String>,
	/// Field representing column `sys_role`
	pub sys_role: Option<String>,
	/// Field representing column `created_at`
	pub created_at: Option<chrono::NaiveDateTime>,
	/// Field representing column `updated_at`
	pub updated_at: Option<chrono::NaiveDateTime>,
}

/// Result of a `.paginate` function
#[derive(Debug, serde::Serialize)]
pub struct PaginationResult<T> {
	/// Resulting items that are from the current page
	pub items: Vec<T>,
	/// The count of total items there are
	pub total_items: i64,
	/// Current page, 0-based index
	pub page: i64,
	/// Size of a page
	pub page_size: i64,
	/// Number of total possible pages, given the `page_size` and `total_items`
	pub num_pages: i64,
}

impl User {
	/// Gets the Hash from the database where email matches,
	/// hashes the password and compares newly generated hash
	/// with hash from the database
	pub async fn check_credentials(
		db: &mut ConnectionType,
		email: &str,
		password: &str,
	) -> anyhow::Result<Self> {
		use crate::schema::user::dsl;

		trace!("Checking credentials for {}", email);
		let rec = dsl::user
			.filter(dsl::email.eq(email))
			.first::<Self>(db)
			.await?;
		debug!("Found user: {:?}", rec.username);
		let hash = PasswordHash::new(&rec.hash);
		if hash.is_err() {
			error!(
				"Hash is invalid.\nuser: {}\nhash: {}\npassword: {}",
				rec.username, rec.hash, password
			);
			return Err(anyhow::Error::msg("Hash is invalid"));
		}
		let hash = hash.unwrap();
		match argon2::Argon2::default().verify_password(password.as_bytes(), &hash) {
			Ok(_) => {
				trace!("Logged in as {}", rec.username);
				Ok(rec)
			}
			Err(_) => {
				trace!("Wrong credentials for {}", email);
				Err(anyhow::Error::msg("Wrong credentials"))
			}
		}
	}

	fn hash_password(password: &[u8]) -> Result<String, argon2::password_hash::Error> {
		let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
		trace!("Hashing password");
		Ok(
			argon2::Argon2::default()
				.hash_password(password, &salt)?
				.to_string(),
		)
	}

	/// Insert a new row into `user` with a given [`CreateUser`]
	pub async fn create(db: &mut ConnectionType, item: &CreateUser) -> QueryResult<Self> {
		use crate::schema::user::dsl::*;

		trace!("Inserting into user table: {:?}", item);
		db.transaction(|mut conn| {
			Box::pin(async move {
				insert_into(user).values(item).execute(&mut conn).await?;
				user
					.select(User::as_select())
					.filter(id.eq(last_insert_id()))
					.first::<Self>(&mut conn)
					.await
			})
		})
		.await
	}

	/// Get a row from `user`, identified by the primary key
	pub async fn read(db: &mut ConnectionType, param_id: i32) -> QueryResult<Self> {
		use crate::schema::user::dsl::*;
		trace!("Reading from user table: {}", param_id);
		user.filter(id.eq(param_id)).first::<Self>(db).await
	}

	/// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
	pub async fn paginate(
		db: &mut ConnectionType,
		page: i64,
		page_size: i64,
	) -> QueryResult<PaginationResult<Self>> {
		use crate::schema::user::dsl::*;

		trace!(
			"Paginating through user table: page {}, page_size {}",
			page,
			page_size
		);
		let page_size = if page_size < 1 { 1 } else { page_size };
		let total_items = user.count().get_result(db).await?;
		let items = user
			.limit(page_size)
			.offset(page * page_size)
			.load::<Self>(db)
			.await?;

		Ok(PaginationResult {
			items,
			total_items,
			page,
			page_size,
			/* ceiling division of integers */
			num_pages: total_items / page_size + i64::from(total_items % page_size != 0),
		})
	}

	/// Update a row in `user`, identified by the primary key with [`UpdateUser`]
	pub async fn update(
		db: &mut ConnectionType,
		param_id: i32,
		item: &UpdateUser,
	) -> QueryResult<Self> {
		use crate::schema::user::dsl::*;

		trace!("Updating user: {} with {:?}", param_id, item);
		db.transaction(|mut conn| {
			Box::pin(async move {
				diesel::update(user.filter(id.eq(param_id)))
					.set(item)
					.execute(&mut conn)
					.await?;
				user.filter(id.eq(param_id)).first::<Self>(&mut conn).await
			})
		})
		.await
	}

	/// Delete a row in `user`, identified by the primary key
	pub async fn delete(db: &mut ConnectionType, param_id: i32) -> QueryResult<usize> {
		use crate::schema::user::dsl::*;

		trace!("Deleting from user table: {}", param_id);
		diesel::delete(user.filter(id.eq(param_id)))
			.execute(db)
			.await
	}
}
