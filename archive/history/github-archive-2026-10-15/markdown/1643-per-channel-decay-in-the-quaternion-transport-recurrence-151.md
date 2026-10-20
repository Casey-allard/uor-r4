# #1643 — Per-channel decay in the quaternion transport recurrence (#1512)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T03:54:08Z
- **Updated:** 2026-10-09T13:17:46Z
- **Closed:** 2026-10-04T04:48:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1643
- **Merged:** not merged
- **Merge commit:** 52471f422201361697193ab6d073af126cf372dc
- **Base / Head:** main / UOR-Foundation:codex/per-channel-decay
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Makes the quaternion transport recurrence's decay **per channel** (`width` values) instead of **per lane** (`width / 4`), with matched parameter accounting. References #1512.

## The change

`crates/uor-r4-training/src/geometric_stack.rs`

| site | before | after |
| --- | --- | --- |
| `StackConfig::shapes` (1125) | `rec.decay` `[lanes]` | `rec.decay` `[width]` |
| variable init (1392) | timescale spread over lanes | spread over channels |
| composed Candle reference (6833-6884) | `opening [b,t,lanes] x log_a [lanes]` | `opening.unsqueeze(3) x log_a [lanes,4]` |
| `RecurrenceCore::parameter_len` (8709) | `4w + w + lanes` | `4w + 2w` |
| `Transition` (8678) | scalar `lambda`, `keep`, `clamped` | `[f32;4]` / `[bool;4]` per channel |
| CPU forward `window` (8757-8795) | `lambda = exp(8 opening log_a[lane])` | `lambda_k = exp(8 opening log_a[4 lane + k])`, `q_k = lambda_k u_k` |
| CPU backward (9173-9250) | scalar `d_lambda`/`d_log_a[lane]` | per-channel `d lambda_k`, `d log a[4 lane + k]`, `d opening` summed over the lane |

The served form stays table-servable under D0-b: the decay is still `exp(...)` times a per-channel table value, with one grid-code read and one `exp` per channel and **no new matmul**.

`crates/uor-r4-training/src/metal_stack_kernels.rs`: `recurrence_log_a` (per channel), `recurrence_core_fwd` and `recurrence_core_bwd` (per-channel `float4` lambda/keep), and the `metal_log_a` launcher in `geometric_stack.rs`.

Served path ported so export/serve stays consistent: `stack_export.rs` (`decay_rate` carries `width` grid codes), `uor-r4-lut/src/stack.rs`, `uor-r4-integer/src/stack/{session.rs,kernels.rs}` (`stack_rotation` / `stack_snap_rotation` take a per-channel lambda).

## Measured parameter delta

Built 1-layer, width-288, heads-6 model (`geometric-stack train`), same flags both arms:

- per-lane: **2,175,984** parameters
- per-channel: **2,176,200** parameters
- delta **+216 per r-layer** = `width - lanes = 288 - 72`, +0.0099% of the model. The prediction (+216) is confirmed exactly from the built artifacts, not assumed.

## Verification

- `cargo fmt --check` clean; touched packages compile (`uor-r4-training`, `uor-r4-lut`, `uor-r4-integer`).
- New test `per_channel_decay_generalises_the_per_lane_core`: with a lane's four decays equal, the forward is **bit-identical** to the frozen pre-per-channel core; with them unequal the forward changes.
- New test `per_channel_decay_gradient_matches_reference_and_finite_differences`: the fused decay gradient matches the composed Candle autodiff reference (relative gap < 1e-3) for rotation on and off, plus central finite differences for the rotation-free model.
- `without_a_snap_the_core_is_bit_identical_to_the_pre_snap_core` adapted to lane-uniform decays: forward still bit-identical, input gradients within reassociation.
- Existing fused-vs-composed checks now cover the per-channel backward: `transport_logits_equal_the_forward_until_snapped`, `u1_recurrence_matches_composed_mixer_and_finite_differences`.
- `cargo test -p uor-r4-training --lib`: **603 passed, 3 failed**; the 3 (`joint_campaign::{calibrated_shadow_preserves..., radial_initialization_resume..., radial_transfer_checkpoint_resume...}`) fail identically with the pre-change test binary, so they are pre-existing and unrelated.
- `uor-r4-lut` 22 passed; `uor-r4-integer` lib 199 passed (its `adversarial_challenger_m1_token_boundary` failures need an external fixture tree that is absent from this machine).
- `cargo test --release -p uor-r4-training --test stack_d11_oracle`: **6 passed** after a clean rebuild of the three touched packages — the multiplier-free D11 engine and the D10 LUT engine return identical logits on exported per-channel artifacts (random, amplified and GPTQ-calibrated), and the corrupted-container cases still agree.
- End-to-end on the trained width-288 model: `export` then `lut-evaluate` and `d11-evaluate` on the same `model.lut` (sha256 `04faf0d9…`): **D10 NLL = D11 NLL = 3.6745829155120724, max absolute logit difference 0, first difference none**.
- `cargo check -p uor-r4-training --features metal --lib` passes, so the Metal path compiles; its MSL kernels are runtime-compiled and were not executed here.

