/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use diesel_async::RunQueryDsl;
use crate::models::activity::Activity;
use crate::models::tracking::Tracking;

type Connection = rocket_db_pools::Connection<crate::DB>;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=tracking_to_activity, primary_key(id), belongs_to(Activity, foreign_key=activity_id) , belongs_to(Tracking, foreign_key=tracking_id))]
pub struct TrackingToActivity {
    pub id: i32,
    pub tracking_id: i32,
    pub activity_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=tracking_to_activity)]
pub struct CreateTrackingToActivity {
    pub id: i32,
    pub tracking_id: i32,
    pub activity_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=tracking_to_activity)]
pub struct UpdateTrackingToActivity {
    pub tracking_id: Option<i32>,
    pub activity_id: Option<i32>,
}


#[derive(Debug, Serialize)]
pub struct PaginationResult<T> {
    pub items: Vec<T>,
    pub total_items: i64,
    /// 0-based index
    pub page: i64,
    pub page_size: i64,
    pub num_pages: i64,
}

impl TrackingToActivity {

    pub async fn create(db: &mut Connection, item: &CreateTrackingToActivity) -> QueryResult<Self> {
        use crate::schema::tracking_to_activity::dsl::*;

        insert_into(tracking_to_activity).values(item).get_result::<Self>(db).await
    }

    pub async fn read(db: &mut Connection, param_id: i32) -> QueryResult<Self> {
        use crate::schema::tracking_to_activity::dsl::*;

        tracking_to_activity.filter(id.eq(param_id)).first::<Self>(db).await
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub async fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::tracking_to_activity::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = tracking_to_activity.count().get_result(db).await?;
        let items = tracking_to_activity.limit(page_size).offset(page * page_size).load::<Self>(db).await?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub async fn update(db: &mut Connection, param_id: i32, item: &UpdateTrackingToActivity) -> QueryResult<Self> {
        use crate::schema::tracking_to_activity::dsl::*;

        diesel::update(tracking_to_activity.filter(id.eq(param_id))).set(item).get_result(db).await
    }

    pub async fn delete(db: &mut Connection, param_id: i32) -> QueryResult<usize> {
        use crate::schema::tracking_to_activity::dsl::*;

        diesel::delete(tracking_to_activity.filter(id.eq(param_id))).execute(db).await
    }

}