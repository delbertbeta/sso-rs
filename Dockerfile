FROM rust:bookworm AS builder
WORKDIR /app

RUN rustup component add rustfmt

RUN apt-get -o Acquire::ForceIPv4=true update \
 && apt-get install -y --no-install-recommends pkg-config libssl-dev protobuf-compiler \
 && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY entity ./entity
COPY migration ./migration
COPY volo-gen ./volo-gen
COPY qcloud ./qcloud
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get -o Acquire::ForceIPv4=true update \
 && apt-get install -y --no-install-recommends ca-certificates openssl \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sso-rs /sso-rs

EXPOSE 3000
EXPOSE 2999

CMD ["/sso-rs"]
