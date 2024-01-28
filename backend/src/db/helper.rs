use diesel::Table;
use crate::DB;
use rocket_db_pools::Connection;
use super::QueryResult;
use diesel::prelude::*;
use diesel::query_dsl::methods::SelectDsl;
use diesel::dsl::CountStar;
use rocket_db_pools::diesel::RunQueryDsl;
use tracing::trace;

async fn last_page<T: Table>(db: &mut Connection<DB>, page_size: i64, table: T) -> QueryResult<i64>
where T: SelectDsl<CountStar>,
	T: RunQueryDsl<Connection<DB>>,
{

	trace!("Getting last page of user table for page_size {page_size}");

	let total_items: i64 = table.count().get_result(db).await?;
	Ok(total_items / page_size + i64::from(total_items % page_size != 0))
}