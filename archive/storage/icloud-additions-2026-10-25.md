# iCloud cold archive: additions of 24–25 October 2026

The cold archive was read-only from 20 October 2026, with new results going to the HF store. On 24 October 2026 the owner reopened it for the close-out, because the HF store is too small for this data:

> "if it seems Huggingface is too small for the store, we can use the icloud store where we have been putting this stuff previously (I think in tar's also but you should check) that way we can have them upload to icloud and remove them locally and only fetch them if we need them later (if we are going to need them for like the next ticket there is no need to tar upload and then download and reinflate between tickets obviously)"

Six tars were added under `UOR-R4/results/`, each with a one-line `UOR-R4/index/<name>.tsv`. The rows are in [`icloud-index-2026-10-25.tsv`](icloud-index-2026-10-25.tsv), in the same columns as [`icloud-index-2026-10-20.tsv`](icloud-index-2026-10-20.tsv). Each md5 was checked against the iCloud copy, and the copy was evicted, which proves the upload, before the local data was removed.

| Tar | What it holds |
|---|---|
| `kimi-local-closeout-20261024` | `~/uor-r4/local/kimi` without build folders (verify-3130 checkout, verify logs, PM legacy notes, m2-fresh256 readback, reviews of #3136–#3144), plus the local folders of the `kimi-loop-check` and `kimi-review-3152` worktrees |
| `deepseek-local-closeout-20261024` | `~/uor-r4/local/deepseek` without its harness build (patches and PICKUP), plus the `deepseek-verify` worktree's local folder |
| `antigravity-local-closeout-20261024` | `~/uor-r4/local/antigravity` (sealed m2-fresh256 copy, tokenizer, loop probes, PR reviews), plus the `antigravity-main-verify` worktree's local folder |
| `codex-cf1-worktree-closeout-20261024` | The `codex-cf1-audited-dose-runbook` worktree's uncommitted diff and untracked runbook folder. The runbook itself is on main through #3167, and the branch had no commits of its own |
| `claude-local-closeout-20261025` | `~/uor-r4/local/claude` without build folders and without the re-extractable legal case (`codex-direct-legal-centered-20261009`, already in this archive with md5 `5a965dade6ed7b04b8238b4d1e020be3`). It also holds this session's evidence (the #3152 replays, the #3111 RTX 5090 qualification logs, review records, the owner-checkout safety copy) and the staged worktree logs |
| `vast-archive-20261025` | The laptop's `~/.local/share/uor-r4/compute/vast-archive/`, the archived `/workspace` results of Vast pods destroyed 12–23 Oct |

To fetch one: download it from iCloud, check its md5 against the row, then `tar -xf`.
