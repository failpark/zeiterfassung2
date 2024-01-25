use rocket::{
	fairing::AdHoc,
	serde::json::Json,
};
use rocket_db_pools::Connection;

use crate::{
	db::{
		client::{
			Client,
			CreateClient,
			UpdateClient,
		},
		PaginationResult,
	},
	Error,
	Result,
	User,
	DB,
};

#[post("/", data = "<create_client>")]
async fn create(
	user: User,
	mut db: Connection<DB>,
	create_client: Json<CreateClient>,
) -> Result<Json<Client>> {
	if user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	let client = Client::create(&mut db, &create_client).await;
	if let Ok(client) = client {
		Ok(Json(client))
	} else {
		Err(Error::BadRequest("Client already exists".to_string()))
	}
}

#[patch("/<id>", data = "<update_client>")]
async fn update(
	_user: User,
	mut db: Connection<DB>,
	update_client: Json<UpdateClient>,
	id: i32,
) -> Result<Json<Client>> {
	Ok(Json(Client::update(&mut db, id, &update_client).await?))
}

#[get("/<id>")]
async fn get(_user: User, mut db: Connection<DB>, id: i32) -> Result<Json<Client>> {
	Ok(Json(Client::read(&mut db, id).await?))
}

#[get("/page/<page_size>/<page>")]
async fn get_page(
	_user: User,
	mut db: Connection<DB>,
	page_size: i64,
	page: i64,
) -> Result<Json<PaginationResult<Client>>> {
	Ok(Json(Client::paginate(&mut db, page, page_size).await?))
}

#[get("/page/<page_size>/last", rank = 2)]
async fn get_last_page(
	_user: User,
	mut db: Connection<DB>,
	page_size: i64,
) -> Result<Json<PaginationResult<Client>>> {
	let last_page = Client::last_page(&mut db, page_size).await?;
	Ok(Json(Client::paginate(&mut db, last_page, page_size).await?))
}

#[delete("/<id>")]
async fn delete(user: User, mut db: Connection<DB>, id: i32) -> Result<Json<usize>> {
	if user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	Ok(Json(Client::delete(&mut db, id).await?))
}

pub fn mount() -> AdHoc {
	AdHoc::on_ignite("Client", |rocket| async {
		rocket.mount(
			"/client",
			routes![create, get, update, delete, get_page, get_last_page,],
		)
	})
}

#[cfg(test)]
mod test {
	use pretty_assertions::{
		assert_eq,
		assert_ne,
	};
	use rocket::{
		http::Status,
		local::blocking::Client as RocketClient,
		serde::json::to_string,
	};

	use crate::{
		db::{
			client::{
				Client,
				UpdateClient,
			},
			PaginationResult,
		},
		error::ErrorJson,
		rocket,
		test::{
			generate_client,
			get_token_admin,
			get_token_user,
			AuthHeader,
		},
	};

