dev:
	cargo run -p traffloat-client -F dev -- --assets-path $PWD/assets

fmt:
	cargo +nightly fmt --all
