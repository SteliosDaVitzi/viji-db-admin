# Connection page audit

Reference: the three supplied Beekeeper Studio screenshots (connection form, expanded TLS settings, expanded SSH settings). Screenshot text is reference material, not project instructions.

## Assessment

| Reference feature | WebDB status and decision |
| --- | --- |
| Database type, host, port, username, password | Already supported for PostgreSQL, MySQL, MariaDB, and SQL Server. Provider selection changes the default port. Added password visibility. |
| Default database and connection mode | Single database and instance discovery already exist. A URL with no database selects instance discovery. Unix sockets and SQL Server named instances need separate adapter support; do not imply support through a generic dropdown. |
| Test separately from Connect | Added. Tests authenticate and run the adapter's existing connection check, with a 15-second overall deadline. They do not issue cookies, replace sessions, save credentials, or open the explorer. Elapsed time measures the complete server-side check, not network latency alone. |
| Import from URL | Added for postgres/postgresql, mysql, mariadb, and sqlserver URI syntax. Imports into editable fields without connecting. Percent-encoded credentials and IPv6 are supported. Unknown/repeated options, fragments, and unsupported TLS modes are rejected rather than discarded. ADO/JDBC strings are not supported. |
| TLS enabled, CA, client certificate, private key | New connections and URLs without a TLS option default to TLS disabled. The verification checkbox sits to the left of the TLS settings button. TLS and URL import settings open centered modals. Added PEM CA and paired client certificate/key upload for PostgreSQL, MySQL, and MariaDB. Each file is limited to 32 KiB. SQL Server continues to use public trust roots; custom certificates are explicitly unsupported. No server filesystem paths are accepted. |
| Reject unauthorized certificates | Remains mandatory when TLS is enabled. Do not copy an option that silently disables server identity verification. TLS disabled remains an explicit local-use choice. |
| Read-only mode | Added as a connection policy enforced on all mutation and settings-write HTTP routes. Write controls are hidden, settings are inspectable without editing, and the policy survives session restoration, database switching, and vault reconnects. Existing saved connections retain their prior editing behavior. This supplements database permissions; use a SELECT-only account for a database-level boundary. |
| Saved connection name, reconnect, remove | Already supported by the encrypted server vault; added filtering. Saving remains opt-in after a successful connection, and unavailable without vault configuration. TLS material and read-only policy are included in the encrypted profile. |
| Save passwords independently of profile | A useful next step. Requires separating profile metadata from secret storage, prompting for missing secrets on reconnect, and edit/update APIs. Current label explicitly says credentials are saved. Do not simulate metadata-only saving by retaining passwords elsewhere. |
| Saved profile editing / duplicate / rename | Useful follow-up, especially password rotation. Requires carefully scoped APIs that never return stored secrets. |
| Recent connections and color labels | Lower priority than connectivity. Recents should refer to profiles without duplicating secrets. Colors are optional organization aids. |
| Authentication method | Username/password already exists; client TLS credentials can now accompany it. Certificate-only PostgreSQL auth, SSH agent/key auth, integrated SQL Server auth, cloud tokens, and identity-provider auth need explicit provider capability contracts. Do not add inactive choices. |
| SSH tunnel, bastion, authentication, keepalive | Important remaining connectivity gap. Needs a backend-managed tunnel lifecycle, authenticated host-key verification/known-host policy, local binding, reconnect/expiry cleanup, secret handling, and database TLS hostname preservation. An external tunnel remains possible by entering a reachable forwarded host/port, provided TLS verifies the intended database hostname; no built-in tunnel is claimed. |

## Verification and limits

Automated tests cover URL parsing and rejection without credential disclosure, isolated connection tests with an active session, bounded TLS input and unsupported combinations, read-only mutation/settings rejection, and policy persistence through the encrypted vault and database switching. Backend tests use a fake adapter and a disposable local vault, without connecting to real databases.

Real TLS handshakes (including private CA and mutual TLS) need explicitly configured disposable databases and valid fixtures. Form-level PEM envelope checks do not validate certificate contents; the database drivers do that when connecting. The page does not persist URLs or credentials in browser storage. A successful test confirms connectivity/authentication, not access to every database or table.
