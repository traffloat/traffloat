client-build path_prefix: client-clean pp
	cd client && trunk build --public-url {{path_prefix}} --release release.html
client-build-dev: client-clean pp
	cd client && trunk build dev.html
client-watch: client-clean
	cd client && trunk serve dev.html --watch .. --open

doc: client-glsl
	cargo doc --lib --document-private-items

guide: guide-clean
	cd docgen && cargo run -- --site-url https://traffloat.github.io/guide/master/
	cd docgen/output && mkdocs build

client-clean:
	test ! -d client/dist || rm -r client/dist
	test ! -d client/pkg || rm -r client/pkg
pp: client-texture client-glsl
client-texture:
	cd client/textures && python3 aggregate.py
client-glsl:
	cd client && ./glsl_min.rb
guide-clean:
	test ! -d docgen/output || rm -r docgen/output

deps:
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
