# #1704 — Read key shift: saved option, production-pattern bench, and serving path (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T00:34:03Z
- **Updated:** 2026-10-09T13:08:44Z
- **Closed:** 2026-10-05T00:35:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1704
- **Merged:** 2026-10-05T00:35:55Z
- **Merge commit:** 2cdae05a085a90d9826739179e6dd63d87e2648e
- **Base / Head:** main / UOR-Foundation:codex/key-shift-production
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. This PR follows #1701 (merged) and is based on `main` (`84715ed2`).

## Results

1. **Production pattern.** With the read key shift on, the recurrence no longer crowds out the reads: `rrarra` scores **1.000 at every distance with both seeds**.
   - **Correction to #1698:** the earlier `rrarra` failure was seed-dependent. That result (0.007 at seed 1) did not reproduce at seed 2, where `rrarra` without the shift reaches 0.9995.
   - Recurrence layers placed before a read can supply predecessor identity through their width-4 causal convolution. Without the shift, training finds that route only sometimes.
   - The all-read arm A fails on **both** seeds. It has no other source of predecessor identity.
   - The shift makes the outcome reliable for every pattern tested.
2. **`key_shift` is now a saved, trainable option** for `train` and `dialogue-train`. It is checked end to end on the CLI.
3. **D11 serving form:** a design note only (below). I estimate the work above about 200 lines, mainly because the session snapshot and rollback paths would change. Until it exists, every integer and served path refuses a key-shift model explicitly.

## 1. Production pattern on the bench

The bench settings are those of #1698 and #1701: CPU at 4 threads, context 512, width 128, 4 heads, MLP 384, 6 layers, 1800 steps of batch 8, lr 1e-3, l2 read.
- Each cell is the share of 512 fresh-pairing queries answered correctly per bucket.
- The binary was built from `787c350d`, whose bench and forward sources match this head.
- Report roots are sealed under `~/uor-r4-local/mqar-bench/key-shift-production-787c350d/`. The seed-1 `rrarra` row without the shift is the #1698 root `grid3-ctx512-eba6a15f/B-rrarra-l2`.

