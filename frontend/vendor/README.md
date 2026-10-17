# Maintained dependency patches

These local packages replace vulnerable upstream releases in the frontend build
tree. Their original licenses and copyright notices remain in each package.

| Package | Upstream base | Local version | Advisory | Maintained fix |
| --- | --- | --- | --- | --- |
| `braces` | 3.0.3 | 3.0.4 | [GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm) | Bounds parser nesting and recursive AST walks at 100 levels; rejects cyclic parent chains in expansion. |
| `node-forge` | 1.4.0 | 1.4.1 | [GHSA-86w9-cpqp-85rv](https://github.com/advisories/GHSA-86w9-cpqp-85rv) | Requires the nested `DigestAlgorithm` sequence to contain only its OID and optional NULL parameters. |

The `braces` hardening follows the reviewed changes proposed in
[micromatch/braces#72](https://github.com/micromatch/braces/pull/72), with depth
checks retained across parse, compile, expand, and stringify. The `node-forge`
check follows the upstream proposal in
[digitalbazaar/forge#1152](https://github.com/digitalbazaar/forge/pull/1152).
Both patches stay local until upstream publishes fixed releases. The regression
tests live in `frontend/tests/dependency-hardening.test.mjs`.
