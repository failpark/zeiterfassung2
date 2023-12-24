alias c := check
alias r := run
alias t := nextest

_default:
	@just --list

fmt *args='--check':
	cargo +nightly fmt {{ if args == "--write" { "" } else if args == "-w" { "" } else { args } }}

add *args:
	cargo add {{ args }}

check *args:
	cargo clippy {{ args }}

run *args:
	cargo run {{ args }}

test *args:
	cargo test {{ args }}

nextest *args:
	cargo nextest run {{ args }}

# send requests without having to manually get the token
xh *args:
	xh ":8000{{args}}" -A bearer -a `xh POST :8000/login email='admin@localhost' password='admin' | jq -r .token`
