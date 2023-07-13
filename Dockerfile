FROM rust:1.70 as builder

RUN USER=root cargo new --bin short_link
# user github-action don't need this config
COPY ./.config/config.toml /usr/local/cargo/config.toml

WORKDIR ./short_link
COPY ./Cargo.toml ./Cargo.toml
# Build empty app with downloaded dependencies to produce a stable image layer for next build
RUN cargo build --release

# Build web app with own code
RUN rm src/*.rs
ADD . ./
RUN rm ./target/release/deps/short_link*
RUN cargo build --release


FROM ubuntu:latest

ARG APP=/usr/app

EXPOSE 8008

ENV TZ=Asia/Shanghai

COPY --from=builder /short_link/target/release/short_link ${APP}/short_link

RUN ln -sf /usr/share/zoneinfo/$TZ /etc/localtime \
    && echo $TZ > /etc/timezone

WORKDIR ${APP}

CMD ["./short_link"]
