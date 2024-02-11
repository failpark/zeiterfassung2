use rocket_db_pools::{
	diesel::{
		insert_into,
		prelude::*,
	},
	Connection,
};
use tracing::trace;

use super::tracking::Tracking;
use crate::{
	db::{
		activity::Activity,
		last_insert_id,
		PaginationResult,
	},
	schema::*,
	DB,
};

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

impl TrackingToActivity {
	/// Insert a new row into `tracking_to_activity` with a given [`CreateTrackingToActivity`]
	pub async fn create(
		db: &mut Connection<DB>,
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
	pub async fn read(db: &mut Connection<DB>, param_id: i32) -> QueryResult<Self> {
		use crate::schema::tracking_to_activity::dsl::*;

		trace!("Reading from tracking_to_activity table: {:?}", param_id);
		tracking_to_activity
			.filter(id.eq(param_id))
			.first::<Self>(db)
			.await
	}

	/// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
	pub async fn paginate(
		db: &mut Connection<DB>,
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
		db: &mut Connection<DB>,
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
	pub async fn delete(db: &mut Connection<DB>, param_id: i32) -> QueryResult<usize> {
		use crate::schema::tracking_to_activity::dsl::*;

		trace!("Deleting from tracking_to_activity table: {}", param_id);
		diesel::delete(tracking_to_activity.filter(id.eq(param_id)))
			.execute(db)
			.await
	}

	pub async fn delete_by_tracking_id(db: &mut Connection<DB>, param_id: i32) -> QueryResult<usize> {
		use crate::schema::tracking_to_activity::dsl::*;

		trace!(
			"Deleting from tracking_to_activity table with tracking_id: {}",
			param_id
		);
		diesel::delete(tracking_to_activity.filter(tracking_id.eq(param_id)))
			.execute(db)
			.await
	}

	pub async fn from_tracking(db: &mut Connection<DB>, param_id: i32) -> QueryResult<Vec<Self>> {
		use crate::schema::tracking_to_activity::dsl::*;

		trace!("Reading from tracking_to_activity table: {:?}", param_id);
		tracking_to_activity
			.filter(tracking_id.eq(param_id))
			.load::<Self>(db)
			.await
	}

	pub async fn get_activity_ids(db: &mut Connection<DB>, param_id: i32) -> QueryResult<Vec<i32>> {
		let activities = Self::from_tracking(db, param_id).await?;
		let mut result = Vec::new();
		for i in activities {
			result.push(i.activity_id);
		}
		Ok(result)
	}

	pub async fn get_activities(
		db: &mut Connection<DB>,
		param_id: i32,
	) -> QueryResult<Vec<Activity>> {
		let activities = Self::from_tracking(db, param_id).await?;
		let mut result = Vec::new();
		for i in activities {
			result.push(Activity::read(db, i.activity_id).await?);
		}
		Ok(result)
	}
}
