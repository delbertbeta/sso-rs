FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates openssl \
 && rm -rf /var/lib/apt/lists/*
COPY ./target/release/sso-rs /sso-rs
EXPOSE 3000
EXPOSE 2999
CMD ["/sso-rs"]
