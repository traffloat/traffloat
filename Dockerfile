FROM rust:1.49-alpine3.12 AS sources

RUN apk add --no-cache --update pkgconfig openssl-dev musl-dev nodejs npm
RUN cargo install wasm-pack

RUN mkdir -p /build
WORKDIR /build

ADD Cargo.toml Cargo.toml
ADD Cargo.lock Cargo.lock
ADD client client
ADD server server
ADD common common
ADD codegen codegen

FROM sources AS server-build

RUN cargo build --release --package traffloat-server

FROM alpine:3.12 AS server

RUN adduser traffloat --system --disabled-password --no-create --uid 1000
RUN mkdir /app
WORKDIR /app
RUN chown 1000:1000 .

COPY --from=server-build /build/target/release/traffloat-server traffloat-server

ENV RUST_LOG=info
ENTRYPOINT ["./traffloat-server"]

FROM sources AS client-build

WORKDIR /build/client
RUN npm install
RUN npm run build

FROM nginx:1-alpine AS client
COPY --from=client-build /build/client/dist /var/www/html
