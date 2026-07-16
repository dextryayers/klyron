FROM rust:1.85-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev cmake clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

RUN cargo build -p klyron_cli --bin klyron --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r klyron && useradd -r -g klyron -d /app -s /sbin/nologin klyron

WORKDIR /app

COPY --from=builder /build/target/release/klyron /usr/local/bin/klyron

RUN mkdir -p /app/.klyron && chown -R klyron:klyron /app

USER klyron

ENV KLYRON_HOME=/app/.klyron

ENTRYPOINT ["klyron"]
CMD ["--help"]
