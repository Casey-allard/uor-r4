# #2182 — SoftSort read: parallel host flock selection; evaluate softsort_tau= (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T22:54:45Z
- **Updated:** 2026-10-10T22:57:49Z
- **Closed:** 2026-10-10T22:55:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2182
- **Merged:** 2026-10-10T22:55:54Z
- **Merge commit:** d97fe39894165a3bddafb337340a463946676505
- **Base / Head:** main / UOR-Foundation:claude/softsort-speed
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line "wide learned flock read" · count 1/3 · headline before → after: softmax at runtime yes, served 0.886838 BPB → unchanged (preparation PR 2 of 2 for arm S).**

Names its run: **arm S** of the [pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6102148887) as [amended](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6102170500), `TEST FITNESS: FIT` ([review](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6102348077)).

## Why (a tooling blocker, not a result)

The arm S retune at f07d2ff01 made no step-1000 dev eval in 45 minutes on a 5090. Each trainer sat at 100 % of one CPU core while the pod had 120. The cause is `softsort_learned_read`: it selected each row's flock support in a **serial** host loop. That is about 98k rows per read layer per step, each a sort of up to 384 scores. The learned-rank read that S replaces already runs the same selection under rayon. At that speed the decisive pair would take more than 9 h per seed, far outside the pre-registered cost.

## What changes

- **`geometric_stack.rs` `softsort_learned_read`:** the row loop now runs as `par_chunks_mut` over each row's own slots. Rows are independent and each writes only its own slots, so the output is identical to the serial order. The forward and the backward are unchanged.
- **`geometric-stack evaluate`:**
  - `softsort_tau=T` scores a `softsort_rank` model with its SoftSort training forward at `T` instead of the hard served read. Any other weighting is refused.
  - `max_blocks=N` bounds the blocks scored.
  - These cover the FIT report items (a), the soft-vs-hard end gap, and (b), the score scale (the τ at which the soft and hard NLL part).

## Checks at the head

- `cargo fmt --check`: clean.
- `cargo test --release -p uor-r4-training --lib softsort`: 3 passed. These include τ → 0 equals the hard rank read and the finite-difference gradient.
- `cargo build --release --example geometric-stack`: OK.
- Pod bootstrap at this head (sm_120): build OK, parity **PASS** (37 passed).
- The re-run retune uses this binary. Its step rate goes in the result PR.

Would the next step differ if this came out the other way? Yes. Without the fix arm S cannot run within its pre-registered cost.

References #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6103063683 — 2026-10-10T22:55:02Z — Casey-allard

**Exact-head checks at `6a9a24f6d6ba2bcf28e517ebbc2b0e5a11cc91ae`** (laptop, macOS arm64; pod jg1ztimihpt7m5, sm_120):
- `cargo fmt --check -p uor-r4-training`: exit 0.
- `cargo test --release -p uor-r4-training --lib softsort`: exit 0, `test result: ok. 3 passed; 0 failed`.
- `cargo build --release -p uor-r4-training --example geometric-stack`: exit 0.
- `uor-pod bootstrap --ref 6a9a24f6d6ba2bcf28e517ebbc2b0e5a11cc91ae`: build 105 s, parity **PASS** (`37 passed; 0 failed; 3 ignored`).

**Exact-head self-review (Claude lab, owner rule of 30 September):**
- The parallel loop gives each row exactly its own `kept` slots of `positions`/`flat`/`live` and its own `table_rows` entry, through `par_chunks_mut(kept)` zipped with `par_iter_mut`. `enumerate` gives the same `row` index as the serial loop, so the values written are identical. An error in any row still fails the call.
- `host` is shared read-only. Nothing else in the read changed.
- `evaluate softsort_tau=` refuses a model that is not `softsort_rank` and records the τ in `evaluation.json`. `set_softsort_temperature` validates τ > 0. `max_blocks` only lowers `blocks`, and the existing `tune_blocks` check still applies.
- APPROVE.


---
_Comment/review ids in this file: 6103063683_

