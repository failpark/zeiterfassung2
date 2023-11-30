_default:
	@just --list

fmt *args='--check':
	cargo +nightly fmt {{ if args == "--write" { "" } else { args } }}

add *args:
	cargo add {{ args }}

dsync:
	dsync -i backend/src/schema.rs -o backend/src/models --connection-type="rocket_db_pools::Connection<crate::DB>" --async -g created_at -g updated_at