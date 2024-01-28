use rocket::{
	delete,
	fairing::AdHoc,
	get,
	patch,
	post,
	routes,
	serde::json::Json,
};
use rocket_db_pools::Connection;

use crate::{
	db::{
		project::{
			CreateProject,
			Project,
			UpdateProject,
		},
		PaginationResult,
	},
	Error,
	Result,
	User,
	DB,
};

#[post("/", data = "<create_project>")]
async fn create(
	user: User,
	mut db: Connection<DB>,
	create_project: Json<CreateProject>,
) -> Result<Json<Project>> {
	if user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	let project = Project::create(&mut db, &create_project).await;
	if let Ok(project) = project {
		Ok(Json(project))
	} else {
		Err(Error::BadRequest("Project already exists".to_string()))
	}
}

#[patch("/<id>", data = "<update_project>")]
async fn update(
	_user: User,
	mut db: Connection<DB>,
	update_project: Json<UpdateProject>,
	id: i32,
) -> Result<Json<Project>> {
	Ok(Json(Project::update(&mut db, id, &update_project).await?))
}

#[get("/<id>")]
async fn get(_user: User, mut db: Connection<DB>, id: i32) -> Result<Json<Project>> {
	Ok(Json(Project::read(&mut db, id).await?))
}

#[get("/page/<page_size>/<page>")]
async fn get_page(
	_user: User,
	mut db: Connection<DB>,
	page_size: i64,
	page: i64,
) -> Result<Json<PaginationResult<Project>>> {
	Ok(Json(Project::paginate(&mut db, page, page_size).await?))
}

#[get("/page/<page_size>/last", rank = 2)]
async fn get_last_page(
	_user: User,
	mut db: Connection<DB>,
	page_size: i64,
) -> Result<Json<PaginationResult<Project>>> {
	let last_page = Project::last_page(&mut db, page_size).await?;
	Ok(Json(
		Project::paginate(&mut db, last_page, page_size).await?,
	))
}

#[delete("/<id>")]
async fn delete(user: User, mut db: Connection<DB>, id: i32) -> Result<Json<usize>> {
	if user.sys_role != "admin" {
		return Err(Error::ForbiddenAccess);
	}
	Ok(Json(Project::delete(&mut db, id).await?))
}

pub fn mount() -> AdHoc {
	AdHoc::on_ignite("Project", |rocket| async {
		rocket.mount(
			"/project",
			routes![create, get, update, delete, get_page, get_last_page,],
		)
	})
}

#[cfg(test)]
mod test {
	use fake::{
		faker::lorem::en::Word,
		Fake,
	};
	use pretty_assertions::{
		assert_eq,
		assert_ne,
	};
	use rocket::{
		http::Status,
		local::blocking::Client as LocalClient,
		serde::json::to_string,
	};

	use crate::{
		db::{
			client::Client,
			project::{
				Project,
				UpdateProject,
			},
			PaginationResult,
		},
		rocket,
		test::{
			generate_client,
			generate_project,
			methods::{
				delete,
				get,
				patch,
				post,
			},
			token::{
				get_token_admin,
				get_token_user,
			},
		},
	};

	#[test]
	fn project() {
		let client = LocalClient::tracked(rocket()).unwrap();
		let mut project = generate_project();
		let token = get_token_admin(&client);
		// Create new client for project
		let project_client = generate_client();
		let res = post(
			&client,
			"/client",
			to_string(&project_client).unwrap(),
			token,
		);
		let project_client = res.into_json::<Client>().unwrap();
		project.client_id = project_client.id;
		let base_url = String::from("/project");

		// Test unauthorized
		let res = post(&client, &base_url, to_string(&project).unwrap(), "");
		assert_eq!(res.status(), Status::Unauthorized);

		// Create new project
		let res = post(&client, &base_url, to_string(&project).unwrap(), token);
		let inserted_project = res.into_json::<Project>().unwrap();
		assert_eq!(inserted_project, project);
		let project_id = inserted_project.id;

		// Update project
		let new_project_name = Word().fake::<String>() + " " + &Word().fake::<String>();
		let update_project = UpdateProject {
			name: Some(new_project_name.clone()),
			..Default::default()
		};
		let url = format!("{base_url}/{}", project_id);
		let res = patch(&client, &url, to_string(&update_project).unwrap(), token);
		assert_eq!(res.status(), Status::Ok);
		let updated_project = res.into_json::<Project>().unwrap();
		assert_eq!(updated_project.name, new_project_name);
		assert_ne!(updated_project, inserted_project);

		// Get project
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		let res = res.into_json::<Project>().unwrap();
		assert_eq!(res, updated_project);

		// Test inserting as normal user
		let user_token = get_token_user(&client);
		let res = post(&client, &base_url, to_string(&project).unwrap(), user_token);
		assert_eq!(res.status(), Status::Forbidden);

		// delete project
		let res = delete(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
	}

	#[test]
	fn projects() {
		let client = LocalClient::tracked(rocket()).unwrap();
		let token = get_token_admin(&client);
		let base_url = String::from("/project");

		// Create new client for project
		let project_client = generate_client();
		let res = post(
			&client,
			"/client",
			to_string(&project_client).unwrap(),
			token,
		);
		let project_client = res.into_json::<Client>().unwrap();

		let mut project_list: Vec<Project> = Vec::new();
		// insert multiple projects
		for _ in 0..10 {
			let mut project = generate_project();
			project.client_id = project_client.id;
			let res = post(&client, &base_url, to_string(&project).unwrap(), token);
			if res.status() != Status::Ok {
				dbg!(res.into_string());
				panic!();
			}
			project_list.push(res.into_json::<Project>().unwrap());
		}

		let url = format!("{base_url}/{}", project_list[0].id);
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		assert_eq!(res.into_json::<Project>().unwrap(), project_list[0]);

		// get first page of users with page_size 5
		let url = format!("{base_url}/page/5/0");
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		let pagination = res.into_json::<PaginationResult<Project>>().unwrap();
		assert_eq!(pagination.page, 0);
		assert_eq!(pagination.page_size, 5);

		// get last page of users with page_size 5 with number
		let url = format!("{base_url}/page/5/{}", pagination.num_pages - 1);
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		let pagination = res.into_json::<PaginationResult<Project>>().unwrap();
		// the last page contains 5 OR LESS items
		let last_page_items: usize = (pagination.total_items - (pagination.num_pages - 1) * 5)
			.try_into()
			.unwrap();
		// reverse to get items from the bottom
		let last_page_items = 10 - last_page_items;
		// some race conditions could arrise here, but in prod it doesn't matter
		assert_eq!(pagination.items, project_list[last_page_items..]);

		let url = format!("{base_url}/page/5/last");
		let res = get(&client, &url, token);
		assert_eq!(res.status(), Status::Ok);
		let last_page = res.into_json::<PaginationResult<Project>>().unwrap();
		assert_eq!(last_page, pagination);

		// delete all projects
		for project in project_list {
			let url = format!("{base_url}/{}", project.id);
			let res = delete(&client, &url, token);
			assert_eq!(res.status(), Status::Ok);
		}
	}
}
