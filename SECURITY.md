# Dependency review — 2026-10-16

The requested zero-vulnerability acceptance criterion is **not yet met** for the complete development dependency tree.

- `cargo audit --file backend/Cargo.lock`: **0 vulnerabilities, 0 warnings** against RustSec, with no ignored advisories.
- `npm audit --prefix frontend/.output/server` after generating its deployment lockfile: **0 vulnerabilities** across the generated production server dependencies.
- `npm audit --prefix frontend`: **11 high-severity findings**, arising from two unpatched transitive packages and their dependent packages. No audit filters or advisory suppressions have been added.

The current Nuxt 4.6.1 / Nitro 2.13.4 toolchain brings in:

1. `braces@3.0.3`, via `micromatch`, `fast-glob`, and `globby`: recursive pattern parsing can exhaust the stack. [GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm) lists no patched release.
2. `node-forge@1.4.0`, via `listhen`: malformed RSA signature structures can bypass verification. [GHSA-86w9-cpqp-85rv](https://github.com/advisories/GHSA-86w9-cpqp-85rv) lists no patched release.

The application does not accept user-controlled glob patterns or use Forge signature verification. These packages are still present in the build/development toolchain and the audit remains failing. Do not describe this project as having zero vulnerabilities.

To reproduce the production artifact scan after `npm run build --prefix frontend`:

```sh
npm install --prefix frontend/.output/server --package-lock-only --ignore-scripts
npm audit --prefix frontend/.output/server
```

Recheck upstream releases and upgrade when fixes are available, then rerun type checking, linting, backend tests, production build, smoke checks, and both dependency audits. Do not force-downgrade Nuxt to satisfy an audit suggestion without checking compatibility and the resulting tree. Do not relabel vulnerable package versions or suppress advisories to obtain a green result.

Scans cover known dependency advisories, not application penetration testing, browser behavior, or Docker base-image vulnerabilities. The application defaults to loopback access and is intended for a trusted single-user workstation.
