FROM rust:slim-buster as builder
WORKDIR /code

ENV SQLX_OFFLINE=1
COPY . .
RUN cargo b --release \
    && strip target/release/rss-transmission

# 
FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /code/target/release/rss-transmission .
ENV CONFIG_FILE /config/config.toml
ENTRYPOINT [ "./rss-transmission" ]
CMD []
