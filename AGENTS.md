# Repository Guidelines

## Pre-Generation Invariants (read BEFORE writing any code)

These rules are enforced by CI (`cargo clippy -D warnings`, Biome). Fixing them afterwards is wasted effort — emit code in the correct shape the FIRST time. Every CI failure caused by one of these rules means the agent didn't read this section.

### Zero-tolerance rules
- **Default to no code comments. Add a comment only after solving a bug or working through a complex issue, and only when it captures non-obvious context that a future investigator or reviewer genuinely needs** — e.g. why the fix looks the way it does, the upstream/platform bug being worked around, a non-obvious invariant or trade-off chosen after investigation, or a link to the PR/issue that explains the decision. Bad cases that remain banned: narrating what the code does, restating types, JSDoc that paraphrases parameter names, "TODO: refactor" or "this should be cleaner" notes, and any comment that just describes the change you are currently making. When in doubt, prefer better naming/types over a comment. Applies to every language: Rust, TS, JS, Python, shell, SQL, TOML, etc.
- **Never hand-edit generated files**: `**/tauri.ts`, `apps/desktop/src-tauri/gen/**`, `packages/ui-solid/src/auto-imports.d.ts`. These are regenerated (e.g. `tauri.ts` only on debug desktop runs) but stay committed because CI typecheck and fresh clones depend on them; commit binding changes alongside the Rust change that produced them. Generating and committing these artifacts is required; modifying generated output by hand is prohibited. Note: `apps/desktop/src/utils/queries.ts` is hand-written, not generated — edit it normally.
- **Never start additional dev servers** (`bun run dev:desktop`). Assume it is already running. Do not launch a native app against the user's existing profile.

### Post-edit checks (run before you say "done")
- Prefer scoped, fast checks over full workspace gates. Do not run long full-repo checks by default.
- Touched any Rust file → `cargo fmt --all` and `cargo check -p <crate>`. Add `--all-targets`, `--workspace`, or clippy only when explicitly requested, when preparing CI/PR final validation, or when the change needs broader coverage.
- Touched any TS / JS / JSON / CSS / MD file → run the narrowest applicable formatter/linter on touched files first, such as `bun run biome check --write <files>`. Use full `bun run format`, `bun run lint`, and `bun run typecheck` only when explicitly requested or when the change spans shared types/packages.

### Rust — write the clippy-clean form the FIRST time
All patterns below are `deny` in the workspace `[workspace.lints]` in `Cargo.toml`. Do not emit the left column; always emit the right column.

| ❌ Don't write | ✅ Write instead | Lint |
|---|---|---|
| `dbg!(x)` | `tracing::debug!(?x)` (or delete it) | `dbg_macro` |
| `let _ = async_fn();` | `async_fn().await;` or `tokio::spawn(async_fn());` | `let_underscore_future` |
| `a - b` for `Duration`/`Instant` | `a.saturating_sub(b)` | `unchecked_time_subtraction` |
| `if a { if b { … } }` | `if a && b { … }` | `collapsible_if` |
| `x.clone()` when `x: Copy` | `x` | `clone_on_copy` |
| `iter.map(\|x\| foo(x))` | `iter.map(foo)` | `redundant_closure` |
| `fn f(v: &Vec<T>)` / `fn f(s: &String)` | `fn f(v: &[T])` / `fn f(s: &str)` | `ptr_arg` |
| `v.len() == 0` / `v.len() > 0` | `v.is_empty()` / `!v.is_empty()` | `len_zero` |
| `let _ = unit_returning();` | `unit_returning();` | `let_unit_value` |
| `opt.unwrap_or_else(\|\| 42)` (cheap default) | `opt.unwrap_or(42)` | `unnecessary_lazy_evaluations` |
| `for i in 0..v.len() { v[i] … }` | `for item in &v { … }` or `.iter().enumerate()` | `needless_range_loop` |
| `value.min(max).max(min)` | `value.clamp(min, max)` | `manual_clamp` |

Additionally, `unused_must_use = "deny"` applies to all Rust code: every `Result`, `Option`, and `#[must_use]` value must be explicitly handled (`?`, `.unwrap()`, `.ok()`, `let _ = …;` **is not allowed** for unit-returning calls — see `let_unit_value`; it is the correct escape hatch for `Result`-returning calls you consciously discard, e.g. `let _ = tx.send(msg);`).

### TypeScript / JavaScript — write the Biome-clean form the FIRST time
`biome.json` at repo root enforces (do not override locally):
- **Indent: tab.** Not two spaces, not four spaces. New files and edits must use tabs.
- **Quotes: double.** `"foo"`, never `'foo'`, for JS/TS string literals.
- **`organizeImports: on`** — imports are sorted/grouped automatically; don't leave unused imports or hand-sort against the grain.
- **Recommended lint ruleset is on**, with `suspicious.noShadowRestrictedNames` disabled. Everything else (unused vars, `noExplicitAny`, dead code, etc.) applies.
- Desktop code under `apps/desktop/**` has a11y rules disabled; they are enforced everywhere else (`packages/ui-solid`).
- CSS overrides: `noUnknownAtRules`, `noUnknownTypeSelector`, `noDescendingSpecificity` are off for `**/*.css`.

