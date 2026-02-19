use diesel::{
	dsl::CountStar,
	prelude::*,
	query_dsl::{
		methods::SelectDsl,
		LoadQuery,
	},
	MysqlConnection,
	QueryResult,
	Table,
};
use tracing::trace;

pub fn last_page<T>(db: &mut MysqlConnection, page_size: i64, table: T) -> QueryResult<i64>
where
	T: Table + SelectDsl<CountStar>,
	<T as SelectDsl<CountStar>>::Output: RunQueryDsl<MysqlConnection>
		+ LoadQuery<'static, MysqlConnection, i64>,
{
	trace!("Getting last page of user table for page_size {page_size}");

	let total_items: i64 = table.count().get_result(db)?;
	Ok(total_items / page_size + i64::from(total_items % page_size != 0))
}
