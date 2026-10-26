# REPORT: infra-loop-harness, antigravity-98

Lab: antigravity. Order: [ORDER.md](ORDER.md).
References #2904
References #2841

## What did not work, first

- **`uor-harness loop claim infra-loop-harness` refused.** It returned `item infra-loop-harness is already claimed by PR #3169 (docs/loop/tasks/infra-loop-harness/codex-1/ORDER.md)`. Because `#3169` is an idle draft touching only `PLAN.md`, `DECISIONS.md`, and `docs/loop/tasks/infra-loop-harness/codex-1/*`, and our unit shares zero files with `#3169`, `#3283`, `#3289`, or `#3290`, this `ORDER.md` was written by hand per the umbrella-item rule (`run-antigravity.txt` Part 3.2, matching `claude-90` and `antigravity-95`).
- **`uor-harness loop next` skipped Step 7 of the M1 path (`m1-fresh-panel-transfer`, `#2841`).** Even though Step 6 (`m1-export-gap-and-toks`, `#2839`) merged as `#3287`, `m1-fresh-panel-transfer.md` still held the default `crates/uor-r4-discrete/**` glob from `#3171`, which collided with active discrete-crate orders (`#3290`, `r-rank-objective-keeps-cells`) for no file reason.
- **`uor-harness loop close --verify --pr 3287` failed `remove_worktree`.** It reported `FAIL worktree holds ignored files: local/antigravity/gates/1793034975-62704/...; kept` because `ignored_files` in `crates/uor-r4-harness/src/loopclose.rs` treated `target/`, `local/<lab>/views/`, and `local/<lab>/gates.jsonl` as disposable, but omitted `local/<lab>/gates/` (written by every `uor-harness loop gates` run inside a worktree, see `loopclose.rs:760`), `local/target/`, and `local/harness-target/`.

## What changed

1. **`docs/roadmap/items/m1-fresh-panel-transfer.md` (`#2841`, `PLAN.md` Step 7):**
   - Verified that `crates/uor-r4-discrete/src/attention_transfer.rs` (`validate_decision_header` at lines 33–64 and `transfer_metadata_requires_explicit_prospective_decision_and_all_pins` at lines 720–750) already refuses to open the fresh slice before a sealed DEV decision, and that M1 bar (e) ("integer top-1 through the D11 export beating the unigram table (1,226 of 14,287) and the next-word table (6,300 of 14,287) on each of two seeds") is scored by `d11-scorer` in `docs/labs/m1-d11-artifact-scorer/instrument/**`.
   - Narrowed `scope:` from `[crates/uor-r4-discrete/**, ...]` to `[docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-fresh-panel-transfer*, docs/labs/m1-fresh-panel-transfer/**, docs/loop/tasks/m1-fresh-panel-transfer/**, docs/research/notebook/m1-discrete-attention.md, docs/research/open-questions.md, docs/start/faq.md]`, and added `notebook: docs/research/notebook/m1-discrete-attention.md`.
   - Updated `How to start` steps 2 and 4 from pre-D67 owner-release wording (`"and the owner has typed the release"`) to the current D67/D68/D77 rules (fresh-context D68 decision review logged for the owner; fresh-context D67 pre-run review after FIT).
   - Recorded the fresh-context D68 decision review as an `[antigravity] owner log:` comment on `#2841`.
2. **`crates/uor-r4-harness/src/loopclose.rs` (`ignored_files`):**
   - Added `local/target/`, `local/harness-target/`, and `local/<lab>/gates/` to `disposable_prefixes` in `ignored_files` so that gate logs and worktree-local Cargo build directories do not block `uor-harness loop close --verify` from removing a clean merged worktree.
   - Added unit test `loopclose::tests::ignored_files_treats_gate_logs_and_local_targets_as_disposable`, verified it failed before the fix (`got: ["local/antigravity/gates/123-456/fmt-harness.exit", "local/antigravity/gates/123-456/fmt-harness.log", "local/harness-target/debug/bin", "local/target/release/bin"]`), and verified it passes with the fix while still catching non-disposable `local/<lab>/` files (`local/antigravity/unarchived-receipt.json`).

## Measurements

- `loopclose::tests::ignored_files_treats_gate_logs_and_local_targets_as_disposable`: failed pre-fix (`0 passed; 1 failed`), passed post-fix (`2 passed; 0 failed` in `loopclose::tests`).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --lib loopclose::tests` (`2 passed; 0 failed`)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo fmt -p uor-r4-harness -- --check` (`exit 0`)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo clippy --locked -p uor-r4-harness --all-targets -- -D warnings` (`exit 0`)
- `uor-harness loop report --check --order docs/loop/tasks/infra-loop-harness/antigravity-98/ORDER.md` (`PASS`)
- `uor-harness loop pr-check --local` (`PASS`)
- `uor-harness loop gates --full` (`11 / 11 GREEN`)

## Skills used

- `scope-guard` and `solution-integrity` principles applied to keep the diff strictly within the two targeted defects.

## Still open

- **The umbrella-item lock (`#3169`).** As noted in `claude-90`, `loop claim` locks `infra-loop-harness` per item across all labs while `#3169` is open.
- **Step 7 of the M1 path (`m1-fresh-panel-transfer`, `#2841`).** Once this PR merges, `m1-fresh-panel-transfer` is immediately claimable without colliding with `crates/uor-r4-discrete/**` orders.

## Plain summary

This unit unblocks Step 7 of the M1 critical path (`m1-fresh-panel-transfer`, `#2841`) so it can be claimed in parallel with discrete-crate mechanism work, updates its pre-D67 owner-release text to current D67/D68 rules, and fixes `uor-harness loop close --verify` so worktree gate logs (`local/<lab>/gates/`) and local Cargo build directories (`local/target/`, `local/harness-target/`) no longer prevent automatic worktree removal after merge.
