FROM ubuntu:24.04
RUN apt update && apt install -y openssl ca-certificates
COPY ./target/release/sso-rs /sso-rs
EXPOSE 3000
EXPOSE 2999
CMD ["/sso-rs"]