### TypeScript — strictness
- Avoid `any`. Use `unknown` + narrowing, or existing shared types from the generated bindings.
- Do not introduce `@ts-expect-error` / `@ts-ignore` without a concrete reason. Prefer fixing the type.

## Project Structure & Modules
- Turborepo monorepo:
  - `apps/desktop` (Tauri v2 + SolidStart), `apps/cli` (Rust CLI and export sidecar).
  - `packages/*` shared libs (`ui-solid`, `tsconfig`, `config`).
  - `crates/*` Rust media/recording/rendering/camera crates.
  - `scripts/*` for build tooling.

## Build, Test, Develop
- Install: `bun install --frozen-lockfile`; setup: `bun run cap-setup`.
- Dev: `bun run dev:desktop`.
- Build: `bun run build` (Turbo). Desktop release: `bun run tauri:build`.
- Quality: `bun run lint`, `bun run format`, `bun run typecheck`. Rust: `cargo build -p <crate>`, `cargo test -p <crate>`.

## Coding Style & Naming
- TypeScript / JS / JSON / CSS: **tab indent** and **double-quoted** strings, enforced by Biome (see `biome.json`). Do not configure per-file overrides.
- Rust: `rustfmt` default style + the denied clippy lints in the Pre-Generation Invariants above.
- Naming: files kebab‑case (`user-menu.tsx`); React/Solid components PascalCase; hooks `useX`; Rust modules snake_case; crates kebab‑case.
- Runtime: Node 20, Bun 1.4.x, Rust 1.88+.

(See **Pre-Generation Invariants** at the top of this file for the comments policy and the denied clippy/Biome patterns. Those are the source of truth — do not duplicate or weaken them here.)

## Testing
- TS/JS: Vitest where present (e.g., desktop). Name tests `*.test.ts(x)` near sources.
- Rust: `cargo test` per crate; tests in `src` or `tests`.
- Prefer unit tests for logic and light smoke tests for flows; no strict coverage yet.

## Commits & PRs
- Conventional style: `feat:`, `fix:`, `chore:`, `improve:`, `refactor:`, `docs:` (e.g., `fix: hide watermark for pro users`).
- Never add `Co-authored-by` or other attribution trailers (Cursor, Cursor Agent, or anyone else). Commit as the user only.
- PRs: clear description, linked issues, screenshots/GIFs for UI, env/migration notes. Keep scope tight and update docs when behavior changes.

## Agent‑Specific Practices
- Do not start extra servers; use `bun run dev:desktop` as needed.
- Prefer existing scripts and Turbo filters over ad‑hoc commands; clear `.turbo` only when necessary.
- Keep secrets out of VCS; configure via `.env`.
- macOS note: desktop permissions (screen/mic) apply to the terminal running `bun run dev:desktop`.
- All other agent-facing rules (comments policy, no editing generated files, clippy/Biome shape, post-edit gates) live in **Pre-Generation Invariants** at the top of this file.

## Deep Investigation Default
When asked to inspect, review, optimize, secure, or fix something, do not stop at the obvious local change. First trace the full path and run a second-pass blast-radius review:

- identify the real root cause, not only the symptom
- trace callers, side effects, async/runtime behavior, generated artifacts, caches, exports, old data, and platform-specific paths
- compare old vs new behavior when reviewing a diff
- call out what is verified vs merely plausible
- consider likely follow-up reviewer or user reports before calling it done
- verify the actual user-visible outcome where practical, not only compile/lint success

Prefer the smallest correct fix, but only after checking whether the narrow fix misses related consequences.

## Code Formatting & Lint Checks
Before declaring any task complete, the agent should run the fastest useful check for every file type it touched and report anything skipped.

- **Rust**: `cargo fmt --all` and `cargo check -p <crate>` for the touched crate. Add `--all-targets`, `--workspace`, or `cargo clippy -p <crate> --all-targets -- -D warnings` only for explicit requests, CI/PR final validation, or changes that need broader coverage.
- **TS / JS / JSON / CSS / MD**: prefer scoped checks such as `bun run biome check --write <files>`. Use full `bun run format`, `bun run lint`, and `bun run typecheck` only when explicitly requested or when the change is broad enough to justify it.
- If a scoped check fails, fix the violation in the source (do NOT suppress with `#[allow(...)]`, `// biome-ignore`, or `any` unless explicitly approved). The Pre-Generation Invariants show the correct form for every denied lint.
