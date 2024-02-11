use rocket_db_pools::{
	diesel::prelude::*,
	Connection,
};
use serde::{
	Deserialize,
	Serialize,
};
use tracing::{
	error,
	trace,
};

use super::{
	tracking::{
		CreateTracking as CreateTrackingDB,
		Tracking as TrackingDB,
		UpdateTracking as UpdateTrackingDB,
	},
	tracking_to_activity::{
		CreateTrackingToActivity as CreateTrackingToActivityDB,
		TrackingToActivity as TrackingToActivityDB,
	},
};
use crate::{
	db::{
		activity::Activity,
		PaginationResult,
	},
	DB,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Tracking {
	pub id: i32,
	pub client_id: i32,
	pub user_id: i32,
	pub project_id: i32,
	pub date: chrono::NaiveDate,
	pub begin: chrono::NaiveTime,
	pub end: chrono::NaiveTime,
	pub pause: Option<chrono::NaiveTime>,
	pub performed: f32,
	pub billed: f32,
	pub description: Option<String>,
	pub created_at: chrono::NaiveDateTime,
	pub updated_at: chrono::NaiveDateTime,
	pub activities: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateTracking {
	pub client_id: i32,
	pub user_id: i32,
	pub project_id: i32,
	pub date: chrono::NaiveDate,
	pub begin: chrono::NaiveTime,
	pub end: chrono::NaiveTime,
	pub pause: Option<chrono::NaiveTime>,
	pub performed: f32,
	pub billed: f32,
	pub description: Option<String>,
	pub activities: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct UpdateTracking {
	pub client_id: Option<i32>,
	pub user_id: Option<i32>,
	pub project_id: Option<i32>,
	pub date: Option<chrono::NaiveDate>,
	pub begin: Option<chrono::NaiveTime>,
	pub end: Option<chrono::NaiveTime>,
	pub pause: Option<Option<chrono::NaiveTime>>,
	pub performed: Option<f32>,
	pub billed: Option<f32>,
	pub description: Option<String>,
	pub created_at: Option<chrono::NaiveDateTime>,
	pub updated_at: Option<chrono::NaiveDateTime>,
	pub activities: Option<Vec<i32>>,
}

impl Tracking {
	pub async fn create(
		db: &mut Connection<DB>,
		tracking: &CreateTracking,
	) -> Result<Tracking, diesel::result::Error> {
		trace!("Tracking middle layer: create");

		let tracking_db = CreateTrackingDB {
			client_id: tracking.client_id,
			user_id: tracking.user_id,
			project_id: tracking.project_id,
			date: tracking.date,
			begin: tracking.begin,
			end: tracking.end,
			pause: tracking.pause,
			performed: tracking.performed,
			billed: tracking.billed,
			description: tracking.description.to_owned(),
		};
		trace!("Creating Tracking");
		let tracking_db = TrackingDB::create(db, &tracking_db).await.map_err(|e| {
			error!("Error creating tracking: {:#?}", e);
			e
		})?;
		trace!("Iterating over activities to create tracking_to_activity");
		for i in tracking.activities.clone() {
			let tracking_to_activity = CreateTrackingToActivityDB {
				tracking_id: tracking_db.id,
				activity_id: i,
			};
			TrackingToActivityDB::create(db, &tracking_to_activity)
				.await
				.map_err(|e| {
					error!("Error creating tracking to activity: {:#?}", e);
					e
				})?;
		}
		Ok(Tracking {
			id: tracking_db.id,
			client_id: tracking_db.client_id,
			user_id: tracking_db.user_id,
			project_id: tracking_db.project_id,
			date: tracking_db.date,
			begin: tracking_db.begin,
			end: tracking_db.end,
			pause: tracking_db.pause,
			performed: tracking_db.performed,
			billed: tracking_db.billed,
			description: tracking_db.description,
			created_at: tracking_db.created_at,
			updated_at: tracking_db.updated_at,
			activities: tracking.activities.to_owned(),
		})
	}

	pub async fn read(
		db: &mut Connection<DB>,
		param_id: i32,
	) -> Result<Tracking, diesel::result::Error> {
		trace!("Tracking middle layer: read");
		let tracking_db = TrackingDB::read(db, param_id).await.map_err(|e| {
			error!("Error reading tracking: {:#?}", e);
			e
		})?;
		let activities = TrackingToActivityDB::get_activity_ids(db, param_id)
			.await
			.map_err(|e| {
				error!("Error getting activities: {:#?}", e);
				e
			})?;
		Ok(Tracking {
			id: tracking_db.id,
			client_id: tracking_db.client_id,
			user_id: tracking_db.user_id,
			project_id: tracking_db.project_id,
			date: tracking_db.date,
			begin: tracking_db.begin,
			end: tracking_db.end,
			pause: tracking_db.pause,
			performed: tracking_db.performed,
			billed: tracking_db.billed,
			description: tracking_db.description,
			created_at: tracking_db.created_at,
			updated_at: tracking_db.updated_at,
			activities,
		})
	}

	pub async fn paginate(
		db: &mut Connection<DB>,
		page: i64,
		page_size: i64,
	) -> Result<PaginationResult<Tracking>, diesel::result::Error> {
		trace!("Tracking middle layer: paginate");
		use rocket_db_pools::diesel::prelude::*;

		use crate::schema::activity;

		trace!(
			"Paginating through tracking table: page {}, page_size {}",
			page,
			page_size
		);
		let pagination = TrackingDB::paginate(db, page, page_size)
			.await
			.map_err(|e| {
				error!("Error paginating tracking: {:#?}", e);
				e
			})?;
		let tracking_db = pagination.items;
		trace!("Getting all activities belonging to each tracking");
		let activities = TrackingToActivityDB::belonging_to(&tracking_db)
			.inner_join(activity::table)
			.select((TrackingToActivityDB::as_select(), Activity::as_select()))
			.load(db)
			.await
			.map_err(|e| {
				error!("Error getting activities: {:#?}", e);
				e
			})?;

		let activities_per_tracking: Vec<(TrackingDB, Vec<Activity>)> = activities
			.grouped_by(&tracking_db)
			.into_iter()
			.zip(tracking_db)
			.map(|(activities, tracking)| {
				(
					tracking,
					activities
						.into_iter()
						.map(|(_, activity)| activity)
						.collect(),
				)
			})
			.collect();

		Ok(PaginationResult {
			items: activities_per_tracking
				.into_iter()
				.map(|(tracking, activities)| Self::new(tracking, activities))
				.collect(),
			total_items: pagination.total_items,
			page: pagination.page,
			page_size: pagination.page_size,
			num_pages: pagination.num_pages,
		})
	}

	fn new(tracking_db: TrackingDB, activities: Vec<Activity>) -> Tracking {
		Tracking {
			id: tracking_db.id,
			client_id: tracking_db.client_id,
			user_id: tracking_db.user_id,
			project_id: tracking_db.project_id,
			date: tracking_db.date,
			begin: tracking_db.begin,
			end: tracking_db.end,
			pause: tracking_db.pause,
			performed: tracking_db.performed,
			billed: tracking_db.billed,
			description: tracking_db.description,
			created_at: tracking_db.created_at,
			updated_at: tracking_db.updated_at,
			activities: activities.into_iter().map(|activity| activity.id).collect(),
		}
	}

	fn from_tracking(tracking_db: TrackingDB) -> Tracking {
		Tracking {
			id: tracking_db.id,
			client_id: tracking_db.client_id,
			user_id: tracking_db.user_id,
			project_id: tracking_db.project_id,
			date: tracking_db.date,
			begin: tracking_db.begin,
			end: tracking_db.end,
			pause: tracking_db.pause,
			performed: tracking_db.performed,
			billed: tracking_db.billed,
			description: tracking_db.description,
			created_at: tracking_db.created_at,
			updated_at: tracking_db.updated_at,
			activities: vec![],
		}
	}

	pub async fn update(
		db: &mut Connection<DB>,
		param_id: i32,
		tracking: &UpdateTracking,
	) -> Result<Tracking, diesel::result::Error> {
		trace!("Tracking middle layer: update");

		let tracking_db = UpdateTrackingDB {
			client_id: tracking.client_id,
			user_id: tracking.user_id,
			project_id: tracking.project_id,
			date: tracking.date,
			begin: tracking.begin,
			end: tracking.end,
			pause: tracking.pause,
			performed: tracking.performed,
			billed: tracking.billed,
			description: tracking.description.to_owned(),
			created_at: tracking.created_at,
			updated_at: tracking.updated_at,
		};
		let mut tracking_update;
		let default_update = UpdateTrackingDB::default();
		if tracking_db == default_update {
			trace!("No update needed for tracking");
			let tracking_db = TrackingDB::read(db, param_id).await.map_err(|e| {
				error!("Error reading tracking: {:#?}", e);
				e
			})?;
			tracking_update = Self::from_tracking(tracking_db);
		} else {
			let tracking_db = TrackingDB::update(db, param_id, &tracking_db)
				.await
				.map_err(|e| {
					error!("Error updating tracking: {:#?}", e);
					e
				})?;
			tracking_update = Self::from_tracking(tracking_db);
		}
		if tracking.activities.is_some() {
			let activities = tracking.activities.clone().unwrap();
			// just drop the old ones and add the new ones
			TrackingToActivityDB::delete_by_tracking_id(db, param_id)
				.await
				.map_err(|e| {
					error!("Error deleting tracking to activity: {:#?}", e);
					e
				})?;
			for i in activities {
				let tracking_to_activity = CreateTrackingToActivityDB {
					tracking_id: param_id,
					activity_id: i,
				};
				TrackingToActivityDB::create(db, &tracking_to_activity)
					.await
					.map_err(|e| {
						error!("Error creating tracking to activity: {:#?}", e);
						e
					})?;
			}
			tracking_update.activities = tracking.activities.to_owned().unwrap();
		}
		Ok(tracking_update)
	}

	pub async fn delete(db: &mut Connection<DB>, param_id: i32) -> QueryResult<usize> {
		trace!("Tracking middle layer: delete");
		trace!(
			"Tracking middle layer tracking to activity delete by tracking id {}",
			param_id
		);
		TrackingToActivityDB::delete_by_tracking_id(db, param_id).await?;
		trace!("Tracking middle layer tracking delete by id {}", param_id);
		TrackingDB::delete(db, param_id).await
	}

	pub async fn last_page(db: &mut Connection<DB>, page_size: i64) -> QueryResult<i64> {
		trace!("Tracking middle layer: last_page");
		TrackingDB::last_page(db, page_size).await
	}
}

#[cfg(test)]
impl PartialEq<CreateTracking> for Tracking {
	fn eq(&self, other: &CreateTracking) -> bool {
		self.client_id == other.client_id
			&& self.user_id == other.user_id
			&& self.project_id == other.project_id
			&& self.date == other.date
			&& self.begin == other.begin
			&& self.end == other.end
			&& self.pause == other.pause
			&& self.performed == other.performed
			&& self.billed == other.billed
			&& self.description == other.description
			&& self.activities == other.activities
	}
}
