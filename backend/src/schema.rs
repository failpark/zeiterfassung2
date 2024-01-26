// @generated automatically by Diesel CLI.

diesel::table! {
		activity (id) {
				id -> Integer,
				#[max_length = 255]
				token -> Nullable<Varchar>,
				#[max_length = 255]
				name -> Varchar,
				created_at -> Timestamp,
				updated_at -> Timestamp,
		}
}

diesel::table! {
		client (id) {
				id -> Integer,
				#[max_length = 255]
				name -> Varchar,
				created_at -> Timestamp,
				updated_at -> Timestamp,
		}
}

diesel::table! {
		project (id) {
				id -> Integer,
				client_id -> Integer,
				#[max_length = 255]
				name -> Varchar,
				created_at -> Timestamp,
				updated_at -> Timestamp,
		}
}

diesel::table! {
		tracking (id) {
				id -> Integer,
				client_id -> Integer,
				user_id -> Integer,
				project_id -> Integer,
				date -> Date,
				begin -> Time,
				end -> Time,
				pause -> Nullable<Time>,
				performed -> Float,
				billed -> Float,
				description -> Nullable<Text>,
				created_at -> Timestamp,
				updated_at -> Timestamp,
		}
}

diesel::table! {
		tracking_to_activity (id) {
				id -> Integer,
				tracking_id -> Integer,
				activity_id -> Integer,
		}
}

diesel::table! {
		user (id) {
				id -> Integer,
				#[max_length = 50]
				username -> Varchar,
				#[max_length = 40]
				firstname -> Varchar,
				#[max_length = 40]
				lastname -> Varchar,
				#[max_length = 255]
				email -> Varchar,
				#[max_length = 255]
				hash -> Varchar,
				#[max_length = 255]
				sys_role -> Varchar,
				created_at -> Timestamp,
				updated_at -> Timestamp,
		}
}

diesel::joinable!(project -> client (client_id));
diesel::joinable!(tracking -> client (client_id));
diesel::joinable!(tracking -> project (project_id));
diesel::joinable!(tracking -> user (user_id));
diesel::joinable!(tracking_to_activity -> activity (activity_id));
diesel::joinable!(tracking_to_activity -> tracking (tracking_id));

diesel::allow_tables_to_appear_in_same_query!(
	activity,
	client,
	project,
	tracking,
	tracking_to_activity,
	user,
);
