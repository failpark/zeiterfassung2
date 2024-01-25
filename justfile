alias c := check
alias ct := compile-test
alias r := run
alias t := nextest
alias diff := difftastic

_default:
	@just --list

fmt *args='--check':
	cargo +nightly fmt {{ if args == "--write" { "" } else if args == "-w" { "" } else { args } }}

add *args:
	cargo add {{ args }}

check *args:
	cargo clippy {{ if args == "-t" { "--all-targets" } else { args } }}

run *args:
	cargo run {{ args }}

test *args:
	cargo test {{ args }}

compile-test *args:
	cargo test --no-run {{ args }}

nextest *args:
	cargo nextest run {{ args }}

# send requests without having to manually get the token
xh *args:
	xh ":8000{{args}}" -A bearer -a `xh POST :8000/login email='admin@localhost' password='admin' | jq -r .token`

# use difftastic
difftastic *args:
	GIT_EXTERNAL_DIFF=difft git diff {{ args }}