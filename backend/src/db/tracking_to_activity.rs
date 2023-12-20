use rocket_db_pools::diesel::{
	insert_into,
	prelude::*,
};

use super::{
	activity::Activity,
	last_insert_id,
	tracking::Tracking,
};
use crate::schema::*;

pub type ConnectionType = rocket_db_pools::Connection<crate::DB>;

/// Struct representing a row in table `tracking_to_activity`
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
#[diesel(table_name=tracking_to_activity, primary_key(id), belongs_to(Activity, foreign_key=activity_id) , belongs_to(Tracking, foreign_key=tracking_id))]
pub struct TrackingToActivity {
	/// Field representing column `id`
	pub id: i32,
	/// Field representing column `tracking_id`
	pub tracking_id: i32,
	/// Field representing column `activity_id`
	pub activity_id: i32,
}

/// Create Struct for a row in table `tracking_to_activity` for [`TrackingToActivity`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Insertable)]
#[diesel(table_name=tracking_to_activity)]
pub struct CreateTrackingToActivity {
	/// Field representing column `tracking_id`
	pub tracking_id: i32,
	/// Field representing column `activity_id`
	pub activity_id: i32,
}

/// Update Struct for a row in table `tracking_to_activity` for [`TrackingToActivity`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, AsChangeset, PartialEq, Default)]
#[diesel(table_name=tracking_to_activity)]
pub struct UpdateTrackingToActivity {
	/// Field representing column `tracking_id`
	pub tracking_id: Option<i32>,
	/// Field representing column `activity_id`
	pub activity_id: Option<i32>,
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

impl TrackingToActivity {
	/// Insert a new row into `tracking_to_activity` with a given [`CreateTrackingToActivity`]
	pub async fn create(
		db: &mut ConnectionType,
		item: &CreateTrackingToActivity,
	) -> QueryResult<Self> {
		use crate::schema::tracking_to_activity::dsl::*;

		trace!("Inserting into tracking_to_activity table: {:?}", item);
		db.transaction(|mut conn| {
			Box::pin(async move {
				insert_into(tracking_to_activity)
					.values(item)
					.execute(&mut conn)
					.await?;
				tracking_to_activity
					.select(TrackingToActivity::as_select())
					.filter(id.eq(last_insert_id()))
					.first::<Self>(&mut conn)
					.await
			})
		})
		.await
	}

	/// Get a row from `tracking_to_activity`, identified by the primary key
	pub async fn read(db: &mut ConnectionType, param_id: i32) -> QueryResult<Self> {
		use crate::schema::tracking_to_activity::dsl::*;

		trace!("Reading from tracking_to_activity table: {:?}", param_id);
		tracking_to_activity
			.filter(id.eq(param_id))
			.first::<Self>(db)
			.await
	}

	/// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
	pub async fn paginate(
		db: &mut ConnectionType,
		page: i64,
		page_size: i64,
	) -> QueryResult<PaginationResult<Self>> {
		use crate::schema::tracking_to_activity::dsl::*;

		trace!(
			"Paginating through tracking_to_activity table: page {}, page_size {}",
			page,
			page_size
		);
		let page_size = if page_size < 1 { 1 } else { page_size };
		let total_items = tracking_to_activity.count().get_result(db).await?;
		let items = tracking_to_activity
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

	/// Update a row in `tracking_to_activity`, identified by the primary key with [`UpdateTrackingToActivity`]
	pub async fn update(
		db: &mut ConnectionType,
		param_id: i32,
		item: &UpdateTrackingToActivity,
	) -> QueryResult<Self> {
		use crate::schema::tracking_to_activity::dsl::*;

		trace!(
			"Updating tracking_to_activity table: {} with {:?}",
			param_id,
			item
		);
		db.transaction(|mut conn| {
			Box::pin(async move {
				diesel::update(tracking_to_activity.filter(id.eq(param_id)))
					.set(item)
					.execute(&mut conn)
					.await?;
				tracking_to_activity
					.filter(id.eq(param_id))
					.first::<Self>(&mut conn)
					.await
			})
		})
		.await
	}

	/// Delete a row in `tracking_to_activity`, identified by the primary key
	pub async fn delete(db: &mut ConnectionType, param_id: i32) -> QueryResult<usize> {
		use crate::schema::tracking_to_activity::dsl::*;

		trace!("Deleting from tracking_to_activity table: {}", param_id);
		diesel::delete(tracking_to_activity.filter(id.eq(param_id)))
			.execute(db)
			.await
	}
}
