FROM rust:1.33 as build

WORKDIR /usr/src/hemtt
COPY . .
RUN cargo install --path .

FROM ubuntu:18.04

RUN \
	apt-get update && apt-get install -y git python3 zip ca-certificates libcurl3 --no-install-recommends && \
	rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/local/cargo/bin/hemtt /usr/local/bin/hemtt
