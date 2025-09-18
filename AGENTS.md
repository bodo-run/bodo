# General rules

* Always make sure test, clippy and fmt passes before committing 
* Write tests for added functionalities 
* Prefer clean and simple solutions and avoid over complicating things

# Rust rules
* No warnings: treat compiler and clippy warnings as errors in CI. Either fix or justify with a targeted allow and a short comment.
* Avoid `unwrap`/`expect`/`panic!` in non-test code. Return `Result` and bubble up errors. A single well-justified `expect` at startup is acceptable with a clear message.
* `unsafe` is forbidden unless absolutely necessary. If used, isolate it, document invariants, add tests, and link to justification.
* Error handling: use `thiserror` for library error types; use `anyhow` in binaries for flexible error contexts. Include actionable `Display` messages.
* Logging/observability: use `tracing` (not `println!`) for logs. Provide levels (`error`, `warn`, `info`, `debug`, `trace`) and support `RUST_LOG`/env-filter. Keep user-facing CLI output clean and deterministic.
* CLI UX: use `clap` for args. Provide `--help` and `--version`, sensible defaults, clear error messages, non-zero exit codes on failure, and `--quiet`/`--verbose` flags. CLI args > env vars > config files for precedence.
* Public API hygiene: document all public items with rustdoc. Include examples that compile (`rustdoc --test`). Keep the public surface minimal and stable.
* Dependency discipline: keep deps lean. Prefer small, well-maintained crates. Audit regularly (`cargo deny`, `cargo audit`). Optional deps must be behind feature flags; default features should be minimal.
* Lints/style: enable useful clippy groups. Justify any `allow`. Keep formatting standard with rustfmt; do not customize style without team agreement.
* Project structure: share logic in `src/lib.rs`; keep `src/main.rs` thin. Place integration tests in `tests/` and examples in `examples/`. Avoid large god-modules; prefer small, cohesive modules.
* Tests: aim for fast, deterministic tests. Use `assert_cmd`/`trycmd` for CLI tests, temp dirs for filesystem tests, and avoid network by default (mock when feasible). Add regression tests for bugs.
* Performance: measure before optimizing. Use iterators over allocations, avoid needless `clone`, and prefer streaming I/O. Add microbenchmarks with `criterion` for hot paths when changes are performance-motivated.
* Concurrency: be explicit about sync vs async. Avoid blocking in async contexts. Ensure types are `Send + Sync` where needed and prefer channels/async primitives over shared mutability.
* MSRV and portability: define `rust-version` in `Cargo.toml`. Don’t use features beyond MSRV without bumping it. Support macOS and Linux; avoid platform-specific assumptions.
* Security: never log secrets or PII. Validate and normalize untrusted input and paths. Use restrictive file permissions where relevant. Prefer safe wrappers to raw syscalls.
* Versioning/release: follow SemVer. Update CHANGELOG and bump crate version with any public API change. Keep `--version` in sync.
* Config: prefer explicit configuration with clear defaults. Document all environment variables and config keys. Validate config at startup with helpful errors.
* CI expectations: run `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all --all-features` on every PR. Consider `cargo deny` and `cargo audit` in CI.

# Engineeering best practices
* Code reviews: prefer small, focused PRs with clear intent, rationale, and linked issues. Include a checklist (tests, docs, backward-compat, error handling, logs, perf implications). Address all comments or explain why not. At least one reviewer approval required.
* Commits: use Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `perf:`, `test:`, `build:`, `ci:`, `chore:`, `revert:`). Keep commits atomic and messages meaningful; wrap body at ~72 chars.

# Local development
* Use `just` or `make` targets for common workflows (`fmt`, `clippy`, `test`, `audit`). Prefer reproducible commands over ad-hoc scripts.
* Pin toolchain with `rust-toolchain.toml`. Keep MSRV aligned with `Cargo.toml` `rust-version`.
* Run `cargo test --all --all-features` locally before pushing. Use `--nocapture` for debugging test output sparingly.
* Prefer `cargo nextest` for faster local runs when available; CI remains the source of truth.

# Error messages & diagnostics
* Errors should be actionable: say what failed, why, and how to fix. Include a short hint when feasible.
* Wrap external I/O and parsing with context (`anyhow::Context`) to preserve root causes.
* Surface user-facing diagnostics cleanly; reserve stack traces for `-vv` or debug builds.
* Use exit codes consistently: `2` for usage errors, `1` for runtime failures, `0` for success.

