## Description

<!-- A clear description of what this PR does and why. Link to any related issues. -->

Fixes #<!-- issue number -->

---

## Type of Change

- [ ] `feat` — new feature or capability
- [ ] `fix` — bug fix (non-breaking)
- [ ] `chore` — tooling, CI, dependencies, formatting
- [ ] `docs` — documentation only
- [ ] `refactor` — code change without behavior change
- [ ] `perf` — performance improvement
- [ ] `test` — new or updated tests

**Scope** (check all that apply): `core` `cli` `app` `scanner` `classifier` `scorer` `db` `treemap` `types` `ci` `docs`

---

## Testing

- [ ] `cargo test` — all 11 unit tests pass (`cargo test 2>&1 | tail -5`)
- [ ] `cargo clippy -- -D warnings` — no warnings
- [ ] `cargo fmt -- --check` — no formatting diff
- [ ] `npm run build` (in `dmo-app/`) — frontend builds cleanly

For UI changes:
- [ ] Tested `npm run tauri dev` with a real scan on `~/Library/Caches`

---

## Safety Checklist (for classifier / scanner changes)

Skip this section if your PR does not touch `classifier.rs`, `scanner.rs`, `types.rs`, or `scorer.rs`.

- [ ] Protected categories (`UserDocument`, `AppBundle`, `Database`, `CloudSync`, `GitRepository`, `SystemCritical`) still return `type_risk = 0.00` and cannot appear in ranked waste output
- [ ] File contents are not read anywhere in the new code (metadata only)
- [ ] No network calls introduced
- [ ] New waste category tested against edge cases (Documents folder, `.git` trees, app bundles)

---

## Screenshots / Output (if applicable)

<!-- For UI changes, paste a screenshot. For CLI changes, paste a sample output diff. -->

---

## Checklist

- [ ] Branch name follows convention (`feat/`, `fix/`, `chore/`, `research/`)
- [ ] Commits follow Conventional Commits format
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
- [ ] No debug artifacts (`println!`, `console.log`, `dbg!`) in production paths
- [ ] PR description is complete and reviewable
