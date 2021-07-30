FROM rust:1.54-slim as BUILD

RUN apt-get update && \
    apt-get install -y librust-openssl-dev pkg-config

WORKDIR /ddns

COPY . /ddns

RUN cargo build --release

FROM debian:10-slim

RUN apt-get update  && \
    apt-get install -y ca-certificates

COPY --from=build /ddns/target/release/ddns /usr/bin/ddns

ENTRYPOINT ["ddns"]