| pattern | key shift | seed | d16 | d64 | d200 | d400 | all | held-out class | positions scored / query token | first curve point ≥ 0.9 | wall |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `rrarra` | off | 1 (#1698) | 0.014 | 0.008 | 0.002 | 0.006 | 0.007 | 0/1024 | 2,855 | never | 977 s |
| `rrarra` | off | 2 | 1.000 | 1.000 | 0.998 | 1.000 | 1.000 | 1024/1024 | 2,841 | 400 | 673 s |
| `rrarra` | **on** | 1 | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | 1024/1024 | 2,855 | 700 | 1016 s |
| `rrarra` | **on** | 2 | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | 1024/1024 | 2,841 | 600 | 633 s |
| `rararr` (14-layer stand-in) | off | 1 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1022/1024 | 2,855 | 500 | 972 s |
| `rararr` (14-layer stand-in) | **on** | 1 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1024/1024 | 2,855 | **200** | 1129 s |
| `aaaaaa` (arm A) | off | 1 (#1701) | 0.807 | 0.176 | 0.010 | 0.029 | 0.255 | 0/1024 | 8,564 | never | 1041 s |
| `aaaaaa` (arm A) | off | 2 | 0.799 | 0.188 | 0.023 | 0.039 | 0.262 | 1/1024 | 8,523 | never | 798 s |

**The stand-in for the 14-layer `rrarrarrarrarr`** is `rararr`. It keeps three properties of the 14-layer pattern: about the same read fraction (2/6 against 4/14), recurrence first, and two trailing recurrence layers after the last read, which `rrarra` lacks. The true 14-layer model was not run; at width 128 it would exceed the 20-minute budget per arm.

**Read probe, final step:**
- The best head's weight on the value position at d16/d64/d200/d400 is 0.998/0.997/0.995/0.988 for `rrarra` with the shift (layer 2, head 3), and 0.996/0.997/0.993/0.978 for `rararr` with the shift (layer 1, head 3).
- `rararr` without the shift also reaches 0.98–1.00, through layer 1, head 3: a read that sits after one recurrence layer.

**Interpretation, measured at this scale with 1–2 seeds:**
- Retrieving MQAR needs predecessor identity at the read's keys.
- An all-read stack has no source for it, and fails on both seeds.
- A recurrence layer before a read can provide it through its causal convolution, but whether training finds that route depends on the seed. `rrarra` failed at seed 1 and solved at seed 2.
- The key shift provides it exactly. Every pattern with the shift solved every bucket on every seed tried, including the held-out pairing class. It also learned fastest on `rararr` (≥ 0.9 at step 200 against 500).
- So, once the shift is on, the recurrence does not crowd out the reads. #1698's "crowding out" reading should be restated as "seed-dependent without the shift".
- Not yet measured: the D19 grounded MQAR cell and a chat model with the shift. That is the GPU-pod training this PR enables.

## 2. `key_shift` as a saved, trainable option

**Saving and loading:**
- `StackModel::save` writes `"read_key_shift": true` into `config.json` only when the shift is set (`READ_KEY_SHIFT_FIELD`). Every other model keeps its exact legacy bytes; a test compares them to `to_vec_pretty(StackConfig)`.
- `StackModel::load` restores the shift. `saved_read_key_shift` refuses any value other than absent or `true`, so a model cannot be silently read as unshifted.
- The sealed inference checkpoint (`stack_checkpoint`) saves through `StackModel::save` and loads through `StackModel::load`, so it carries the shift as well.

**CLI** (`examples/geometric-stack.rs`): `train` and `dialogue-train` take `key_shift=false|true|add`, default false. The report records `key_shift` as `false`, `true` or `added_to_init`. The resume lineage records it only when set, so earlier checkpoints still resume.

**The `init=` policy:** the setting must equal the saved model's.
- `key_shift=false` from a shifted model is refused, because it would silently drop the channel.
- `key_shift=true` from an unshifted model is refused.
- **Design choice:** `key_shift=add` is the explicit way to add the channel to a model trained without it. The model's weights are kept, its reads change from the first step, and the report and lineage say `added_to_init`.
- A resume must match its lineage.
- `qat=true` with the shift is refused, because the shift has no served representation yet.

**Refusals until a served form exists:**
- `StackModel::set_served_representation` refuses the shift in both orders.
- Integer export (`export_stack`) and `stack_grid_reference` refuse it.
- The attention-session float replay host refuses it.
- So no LUT or D11 artifact can be made from a key-shift model that silently drops the shift.

**CUDA:** the op is a reshape, narrow, cat, neg and add on the key projection, all ordinary candle ops. The fused read kernels never see the shift, because it is applied before them. The CUDA path itself is **UNAVAILABLE** here: there is no GPU on this machine. The first pod run should compare against the CPU before training.

**Tests** (all at this head):
- `cargo fmt --all -- --check` passes.
- `cargo test --release -p uor-r4-training --example geometric-stack --example mqar-bench`: 11 and 8 tests pass.
- The lib tests filtered to `stack_export stack_checkpoint read_identity_carry save geometric_attention_native` pass (54), including the new ones:
  - `geometric_stack::read_key_shift_save_load_is_bit_exact_and_default_format_stays_legacy`: on CPU, save then load reproduces the logits bit for bit with the shift on; an unshifted model keeps legacy bytes; non-canonical values are refused; served mode is refused in both orders.
  - `stack_export::read_key_shift_refuses_export_and_grid_reference`.
  - `geometric-stack::key_shift_is_never_changed_silently`: the full `init=`, add and resume matrix, plus the `qat` and `add` argument checks.

**CLI smoke run** on random tokens: width 32, `rra`, l2, context 32. Each case behaved as intended:
- `key_shift=true` training saved the field.
- `init=` of that model with `key_shift=false` was refused; with `key_shift=true` it ran.
- `init=` of an unshifted model with `key_shift=true` was refused; with `key_shift=add` it ran and saved the field.
- Resuming a shifted checkpoint without `key_shift` was refused ("differs from the checkpoint's lineage in: key_shift"); with `key_shift=true` it resumed.
- `qat=true` with `key_shift=true` was refused.
- `export` of the shifted model was refused.

## 3. D11 serving form: design note (not built)

**Served arithmetic.** At read layer ℓ and position t, the served key is `k'_t = k_t + j·k_{t−1}`.
- `k_t` is the existing 4-bit integer key projection, at exponent −16.
- `j·(a,b,c,d) = (−c, d, a, −b)` is a fixed signed permutation of each four-channel lane.
- So the served cost is one signed permutation and one i32 add per key element: no multiplier, no float, no table, and no parameters. This fits D0-b and D11.
- The sum of two exponent −16 keys needs one guard bit. Use a checked add, or widen to i64 before the score kernels that already widen.

**Where it goes.** I recommend caching raw keys and shifting on read, which leaves the session schema unchanged:
1. The `IntegerStackSession` read cache keeps storing the **raw** `k_t`. `SerializedStackLayerState::Read`, `save_state` and `restore_state` stay at schema `uor-r4.stack-session/1`, because `k'_s` is a function of the cached raw keys `k_s` and `k_{s−1}` alone.
2. At score time (`stack_l2_distance`, `stack_dot`, the Lorentz path), each head's key slice for source `s` is formed as `k_s + j·k_{s−1}` in a per-head scratch row. That costs hd adds and one extra key-row read per scored source.
3. L2 and Lorentz lifts are computed at write time from `k'_t`, since `k_{t−1}` is in the cache then, and stored as now.
4. The container header gains `read_key_shift: bool` with `#[serde(default)]`. Old artifacts parse unchanged, and the exporter writes `true` only for shifted models. A runtime that predates the field would ignore it, so the artifact schema string should change for shifted models. That way an old runtime refuses them instead of serving them unshifted.
5. `stack_export::export_stack` writes the flag. `stack_grid_reference` applies the same shift to its quantized keys.
6. The decisive test would export a small key-shift model, then compare `IntegerStackSession::step` logits bit for bit against `stack_grid_reference`, including across `save_state` and `restore_state` mid-sequence.

**Rejected alternative:** caching shifted keys. It needs a per-layer raw previous key as new session state, which changes the snapshot schema and the rollback paths.

**Estimate:** the format flag and schema (about 25 lines), shifting on read in three score kernels plus the lift (about 60), export and grid reference (about 40), and tests (about 100). That totals 200–250 lines, above the threshold set for this round, so it is left for its own PR.

## Cost

- Agent wall time this round: about 1 h 35 min (19:00–20:35 ET), including the #1701 doc-line fix.
- Bench compute, all on CPU at 4 threads: 6 runs of 633–1,129 s each, for 5,221 s of wall time in total.
- Builds and tests ran at `-j 1`, overlapped with the bench.
- The CLI smoke runs took under 10 s.
- New storage is 876 KB of sealed report roots.
- No GPU or external compute was used.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5986115134 — 2026-10-05T00:35:02Z — Casey-allard

Exact-head review at dec3c096e4e3abe7ee382986a4ec1e83612aba12: **approved**.

**Save and load**
- `read_key_shift` is written to config.json only when set, so legacy config bytes stay exact (tested).
- Load refuses any value other than an absent field or `true`.
- Save→load reproduces logits bit-exactly.

**Refusals and lineage**
- QAT, the served codec, export, the grid reference and replay all refuse a shifted model, so no artifact can drop the shift silently.
- `init=` requires the explicit `key_shift=add` to add the shift to an unshifted model.
- Resume requires matching lineage.

**Checks:** `cargo fmt` and the example and library tests pass at this head, per the PR body.

**Not yet verified:** CUDA parity. It is checked on the pod before any training run.

Queuing.


---
_Comment/review ids in this file: 5986115134_

