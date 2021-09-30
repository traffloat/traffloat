default: client-watch

client-build path_prefix: client-clean pp
	cd client && trunk build --public-url {{path_prefix}} --release release.html
client-build-dev: client-clean pp
	cd client && trunk build dev.html
client-watch: client-clean
	cd client && trunk serve dev.html --watch .. --open

doc: client-glsl
	cargo doc --lib --document-private-items

client-clean:
	test ! -d client/dist || rm -r client/dist
	test ! -d client/pkg || rm -r client/pkg
pp: client-texture client-glsl client-tsv
client-texture:
	cd client/textures && python3 aggregate.py
client-glsl:
	cd client && ./glsl_min.rb
client-tsv:
	rm client/static/*.tsv || true
	find client/static -name "*.tsvt" -exec cargo run --bin tsvtool to-binary {} \;

deps:
	cd client/textures && npm install
	pip3 install -r client/textures/requirements.txt
	test -f client/glsl_min.rb || ( \
		wget -O client/glsl_min.rb \
			https://raw.githubusercontent.com/traffloat/glsl-minifier/master/glsl_min.rb && \
		chmod +x client/glsl_min.rb \
	)

tokei:
	tokei -C -e "*lock*" -e "*.svg"
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
