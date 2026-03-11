# Contributing to Miktos DMO

Thank you for your interest in contributing. DMO is a research-driven project at the intersection of Artificial Life, systems programming, and information visualization. Contributions that advance the science or improve correctness are especially welcome.

---

## Development Setup

### Prerequisites

- **Rust** 1.77+ with `rustfmt` and `clippy` (both installed by `rustup` by default)
- **Node.js** 18+ and **npm**
- **Xcode Command Line Tools**: `xcode-select --install`

### Building

```bash
# Clone
git clone https://github.com/LegnaPetiteTour/Miktos-DMO.git
cd Miktos-DMO

# Build + test the Rust workspace
cargo build
cargo test

# Desktop app (dev mode)
cd dmo-app
npm install
npm run tauri dev
```

All 11 unit tests must pass before submitting a PR. Do not disable or skip tests.

---

## Code Style

### Rust

- Format: `cargo fmt` (enforced in CI — must produce no diff)
- Lint: `cargo clippy -- -D warnings` (all warnings treated as errors in CI)
- Unsafe code: not permitted without explicit justification and review
- Panics: `unwrap()` / `expect()` are only acceptable in tests or cases where the invariant is provably upheld; document why

### Svelte / TypeScript

- Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`) — do not use legacy Options API
- No `any` types without a comment explaining why
- Canvas2D drawing code lives in `Treemap.svelte` — keep it functional and well-commented

### General

- No dead code merged to `main`
- No debugging artifacts (`println!`, `console.log`) in non-debug paths
- File reads: the Rust backend must only read filesystem **metadata** (path, size, mtime, atime). File **contents** must never be read. This is a hard privacy constraint.

---

## Branches

| Branch | Purpose |
|---|---|
| `main` | Stable. Always passes CI. Protected. |
| `feat/<name>` | New features |
| `fix/<name>` | Bug fixes |
| `chore/<name>` | Tooling, CI, deps, docs |
| `research/<name>` | Experimental / ALife work |

Branch names must be lowercase with hyphens.

---

## Commit Convention

This project follows [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <short description>

[optional body]

[optional footer]
```

**Types:** `feat`, `fix`, `chore`, `docs`, `test`, `refactor`, `perf`, `ci`

**Scopes:** `core`, `cli`, `app`, `scanner`, `classifier`, `scorer`, `db`, `treemap`, `types`

**Examples:**

```
feat(classifier): add XcodeCache waste category for DerivedData
fix(scorer): clamp size_weight to [0, 1] for zero-byte files
chore(ci): add cargo fmt check to CI pipeline
docs(readme): update Phase 2 roadmap preview
```

Commits should be atomic — one logical change per commit.

---

## Pull Request Process

1. Branch from `main`, use the correct branch prefix (e.g. `feat/xcode-cache-category`)
2. All CI checks must pass: `cargo test`, `cargo fmt`, `cargo clippy`, `npm run build`
3. Fill in the PR template completely — link any relevant issue
4. For new waste categories: include evidence of why the category is safe (what guarantees it cannot match user documents or app bundles?)
5. For changes to the scoring formula: include before/after scores on a known test case
6. For UI changes: include a screenshot or screen recording
7. At least one review approval is required before merge

---

## Safety Constraints (non-negotiable)

These constraints are not just implementation details — they are the trust contract with users:

1. **No filesystem writes** in Phase 0–1. The scanner and visualizer are strictly read-only.
2. **Protected categories are immutable at the classifier level.** `UserDocument`, `AppBundle`, `Database`, `CloudSync`, `GitRepository`, `SystemCritical` must always return `type_risk = 0.00`. They must never appear in a waste ranking.
3. **No network calls.** DMO has no telemetry, no update checks, no external API calls. Keep it offline-first.
4. **No reading file contents.** Metadata only: path, size, mtime, atime, inode type.

Any PR that relaxes these constraints will be closed without review.

---

## Research Contributions

DMO is grounded in peer-reviewed Artificial Life literature. If you are contributing Phase 2 Physarum simulation or Phase 4 learning loop work:

- Cite your algorithmic sources (paper, DOI, or public implementation)
- Add new citations to `Documentation/DMO_Phase2_Research_Architecture.md`
- Prefer implementations that can be verified against published benchmarks

---

## Reporting Issues

Use the GitHub Issue templates:

- **Bug report** — for incorrect behavior (wrong classification, crashes, false positives)
- **Feature request** — for new ideas

For security vulnerabilities, see [SECURITY.md](SECURITY.md).
