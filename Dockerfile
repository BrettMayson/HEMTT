FROM rust:1.33

WORKDIR /usr/src/hemtt
COPY . .

RUN apt-get update
RUN apt-get install -y \
	git \
	python3 \
	zip

RUN rm -rf /var/lib/apt/lists/*

RUN cargo install --path .
