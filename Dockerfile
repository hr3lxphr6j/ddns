FROM rust:1.53-slim as BUILD

RUN apt-get update && \
    apt-get install -y librust-openssl-dev

WORKDIR /ddns

COPY . /ddns

RUN cargo build --release

FROM debian:10-slim

RUN apt-get update && \
    apt-get install -y openssl

COPY --from=build /ddns/target/release/ddns /usr/bin/ddns

ENTRYPOINT ["opur-dingtalk-bot"]