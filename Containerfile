FROM rust:bookworm as build

WORKDIR /usr/src/dynamic-route53-rs
COPY . .
RUN apt-get update -y && apt-get install -y pkg-config libssl-dev && apt-get clean
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update -y && apt-get install -y openssl && apt clean -y
COPY --from=build /usr/src/dynamic-route53-rs/target/release/dynamic-route53 /usr/local/bin/
RUN useradd -U dynamic-route53
USER dynamic-route53
WORKDIR /
