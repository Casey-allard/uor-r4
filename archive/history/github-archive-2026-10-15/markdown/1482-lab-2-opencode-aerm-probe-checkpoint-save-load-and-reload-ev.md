# #1482 — Lab 2 (OpenCode): AERM probe checkpoint save/load and reload evaluation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T00:09:15Z
- **Updated:** 2026-10-09T14:16:59Z
- **Closed:** 2026-09-29T00:18:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1482
- **Merged:** 2026-09-29T00:18:45Z
- **Merge commit:** 1c94969de0d43d4e10532be7b8222b0f8edefea6
- **Base / Head:** main / UOR-Foundation:lab/opencode/aerm-checkpoint
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 2 (OpenCode). Class A** (lane code and tooling; no result claim). References #973 and #820.

## What and why

D2 and G v1 both recorded the same limitation: **the AERM runner writes reports only, so no trained model can be loaded**. That blocks the next reader unit (separating missed write / wrong key / eviction, then tag/address generalisation), which needs a loadable artifact.

- `stack_aerm.rs`: `AermModel::save` / `AermModel::load` (reusing `StackModel::save`/`load`) write the stack, both heads and — in the memory arm — the full dual-register branch, plus `aerm.json` with the split and memory flag. Load validates shapes and dtype and fails closed on a missing or mismatched entry.
- `aerm-probe.rs`: `run … save=CHECKPOINT_ROOT` writes `<arm>-s<seed>/` and records the path in the run JSON; the new `checkpoint-eval` mode loads one arm and writes the same evaluations (`uor-r4.g1-aerm-checkpoint-eval/1`).

No result claim, no gate, no shared interface, no serving path.

## Checks (commands and outcomes)

- `cargo fmt -p uor-r4-training --check` → pass.
- `cargo check -p uor-r4-training --offline --release --examples` → pass.
- `cargo test -p uor-r4-training --release --offline stack_aerm` → **12 passed, 0 failed**, including the new `aerm_save_and_load_reproduce_logits` (memory arm and control arm both round-trip with identical logits).
- **Save → reload round-trip on a real smoke:** `run … steps=20 save=…` then `checkpoint-eval` on `aerm-s1` reproduces the run's `text.dev_nll` (7.49420166015625) bit-for-bit and the same class counters, `read_events` and free-running count; checkpoint layout `aerm.json`, `heads.safetensors`, `stack/{config.json, model.safetensors}`.
- `python3 scripts/check_claim_wording.py` → pass.

A non-author review is requested before merge (class A).


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5881185130 — 2026-09-29T00:17:47Z — Casey-allard

## Non-author review — **approved** (with two should-fix items, now applied)

Independent reviewer pass `uor-evidence-auditor` (session `ses_f15815184ffejVtjFOulHEyBED`), read-only. Verdicts:

- **Q1 completeness — PASS.** `save` writes the stack (all variables, nested under `stack/`), the four head tensors and — in the memory arm — exactly the ten `MemoryBranch` fields, cross-checked one-for-one against the struct, `parameter_counts` and `optimizer_groups`. Nothing omitted or defaulted.
- **Q2 fail-closed — PASS.** Missing keys, wrong shapes and wrong dtypes are rejected for every tensor on both the AERM and reused stack paths.
- **Q3 test — meaningful; a blind spot was found.** The memory branch is zero-initialised, so a parameter dropped from **both** save and load was invisible to the exact-logit comparison.
- **Q4 wiring — PASS.** `save=` writes a distinct directory per arm/seed and records it; `checkpoint-eval` loads it and runs the same four evaluations with the same fixed seeds; the added `checkpoint` key does not affect `summarize`.
- **Q5** no `unwrap`/`expect` on recoverable paths, no silent fallback.

### Should-fix items, applied in `9af92e89`

1. **Test blind spot closed.** The round-trip test now overwrites all four head and all ten memory variables with deterministic non-zero values before saving, and compares `bottom(...).tags` as well as `top(...)` logits. **Mutation check run by the lead:** dropping `memory.copy_scale_previous` from both the save list and the load construction (with the expected count set to 9) makes the test **fail** at the exact-logit assertion ("loaded memory-arm logits differ"); the file was then restored (clean tree, 12/12 pass, fmt clean).
2. **Fail-closed extras and split.** `load` now rejects a saved `split` of zero or at least the stack's layer count, and a `heads.safetensors` whose tensor count differs from `4 + (10 if memory else 0)` — so any dropped tensor fails closed at load.

### Confirmation at the updated head

The reviewer re-read the delta and returned: **"Confirmed for merge"** — the tags comparison exercises the head path, the count check covers a tensor dropped from both lists, the non-zero fill cannot be folded away, the split check is placed after the stack load, and no unrelated change was visible.

Checks on the head: `cargo fmt --check` pass; `cargo check --examples` pass; `cargo test -p uor-r4-training --release --offline stack_aerm` → **12 passed, 0 failed**; save→`checkpoint-eval` smoke reproduces `text.dev_nll` 7.49420166015625 exactly; `check_claim_wording.py` pass.



---
_Comment/review ids in this file: 5881185130_

