use argon2::password_hash::{
	PasswordHash,
	PasswordVerifier,
};
#[cfg(test)]
use fake::{
	faker::internet::en::*,
	faker::name::en::*,
	Dummy,
};
use rocket_db_pools::{
	diesel::{
		insert_into,
		prelude::*,
	},
	Connection,
};
use serde::{
	Deserialize,
	Serialize,
};

use super::{
	last_insert_id,
	PaginationResult,
};
use crate::{
	schema::*,
	Error,
	Result,
	DB,
};

/// Struct representing a row in table `user`
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, QueryableByName)]
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
	#[cfg_attr(test, dummy(expr = "\"User_01!\".into()"))]
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

impl User {
	/// Gets the Hash from the database where email matches,
	/// hashes the password and compares newly generated hash
	/// with hash from the database
	pub async fn check_credentials(
		db: &mut Connection<DB>,
		email: &str,
		password: &str,
	) -> Result<Self> {
		use crate::schema::user::dsl;

		trace!("Checking credentials for {}", email);
		let rec = dsl::user
			.filter(dsl::email.eq(email))
			.first::<Self>(db)
			.await?;
		debug!("Found user: {:?}", rec.username);
		let hash = PasswordHash::new(&rec.hash);
		if let Err(err) = hash {
			error!(
				"Hash is invalid.\nuser: {}\nhash: {}\npassword: {}",
				rec.username, rec.hash, password
			);
			return Err(Error::Argon2PasswordHash(err));
		}
		let hash = hash.unwrap();
		match argon2::Argon2::default().verify_password(password.as_bytes(), &hash) {
			Ok(_) => {
				trace!("Logged in as {}", rec.username);
				Ok(rec)
			}
			Err(_) => {
				trace!("Wrong credentials for {}", email);
				Err(Error::WrongCredentials)
			}
		}
	}

	/// Insert a new row into `user` with a given [`CreateUser`]
	pub async fn create(db: &mut Connection<DB>, item: &CreateUser) -> QueryResult<Self> {
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
	pub async fn read(db: &mut Connection<DB>, param_id: i32) -> QueryResult<Self> {
		use crate::schema::user::dsl::*;
		trace!("Reading from user table: {}", param_id);
		user.filter(id.eq(param_id)).first::<Self>(db).await
	}

	/// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
	pub async fn paginate(
		db: &mut Connection<DB>,
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
		let items: Vec<User> = if page == 0 {
			user.limit(page_size).load::<Self>(db).await?
		} else {
			user
				.limit(page_size)
				.offset(page * page_size)
				.load::<Self>(db)
				.await?
		};
		Ok(PaginationResult {
			items,
			total_items,
			page,
			page_size,
			/* ceiling division of integers */
			num_pages: total_items / page_size + i64::from(total_items % page_size != 0),
		})
	}

	pub async fn last_page(db: &mut Connection<DB>, page_size: i64) -> QueryResult<i64> {
		use crate::schema::user::dsl::*;

		trace!("Getting last page of user table for page_size {page_size}");

		let total_items: i64 = user.count().get_result(db).await?;
		// index starts at 0
		Ok((total_items / page_size + i64::from(total_items % page_size != 0)) - 1)
	}

	/// Update a row in `user`, identified by the primary key with [`UpdateUser`]
	pub async fn update(
		db: &mut Connection<DB>,
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
	pub async fn delete(db: &mut Connection<DB>, param_id: i32) -> QueryResult<usize> {
		use crate::schema::user::dsl::*;

		trace!("Deleting from user table: {}", param_id);
		diesel::delete(user.filter(id.eq(param_id)))
			.execute(db)
			.await
	}
}

#[cfg(test)]
impl PartialEq for User {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
			&& self.username == other.username
			&& self.firstname == other.firstname
			&& self.lastname == other.lastname
			&& self.email == other.email
			&& self.hash == other.hash
			&& self.sys_role == other.sys_role
			&& self.created_at == other.created_at
			&& self.updated_at == other.updated_at
	}
}

#[cfg(test)]
impl PartialEq<CreateUser> for User {
	fn eq(&self, other: &CreateUser) -> bool {
		self.username == other.username
			&& self.firstname == other.firstname
			&& self.lastname == other.lastname
			&& self.email == other.email
			&& self.sys_role == other.sys_role
	}
}
