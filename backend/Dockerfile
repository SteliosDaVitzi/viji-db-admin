FROM rust:1.98-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 webdb
RUN install -d -m 0700 -o webdb -g webdb /var/lib/webdb
COPY --from=build /app/target/release/webdb /usr/local/bin/webdb
USER webdb
ENV WEBDB_BIND=0.0.0.0:34617
EXPOSE 34617
CMD ["webdb"]
