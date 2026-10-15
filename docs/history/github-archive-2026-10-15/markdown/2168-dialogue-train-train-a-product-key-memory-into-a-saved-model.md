# #2168 — dialogue-train: train a product-key memory INTO a saved model (memory_layers= beside init=) (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T16:22:47Z
- **Updated:** 2026-10-10T17:07:52Z
- **Closed:** 2026-10-10T17:07:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2168
- **Merged:** 2026-10-10T17:07:51Z
- **Merge commit:** a5e1406695a681dce3225310b73678c27bdac056
- **Base / Head:** main / UOR-Foundation:deepseek/addressed-memory-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Preparation for the memory-equipped run pre-registered on [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6099634823) (D22 order 2). The LM trainer has carried `memory_layers=` and its six companions since the matched-ablation rounds; `DialogueSettings` had **none**, so every dialogue artifact this line has produced answers rows named `multi_turn_memory` with a copy pointer and **no memory reader**. D22 makes that a tooling blocker to fix, not an owner question.** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: exact addressed memory · count 0/3 · headline unchanged (preparation; the run it prepares is pre-registered before any compute)

## What changed

| | |
|---|---|
| shared parsing | `memory_config_from_args()` extracted from `stack_config()`; the six companions now **refuse to appear without `memory_layers=`**, since they describe the memory it names |
| `init_extended_config()` | accepts the memory options beside `init=`. On a model saved **without** memory layers they **add** one and report `memory_added`; on a model saved **with** one they must describe exactly that memory, and a different one is refused |
| `StackModel::add_memory_layers()` | drops the named layers' `mlp.gate/up/down` variables and initialises the memory's own from the model's seed. The initialiser advances through `StackConfig::shapes` in the same order `StackModel::new` does, so **the added weights are exactly what a fresh construction with that configuration and seed would have started from**. Refused on a model with a served representation; idempotent for an identical memory |
| `dialogue_train()` | installs the memory after `init=` (after `extend_context`, **before the optimizer is built**), so the mechanism is present **from the first step** of the fine-tune rather than bolted onto a finished model |
| `DialogueSettings` | carries the parsed memory; the seven option names join the dialogue-train allowlist |

## Checks at the exact head

| check | command | result |
|---|---|---|
| library test | `cargo test -p uor-r4-training --lib a_memory_added_to_a_saved_model` | **pass** — the memory's `keys`/`values`/`query.weight` replace layer 1's `mlp.gate/up/down`, layer 0 keeps its MLP, the three added tensors are **equal element-wise to a fresh `StackModel::new` with that configuration and seed**, a second identical call returns `false`, and a different memory is refused |
| CLI test | `cargo test -p uor-r4-training --example geometric-stack memory_options_add` | **pass** — adding on a saved-without-memory config, agreeing on a saved-with one, refusing a different one (`memory_layers=0`, and a companion without `memory_layers=`), leaving a config untouched without the options, and the codebook defaults (`e8` → 240 sub-keys, key width 16) |
| example suite | `cargo test -p uor-r4-training --example geometric-stack` | **13 passed, 0 failed** |
| build | `cargo build --release -p uor-r4-training --example geometric-stack` | clean |
| format | `cargo fmt -p uor-r4-training -- --check` | the two changed files are clean (the package's pre-existing drift in untouched files is left exactly as it was) |
| claim wording | `python3 scripts/check_claim_wording.py` | **pass** |

This is a preparation PR and it names the run it prepares: the **two-seed memory-equipped fine-tune** (`memory_layers=4`, `memory_sub_keys=64`, `top_k=16`, `heads=4`, `key_dim=64`) on the adopted 10 % mixture, seeds `20261010` and `20261011`, exactly D10's recipe with the memory present from step 0, scored on the frozen v5 memory panel and the open reply panel with the bar and the noise declared in that pre-registration.

References #2029


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6099891688 — 2026-10-10T16:50:40Z — Casey-allard

**Validation at the exact head `0487a73d460dd7375cbdb31e1173c91d9cff4139`** (two commits: the memory wiring, then the allowlist the options need to reach it).

| check | command | result |
|---|---|---|
| library test | `cargo test -p uor-r4-training --lib a_memory_added_to_a_saved_model_matches_a_fresh_construction` | **pass** — `memory.keys`/`values`/`query.weight` replace layer 1's `mlp.gate/up/down`, layer 0 keeps its MLP, the three added tensors are **equal element-wise to a fresh `StackModel::new` with that configuration and seed**, a second identical call returns `false`, and a different memory is refused |
| CLI test | `cargo test -p uor-r4-training --example geometric-stack memory_options_add_a_memory_to_a_saved_model_and_must_agree_with_one` | **pass** — adding, agreeing, refusing a different memory, refusing a companion without `memory_layers=`, untouched without the options, codebook defaults |
| example suite | `cargo test -p uor-r4-training --example geometric-stack` | **13 passed, 0 failed** |
| option reaches the config | `geometric-stack dialogue-train memory_layers=4` | now fails on the **next** required argument (`missing tokenizer=`), i.e. the option is accepted — the first head failed here with `unknown argument memory_layers=`, which is why this second commit exists |
| build | `cargo build --release -p uor-r4-training --example geometric-stack` | clean |
| format | `cargo fmt -p uor-r4-training -- --check` | the two changed files are clean; pre-existing drift in untouched files is left exactly as it was |
| claim wording | `python3 scripts/check_claim_wording.py` | **pass** |

**One measured limitation, recorded rather than discovered later:** the product-key memory is a CPU custom op (`CustomOp3` with only `cpu_fwd`), so `device=cuda` refuses it — `Error: Tensor(no cuda implementation for geometric-stack-product-key-memory)`. The two-seed run this PR prepares therefore fits on the pod's CPU at a **measured 3.7 s/step** (~2 h per arm) instead of GPU speed; a device-agnostic or CUDA implementation of the memory op is the tooling piece that removes that limit and is named as the next source work on #2029.



---
_Comment/review ids in this file: 6099891688_

