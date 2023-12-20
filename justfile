_default:
	@just --list

fmt *args='--check':
	cargo +nightly fmt {{ if args == "--write" { "" } else if args == "-w" { "" } else { args } }}

add *args:
	cargo add {{ args }}

check *args:
	cargo clippy {{ args }}