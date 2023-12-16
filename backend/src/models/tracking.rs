use rocket_db_pools::diesel::{
	insert_into,
	prelude::*,
};

use super::{
	client::Client,
	last_insert_id,
	project::Project,
	user::User,
};
use crate::schema::*;

pub type ConnectionType = rocket_db_pools::Connection<crate::DB>;

/// Struct representing a row in table `tracking`
#[derive(
	Debug,
	Clone,
	serde::Serialize,
	serde::Deserialize,
	Queryable,
	Selectable,
	QueryableByName,
	Associations,
	Identifiable,
)]
#[diesel(table_name=tracking, primary_key(id), belongs_to(Client, foreign_key=client_id) , belongs_to(Project, foreign_key=project_id) , belongs_to(User, foreign_key=user_id))]
pub struct Tracking {
	/// Field representing column `id`
	pub id: i32,
	/// Field representing column `client_id`
	pub client_id: i32,
	/// Field representing column `user_id`
	pub user_id: i32,
	/// Field representing column `project_id`
	pub project_id: i32,
	/// Field representing column `date`
	pub date: chrono::NaiveDate,
	/// Field representing column `begin`
	pub begin: chrono::NaiveTime,
	/// Field representing column `end`
	pub end: chrono::NaiveTime,
	/// Field representing column `pause`
	pub pause: Option<chrono::NaiveTime>,
	/// Field representing column `performed`
	pub performed: f32,
	/// Field representing column `billed`
	pub billed: f32,
	/// Field representing column `description`
	pub description: Option<String>,
	/// Field representing column `created_at`
	pub created_at: chrono::NaiveDateTime,
	/// Field representing column `updated_at`
	pub updated_at: chrono::NaiveDateTime,
}

/// Create Struct for a row in table `tracking` for [`Tracking`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Insertable)]
#[diesel(table_name=tracking)]
pub struct CreateTracking {
	/// Field representing column `id`
	pub id: i32,
	/// Field representing column `client_id`
	pub client_id: i32,
	/// Field representing column `user_id`
	pub user_id: i32,
	/// Field representing column `project_id`
	pub project_id: i32,
	/// Field representing column `date`
	pub date: chrono::NaiveDate,
	/// Field representing column `begin`
	pub begin: chrono::NaiveTime,
	/// Field representing column `end`
	pub end: chrono::NaiveTime,
	/// Field representing column `pause`
	pub pause: Option<chrono::NaiveTime>,
	/// Field representing column `performed`
	pub performed: f32,
	/// Field representing column `billed`
	pub billed: f32,
	/// Field representing column `description`
	pub description: Option<String>,
}

/// Update Struct for a row in table `tracking` for [`Tracking`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, AsChangeset, PartialEq, Default)]
#[diesel(table_name=tracking)]
pub struct UpdateTracking {
	/// Field representing column `client_id`
	pub client_id: Option<i32>,
	/// Field representing column `user_id`
	pub user_id: Option<i32>,
	/// Field representing column `project_id`
	pub project_id: Option<i32>,
	/// Field representing column `date`
	pub date: Option<chrono::NaiveDate>,
	/// Field representing column `begin`
	pub begin: Option<chrono::NaiveTime>,
	/// Field representing column `end`
	pub end: Option<chrono::NaiveTime>,
	/// Field representing column `pause`
	pub pause: Option<Option<chrono::NaiveTime>>,
	/// Field representing column `performed`
	pub performed: Option<f32>,
	/// Field representing column `billed`
	pub billed: Option<f32>,
	/// Field representing column `description`
	pub description: Option<Option<String>>,
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

impl Tracking {
	/// Insert a new row into `tracking` with a given [`CreateTracking`]
	pub async fn create(db: &mut ConnectionType, item: &CreateTracking) -> QueryResult<Self> {
		use crate::schema::tracking::dsl::*;

		trace!("Inserting into tracking table: {:?}", item);
		db.transaction(|mut conn| {
			Box::pin(async move {
				insert_into(tracking)
					.values(item)
					.execute(&mut conn)
					.await?;
				tracking
					.select(Tracking::as_select())
					.filter(id.eq(last_insert_id()))
					.first::<Self>(&mut conn)
					.await
			})
		})
		.await
	}

	/// Get a row from `tracking`, identified by the primary key
	pub async fn read(db: &mut ConnectionType, param_id: i32) -> QueryResult<Self> {
		use crate::schema::tracking::dsl::*;

		trace!("Reading from tracking table: {}", param_id);
		tracking.filter(id.eq(param_id)).first::<Self>(db).await
	}

	/// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
	pub async fn paginate(
		db: &mut ConnectionType,
		page: i64,
		page_size: i64,
	) -> QueryResult<PaginationResult<Self>> {
		use crate::schema::tracking::dsl::*;

		trace!(
			"Paginating through tracking table: page {}, page_size {}",
			page,
			page_size
		);
		let page_size = if page_size < 1 { 1 } else { page_size };
		let total_items = tracking.count().get_result(db).await?;
		let items = tracking
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

	/// Update a row in `tracking`, identified by the primary key with [`UpdateTracking`]
	pub async fn update(
		db: &mut ConnectionType,
		param_id: i32,
		item: &UpdateTracking,
	) -> QueryResult<Self> {
		use crate::schema::tracking::dsl::*;

		trace!("Updating tracking table: {} with {:?}", param_id, item);
		db.transaction(|mut conn| {
			Box::pin(async move {
				diesel::update(tracking.filter(id.eq(param_id)))
					.set(item)
					.execute(&mut conn)
					.await?;
				tracking
					.filter(id.eq(param_id))
					.first::<Self>(&mut conn)
					.await
			})
		})
		.await
	}

	/// Delete a row in `tracking`, identified by the primary key
	pub async fn delete(db: &mut ConnectionType, param_id: i32) -> QueryResult<usize> {
		use crate::schema::tracking::dsl::*;

		trace!("Deleting from tracking table: {}", param_id);
		diesel::delete(tracking.filter(id.eq(param_id)))
			.execute(db)
			.await
	}
}
