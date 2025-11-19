FROM rust:1.91-slim AS builder
ARG TARGETARCH
ARG TARGETVARIANT

WORKDIR /usr/local/src
COPY . .

RUN apt-get update && apt-get install -y --no-install-recommends musl-tools binutils && rm -rf /var/lib/apt/lists/*
RUN set -eux; \
    if [ "$TARGETARCH" = "amd64" ]; then TARGET=x86_64-unknown-linux-musl; \
    elif [ "$TARGETARCH" = "arm64" ]; then TARGET=aarch64-unknown-linux-musl; \
    elif [ "$TARGETARCH" = "arm" ] && [ "$TARGETVARIANT" = "v7" ]; then TARGET=armv7-unknown-linux-musleabihf; \
    else echo "unsupported arch: ${TARGETARCH}${TARGETVARIANT}"; exit 1; fi; \
    rustup target add "$TARGET"; \
    cargo build --release --target "$TARGET"; \
    strip "target/$TARGET/release/short_link"; \
    cp "target/$TARGET/release/short_link" "target/release/short_link"


FROM alpine:3.22
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

ENV TZ=Asia/Shanghai \
    RUST_LOG=info
COPY --from=builder /usr/local/src/target/release/short_link  /usr/app/short_link

WORKDIR /usr/app
USER appuser

EXPOSE 8008
CMD ["./short_link"]
