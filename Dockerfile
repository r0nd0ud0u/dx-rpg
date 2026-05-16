FROM rust:1 AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY src src
COPY Cargo.toml .
COPY Cargo.lock .
COPY assets assets
COPY offlines offlines
COPY Dioxus.toml .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=ssh cargo chef cook --release --recipe-path recipe.json
COPY src src
COPY Cargo.toml .
COPY Cargo.lock .
COPY assets assets
COPY offlines/ offlines/
COPY Dioxus.toml .

# Install `dx`
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli@0.7.9 --root /.cargo -y --force
ENV PATH="/.cargo/bin:$PATH"

# Create the final bundle folder. Bundle always executes in release mode with optimizations enabled
RUN --mount=type=ssh dx bundle --platform web --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 pkg-config curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/dx/dx-rpg/release/web/ /usr/local/app
COPY ./offlines/ /usr/local/app/offlines/

# Create directories for persistent data volumes
RUN mkdir -p /data /usr/local/app/saved_data

# Set correct permissions for the app directory and data volumes
RUN chmod -R 755 /usr/local/app && chmod 777 /data /usr/local/app/saved_data

# set our port and make sure to listen for all connections
ENV PORT=8080
ENV IP=0.0.0.0
# DATABASE_URL must be provided at runtime (e.g. via docker-compose environment)
ENV DATABASE_URL=sqlite:///data/db.sqlite

# expose the port 8080
EXPOSE 8080

WORKDIR /usr/local/app

# Entrypoint script: ensure /data/db.sqlite exists before starting the app
RUN printf '#!/bin/sh\nmkdir -p /data\ntouch /data/db.sqlite\nexec /usr/local/app/server "$@"\n' > /entrypoint.sh \
    && chmod +x /entrypoint.sh

ENTRYPOINT [ "/entrypoint.sh" ]
