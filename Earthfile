VERSION 0.8
IMPORT github.com/earthly/lib/rust:3.0.1 AS rust

assets:
    FROM python:3.12.6-alpine3.20
    CACHE /var/pycache
    ENV PYTHONPYCACHEPREFIX /var/pycache
    COPY scenarios/requirements.txt /src/
    WORKDIR /src
    RUN pip install -r requirements.txt
    COPY scenarios .
    RUN python .
    SAVE ARTIFACT assets AS LOCAL assets

compile:
    FROM rust:1.81-slim-bullseye
    RUN apt-get update && apt-get install -y pkg-config make g++ libssl-dev
    DO rust+INIT --keep_fingerprints=true
    COPY --keep-ts --dir Cargo.toml Cargo.lock base desktop fluid graph tools version view .
    DO rust+CARGO --args='build --release --bin traffloat-desktop' --output='release/traffloat-desktop'

save-schema:
    FROM +compile
    DO rust+CARGO --args='run --bin traffloat-save-schema'

    SAVE ARTIFACT assets/save-schema.json AS LOCAL output/save-schema.json