## Training comparison (both arms run)

600 steps, batch 8, context 128, width 288, one `r` layer, Lorentz read, rotation on, seed 1, identical 4M-token train store and 250K-token dev store, 2 threads, same build configuration (release + `cpu-accelerate`), arms run sequentially.

| step | per-lane dev NLL | per-channel dev NLL |
| --- | --- | --- |
| 50 | 4.513551 | 4.506875 |
| 100 | 4.148134 | 4.132269 |
| 150 | 3.961084 | 3.972649 |
| 200 | 3.835362 | 3.834629 |
| 250 | 3.759891 | 3.751782 |
| 300 | 3.675105 | 3.669148 |
| 350 | 3.589051 | 3.587557 |
| 400 | 3.513523 | 3.518828 |
| 450 | 3.465322 | 3.474808 |
| 500 | 3.430826 | 3.437990 |
| 550 | 3.404445 | 3.414931 |
| 600 | 3.378708 | 3.393137 |
| final (65,536 targets) | **3.387052** | **3.394493** |

No measurable advantage in this run: per-channel is 0.00744 nats **higher** on the final development evaluation (0.22% relative), a single short run that is far from converged. Training throughput was the same within noise (6,650 vs 6,905 tokens/s average; a 20-step timing run had suggested per-channel about 25% slower). The per-channel benefit claimed in the GLA/Mamba literature is not reproduced or refuted by a 600-step run; a longer, converged comparison would be needed.

## Uncertainties

- The Metal kernels are updated but could not be executed or compiled on this machine (no GPU authorized); the Rust side is behind `--features metal`, and the MSL is compiled at runtime.
- `float_exponent`-style shared-timescale initialization is unchanged in kind but now spreads 288 channels over the same fast→slow range, so the initial decay values are not identical between the two parameterisations (all other variables initialize from the same RNG stream, since the decay branch draws no random values).
- The Metal parity integration test (`tests/metal_stack_ops_parity.rs`) is not exercised here.
- Release test artifacts were built through a target directory shared with a baseline control worktree; an oracle run before `cargo clean -p uor-r4-lut -p uor-r4-integer -p uor-r4-training --release` mixed the two sources' rlibs and reported a spurious D10 rejection. The 6/6 pass above is from the clean rebuild, and the debug suite and the two training binaries were each verified to carry their own source's behaviour (`decay count differs from the lane count` in the control, `... from the width` in this branch; 2,175,984 against 2,176,200 parameters).


## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### comment 5970918170 — 2026-10-03T16:09:38Z — Casey-allard

Owner-directed merge audit: this draft has real new-artifact CPU/integer evidence, but is not safe to merge as an unconditional replacement. It changes rec.decay training shape lanes→width and LUT/integer decay_rate acceptance lanes→width without an old-shape loader, migration or explicit format/selector admission. That can reject accepted parent artifacts. Keep draft until compatibility is preserved and the relevant Metal path is exercised; the neutral short-fit result is not a family-wide negative. Serving exp wording must also distinguish offline/reference from native served arithmetic. No new fit or peer source edit was made by this audit.

