# WebDB

A database workspace with a Nuxt 4 / Vue 3 / TypeScript interface and a Rust / Axum backend. PostgreSQL supports browsing, record editing, and schema changes. A clearly labeled read-only sample workspace works without a database.

## Editing and schema changes

Browse a PostgreSQL base table and double-click a cell (or press Enter on a focused cell) to edit its value in a compact horizontal dialog. Click **Save value** or press **Ctrl/⌘ + Enter** to save. A single row click selects and highlights it, revealing **Delete row** in the table toolbar. Use **Insert record** above the table to add one. Values use PostgreSQL text input (for example `true`, a UUID, JSON text, or `{1,2}` for an array). NULL, empty text, and omitted fields are distinct. Omitted insert fields use database defaults; generated columns are maintained by PostgreSQL. Inserts, deletes, and schema changes require typing `APPLY` and confirming; individual cell edits use **Save value**. Failed changes preserve the draft.

PostgreSQL enforces foreign keys (including configured cascades), UNIQUE, NOT NULL, CHECK, exclusion constraints, column types, permissions, and triggers. Safe error messages explain failures without returning driver details. Updates and deletes lock and compare the complete browsed record before changing it; stale, deleted, or identical duplicate records are refused. This also supports tables without primary keys when the complete record is unique. Views and query results remain read only. Writes commit immediately; there is no undo. Foreign key values are entered manually and checked by PostgreSQL.

Open **Database changes** to create or alter tables, columns, indexes, and constraints using one PostgreSQL SQL statement. The dialog starts with a CREATE TABLE example. This is an explicit write workspace, separate from the read-only query editor. CREATE, ALTER, DROP, TRUNCATE, INSERT, UPDATE, and DELETE are supported inside a transaction. Destructive SQL can affect dependent objects when explicitly specified; review the entire statement before confirming. **Drop table** uses RESTRICT by default. Connect in instance mode to **Create database** or **Drop database**; switch away from a database before dropping it. The dedicated database actions protect system databases and never force-disconnect clients. Commands requiring execution outside a transaction, such as CREATE INDEX CONCURRENTLY, are unsupported in the SQL change dialog.

Record field values are bound parameters, identifiers are quoted, and operations use statement and lock timeouts. The write API requires an active session and explicit confirmation. The application remains a trusted single-user tool, not a shared authenticated database administration service. Use an account with only the privileges needed for the operations you intend to perform.

Write integration tests are opt-in: set `WEBDB_TEST_CONNECTION_JSON` to a ConnectionInput JSON object for an explicitly disposable PostgreSQL database, then run `cargo test --manifest-path backend/Cargo.toml --test postgres_write -- --ignored`. The test creates and drops randomly named schemas and a database and covers typed values, defaults, generated columns, constraints, rollback, stale records, and SQL injection boundaries. With the application running, `npm run test:write-smoke` uses the same explicit connection configuration and `WEBDB_URL` (default `http://127.0.0.1:34616`) to exercise the write API through the frontend proxy. Never point these tests at production.

The interface has a table explorer, sortable and paginated rows, a filter for the current page, column metadata, a record inspector, clipboard copying, and current-page JSON export. Surfaces use contrast and shadows; keyboard focus uses a visible light halo.

The full-width query workspace sits above the table section. Use **+** to open independent query tabs; each tab retains its draft and result page during the current connection. The editor has a scrolling line-number gutter. Click **Run** or press **Ctrl/⌘ + Enter** to run a single read-only SELECT (including CTEs, joins, and aggregates on PostgreSQL). Below it, a compact title and one toolbar hold the table views, filter, refresh, JSON export, and pagination. Results retain cell selection and a compact read-only value dialog on double-click; put ordering in your SQL. **Back to table** restores normal browsing. An ending semicolon is optional; output columns must have unique names, so alias duplicate names in joins. Queries have a 12,000-byte limit, a 10-second statement timeout, and up to 100 result rows per page. Page navigation reruns the executed query; use an explicit ORDER BY for predictable paging.

Use **Save** to name a query, **Saved** to open it in its own tab, and **Update** to change its name or SQL. Opening an already open saved query switches to its tab and preserves any draft edits. The saved list lets you remove queries. Saved queries persist only in this browser's local storage (up to 100); loading never executes them. Connection credentials and result rows are not stored with them. Avoid putting secrets in saved SQL. The sample workspace supports `SELECT * FROM public.customers` (also `orders` and `products`) with an optional `LIMIT`; connect PostgreSQL for full SQL support.

