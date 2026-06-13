dev *args:
	cargo run -p traffloat-client -F dev -- --assets-path $PWD/assets {{args}}

dev-log level *args:
	RUST_LOG=info,traffloat_physics={{level}},traffloat_client={{level}} cargo run -p traffloat-client -F dev -- --assets-path $PWD/assets {{args}}

fmt:
	cargo +nightly fmt --all

imports:
	cargo clippy --fix --allow-staged -- -D unused_imports

# This is currently not enforced, will be fixed in the future after project gets more mature
precommit:
	cargo +nightly fmt --all
	cargo clippy -- \
		-W clippy::dbg_macro \
		-W clippy::unused_self \
		-W unused_imports \
		-W dead_code \
		-W unused_variables