	#[test]
	fn client() {
		let rocket_client = RocketClient::tracked(rocket()).unwrap();
		let client = generate_client();

		// Test unauthorized
		let res = rocket_client
			.post("/client")
			.body(to_string(&client).unwrap())
			.dispatch();
		assert_eq!(res.status(), Status::Unauthorized);

		let token = get_token_admin(&rocket_client);
		// Create new client
		let res = rocket_client
			.post("/client")
			.body(to_string(&client).unwrap())
			.add_auth_header(token)
			.dispatch();
		let inserted_client = res.into_json::<Client>().unwrap();
		assert_eq!(inserted_client, client);
		let client_id = inserted_client.id;

		// Check duplicate client insert
		let res = rocket_client
			.post("/client")
			.body(to_string(&client).unwrap())
			.add_auth_header(token)
			.dispatch();
		assert_eq!(res.status(), Status::BadRequest);
		assert_eq!(
			res.into_string(),
			Some(
				to_string(&ErrorJson::new(400, "Client already exists",))
					.expect("Could not serialize ErrorJson")
			)
		);

		// Update client
		let mut new_company_name = client.name.clone();
		// Companys name should end with GmbH on the bill or smth... 🤷
		new_company_name.push_str(" GmbH");
		let update_client = UpdateClient {
			name: Some(new_company_name.clone()),
			..Default::default()
		};
		let res = rocket_client
			.patch(format!("/client/{client_id}"))
			.body(to_string(&update_client).expect("Could not serialize UpdateClient"))
			.add_auth_header(token)
			.dispatch();
		assert_eq!(res.status(), Status::Ok);
		let updated_client = res.into_json::<Client>().unwrap();
		assert_eq!(updated_client.name, new_company_name);
		assert_ne!(updated_client, inserted_client);

		// Get client
		let res = rocket_client
			.get(format!("/client/{client_id}"))
			.add_auth_header(token)
			.dispatch();
		assert_eq!(res.status(), Status::Ok);
		let res = res.into_json::<Client>().unwrap();
		assert_eq!(res, updated_client);

		// Test inserting as normal user
		let user_token = get_token_user(&rocket_client);
		let res = rocket_client
			.post("/client")
			.body(to_string(&client).unwrap())
			.add_auth_header(user_token)
			.dispatch();
		assert_eq!(res.status(), Status::Forbidden);

		// delete client
		let res = rocket_client
			.delete(format!("/client/{client_id}"))
			.add_auth_header(token)
			.dispatch();
		assert_eq!(res.status(), Status::Ok);
	}

	#[test]
	fn clients() {
		let rocket_client = RocketClient::tracked(rocket()).unwrap();
		let token = get_token_admin(&rocket_client);

		let mut client_list: Vec<Client> = Vec::new();
		// insert multiple clients
		for _ in 0..10 {
			let client = generate_client();
			let res = rocket_client
				.post("/client")
				.body(to_string(&client).unwrap())
				.add_auth_header(token)
				.dispatch();
			if res.status() != Status::Ok {
				dbg!(res.into_string());
				panic!();
			}
			client_list.push(res.into_json::<Client>().unwrap());
		}

		let res = rocket_client
			.get(format!("/client/{}", client_list[0].id))
			.add_auth_header(token)
			.dispatch();
		assert_eq!(res.status(), Status::Ok);
		assert_eq!(res.into_json::<Client>().unwrap(), client_list[0]);

		// get first page of users with page_size 5
		let res = rocket_client
			.get("/client/page/5/0")
			.add_auth_header(token)
			.dispatch();
		assert_eq!(res.status(), Status::Ok);
		let pagination = res.into_json::<PaginationResult<Client>>().unwrap();
		assert_eq!(pagination.page, 0);
		assert_eq!(pagination.page_size, 5);

		// get last page of users with page_size 5 with number
		let res = rocket_client
			.get(format!("/client/page/5/{}", pagination.num_pages - 1))
			.add_auth_header(token)
			.dispatch();
		assert_eq!(res.status(), Status::Ok);
		let pagination = res.into_json::<PaginationResult<Client>>().unwrap();
		// the last page contains 5 OR LESS items
		let last_page_items: usize = (pagination.total_items - (pagination.num_pages - 1) * 5)
			.try_into()
			.unwrap();
		// reverse to get items from the bottom
		let last_page_items = 10 - last_page_items;
		assert_eq!(pagination.items, client_list[last_page_items..]);

		let res = rocket_client
			.get("/client/page/5/last")
			.add_auth_header(token)
			.dispatch();
		assert_eq!(res.status(), Status::Ok);
		let last_page = res.into_json::<PaginationResult<Client>>().unwrap();
		assert_eq!(last_page, pagination);

		// delete all clients
		for client in client_list {
			let res = rocket_client
				.delete(format!("/client/{}", client.id))
				.add_auth_header(token)
				.dispatch();
			assert_eq!(res.status(), Status::Ok);
		}
	}
}
