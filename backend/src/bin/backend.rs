#[rocket::main]
async fn main() -> std::result::Result<(), rocket::Error> {
	let _guard = zeiterfassung_backend::tracing::init();
	let _rocket = zeiterfassung_backend::rocket()
		.ignite()
		.await?
		.launch()
		.await?;
	Ok(())
}
