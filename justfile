default: client-watch

client-build path_prefix: client-clean
	cd client && trunk build --public-url {{path_prefix}} --release release.html
client-build-dev: client-clean
	cd client && trunk build dev.html
client-watch: client-clean
	cd client && trunk serve dev.html --watch .. --open

doc:
	cargo doc --lib --document-private-items

client-clean:
	test ! -d client/dist || rm -r client/dist
	test ! -d client/pkg || rm -r client/pkg

client-scenarios *ARGS: client-scenarios-clean
	cargo run --release --bin tfsave-builder -- {{ARGS}} scenarios/vanilla/main.toml client/gen/scenarios/vanilla
client-scenarios-dev *ARGS: client-scenarios-clean
	cargo run --bin tfsave-builder -- {{ARGS}} scenarios/vanilla/main.toml client/gen/scenarios/vanilla

client-scenarios-clean:
	rm -r client/gen/scenarios || true
	mkdir client/gen/scenarios

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
