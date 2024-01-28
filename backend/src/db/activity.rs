#[cfg(test)]
use fake::{
	faker::company::en::*,
	Dummy,
	Fake,
};
use rocket_db_pools::{
	diesel::{
		insert_into,
		prelude::*,
		RunQueryDsl,
	},
	Connection,
};
use tracing::trace;

use super::{
	last_insert_id,
	PaginationResult,
};
use crate::{
	schema::*,
	DB,
};

/// Struct representing a row in table `activity`
#[derive(
	Debug, Clone, serde::Serialize, serde::Deserialize, Queryable, Selectable, QueryableByName,
)]
#[diesel(table_name = activity, primary_key(id))]
#[cfg_attr(test, derive(PartialEq))]
pub struct Activity {
	/// Field representing column `id`
	pub id: i32,
	/// Field representing column `token`
	pub token: Option<String>,
	/// Field representing column `name`
	pub name: String,
	/// Field representing column `created_at`
	pub created_at: chrono::NaiveDateTime,
	/// Field representing column `updated_at`
	pub updated_at: chrono::NaiveDateTime,
}

/// Create Struct for a row in table `activity` for [`Activity`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Insertable)]
#[diesel(table_name = activity)]
#[cfg_attr(test, derive(Dummy))]
pub struct CreateActivity {
	/// Field representing column `token`
	pub token: Option<String>,
	/// Field representing column `name`
	#[cfg_attr(
		test,
		dummy(
			expr = "BsVerb().fake::<String>() + \" \" + &BsAdj().fake::<String>() + \" \" + \
			        &BsNoun().fake::<String>()"
		)
	)]
	pub name: String,
}

/// Update Struct for a row in table `activity` for [`Activity`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, AsChangeset, PartialEq, Default)]
#[diesel(table_name = activity)]
pub struct UpdateActivity {
	/// Field representing column `token`
	pub token: Option<Option<String>>,
	/// Field representing column `name`
	pub name: Option<String>,
	/// Field representing column `created_at`
	pub created_at: Option<chrono::NaiveDateTime>,
	/// Field representing column `updated_at`
	pub updated_at: Option<chrono::NaiveDateTime>,
}

impl Activity {
	/// Insert a new row into `activity` with a given [`CreateActivity`]
	pub async fn create(db: &mut Connection<DB>, item: &CreateActivity) -> QueryResult<Self> {
		use crate::schema::activity::dsl::*;

		trace!("Inserting into activity table: {:?}", item);
		db.transaction(|mut conn| {
			Box::pin(async move {
				insert_into(activity)
					.values(item)
					.execute(&mut conn)
					.await?;
				activity
					.select(Activity::as_select())
					.filter(id.eq(last_insert_id()))
					.first::<Self>(&mut conn)
					.await
			})
		})
		.await
	}

	/// Get a row from `activity`, identified by the primary key
	pub async fn read(db: &mut Connection<DB>, param_id: i32) -> QueryResult<Self> {
		use crate::schema::activity::dsl::*;

		trace!("Reading from activity table: {}", param_id);
		activity.filter(id.eq(param_id)).first::<Self>(db).await
	}

	/// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
	pub async fn paginate(
		db: &mut Connection<DB>,
		page: i64,
		page_size: i64,
	) -> QueryResult<PaginationResult<Self>> {
		use crate::schema::activity::dsl::*;

		trace!(
			"Paginating through activity table: page {}, page_size {}",
			page,
			page_size
		);
		let page_size = if page_size < 1 { 1 } else { page_size };
		let total_items = activity.count().get_result(db).await?;
		let items = activity
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

	/// Update a row in `activity`, identified by the primary key with [`UpdateActivity`]
	pub async fn update(
		db: &mut Connection<DB>,
		param_id: i32,
		item: &UpdateActivity,
	) -> QueryResult<Self> {
		use crate::schema::activity::dsl::*;

		trace!("Updating activity table: {} with {:?}", param_id, item);
		db.transaction(|mut conn| {
			Box::pin(async move {
				diesel::update(activity.filter(id.eq(param_id)))
					.set(item)
					.execute(&mut conn)
					.await?;
				activity
					.filter(id.eq(param_id))
					.first::<Self>(&mut conn)
					.await
			})
		})
		.await
	}

	/// Delete a row in `activity`, identified by the primary key
	pub async fn delete(db: &mut Connection<DB>, param_id: i32) -> QueryResult<usize> {
		use crate::schema::activity::dsl::*;

		trace!("Deleting from activity table: {}", param_id);
		diesel::delete(activity.filter(id.eq(param_id)))
			.execute(db)
			.await
	}

	pub async fn last_page(db: &mut Connection<DB>, page_size: i64) -> QueryResult<i64> {
		use crate::schema::activity::dsl::*;

		trace!("Getting last page of activity table for page_size {page_size}");

		let total_items: i64 = activity.count().get_result(db).await?;
		// index starts at 0
		Ok((total_items / page_size + i64::from(total_items % page_size != 0)) - 1)
	}
}

#[cfg(test)]
impl PartialEq<CreateActivity> for Activity {
	fn eq(&self, other: &CreateActivity) -> bool {
		self.name == other.name && self.token == other.token
	}
}
