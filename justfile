dev *args:
	cargo run -p traffloat-client -F dev -- --assets-path $PWD/assets {{args}}

dev-log level *args:
	RUST_LOG=info,traffloat_physics={{level}},traffloat_client={{level}} cargo run -p traffloat-client -F dev -- --assets-path $PWD/assets {{args}}

fmt:
	cargo +nightly fmt --all
