#[cfg(test)]
use fake::{
	faker::company::en::*,
	Dummy,
	Fake,
};
use diesel::{
	insert_into,
	prelude::*,
	MysqlConnection,
};
use tracing::trace;

use super::{
	client::Client,
	last_insert_id,
	PaginationResult,
};
use crate::schema::*;

/// Struct representing a row in table `project`
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
#[diesel(table_name=project, primary_key(id), belongs_to(Client, foreign_key=client_id))]
#[cfg_attr(test, derive(PartialEq))]
pub struct Project {
	/// Field representing column `id`
	pub id: i32,
	/// Field representing column `client_id`
	pub client_id: i32,
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
#[cfg_attr(test, derive(Dummy))]
pub struct CreateProject {
	/// Field representing column `client_id`
	pub client_id: i32,
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

impl Project {
	/// Insert a new row into `project` with a given [`CreateProject`]
	pub fn create(db: &mut MysqlConnection, item: &CreateProject) -> QueryResult<Self> {
		use crate::schema::project::dsl::*;

		trace!("Inserting into project table: {:?}", item);
		db.transaction(|conn| {
			insert_into(project).values(item).execute(conn)?;
			project
				.select(Project::as_select())
				.filter(id.eq(last_insert_id()))
				.first::<Self>(conn)
		})
	}

	/// Get a row from `project`, identified by the primary key
	pub fn read(db: &mut MysqlConnection, param_id: i32) -> QueryResult<Self> {
		use crate::schema::project::dsl::*;

		trace!("Reading from project table: {:?}", param_id);
		project.filter(id.eq(param_id)).first::<Self>(db)
	}

	/// Paginates through the table where page is a 0-based index (i.e. page 0 is the first page)
	pub fn paginate(
		db: &mut MysqlConnection,
		page: i64,
		page_size: i64,
	) -> QueryResult<PaginationResult<Self>> {
		use crate::schema::project::dsl::*;

		trace!(
			"Paginating through project table: page {}, page_size {}",
			page,
			page_size
		);
		let page_size = if page_size < 1 { 1 } else { page_size };
		let total_items = project.count().get_result(db)?;
		let items = project
			.limit(page_size)
			.offset(page * page_size)
			.load::<Self>(db)?;

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
	pub fn update(
		db: &mut MysqlConnection,
		param_id: i32,
		item: &UpdateProject,
	) -> QueryResult<Self> {
		use crate::schema::project::dsl::*;

		trace!("Updating project table: {} with {:?}", param_id, item);
		db.transaction(|conn| {
			diesel::update(project.filter(id.eq(param_id)))
				.set(item)
				.execute(conn)?;
			project.filter(id.eq(param_id)).first::<Self>(conn)
		})
	}

	/// Delete a row in `project`, identified by the primary key
	pub fn delete(db: &mut MysqlConnection, param_id: i32) -> QueryResult<usize> {
		use crate::schema::project::dsl::*;

		trace!("Deleting from project table: {:?}", param_id);
		diesel::delete(project.filter(id.eq(param_id))).execute(db)
	}

	pub fn last_page(db: &mut MysqlConnection, page_size: i64) -> QueryResult<i64> {
		use crate::schema::project::dsl::*;

		trace!("Getting last page of project table for page_size {page_size}");

		let total_items: i64 = project.count().get_result(db)?;
		// index starts at 0
		Ok((total_items / page_size + i64::from(total_items % page_size != 0)) - 1)
	}
}

#[cfg(test)]
impl PartialEq<CreateProject> for Project {
	fn eq(&self, other: &CreateProject) -> bool {
		self.name == other.name
	}
}
