FROM rust:1.70 as builder

# user github-action don't need this config
COPY ./.config/config.toml /usr/local/cargo/config.toml

WORKDIR /usr/local/src/
COPY . ./

RUN apt-get update && apt-get install -y default-mysql-client
RUN cargo install diesel_cli --no-default-features --features "mysql"
RUN cargo install --path .

FROM ubuntu:latest
ENV TZ=Asia/Shanghai

COPY --from=builder /usr/local/cargo/bin/short_link /usr/local/bin/short_link

WORKDIR /usr/local/bin/

CMD ["./short_link"]