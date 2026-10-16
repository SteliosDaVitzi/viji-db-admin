# WebDB conventions

- Frontend: Nuxt 4, Vue Composition API, strict TypeScript; use `app/` conventions.
- Backend: Rust, Axum, SQLx. Do not add .NET projects.
- Dependencies point inward: HTTP routes → application contracts ← database adapters.
- Add providers through `DatabaseAdapter` and register them in the composition root.
- Browsing stays read only. Parameterize values and quote every dynamic SQL identifier.
- Never log credentials or return raw driver errors. Never persist credentials in browser storage.
- Keep dependency lockfiles committed. Do not suppress audit findings or misrepresent scan results.
- Run `npm run typecheck`, `npm run lint`, `npm test`, and build checks for relevant changes.
- Real database tests require an explicitly configured disposable database.
