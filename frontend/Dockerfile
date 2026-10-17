FROM cgr.dev/chainguard/node:latest-dev AS build
WORKDIR /app
COPY --chown=65532:65532 package*.json ./
COPY --chown=65532:65532 vendor ./vendor
USER 65532
RUN npm ci
COPY --chown=65532:65532 . .
RUN npm run build
FROM cgr.dev/chainguard/node:latest
WORKDIR /app
COPY --from=build --chown=65532:65532 /app/.output ./.output
ENV HOST=0.0.0.0 PORT=34616
EXPOSE 34616
CMD [".output/server/index.mjs"]
