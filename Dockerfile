FROM rust:1.70 as builder

# user github-action don't need this config
#COPY ./.config/config.toml /usr/local/cargo/config.toml

WORKDIR /usr/local/src/
COPY . ./

RUN cargo install --path .

FROM ubuntu:latest
ENV TZ=Asia/Shanghai

COPY --from=builder /usr/local/cargo/bin/short_link /usr/local/bin/short_link

WORKDIR /usr/local/bin/

CMD ["./short_link"]