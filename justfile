_default:
	@just --list

fmt *args='--check':
	cargo +nightly fmt {{ if args == "--write" { "" } else if args == "-w" { "" } else { args } }}

add *args:
	cargo add {{ args }}

fix_ids:
	sed -i 's/\sid\ ->\ Integer\,/\tid\ ->\ Nullable\<Integer\>,/g' backend/src/schema.rs