FROM cgr.dev/chainguard/rust:latest-dev AS build
WORKDIR /work
COPY --chown=65532:65532 Cargo.toml Cargo.lock ./
COPY --chown=65532:65532 src ./src
USER 65532
RUN --mount=type=cache,target=/home/nonroot/.cargo/registry,uid=65532,gid=65532 cargo build --release --locked
RUN mkdir -m 0700 /work/webdb-data
FROM cgr.dev/chainguard/glibc-dynamic:latest
COPY --from=build --chown=65532:65532 /work/target/release/dbsaur /usr/local/bin/dbsaur
COPY --from=build --chown=65532:65532 /work/webdb-data /var/lib/webdb
USER 65532
ENV WEBDB_BIND=0.0.0.0:34617
EXPOSE 34617
CMD ["dbsaur"]
