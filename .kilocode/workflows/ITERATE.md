# ITERATE.md

Roadmap-driven workflow for planning, implementing, reviewing, and merging changes.

## Overview

This loop keeps development aligned with the roadmap while maintaining high code quality.
It complements the rules in `AGENTS.md` (CI, testing, commits, and review discipline).

## Prerequisites

- `git` and GitHub CLI (`gh`) installed and authenticated: `gh auth status`
- Rust toolchain installed; be able to run `cargo` locally
- Familiarity with `AGENTS.md` guidelines (tests, clippy, fmt, commits)

## Iteration Loop

Repeat forever:

1. Review `ROADMAP.md`: confirm current milestone and next actionable task.
2. Create a focused branch for the task.
3. Plan changes carefully; update or add tests first when feasible (TDD encouraged).
4. Implement the change with a clean, minimal diff.
5. Ensure CI will pass locally (fmt, clippy, tests) and commit using Conventional Commits.
6. Open a PR with clear description, context, and links to roadmap items.
7. Address code reviews; iterate until approvals and green checks.
8. Update `ROADMAP.md` to mark the task as done (and note follow-ups if any).
9. Merge via `gh` (use squash unless otherwise agreed), then delete the branch.
10. Switch back to `main`, pull latest, and pick the next task.

## Helpful Commands (fish shell)

- Sync `main` and branch for a task:
	```fish
	git fetch origin
	git switch main
	git pull --ff-only
	set -l task "feat/short-task-name"
	git switch -c $task
	```

- Run local CI checks (required before PR):
	```fish
	cargo fmt --all --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --all --all-features
	```

- Conventional Commits examples:
	```
	feat: add X to improve Y
	fix: handle Z edge case in parser
	docs: clarify setup in README
	```

- Create a PR (fills from commits and editor if needed):
	```fish
	gh pr create --fill --base main --head (git branch --show-current)
	```

- View/monitor PR:
	```fish
	gh pr view --web
	```

- Merge once approved and green (squash + delete branch):
	```fish
	gh pr merge --squash --delete-branch
	```

- After merge, get ready for next task:
	```fish
	git switch main
	git pull --ff-only
	```

## Pre-PR Checklist

- [ ] Code compiles with no warnings; clippy clean (`-D warnings`).
- [ ] `cargo fmt --all` produces no diffs.
- [ ] Tests added/updated; `cargo test --all --all-features` passes locally.
- [ ] Error handling uses `thiserror`/`anyhow` patterns where appropriate.
- [ ] No `unwrap`/`expect`/`panic!` in non-test code (unless justified at startup).
- [ ] Logging via `tracing` (no stray `println!`).
- [ ] Public APIs documented; examples compile if added.
- [ ] Commit messages follow Conventional Commits.
- [ ] PR description references the relevant `ROADMAP.md` item and provides rationale.

## Code Review Loop

- Be responsive and iterate in small, focused commits.
- Provide rationale when disagreeing; link to docs or measurements.
- Keep the PR scope tight; defer extras to follow-ups.
- Update `ROADMAP.md` when the PR lands; create follow-up issues for TODOs.

## Notes

- Prefer simple, clean solutions over cleverness.
- Avoid unrelated refactors in feature/fix PRs; use separate PRs.
- Keep the main branch green; rebase if the base has moved.