# Tracing & metrics
* Initialize `tracing` once at startup with env-filter support; honor `--verbose`/`--quiet` flags.
* Add spans around network/FS boundaries and expensive operations. Record key fields (sizes, durations, counts) without PII.
* When metrics are needed, use a lightweight client gated behind a feature flag. Avoid adding heavy deps by default.

# Feature flags & builds
* Keep default features minimal and safe. Place optional integrations behind named features with clear docs.
* Provide `--no-default-features` builds in CI to catch coupling issues.
* Ensure features are additive and orthogonal; avoid hidden feature interactions.

# CLI UX patterns
* Prefer subcommands for distinct actions; flags modify behavior, not meaning.
* Stable, deterministic output by default; provide `--json` or `--yaml` for machine-readability.
* Validate inputs early with clear messages. Support `--dry-run` for destructive operations.
* Keep help text concise with examples. Include `EXAMPLES` section when non-trivial.

# Testing conventions
* Unit tests live alongside code; integration tests in `tests/`. Use realistic fixtures and temp dirs.
* For CLI, use `assert_cmd` and `predicates` or `trycmd` to verify stdout/stderr and exit codes.
* Avoid time and network flakiness: prefer fakes/mocks; freeze time where needed.
* Add regression tests for every bug fix with a reference to the issue.

# Security & privacy
* Treat all file paths and input as untrusted: normalize and restrict traversal.
* Redact secrets in logs and error messages. Never echo tokens by default.
* Use permissions narrowly when creating files (`0o600` for secrets). Avoid world-writable dirs.

# Performance playbook
* Profile before optimizing; set hypotheses and compare with benchmarks.
* Favor iterators and slices; avoid needless heap allocations and clones.
* Stream large I/O; avoid loading entire files unless necessary. Consider backpressure.

# Release & packaging
* Build reproducible binaries with locked dependencies (`-Z minimal-versions` in a periodic job, `cargo update -p` as needed).
* Produce checksums and SBOM where applicable. Sign releases if feasible.
* Keep `--version` output in sync with `Cargo.toml` and include git rev when building from non-tagged commits.
* Branching/merges: trunk-based with short-lived feature branches. Rebase onto main before merge; prefer fast-forward/squash to keep history clean. Keep main always green.
* Documentation: update `README.md`, `USAGE.md`, `CHANGELOG.md`, and rustdoc with each change. Keep examples runnable and tested. Record significant decisions as lightweight ADRs in `docs/adr/`.
* Testing strategy: pyramid of unit → integration → CLI. Use temp dirs for FS tests; avoid real network and time dependencies (mock/fixture instead). Add regression tests for every bug. Keep tests fast and deterministic.
* CI discipline: fail fast and keep pipelines under a few minutes. Cache builds prudently. Required checks: fmt, clippy (deny warnings), tests. Optional but recommended: `cargo deny` and `cargo audit`.
* Observability: structured logs via `tracing` with appropriate levels; avoid logging PII/secrets. Add spans around I/O and external calls. Gate verbosity behind `-v/-q`. Provide `--json` for machine-readable output when relevant.
* Security: never commit secrets; provide `.env.example` and document required vars. Validate and normalize all untrusted inputs and paths. Use least-privilege file perms. Audit deps regularly and pin MSRV/toolchain.
* Reproducibility/tooling: check in `rust-toolchain.toml`. Provide `just`/`make` tasks for common workflows. Scripts must be idempotent and cross-platform (macOS/Linux). Ensure deterministic, sorted outputs where applicable.
* UX and safety: default to safe operations; support `--dry-run` and `--yes`/`--no-confirm`. Use atomic writes and backups for destructive file operations. Prefer clear, actionable error messages with hints.
* Performance: measure before optimizing; use `criterion` and profiling tools (e.g., `cargo flamegraph`). Avoid needless allocations/copies; stream where possible. Set and monitor performance budgets for hot paths.
* Release process: follow SemVer; tag releases (`vX.Y.Z`), update `CHANGELOG.md` and verify `--version`. Provide migration notes for breaking changes and deprecations with timelines.
* Backward compatibility: avoid breaking CLI flags and output formats. When changes are necessary, add deprecation warnings, feature flags/compat modes, and provide a migration guide.
* Issue hygiene: use templates, labels, and milestones. Keep issues small and outcome-focused. Link PRs to issues and close them automatically when merged.
* Tech debt & TODOs: track TODOs with issue references (e.g., `TODO(#123)`) or create follow-up issues before merging. Avoid long-lived `allow` annotations without justification.