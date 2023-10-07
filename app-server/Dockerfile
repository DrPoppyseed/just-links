# Compute a recipe file
FROM rust:1-bullseye as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Cache project dependencies
FROM rust:1-bullseye as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build binary
FROM rust:1-bullseye as builder
ARG CARGO_FEATURES="--features rustls/tls-rustls rustls/tokio-rustls-comp"
WORKDIR /app
COPY . .
# Copy over the cached dependencies from above
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release $CARGO_FEATURES --bin app-server

# Create a tiny output image. That only contains our final binary.
FROM rust:1-bullseye as runtime
WORKDIR /app
COPY --from=builder /app/target/release/app-server /usr/local/bin
ENTRYPOINT ["/usr/local/bin/app-server"]

