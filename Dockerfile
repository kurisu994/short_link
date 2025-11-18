FROM rust:1.91-alpine AS builder

# 安装构建依赖
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    postgresql-dev \
    openssl-dev \
    gcc

RUN USER=root cargo new --bin short_link

WORKDIR ./short_link
COPY ./Cargo.toml ./Cargo.toml
# Build empty app with downloaded dependencies to produce a stable image layer for next build
RUN cargo build --release

RUN rm src/*.rs
ADD . ./
RUN rm ./target/release/deps/short_link*
RUN cargo build --release


FROM alpine:3.22

ARG APP=/usr/app

# 安装运行时依赖
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    && rm -rf /var/cache/apk/*

# 创建非root用户
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

EXPOSE 8008

ENV TZ=Asia/Shanghai

COPY --from=builder /short_link/target/release/short_link ${APP}/short_link

RUN ln -sf /usr/share/zoneinfo/$TZ /etc/localtime \
    && echo $TZ > /etc/timezone \
    && chown -R appuser:appgroup ${APP}

WORKDIR ${APP}
USER appuser

CMD ["./short_link"]
