# REPORT: infra-loop-harness, claude-43 (a notebook row added inside the table passes the scope rule)

Lab: claude. Base: `8d488457e`. Order: [ORDER.md](ORDER.md), committed first (`bc76a54ae`). The cancel probe found that at the base, a notebook row added before the page's trailing prose is refused. My #2829 close (claude-42) failed `pr-check --local` with "docs/research/notebook/m1-discrete-attention.md is outside item m1-a8-erosion-diagnosis's scope". So the task stands.

## What did not work, first

- **The designed notebook path never worked on a real page.** `scope` allowed the item's `notebook:` page through `append_only`: the head had to start with the base text, which means the new text had to come at the very end. Every notebook page puts prose after its results table. That includes `m1-discrete-attention.md` (last row line 64 of 85) and the five other pages (`data-tokenizer`, `harness-governance`, `m1-prime-zeta`, `m3-durable-memory`, `m4-serving`). So a row placed where the notebook README says rows go was always refused.
- **The test fixture hid the bug.** Its notebook ended with the table, so appending at the very end passed.
- **It would have blocked an owner decision.** Casey's 22 Oct answer on #2857, "Add notebook: field (Recommended)", relies on this path. #2857 and #2854 would have hit the same refusal.

## What changed

| Commit | What |
|---|---|
| `bc76a54ae` | This ORDER, alone. |
| `33ebafb94` | See the list below. |
| this REPORT's commit | This REPORT. |

What `33ebafb94` changes:
- **`loopprcheck.rs`:** the declared notebook is judged with the existing `keeps_base_lines` (every base line kept, in order, so rows are only added), as `docs/start/FILEMAP.md` already is. The unused `append_only` is removed.
- **`tests/loop_prcheck.rs`:** the fixture notebook gains trailing prose. A new pass case adds a row inside the table. The rewritten-row fail case still fails.
- **`docs/harness/README.md`:** the scope sentence says rows may land inside the table.

## Measurements

None. This is harness behaviour.

## Checks run at the head

- `cargo test -p uor-r4-harness --test loop_prcheck`: 28 passed.
- **Negative gate:** with the `loopprcheck.rs` change set aside, `scope_always_allows_status_notebook_append_mistakes_and_filemap_rows` fails at `loop_prcheck.rs:222` on the in-table row. Restored, it passes.
- `clippy --all-targets -D warnings`: exit 0. fmt: clean.
- `loop gates --full`: 10 GREEN, 0 RED.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one condition in `loopprcheck.rs`, its test fixture and case, and one README sentence.

## Still open

- **The #2829 closing PR (claude-42)** waits for this to merge, then merges `main` and passes `pr-check`.
- **DeepSeek's #2857 close**, and **#2854**, use the same path once this lands.

## Plain summary

Research notebooks keep their results table near the top, with explanations below it. The check only allowed new results at the very bottom of the file, so no task could ever log its result in the table. Now a row can go where it belongs, while existing rows still cannot be changed.
