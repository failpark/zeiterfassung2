/* This file is generated and managed by dsync */

use crate::diesel::*;
use crate::schema::*;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use diesel_async::RunQueryDsl;


type Connection = rocket_db_pools::Connection<crate::DB>;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[diesel(table_name=client, primary_key(id))]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=client)]
pub struct CreateClient {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name=client)]
pub struct UpdateClient {
    pub name: Option<String>,
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

impl Client {

    pub async fn create(db: &mut Connection, item: &CreateClient) -> QueryResult<Self> {
        use crate::schema::client::dsl::*;

        insert_into(client).values(item).get_result::<Self>(db).await
    }

    pub async fn read(db: &mut Connection, param_id: i32) -> QueryResult<Self> {
        use crate::schema::client::dsl::*;

        client.filter(id.eq(param_id)).first::<Self>(db).await
    }

    /// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
    pub async fn paginate(db: &mut Connection, page: i64, page_size: i64) -> QueryResult<PaginationResult<Self>> {
        use crate::schema::client::dsl::*;

        let page_size = if page_size < 1 { 1 } else { page_size };
        let total_items = client.count().get_result(db).await?;
        let items = client.limit(page_size).offset(page * page_size).load::<Self>(db).await?;

        Ok(PaginationResult {
            items,
            total_items,
            page,
            page_size,
            /* ceiling division of integers */
            num_pages: total_items / page_size + i64::from(total_items % page_size != 0)
        })
    }

    pub async fn update(db: &mut Connection, param_id: i32, item: &UpdateClient) -> QueryResult<Self> {
        use crate::schema::client::dsl::*;

        diesel::update(client.filter(id.eq(param_id))).set(item).get_result(db).await
    }

    pub async fn delete(db: &mut Connection, param_id: i32) -> QueryResult<usize> {
        use crate::schema::client::dsl::*;

        diesel::delete(client.filter(id.eq(param_id))).execute(db).await
    }

}