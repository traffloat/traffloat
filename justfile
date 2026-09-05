dev *args:
	RUST_BACKTRACE=1 cargo run -p traffloat-client -F dev -- --assets-path $PWD/assets {{args}}

dev-log level *args:
	RUST_BACKTRACE=1 RUST_LOG=info,traffloat_physics={{level}},traffloat_client={{level}} cargo run -p traffloat-client -F dev -- --assets-path $PWD/assets {{args}}

fmt:
	cargo +nightly fmt --all

imports:
	cargo clippy --fix --tests --allow-staged -- -D unused_imports

precommit:
	cargo +nightly fmt --all
	cargo clippy --tests --benches --examples -- \
		-W clippy::dbg_macro \
		-W clippy::unused_self \
		-W unused_imports \
		-W unused_variables \
		# -W dead_code # suppressed until the project is more mature

test module *args:
	RUST_BACKTRACE=1 cargo test -p traffloat-{{module}} -F bevy/dynamic_linking,bevy/debug --lib -- --nocapture --color always {{args}}
