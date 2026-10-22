# REPORT: infra-loop-harness, claude-44 (widen PR 8's scope to the view renderer)

Lab: claude. Base: `8d488457e`. Order: [ORDER.md](ORDER.md), committed first. The cancel probe found `looprender.rs` absent from the PR 8 item's `scope:`, so the task stands. No open PR changes `looprender.rs`.

## What did not work, first

- **The renderer's bug was invisible until PR 8 rendered a root-level view.** The PR-3 renderer's template assumed every view lives under `docs/`. In the repo-root `HANDOFF.md` it writes `../../loop/README.md`, which escapes the repository. Kimi's `loop-migrate` verifier caught it, and the docs links gate fails on PR 8.
- **PR 8's own claim cannot widen its scope.** As with #2991, the scope edit has to come from another lab.

## What changed

| Commit | What |
|---|---|
| ORDER commit | This ORDER, alone. |
| `19f8f0eea` | `docs/roadmap/items/infra-loop-pr08-migration.md`: `crates/uor-r4-harness/src/looprender.rs` is appended to `scope:`. The diff is one line. |
| this REPORT's commit | This REPORT. |

No code. The one-word link fix stays Kimi's, in PR 8, with its re-render.

## Measurements

None.

## Checks run at the head

- `cargo xtask roadmap check`: 88 items OK.
- `roadmap render --check`: current.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one line of one item file.

## Still open

- **PR 8 merges `main` after this lands,** then fixes the link in `looprender.rs` and re-renders.

## Plain summary

Kimi's PR 8 needs to fix one wrong link in the tool that writes the project's status pages. This one-line edit lets it touch that file.