### comment 5971076280 — 2026-10-03T16:27:37Z — Casey-allard

**DISPOSITION: PARKED — do not merge as implemented. Preserved for its evidence, not adopted as a replacement.**

Codex's audit is correct and I accept it. Parking rather than migrating, with reasons and the exact correction below.

## Why parked, not migrated

The change is **structurally incompatible with accepted artifacts**, and it does not need to be compatible because nothing needs it:

```
- shapes.insert(name("rec.decay"), vec![lanes]);   // 72
+ shapes.insert(name("rec.decay"), vec![d]);       // 288
```

`rec.decay` (and the exported `decay_rate`) move from **lanes to width** with **no admission or migration path for accepted lane-decay artifacts**. Codex states the point precisely: **new-artifact D10/D11 parity does not establish old-artifact compatibility.** Those are different artifacts; parity between the new ones says nothing about loading the old ones.

**And per-channel decay is not needed by the active plan.** The experiment returned a **neutral result** — the implementation is correct (forward reduces bit-identically to the frozen per-lane core when the four channel decays are equal) but it measures **0.00744 nats worse (0.22%)** over 600 steps. A neutral change is not worth a migration layer, and Codex is right that the neutral short-fit **does not justify another dose or sweep**.

So: **preserve the source and results on the branch; do not merge.**

## CORRECTION to my own reported wording

I wrote: *"end-to-end `export` -> `lut-evaluate`/`d11-evaluate` gives D10 = D11 = 3.6745829155120724 NLL with max logit difference 0. The per-channel table form serves with no new matmul."*

**That sentence conflates two different things and I am withdrawing the serving half of it.** What was actually verified is **offline/reference computation**: an export followed by a reference evaluation path. What was **not** verified is the **native served contract** for this variant. Those are not interchangeable, and "serves exactly" implied the second from evidence about the first.

Corrected statement:

> **Offline/reference parity holds** — `export` then `lut-evaluate`/`d11-evaluate` gives **D10 = D11 = 3.6745829155120724 NLL, max logit difference 0**, on **newly exported** per-channel artifacts. This is reference-path agreement on one artifact pair. It is **not** evidence that the native served contract holds for this variant, and it is **not** evidence that previously accepted lane-decay artifacts remain loadable.

**Additionally: Metal execution for this variant was never verified.** The Metal kernels were not executed or GPU-compiled — MSL compiles at runtime and no GPU was available — and this variant changes the decay shapes the Metal kernels touch. Given that this same Metal path produced a non-finite divergence (`RecurrenceOut(4)`, fixed for GELU fast-tanh in #1636), the Metal side of this change is **unverified, not passing**.

## What is preserved

- **Branch** `codex/per-channel-decay` @ `ed2e7261db1319966655b68f1a88188d5f70f7da` — pushed, source intact
- **Results**: `+216` params per r-layer measured from built artifacts (`width − lanes` = 288 − 72), matching prediction
- **Forward-invariance test**: with a lane's four decays equal, the new core is **bit-identical** to the frozen `LegacyRecurrenceCore` — the one result here that has lasting value, since it is the correctness precondition any future attempt would need
- **Training comparison**: per-lane **3.387052** vs per-channel **3.394493** at 600 steps, one r-layer, width 288, matched data/seed/threads

This PR stays **draft and not-for-merge**. No merge, no migration, no further fit.

References #1512


### comment 5976559307 — 2026-10-04T04:26:26Z — Casey-allard

Closing and reopening only to trigger a fresh advisory delivery-evidence run on the current workflow (#1651 reports drafts as not applicable). No content change.

### comment 5976691287 — 2026-10-04T04:48:26Z — Casey-allard

Closed on owner direction (4 October): the per-channel decay experiment is recorded as a negative in the DeepSeek lane (#1512) and the retired-mechanisms catalogue. The branch `codex/per-channel-decay` stays on GitHub. Reopen if the line resumes.


---
_Comment/review ids in this file: 5970918170, 5971076280, 5976559307, 5976691287_

