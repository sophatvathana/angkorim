FROM rust:1.85-alpine3.21 AS builder

# Install dependencies including OpenSSL and Musl tools
RUN apk add --no-cache \
    ca-certificates \
    openssl-dev \
    libssl3 \
    libcrypto3 \
    musl-dev \
    gcc \
    g++ \
    make \
    pkgconfig \
    protobuf-dev
 
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app
COPY . .

ENV OPENSSL_DIR="/usr"
ENV OPENSSL_LIB_DIR="/usr/lib"
ENV OPENSSL_INCLUDE_DIR="/usr/include"

RUN cargo build --release --target x86_64-unknown-linux-musl --verbose

FROM alpine:3.21

RUN apk add --no-cache \
    ca-certificates \
    openssl \
    libssl3 \
    libcrypto3

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/angkorim /usr/local/bin/angkorim

ENTRYPOINT ["angkorim"]
