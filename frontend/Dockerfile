FROM node:24-bookworm-slim AS build
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build
FROM node:24-bookworm-slim
WORKDIR /app
COPY --from=build --chown=node:node /app/.output ./.output
USER node
ENV HOST=0.0.0.0 PORT=34616
EXPOSE 34616
CMD ["node", ".output/server/index.mjs"]
