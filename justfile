_default:
	@just --list

fmt *args='--check':
	cargo +nightly fmt {{ if args == "--write" { "" } else { args } }}
