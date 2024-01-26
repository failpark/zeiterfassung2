use fake::{
	Fake,
	Faker,
};
use paste::paste;
use rand::{
	rngs::StdRng,
	SeedableRng,
};
use rocket::{
	local::blocking::{
		Client as LocalClient,
		LocalResponse,
	},
	serde::json::to_string,
};

use super::AuthHeader;

macro_rules! build_faker_fn {
	($name:tt) => {
		paste! {
			pub fn [<generate_ $name:lower>]() -> crate::db::[<$name:lower>]::[<Create $name>] {
				let mut rng = StdRng::from_entropy();
				Faker.fake_with_rng(&mut rng)
			}
		}
	};
}

build_faker_fn!(Activity);
build_faker_fn!(Client);
build_faker_fn!(User);
build_faker_fn!(Project);

macro_rules! build_create_fn {
	($name:tt) => {
		paste! {
			pub fn [<create_ $name:lower>]<'a>(client: &'a LocalClient, item: &crate::db::[<$name:lower>]::[<Create $name>], token: &str) -> LocalResponse<'a> {
				client
					.post(format!("/{}", stringify!([<$name:lower>])))
					.body(to_string(&item).expect(concat!("Could not serialize ", stringify!([<Create $name>]))))
					.add_auth_header(token)
					.dispatch()
			}
		}
	};
}

build_create_fn!(Client);
build_create_fn!(Project);
