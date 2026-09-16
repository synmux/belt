FROM rust:1.98.1-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --locked --release --bin belt

FROM debian:bookworm-slim AS runtime

HEALTHCHECK NONE

RUN apt-get update \
  && apt-get install --no-install-recommends -y ca-certificates=20250419~deb12u1 libgcc-s1=12.2.0-14+deb12u1 \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/belt /usr/local/bin/belt

USER 10001:10001

ENTRYPOINT ["/usr/local/bin/belt"]
