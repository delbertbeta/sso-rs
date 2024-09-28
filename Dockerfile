FROM alpine:3.20.3
COPY ./target/release/sso-rs /sso-rs
EXPOSE 3000
EXPOSE 2999
CMD ["/sso-rs"]
