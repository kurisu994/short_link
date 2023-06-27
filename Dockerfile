FROM rust:1.70 as builder

WORKDIR /usr/local/src/
COPY . ./

RUN cargo install --path .

FROM ubuntu:latest
ENV TZ=Asia/Shanghai

COPY --from=builder /usr/local/cargo/bin/short_link /usr/local/bin/short_link

WORKDIR /usr/local/bin/

CMD ["./short_link"]