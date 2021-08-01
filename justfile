client-build: client-clean pp
	cd client && $(npm bin)/webpack
client-build-dev: client-clean pp
	cd client && $(npm bin)/webpack --mode development
client-watch: client-clean
	cd client && $(npm bin)/webpack serve --mode development --open

doc: client-glsl
	cargo doc --lib

guide: guide-clean
	cd docgen && cargo run -- --site-url https://traffloat.github.io/guide/master/
	cd docgen/output && mkdocs build

client-clean:
	cd client && $(npm bin)/rimraf dist pkg
pp: client-texture client-glsl
client-texture:
	cd client/textures && python3 aggregate.py
client-glsl:
	cd client && ./glsl_min.rb
guide-clean:
	test ! -d docgen/output || rm -r docgen/output

test:
	cargo test && wasm-pack test --headless

deps:
	cd client && npm install
	cd client/textures && npm install
	pip3 install -r client/textures/requirements.txt 

tokei:
	tokei -e "*lock*" -e "*.svg" 
depgraph:
	#!/usr/bin/env sh
	(
		echo 'digraph G {'
		echo '  rankdir="LR";'
		cargo metadata --format-version 1 |
			jq '.packages |
				map(
					select(
						.name |
							contains("traffloat")
					)
				) |
				map({
					name,
					dependencies: ("{" + (
						.dependencies |
							map("\"" + .name + "\"") |
							join(";")
					) + "}")
				}) |
				map("\"" + .name + "\" -> " + .dependencies) |
				join("\n")
			' -r
		echo "}"
	) | \
		dot -T png
