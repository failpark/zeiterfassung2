use rocket_db_pools::diesel::{
	insert_into,
	prelude::*,
	RunQueryDsl,
};

use super::last_insert_id;
use crate::schema::*;

pub type ConnectionType = rocket_db_pools::Connection<crate::DB>;

/// Struct representing a row in table `client`
#[derive(
	Debug, Clone, serde::Serialize, serde::Deserialize, Queryable, Selectable, QueryableByName,
)]
#[diesel(table_name = client, primary_key(id))]
pub struct Client {
	/// Field representing column `id`
	pub id: i32,
	/// Field representing column `name`
	pub name: String,
	/// Field representing column `created_at`
	pub created_at: chrono::NaiveDateTime,
	/// Field representing column `updated_at`
	pub updated_at: chrono::NaiveDateTime,
}

/// Create Struct for a row in table `client` for [`Client`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Insertable)]
#[diesel(table_name = client)]
pub struct CreateClient {
	/// Field representing column `name`
	pub name: String,
}

/// Update Struct for a row in table `client` for [`Client`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, AsChangeset, PartialEq, Default)]
#[diesel(table_name = client)]
pub struct UpdateClient {
	/// Field representing column `name`
	pub name: Option<String>,
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

impl Client {
	/// Insert a new row into `client` with a given [`CreateClient`]
	pub async fn create(db: &mut ConnectionType, item: &CreateClient) -> QueryResult<Self> {
		use crate::schema::client::dsl::*;

		trace!("Inserting into client table: {:?}", item);
		db.transaction(|mut conn| {
			Box::pin(async move {
				insert_into(client).values(item).execute(&mut conn).await?;
				client
					.select(Client::as_select())
					.filter(id.eq(last_insert_id()))
					.first::<Self>(&mut conn)
					.await
			})
		})
		.await
	}

	/// Get a row from `client`, identified by the primary key
	pub async fn read(db: &mut ConnectionType, param_id: i32) -> QueryResult<Self> {
		use crate::schema::client::dsl::*;

		trace!("Reading from client table: {:?}", param_id);
		client.filter(id.eq(param_id)).first::<Self>(db).await
	}

	/// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
	pub async fn paginate(
		db: &mut ConnectionType,
		page: i64,
		page_size: i64,
	) -> QueryResult<PaginationResult<Self>> {
		use crate::schema::client::dsl::*;

		trace!(
			"Paginating through client table: page {}, page_size {}",
			page,
			page_size
		);
		let page_size = if page_size < 1 { 1 } else { page_size };
		let total_items = client.count().get_result(db).await?;
		let items = client
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

	/// Update a row in `client`, identified by the primary key with [`UpdateClient`]
	pub async fn update(
		db: &mut ConnectionType,
		param_id: i32,
		item: &UpdateClient,
	) -> QueryResult<Self> {
		use crate::schema::client::dsl::*;

		trace!("Updating client table: {} with {:?}", param_id, item);
		db.transaction(|mut conn| {
			Box::pin(async move {
				diesel::update(client.filter(id.eq(param_id)))
					.set(item)
					.execute(&mut conn)
					.await?;

				client
					.select(Client::as_select())
					.filter(id.eq(last_insert_id()))
					.first::<Self>(&mut conn)
					.await
			})
		})
		.await
	}

	/// Delete a row in `client`, identified by the primary key
	pub async fn delete(db: &mut ConnectionType, param_id: i32) -> QueryResult<usize> {
		use crate::schema::client::dsl::*;

		trace!("Deleting from client table: {}", param_id);
		diesel::delete(client.filter(id.eq(param_id)))
			.execute(db)
			.await
	}
}
