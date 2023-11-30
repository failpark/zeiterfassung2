/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use diesel_async::RunQueryDsl;
use crate::models::client::Client;
use crate::models::project::Project;
use crate::models::user::User;

type Connection = rocket_db_pools::Connection<crate::DB>;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Identifiable, Associations, Selectable)]
#[diesel(table_name=tracking, primary_key(id), belongs_to(Client, foreign_key=client_id) , belongs_to(Project, foreign_key=project_id) , belongs_to(User, foreign_key=user_id))]
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
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=tracking)]
pub struct CreateTracking {
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
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=tracking)]
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
    pub description: Option<Option<String>>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
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

impl Tracking {

    pub async fn create(db: &mut Connection, item: &CreateTracking) -> QueryResult<Self> {
        use crate::schema::tracking::dsl::*;

        insert_into(tracking).values(item).get_result::<Self>(db).await
    }

    pub async fn read(db: &mut Connection, param_id: i32) -> QueryResult<Self> {
        use crate::schema::tracking::dsl::*;

        tracking.filter(id.eq(param_id)).first::<Self>(db).await
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub async fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::tracking::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = tracking.count().get_result(db).await?;
        let items = tracking.limit(page_size).offset(page * page_size).load::<Self>(db).await?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub async fn update(db: &mut Connection, param_id: i32, item: &UpdateTracking) -> QueryResult<Self> {
        use crate::schema::tracking::dsl::*;

        diesel::update(tracking.filter(id.eq(param_id))).set(item).get_result(db).await
    }

    pub async fn delete(db: &mut Connection, param_id: i32) -> QueryResult<usize> {
        use crate::schema::tracking::dsl::*;

        diesel::delete(tracking.filter(id.eq(param_id))).execute(db).await
    }

}