use rocket::local::blocking::{
	Client,
	LocalRequest,
	LocalResponse,
};

use crate::test::token::AuthHeader;

fn setup<'b>(req: LocalRequest<'b>, item: Option<String>, token: &'b str) -> LocalResponse<'b> {
	let req = req.add_auth_header(token);
	if let Some(item) = item {
		req.body(item).dispatch()
	} else {
		req.dispatch()
	}
}

pub fn delete<'b>(client: &'b Client, uri: &'b str, token: &'b str) -> LocalResponse<'b> {
	setup(client.delete(uri), None, token)
}

pub fn get<'b>(client: &'b Client, uri: &'b str, token: &'b str) -> LocalResponse<'b> {
	setup(client.get(uri), None, token)
}

pub fn post<'b>(
	client: &'b Client,
	uri: &'b str,
	item: String,
	token: &'b str,
) -> LocalResponse<'b> {
	setup(client.post(uri), Some(item), token)
}

#[allow(dead_code)]
pub fn put<'b>(
	client: &'b Client,
	uri: &'b str,
	item: String,
	token: &'b str,
) -> LocalResponse<'b> {
	setup(client.put(uri), Some(item), token)
}

pub fn patch<'b>(
	client: &'b Client,
	uri: &'b str,
	item: String,
	token: &'b str,
) -> LocalResponse<'b> {
	setup(client.patch(uri), Some(item), token)
}
