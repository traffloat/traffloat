client-build: client-clean pp
	cd client && $(npm bin)/webpack
client-build-dev: client-clean pp
	cd client && $(npm bin)/webpack --mode development
client-watch: client-clean pp
	cd client && $(npm bin)/rimraf dist pkg
	cd client && $(npm bin)/webpack serve --mode development --open

doc: pp
	cargo doc --lib
guide:
	cd docgen && cargo run -- --site-url https://traffloat.github.io/guide/master/
	cd docgen/output && mkdocs build

client-clean:
	cd client && $(npm bin)/rimraf dist pkg
pp: client-texture client-glsl
client-texture:
	python3 client/textures/combine.py
client-glsl:
	cd client && ./glsl_min.rb

test:
	cargo test && wasm-pack test --headless

deps:
	cd client && npm install
	pip3 install -r client/textures/requirements.txt 
