use rocket::local::blocking::{
	Client,
	LocalRequest,
	LocalResponse,
};

use crate::{
	rocket,
	test::token::AuthHeader,
};

fn setup<'b>(req: LocalRequest<'b>, item: Option<String>, token: &'b str) -> LocalResponse<'b> {
	let req = req.add_auth_header(token);
	if let Some(item) = item {
		req.body(item).dispatch()
	} else {
		req.dispatch()
	}
}

pub fn delete<'b>(client: &'b Client, path: &'b str, token: &'b str) -> LocalResponse<'b> {
	setup(client.delete(path), None, token)
}

pub fn get<'b>(client: &'b Client, path: &'b str, token: &'b str) -> LocalResponse<'b> {
	setup(client.get(path), None, token)
}

pub fn post<'b>(
	client: &'b Client,
	path: &'b str,
	item: String,
	token: &'b str,
) -> LocalResponse<'b> {
	setup(client.post(path), Some(item), token)
}

pub fn put<'b>(
	client: &'b Client,
	path: &'b str,
	item: String,
	token: &'b str,
) -> LocalResponse<'b> {
	setup(client.put(path), Some(item), token)
}
