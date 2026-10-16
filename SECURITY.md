# Security policy

## Reporting a vulnerability

Do not disclose vulnerabilities, credentials, private connection strings, or real
database records in public issues. Use GitHub's **Report a vulnerability** option
on the repository Security tab when enabled. Otherwise email
**steliosvitzi@gmail.com** with the subject **WebDB security report**.

Include the affected commit/version, provider, deployment method, impact, and a
minimal reproduction with synthetic data. Maintainers review reports as time
permits; there is no guaranteed response time or paid bug bounty. Please coordinate
public disclosure while a report is investigated.

## Supported versions and threat model

Security fixes target the latest default-branch code. No older release branch has
a promised support window. WebDB is early-development software for a trusted,
single-user workstation, with no built-in multi-user authentication or per-user
authorization. Session cookies are not an independent login system.

- Keep both the frontend and API on loopback or a trusted isolated network.
  Remote use requires authentication at the entrypoint, HTTPS, network controls,
  and `WEBDB_SECURE_COOKIES=true`. Never expose the backend directly to the public.
- The API intentionally connects to user-selected hosts. It is not an SSRF-safe
  public proxy. All locally saved connections belong to the same installation.
- TLS is disabled by default. Enable verified TLS on untrusted networks.
- Query parsing and read-only transactions where supported reduce accidental
  writes; they do not replace database privileges. SQL Server has no read-only
  transaction mode. Database functions, triggers, extensions, and remote systems
  may have side effects outside ordinary transaction rollback.
- Use SELECT-only accounts for browsing. Explicit writes commit immediately and
  can destroy data; keep tested backups. MySQL/MariaDB DDL implicitly commits.
- Credentials stay in bounded server memory unless explicitly saved to the
  encrypted vault. Restrict its directory and keep the key separate from backups.
  Saved SQL, exports, clipboard contents, browser extensions, OS memory/dumps,
  database logs, and reverse-proxy logs need their own protection.
- Pagination limits rows, not bytes. Large values and expensive queries can still
  consume memory and database resources. This is not a hostile multi-tenant service.

## Dependency review — 17 October 2026

Results are dated observations, not a security guarantee:

| Check | Result |
| --- | --- |
| `npm --prefix frontend audit --offline=false --audit-level=low` | FAILED: 11 high-severity findings in the development/build dependency tree |
| `cargo audit --file backend/Cargo.lock` | Passed; no vulnerabilities or warnings reported; 241 dependencies, 1,329 RustSec advisories loaded |
| `npm audit --prefix frontend/.output/server --offline=false` | 0 known vulnerabilities after generating the production-server deployment lockfile |

The online scan confirms the earlier findings. Two vulnerable transitive packages
are pulled in by the Nuxt/Nitro development and build toolchain:

- [braces 3.0.3: stack-exhaustion denial of service](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm).
- [node-forge 1.4.0: RSA signature verification bypass](https://github.com/advisories/GHSA-86w9-cpqp-85rv).

Both are still the latest published versions as checked on this date; the
advisories list no patched versions. WebDB does not expose user-controlled glob
patterns or use Forge signature verification directly. This does not remove the
findings. Do not force-downgrade Nuxt to the audit suggestion without a separate
compatibility review. No package versions were changed or advisories suppressed.

An initial sandbox cache-only scan misleadingly returned zero findings; that
result was discarded after an explicit online scan. The audit command and CI now
use `--offline=false`. Never accept a production scan of an empty dependency tree:
the final generated server lockfile resolved 86 packages during installation.

CI audits the complete frontend dependency tree and Rust lockfile independently,
including on a weekly schedule; failures are not ignored. To reproduce:

```sh
npm --prefix frontend ci
cargo install cargo-audit --locked
npm run audit
```

Generated production-server dependencies can be scanned separately after a build:

```sh
npm install --prefix frontend/.output/server --package-lock-only --ignore-scripts --offline=false
npm audit --prefix frontend/.output/server --offline=false
```

Scans cover known dependency advisories, not application penetration testing,
all transitive license obligations, browser behavior, or Docker base-image
vulnerabilities. Scan container images separately before distributing/deploying
them. Never force incompatible dependency downgrades or hide audit findings.
