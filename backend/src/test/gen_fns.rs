use fake::{
	Fake,
	Faker,
};
use paste::paste;
use rand::{
	rngs::StdRng,
	SeedableRng,
};

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
