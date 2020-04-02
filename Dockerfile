FROM rust as build

WORKDIR /usr/src/hemtt
COPY . .

RUN apt-get update
RUN apt-get install musl-tools libssl-dev -y

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine

RUN \
	apt-get update && apt-get install -y git python3 zip ca-certificates libcurl3 --no-install-recommends && \
	rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/src/hemtt/target/x86_64-unknown-linux-musl/release/hemtt /usr/local/bin/hemtt
