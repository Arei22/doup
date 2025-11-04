FROM rust:alpine AS builder

RUN apk add --no-cache \
    nodejs npm \
    musl-dev \
    libjpeg-turbo-dev libpng-dev zlib-dev libunwind-dev xz-dev \
    postgresql-dev openssl-dev \
    build-base \
    pkgconfig \
    nasm

RUN npm install -g pnpm

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target=x86_64-unknown-linux-musl

COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine:3.20

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/doup /doup

CMD ["/doup"]