## Run locally

Requires Node.js 24.11+ and Rust 1.95+. Docker is optional.

```sh
npm --prefix frontend ci
npm run dev
```

Open **http://127.0.0.1:34616**. The Rust API listens on `127.0.0.1:34617`. The first Rust build downloads and compiles dependencies. Alternatively run `npm run dev:backend` and `npm run dev:frontend` in separate terminals.

Use **Open sample workspace** to try browsing immediately. To connect PostgreSQL, enter its host, port, username, and password. Enter a database name for a single database, or enable **Browse all databases in this instance** to hide that field. Instance mode connects through the `postgres` maintenance database (the account needs CONNECT access to it), discovers non-template databases with CONNECT permission, and lists databases as expandable branches in the left drawer. Expand several databases to inspect their schemas and tables without changing the active database. Choosing a table in another database switches the active database after checking access; table results and query tabs reset for the new database. PostgreSQL permissions still govern which tables can be read. The drawer starts at 320px on desktop; drag its right edge to resize, or focus the separator and use Left/Right arrows (Shift for larger steps), Home/End for the bounds, and double-click to reset. Width is constrained on smaller screens. TLS certificate and hostname verification are enabled by default. A local PostgreSQL instance without TLS needs the checkbox turned off. Docker connections use the host as seen from the backend container: use `host.docker.internal` for a database on the Windows host, or the reachable hostname of your existing PostgreSQL server.

## Saved connections

Enable **Save connection and credentials** to name a connection after it successfully connects. Saved connections appear on the setup screen and in **New connection**, with actions to reconnect or remove them. Reconnect runs on the backend and never returns a saved password to the browser. Disconnect ends the active session; removing a saved entry deletes its stored copy and leaves an active session running.

Saving is disabled until the backend has both `WEBDB_VAULT_KEY` (32 random bytes encoded as 64 hex characters) and `WEBDB_STORE_PATH` (a writable SQLite file in an existing private directory). Configuration errors fail startup. The database stores connection names as plaintext labels and encrypts all connection details using AES-256-GCM, fresh random nonces, and the connection ID as authenticated data. There is no browser credential persistence. Keep the encryption key separate from the SQLite file and its backups; losing the key makes saved credentials unrecoverable. Use names without secrets and restrict the directory and key to your OS account.

For local PowerShell development, start in the project root. This creates a new key only if one does not already exist; use the same private key file on later starts:

```powershell
$vaultDirectory = Join-Path $env:LOCALAPPDATA 'WebDB'
New-Item -ItemType Directory -Force -Path $vaultDirectory | Out-Null
$vaultKeyFile = Join-Path $vaultDirectory 'vault.key'
if (!(Test-Path -LiteralPath $vaultKeyFile)) {
  $vaultKeyBytes = New-Object byte[] 32
  $vaultRng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
  $vaultRng.GetBytes($vaultKeyBytes)
  $vaultRng.Dispose()
  [System.IO.File]::WriteAllText($vaultKeyFile, [BitConverter]::ToString($vaultKeyBytes).Replace('-', '').ToLowerInvariant())
}
$env:WEBDB_VAULT_KEY = [System.IO.File]::ReadAllText($vaultKeyFile).Trim()
$env:WEBDB_STORE_PATH = Join-Path $vaultDirectory 'connections.sqlite'
npm run dev
```

The dev launcher inherits these variables; it does not load them from `.env`. Both Compose files enable saved connections using `/var/lib/webdb/connections.sqlite` inside the backend, backed by separate `dev-connections` and `prod-connections` named volumes. SQLite runs in the backend process and needs no separate service. The production image prepares the volume directory for UID 10001; the rest of its filesystem stays read only. Compose requires a persistent `WEBDB_VAULT_KEY` supplied through `.env` or the process environment. Keep the same key across restarts and backups; changing it makes existing saved credentials unreadable. Keep the API private: saved connections belong to the local tool, with no separate per-user accounts or authorization.

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
  adapters/vault.rs     SQLite storage with authenticated credential encryption
  sessions.rs          Bounded, expiring in-memory connection sessions
  lib.rs               HTTP transport and dependency composition
  main.rs              Process configuration and shutdown
