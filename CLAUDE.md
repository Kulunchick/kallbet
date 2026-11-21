## Working in this codebase

Read `docs/agents/codebase.md` before changing code: it says where a new API area, migration, test, page and locale string go.

Run all three before reporting work as done:

- `cargo test` (repo root): backend tests.
- `npm run build` (in `web/`): frontend typecheck and build.
- `npm run check:locales` (in `web/`): en, ru and uk must have identical keys.

## Agent skills

### Issue tracker

Issues are tracked in GitHub Issues on `Kulunchick/kallbet` (via `gh`). See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one root `GLOSSARY.md` plus `docs/adr/`. See `docs/agents/domain.md`.
