# REPORT: infra-loop-harness, claude-51 (the umbrella's scope gains the archive gate)

Lab: claude. Base: `2987514aa`. Order: [ORDER.md](ORDER.md), committed first (`e7572a4b0`). The cancel probe found `xtask/src/archive_verify.rs` absent from the umbrella's `scope:`, so the task stands. No open PR changes that file.

## What did not work, first

- **The archive gate sat outside every fitting scope.** The rule Casey chose for PR 8's catalog edits ("Append-only (Recommended)", #2982, 6352383423) lives in `xtask/src/archive_verify.rs`, and no item's `scope:` lists that file. So the rule needs this one-line widening first, as a second PR in sequence. This is the same path claude-48 and claude-49 took for the links tool.

## What changed

| Commit | What |
|---|---|
| `e7572a4b0` | This ORDER, alone. |
| `1ae974d30` | `docs/roadmap/items/infra-loop-harness.md`: `xtask/src/archive_verify.rs` is appended to `scope:`. |
| this REPORT's commit | This REPORT. |

## Measurements

None.

## Checks run at the head

- `cargo xtask roadmap check`: passes (ready 34, held 26, decision 4, done 24).
- `roadmap render --check`: current. The scope is not part of the render, so `docs/roadmap/README.md` is unchanged.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one scope entry.

## Still open

- **claude-52: the append-only rule** in `archive_verify.rs`.
  - A modified `archive/catalog.jsonl` passes only when its bytes at the base are a prefix of its bytes now, ending on a line boundary, and the result still parses.
  - A modified `archive/CATALOG.md` or `archive/catalog/*.md` passes only when it equals `docs_catalog::render`'s output for that path.
  - Every other modified or deleted archive file still fails.
  - It needs a planted-fault test and another vendor's review.
- **After claude-52 lands,** PR 8 merges `main`.

## Plain summary

Casey chose how the archive check should treat the archive's catalog, and the umbrella task was not allowed to touch that check. Now it is.
