FROM rust:1.84-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev protobuf-compiler && \
    apt clean && \
    rm -rf /var/lib/apt/lists/*
COPY Cargo.toml build.rs ./
COPY protobuf protobuf
COPY src src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
ARG POSTGRESQL_VERSION
WORKDIR /app
RUN apt-get update && \
    apt-get install -y gnupg wget && \
    echo "deb http://apt.postgresql.org/pub/repos/apt bookworm-pgdg main" > /etc/apt/sources.list.d/pgdg.list && \
    wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor -o /etc/apt/trusted.gpg.d/postgresql.gpg && \
    apt-get update && \
    apt-get install -y postgresql-client-${POSTGRESQL_VERSION} && \
    apt clean && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/firefly ./
ENTRYPOINT ["/app/firefly"]
