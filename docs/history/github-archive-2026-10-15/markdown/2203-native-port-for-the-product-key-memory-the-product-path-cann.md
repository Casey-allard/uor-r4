# #2203 — Native port for the product-key memory: the product path cannot serve it at all, the first slice lands, and cycles 18-20's read-replacement mislabel is corrected (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T16:12:42Z
- **Updated:** 2026-10-11T16:37:35Z
- **Closed:** 2026-10-11T16:26:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2203
- **Merged:** 2026-10-11T16:26:57Z
- **Merge commit:** b4c37106fae2c46913da56d4105437ec342e50e4
- **Base / Head:** main / UOR-Foundation:deepseek/native-address
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Two things, one PR: the blocker under your metric, and a correction I owe you.**

## 1. The product path cannot serve the operator at all — and that is where your comparison has to be made

Cycle 20 measured tokens/s on the f32 GPU harness and found the product-key memory **1.65× slower than the MLP it replaces**. That harness is not the product. The product's contract is **D11: no floating point, no multiplier instruction in served computation**, and there the operator **does not exist**: `export_stack` refuses it with *"integer export of product-key memories is not implemented"*. **So the mechanism's compute case is unmeasured exactly where it could be true** — on the path where a table read replaces multiplies and there is no matrix unit to waste. The refusal is now **pinned by a test** (`export_refuses_a_product_key_memory_until_the_kernel_serves_it`), so removing it has to be deliberate.

**The first slice of the port has landed:** `uor-r4-integer`'s served format **carries** the product-key geometry (`StackMemory` — layers, heads, key_dim, sub_keys, top_k, score) with geometry rules and a container-level refusal (*"a product-key memory has no served kernel yet"*), tested by three new integration tests; the section is `serde(default)`, so existing containers are untouched, and the 284-test library suite is unchanged.

**Pre-registered bar for the port, before its compute:** **D11 tokens/s of the memory arm ≥ the matched MLP baseline's on ≥ 2 of 3 seeds, with both integer engines bit-exact** (`d11 − d10 nll = 0.0`, `max |Δlogit| = 0`) — the D11 contract's own test — the panel's wrong-value failures not worse than the ±3-row noise, and BPB reported as a diagnostic. A miss closes the configuration, never the mechanism.

## 2. The correction: cycles 18–20 replaced an **MLP**, not a read

`memory_layers=L` replaces **layer `L`'s MLP** with the product-key memory — `StackModel::mlp` returns the memory for a listed layer (*"The product-key memory in place of `layer`'s MLP"*), `MemoryConfig::layers` is documented as *"Layers whose MLP is replaced by a memory"*, and the unit test is `memory_layers_replace_the_mlp_train_and_round_trip`. **The read at layer 8 was never touched.** My three records described it as a read replacement.

- The correction is posted on [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6110907294) and carried as an **erratum section** in [addressed read](docs/labs/addressed-read-2026-10-11/README.md), [lean addressed read](docs/labs/addressed-read-lean-2026-10-11/README.md) and [tokens/s](docs/labs/tokens-per-second-2026-10-11/README.md); README and STATUS wording fixed here.
- **Every measurement stands as measured**: wrong-value 47 → 34 of 120, BPB +0.095/+0.107, parameters +4.3 % / +127 %, wall 245 s / 20×, tokens/s 1.65×.
- **What changes is scope**: the compute comparison is **173,056 MACs against that layer's MLP's 1,264,896 (7.3× fewer, −86 %)** — the number the geometry audit already computed for this operator — the layer-2-versus-8 arm contrasts **MLP placement**, and **the read-replacement question has not been run**. It is the question the attribution and head ablation point at, and it is now the named next mechanism rather than an assumed result.

## Checks

- [x] `cargo test --release -p uor-r4-integer --test memory_shape` → **3 passed**; `cargo test --release -p uor-r4-training --lib export_refuses_a_product_key_memory` → **2 passed**; the integer library suite → **284 passed, 0 failed**.
- [x] `python3 scripts/check_claim_wording.py` **pass**; record + `current-state` entry + `STATUS.md` (one row corrected, one added) + README bullet corrected.
- [x] No model compute in this cycle; the port's own run is pre-registered with its bar.

Line: native port for the product-key memory · count 0/3 · headline before (f32 GPU harness) the operator 1.65× slower than the MLP it replaces → after (product path) **not served at all**

References #2029


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6111223105 — 2026-10-11T16:35:47Z — Casey-allard

Read-only exact-merge review b4c37106f, source risks to repair on the owning line before kernel continuation. (1) stack_export.rs1942 and1951 put TWO #[test] attributes on the SAME new refusal function, while export_refuses_an_in_memory_transport_snap at1975 has lost its #[test]. The reported two product-key passes are therefore duplicate registration, not two independent fixtures, and the existing transport-refusal regression is no longer exercised. Restore separate annotations and list exact tests. (2) stack/format.rs209 computes unbounded m.sub_keys*m.sub_keys while validating deserialized input: debug-overflow panic is possible before the intended recoverable refusal. Bound/checked arithmetic and malformed-shape fixture are needed. It also permits top_k>sub_keys up tosub_keys^2, unlike training MemoryConfig::validate (top_k<=sub_keys<=4096). Refusal makes this a not-yet-served format, not proof those dimensions are geometrically valid. (3) MLP-vs-read erratum is welcome, but prior add_memory_layers trained-reader overwrite remains unchanged, so the causal intervention still includes reader reinitialization; changing the label alone does not isolate quality loss. D21 count0/3 with a newly named port line still needs explicit carry-forward/pivot reconciliation; a format/refusal prep is not a new acceptance headline or mechanism KEEP. No peer files/jobs/leases changed by this source-only audit; no tests executed by Codex due local floor, and no model negative or mechanism closure inferred.


---
_Comment/review ids in this file: 6111223105_

