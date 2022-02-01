default: client-run-dev

client-run-dev:
	cargo run --bin traffloat-desktop

client-build:
	cargo build --release --bin traffloat-desktop

doc:
	cargo doc --lib --document-private-items

client-scenarios *ARGS: client-scenarios-clean
	cargo run --release --bin tfsave-builder -- {{ARGS}} scenarios/vanilla/main.toml client/gen/scenarios/vanilla
client-scenarios-dev *ARGS: client-scenarios-clean
	cargo run --bin tfsave-builder -- {{ARGS}} scenarios/vanilla/main.toml client/gen/scenarios/vanilla

client-scenarios-clean:
	rm -r client/gen/scenarios || true
	mkdir -p client/gen/scenarios

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