```

To add a database, implement `DatabaseAdapter`, register it in `router()`, and add any provider-specific form fields. Supported providers are fetched from `/api/providers`. Saved connections use a separate local SQLite application store behind the `ConnectionRepository` contract. Its schema initializes automatically; connected database schemas are never changed.

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

The real PostgreSQL integration tests are explicitly opt-in. The browsing test creates and drops a randomly named test schema, checks exact numeric serialization and quoted names, and verifies that a view cannot write through a read-only transaction. Supply your own explicitly disposable database using the same `WEBDB_TEST_CONNECTION_JSON` configuration as the write tests:

```sh
export WEBDB_TEST_CONNECTION_JSON='{"provider":"postgres","host":"127.0.0.1","port":55432,"database":"webdb_test","username":"postgres","password":"YOUR_TEST_PASSWORD","ssl":false,"allDatabases":false}'
cargo test --manifest-path backend/Cargo.toml -- --include-ignored
```

PowerShell uses `$env:WEBDB_TEST_CONNECTION_JSON='...'` instead of `export`. WebDB does not provision PostgreSQL or seed a database.

## Docker

Copy `.env.example` to `.env`. Generate a key once with the following command and set `WEBDB_VAULT_KEY` in `.env` to its output before starting either environment:

```sh
node -e "console.log(require('node:crypto').randomBytes(32).toString('hex'))"
```

```sh
docker compose -f docker-compose.dev.yml up -d
docker compose -f docker-compose.prod.yml up -d --build
```

Both bind the browser entrypoint to localhost and keep the API private. Development uses bind-mounted source; restart the backend service after Rust edits. Production uses multistage builds, unprivileged users, read-only filesystems, and a frontend health check. Container images need their own vulnerability scans before deployment; dependency scans do not cover operating system packages.

Saved connections persist in the SQLite volumes across container rebuilds and ordinary `down` commands. Back up the SQLite store and retain its encryption key separately. The built-in sample workspace uses in-memory data; connect to your own PostgreSQL server for real database operations.

Optional Bash helpers (Git Bash, WSL, Linux or macOS):

```sh
bash scripts/rollup.sh dev          # or prod; supports --no-cache
bash scripts/logs.sh dev backend
bash scripts/down.sh dev            # preserves volumes
bash scripts/down.sh dev volumes    # explicitly deletes project volumes, including saved connections
```

All scripts support `--help`. `migrate` reports that the SQLite connection-store schema initializes automatically at backend startup.

## Security and scope

WebDB is a local, single-user tool. The browser sends credentials only to the local API through the Nuxt proxy; no third-party services, font CDNs, analytics, or persistent browser credential storage are used. Password fields clear on submission. Unless you opt into saving a connection, credentials stay in bounded server memory, expire after eight hours, and are removed on disconnect or process exit; a periodic cleanup removes expired entries within a minute. Sessions use random opaque IDs in `HttpOnly`, `SameSite=Strict` cookies. Refreshing the page restores a valid session without returning its password.

Queries use read-only transactions, statement and lock timeouts, quoted identifiers, bound parameters, and at most 100 rows per page. Use a PostgreSQL account with only the necessary privileges; browsing needs `SELECT`, while explicit editing requires corresponding write privileges. Large integers and decimals are returned as strings. JSON and array columns retain their database text representation to preserve nested numeric precision. Sorting uses native database values. Pagination is offset-based: concurrent changes can shift rows, and views without a sortable unique key have no guaranteed stable order. Large cell contents are returned in full; page size is not a response-byte limit.

Keep the API private. Remote/multi-user deployment requires an authenticated entrypoint, HTTPS, network access controls for target databases, and `WEBDB_SECURE_COOKIES=true`. Because connecting to private database hosts is an intentional feature, the API must not be exposed as an unauthenticated public connection proxy. Behind an HTTPS reverse proxy, pass the original Host and `X-Forwarded-Proto` consistently so same-origin validation matches the public URL. Configure `NUXT_API_BASE` for the private backend address. Run `WEBDB_BIND=0.0.0.0:34617` only on an appropriately isolated network.

See [SECURITY.md](SECURITY.md) for the measured dependency audit status and remaining upstream blockers. A clean dependency scan cannot guarantee the absence of all vulnerabilities.
