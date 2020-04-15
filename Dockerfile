FROM rust as build

WORKDIR /usr/src/hemtt

RUN apt-get update
RUN apt-get install libssl-dev -y

COPY . .
RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update && \
    apt-get install -y git zip ca-certificates libcurl4 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/src/hemtt/target/release/hemtt /usr/local/bin/hemtt
