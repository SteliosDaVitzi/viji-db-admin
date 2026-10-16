# WebDB

A focused, read-only database viewer with a Nuxt 4 / Vue 3 / TypeScript interface and a Rust / Axum backend. PostgreSQL is the first real database provider. A clearly labeled sample workspace works without a database.

The interface has a table explorer, sortable and paginated rows, a filter for the current page, column metadata, a record inspector, clipboard copying, and current-page JSON export. Surfaces use contrast and shadows; keyboard focus uses a visible light halo.

## Run locally

Requires Node.js 24.11+ and Rust 1.95+. Docker is optional.

```sh
npm --prefix frontend ci
npm run dev
```

Open **http://127.0.0.1:34616**. The Rust API listens on `127.0.0.1:34617`. The first Rust build downloads and compiles dependencies. Alternatively run `npm run dev:backend` and `npm run dev:frontend` in separate terminals.

Use **Open sample workspace** to try browsing immediately. To connect PostgreSQL, enter its host, port, database, username, and password. TLS certificate and hostname verification are enabled by default. A local PostgreSQL instance without TLS needs the checkbox turned off. Docker connections use the host as seen from the backend container: `host.docker.internal` for a database on the Windows host, or `postgres` for the optional Compose sample service.

## Architecture

```text
frontend/
  app/components/       Connection form, data grid, record inspector, icons
  app/composables/      Typed workspace state and request lifecycle
  shared/              Browser API contracts
  server/api/          Same-origin proxy; Rust address stays server-side
backend/src/
  domain.rs            Provider-neutral inputs, table/column/page models
  application.rs       Adapter contract, provider registry, sanitized errors
  adapters/postgres.rs  PostgreSQL connection, metadata and read-only reads
  adapters/demo.rs      Deterministic sample workspace
  sessions.rs          Bounded, expiring in-memory connection sessions
  lib.rs               HTTP transport and dependency composition
  main.rs              Process configuration and shutdown
```

To add a database, implement `DatabaseAdapter`, register it in `router()`, and add any provider-specific form fields. Supported providers are fetched from `/api/providers`. No ORM, application database, migrations, or automatic changes to connected database schemas are needed.

## Commands and checks

```sh
npm run typecheck
npm run lint
npm test
npm run build
npm run test:smoke   # requires the frontend and backend to be running
npm run format
cargo install cargo-audit --locked
npm run audit
```

The real PostgreSQL integration test is explicitly opt-in. It creates and drops a randomly named test schema, checks exact numeric serialization and quoted names, and verifies that a view cannot write through a read-only transaction. Use a disposable PostgreSQL database at `127.0.0.1:55432`, database `playground`, user `postgres`:

```sh
export WEBDB_TEST_PG_URL='postgresql://postgres:YOUR_TEST_PASSWORD@127.0.0.1:55432/playground'
export WEBDB_TEST_PG_PASSWORD='YOUR_TEST_PASSWORD'
cargo test --manifest-path backend/Cargo.toml -- --include-ignored
```

PowerShell uses `$env:WEBDB_TEST_PG_URL='...'` and `$env:WEBDB_TEST_PG_PASSWORD='...'` instead of `export`.

## Docker

```sh
docker compose -f docker-compose.dev.yml up -d
docker compose -f docker-compose.prod.yml up -d --build
```

Both bind the browser entrypoint to localhost and keep the API private. Development uses bind-mounted source; restart the backend service after Rust edits. Production uses multistage builds, unprivileged users, read-only filesystems, and a frontend health check. Container images need their own vulnerability scans before deployment; dependency scans do not cover operating system packages.

To opt into a real sample database, copy `.env.example` to `.env`, set `POSTGRES_PASSWORD`, then run:

```sh
docker compose -f docker-compose.dev.yml --profile sample-db up -d
```

Connect to `postgres:5432`, database `playground`, username `webdb`, TLS off. The sample SQL runs only on the first initialization of its named volume. It never seeds a user-supplied database.

Optional Bash helpers (Git Bash, WSL, Linux or macOS):

```sh
bash scripts/rollup.sh dev          # or prod; supports --no-cache
bash scripts/rollup.sh dev seed     # explicitly enable the sample DB
bash scripts/db-inspect.sh dev
bash scripts/logs.sh dev backend
bash scripts/down.sh dev            # preserves volumes
bash scripts/down.sh dev volumes    # explicitly deletes project volumes
```

All scripts support `--help`. `migrate` reports that there are no migrations. Production sample seeding is rejected.

## Security and scope

WebDB is a local, single-user tool. The browser sends credentials only to the local API through the Nuxt proxy; no third-party services, font CDNs, analytics, or persistent browser credential storage are used. Password fields clear on submission. Credentials stay in bounded server memory, expire after eight hours, and are removed on disconnect or process exit; a periodic cleanup removes expired entries within a minute. Sessions use random opaque IDs in `HttpOnly`, `SameSite=Strict` cookies. Refreshing the page restores a valid session without returning its password.

Queries use read-only transactions, statement and lock timeouts, quoted identifiers, bound parameters, and at most 100 rows per page. Use a PostgreSQL account with only the necessary `SELECT` privileges. Large integers and decimals are returned as strings. JSON and array columns retain their database text representation to preserve nested numeric precision. Sorting uses native database values. Pagination is offset-based: concurrent changes can shift rows, and views without a sortable unique key have no guaranteed stable order. Large cell contents are returned in full; page size is not a response-byte limit.

Keep the API private. Remote/multi-user deployment requires an authenticated entrypoint, HTTPS, network access controls for target databases, and `WEBDB_SECURE_COOKIES=true`. Because connecting to private database hosts is an intentional feature, the API must not be exposed as an unauthenticated public connection proxy. Behind an HTTPS reverse proxy, pass the original Host and `X-Forwarded-Proto` consistently so same-origin validation matches the public URL. Configure `NUXT_API_BASE` for the private backend address. Run `WEBDB_BIND=0.0.0.0:34617` only on an appropriately isolated network.

See [SECURITY.md](SECURITY.md) for the measured dependency audit status and remaining upstream blockers. A clean dependency scan cannot guarantee the absence of all vulnerabilities.
