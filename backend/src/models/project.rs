use rocket_db_pools::diesel::{
	insert_into,
	prelude::*,
};

use super::last_insert_id;
use crate::schema::*;

pub type ConnectionType = rocket_db_pools::Connection<crate::DB>;

/// Struct representing a row in table `project`
#[derive(
	Debug, Clone, serde::Serialize, serde::Deserialize, Queryable, Selectable, QueryableByName,
)]
#[diesel(table_name=project, primary_key(id))]
pub struct Project {
	/// Field representing column `id`
	pub id: i32,
	/// Field representing column `name`
	pub name: String,
	/// Field representing column `created_at`
	pub created_at: chrono::NaiveDateTime,
	/// Field representing column `updated_at`
	pub updated_at: chrono::NaiveDateTime,
}

/// Create Struct for a row in table `project` for [`Project`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Insertable)]
#[diesel(table_name=project)]
pub struct CreateProject {
	/// Field representing column `id`
	pub id: i32,
	/// Field representing column `name`
	pub name: String,
}

/// Update Struct for a row in table `project` for [`Project`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, AsChangeset, PartialEq, Default)]
#[diesel(table_name=project)]
pub struct UpdateProject {
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

impl Project {
	/// Insert a new row into `project` with a given [`CreateProject`]
	pub async fn create(db: &mut ConnectionType, item: &CreateProject) -> QueryResult<Self> {
		use crate::schema::project::dsl::*;

		db.transaction(|mut conn| {
			Box::pin(async move {
				insert_into(project).values(item).execute(&mut conn).await?;
				project
					.select(Project::as_select())
					.filter(id.eq(last_insert_id()))
					.first::<Self>(&mut conn)
					.await
			})
		})
		.await
	}

	/// Get a row from `project`, identified by the primary key
	pub async fn read(db: &mut ConnectionType, param_id: i32) -> QueryResult<Self> {
		use crate::schema::project::dsl::*;

		project.filter(id.eq(param_id)).first::<Self>(db).await
	}

	/// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
	pub async fn paginate(
		db: &mut ConnectionType,
		page: i64,
		page_size: i64,
	) -> QueryResult<PaginationResult<Self>> {
		use crate::schema::project::dsl::*;

		let page_size = if page_size < 1 { 1 } else { page_size };
		let total_items = project.count().get_result(db).await?;
		let items = project
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

	/// Update a row in `project`, identified by the primary key with [`UpdateProject`]
	pub async fn update(
		db: &mut ConnectionType,
		param_id: i32,
		item: &UpdateProject,
	) -> QueryResult<Self> {
		use crate::schema::project::dsl::*;

		db.transaction(|mut conn| {
			Box::pin(async move {
				diesel::update(project.filter(id.eq(param_id)))
					.set(item)
					.execute(&mut conn)
					.await?;
				project
					.filter(id.eq(param_id))
					.first::<Self>(&mut conn)
					.await
			})
		})
		.await
	}

	/// Delete a row in `project`, identified by the primary key
	pub async fn delete(db: &mut ConnectionType, param_id: i32) -> QueryResult<usize> {
		use crate::schema::project::dsl::*;

		diesel::delete(project.filter(id.eq(param_id)))
			.execute(db)
			.await
	}
}
