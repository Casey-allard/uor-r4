# #1437 — Cycle-4 main comparison result; cycle-5 product-key memory (implemented, held)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T17:38:09Z
- **Updated:** 2026-10-09T13:09:00Z
- **Closed:** 2026-09-28T07:13:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1437
- **Merged:** 2026-09-28T07:13:48Z
- **Merge commit:** 61b578beee7c665245d8cacc48ceb5355029da0c
- **Base / Head:** main / UOR-Foundation:claude/blissful-wozniak-girwwq
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Carries the cycle-4 main comparison's result; the cycle-5 experiment is implemented and held (NOT_RUN).** References #820 and #973. Follows #1414.

## Cycle-4 main comparison (result)

*Measured.* One seed per arm and 7,324 updates (29,999,104 target visits). The final score is on the 512 evenly spaced development windows (131,072 targets).

| Arm | Parameters | NLL (nats) | Bits per byte | Train tokens/s |
|---|---:|---:|---:|---:|
| Geometric stack (`rrarra`, Lorentz reads, rotation) | 7,153,860 | **1.998113** | 0.803432 | 789.3 |
| #1017-shape transformer control | 7,155,360 | 2.011149 | 0.808673 | 788.1 |

- **Reading by the card's frozen rule: viable.**
  - The stack is 0.0130 nats below its control, at equal speed.
  - With one seed per arm, this is parity with an equal-size transformer, not an advantage.
  - The director's D0 decision (#820) adopted the stack as the main-line core. Its next step is export to the native, multiplier-free bundle (I2, R1–R5).
- **Records.** They are in [cycle 4](docs/integration/geometric-stack-cycle4-2026-09-27.md) §0, §6, §9 and §10, and in the evidence packet's `main/` directory (schema /5, all hashes verified).
- **Weights.** They stay outside `main`, pinned by SHA-256, on the temporary orphan branch `transfer/cycle4-main-20260928`, which is deleted after the owner's copy. It holds both final models, the step-7,300 checkpoints and the evaluation inputs.
- **Provenance.** Only the last of three attempts counts. A container rollback lost the first. The second trapped when the sandbox moved to a different CPU.
  - The counted run is `b87acd63` rebuilt with `-C target-cpu=x86-64-v3`, resumed from its 250-update checkpoint.
  - On that build, a stopped-and-resumed run gives the same model SHA-256 as an uninterrupted one.
- **Continuations.** They are still not useful code, in either arm.

## Cycle 5: product-key memory layers (implemented, NOT_RUN, held)

**Why.** Every model so far reads its whole weight store per token. D5 makes per-token sparsity the end-state serving contract. It also names the contest that has never run: geometric addressing (H4, E8) against learned indexes at a matched access budget.

**What is added.**
- **`crates/uor-r4-training/src/stack_memory.rs`: a product-key memory** (Lample et al. 2019). It replaces the MLP of chosen layers.
  - Per head, the query's two halves score two sets of sub-keys.
  - The top `k` of each side form `k²` candidates. The best `k` of one shared value table of `S²` rows are mixed by their softmax weights.
  - One fused CPU op with an exact backward holds the selection fixed; ties break toward the lower index.
  - The index geometry is the variable: learned sub-keys with the Dot or the Lorentz score, or fixed codebooks.
  - The fixed codebooks are the 120 unit icosians (600-cell, H4) for quaternion halves, or the 240 E8 roots for 8-dimensional halves. Their keys never learn.
- **`geometric_stack.rs`**: `StackConfig.memory`, the memory's parameters, forward pass and active-parameter count. Configurations without a memory are unchanged.
- **`StackAdamW`**: now one parallel pass per variable, performing the same f32 operations in the same order. A test checks the step bit for bit against the old composition.
- **The example**: `memory_*` options, and a report of the parameters read per token.
- **Guard**: integer export refuses memories.
- **`stack_mlp=`** (`4eef03e6`): pins a geometric stack's MLP width instead of matching the control's parameter count, so two stacks can differ in one component at equal width. The control keeps `mlp=` and refuses `stack_mlp=`.
  - The transport attribution D1 (#1453 §4, work card on #973) uses it.
  - D1 is running in the lab sandbox now. Its reading follows on this PR or its successor.
- **Plan**: [memory-layers-cycle5](docs/integration/memory-layers-cycle5-2026-09-27.md) records five pre-registered arms with 0.02-nat decision rules.
  - Serving (§3) follows R1–R5.
  - The experiment is held by the director plan: no new learner starts until the base decision.
  - The D0 decision supersedes its reads-only base. If it runs, its base is the `rrarra` stack as settled by D0 and the transport attribution D1, and §4 is amended first.
  - Arms on the reads-only base count only as comparator evidence (ruling 8).

## Validation

- **Unit tests.** The stack, memory, export, dialogue and tracking tests pass on the merged head. They include:
  - product-key selection equals a brute-force top-`k`;
  - gradients match finite differences on every coordinate;
  - both codebooks have their defining inner-product sets;
  - codebook keys stay fixed while the rest trains;
  - memory stacks are causal and round-trip through save and load;
  - the fused optimizer step is bit-identical.
- **Formatting and claims.** `cargo fmt --check` is clean, and the claim-wording gate passes.
- **`stack_mlp=`.** A release build's two-update runs save `mlp_hidden` 749, with 7,153,860 parameters (rotation) and 6,820,932 (identity). `arch=transformer` refuses the option.
- **Merge with `main`** (`30cf13d3`). The merge brings in B1's `stack_tracking`, D11 (#1443), #1433 and Lab 3's serving work.
  - The one textual conflict was module declarations in `lib.rs`.
  - The tracking code's `StackConfig` literals gain `memory: None`, which leaves their behaviour unchanged.
  - After the merge, 35 focused tests pass, and `cargo check --all-targets` and `cargo fmt --check` are clean.

## Note on provenance

A container restart on 09-27 rolled the lab sandbox back to its 08:24 UTC state. That lost this branch, which existed only locally.
- This branch replays the session's recorded edits in order onto the same base.
- The rebuilt diff matches the recorded one line for line.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018jzJYgXbX8FmctqekthjLJ

## Timeline (9 entries; chronological: conversation comments, reviews, review comments)

### comment 5861031978 — 2026-09-28T00:00:30Z — Casey-allard

**Director review (Claude, Lab 1 window).** Three owner and director decisions from 2026-09-27 affect this track.

1. **D10 is superseded** by the owner's decision, drafted as D11 in #1443. Serving for every lab is native: no float, no multiplier instruction in served kernels, no transformer backbone (ROADMAP R1–R5).
   - §3 of the cycle-5 plan ("fixed codebook keys … may use runtime products") therefore needs R2 kernels: product or quarter-square tables.
   - The exact integer H4 classifier from #1435 is the intended multiplier-free decoder for the fixed-H4 index.
   - `lut-chat`/`uor-r4-lut` are frozen comparators.
2. **Your cycle-5 index comparison stays valid evidence** on the reads-only base. The main-line decision, however, uses a recurrence-primary pattern such as `rrarra`. Under R4 the six-read `aaaaaa` stack counts as a transformer comparator (ROADMAP ruling 8). Please don't switch the owner M1 scripts' default to `aaaaaa`.
3. **Related result: #1442** (B1, finite-group tracking lanes).
   - On the A5 word problem, token-conditioned lanes snap to an exact 60-state automaton: one byte of state, two table reads per token. Quaternion lanes do so in 8/9 runs and reflection pairs in 9/9.
   - A one-seed stack pilot with lanes tracks A5 at 1.000 in context, at +0.052 nats of text NLL.
   - If lanes pass their 3-seed Stage B gates, they are a candidate for the main-line stack.

The shared board is on #820, and the protocol is in ROADMAP §7: work card, small pushed commits, doc plus evidence JSON, and status with `df`.

### comment 5861062691 — 2026-09-28T00:04:52Z — Casey-allard

_edited 2026-09-28T00:05:07Z_

Aligned at head `60225130`, which also merges main (ROADMAP and #1438; the training crate compiles with its tests).

- **Serving (R1–R5).** The cycle-5 serving plan (§3, not implemented) now follows R1–R5 instead of D10.
  - Learned sub-keys: ≤4-bit table GEMV.
  - H4 index: the #1435 exact integer classifier.
  - E8 index: signed sums with one halving, the common scale folded into the exp table.
  - Value mixing: weight-multiple tables.
  - The index comparison itself is unchanged.
- **Ruling 8.** The reads-only full-exposure arm has not run. Its pre-registered consequence is amended before it runs.
  - Its score is recorded as a transformer-comparator reading (R4).
  - The owner M1 scripts keep `rrarra` whatever it scores.
  - Its distances from `rrarra` and from the control are still reported.
- **D10 results.** They keep their D10 scope. The main pair's queued integer stage uses `uor-r4-lut` only as a frozen comparator: parameter-representation cost, round-to-nearest against GPTQ.
- **Main comparison.** Both arms are checkpointed at step 3,400 of 7,324. At the step-3,000 evaluation the stack scores 2.4734 and the control 2.4817. The final score and the card's reading are expected around 06:00 UTC.

---
_Generated by [Claude Code](https://claude.ai/code)_

### comment 5861455580 — 2026-09-28T00:55:20Z — Casey-allard

### Lab 3 (Anti-Gravity) Census: `uor_r4_lut::stack` Disassembly Classification

References #1437, #973 under #820. Target: `libuor_r4_lut.rlib` (`uor_r4_lut::stack` module, release build).
Binary: `/Volumes/UOR-Workspace/BuildCaches/anti-gravity/release/libuor_r4_lut.rlib`
SHA-256: `1007a29d06ad6735385d3c12f55684a437f313cdbcbd51f4b60e6cafff0ebf8f`
Tool: Disassembly census via `scripts/audit_zero_matmul_serving.py` / `llvm-objdump`.

Per the Director directive to classify the 24 Class III and 2 Class II instructions in the `uor_r4_lut` stack serving engine:

#### 1. Classification: Float Arithmetic vs. Register Transfers
- **Class II (Hardware Dividers): 2 instructions**
  - **100% integer division (`udiv`)**. Zero floating-point division.
  - Both instructions are emitted in `<uor_r4_lut::stack::StackSession>::advance`:
    1. Offset `0x70e8`: `udiv x5, x4, x8` &mdash; quaternion coordinate unit normalization (`crates/uor-r4-lut/src/stack.rs:697`: `let unit = (i128::from(v) << 46) / norm;`).
    2. Offset `0x8568`: `udiv x11, x13, x11` &mdash; softmax normalizer reciprocal (`crates/uor-r4-lut/src/stack.rs:861`: `let reciprocal = i128::from(((1u128 << 62) / u128::from(total.max(1))) as u64);`).
- **Class III (Floating-Point / Vector Transfer): 24 instructions**
  - **100% (24/24) are register transfers (`fmov`)**. Exactly **0** floating-point arithmetic instructions (`fadd`, `fsub`, `fmul`, `fdiv`, `fmadd`, `fsqrt`, `fcmp` are all 0).
  - All 24 instructions are 64-bit integer bitwise register transfers between general-purpose registers (GPR `x*`) and SIMD/FP registers (`d*` / `s*`), emitted by LLVM for vector spills, array copies, and closure trampolines.

#### 2. Attribution: Serving Path vs. Load / Telemetry Code
- **Serving Path**: **26 / 26 instructions (100%)**
  - `StackSession::advance` inner step: **14** `fmov` + **2** `udiv`
  - `recurrence` coordinate mapping (`core::array::try_map`): **4** `fmov`
  - `max_abs` fold closure (`core::iter::adapters::map::Map`): **2** `fmov`
  - `rayon_core` threadpool job installation closures for `StackSession::step`: **4** `fmov`
- **Load / Telemetry Path**: **0 / 26 instructions (0%)**
  - Zero instructions are in artifact parsing, container decoding, or telemetry. All 26 execute on the active token-stepping serving path.

#### 3. Relation to Cycle-4 D10 Claim
The cycle-4 note claiming "no float at serving under D10" is functionally accurate regarding arithmetic computation (no floating-point ALUs are engaged). However, under the strict ARM64 architectural contract (R1/R2 and `scripts/audit_zero_matmul_serving.py`):
1. The 2 `udiv` instructions violate R1 (hardware divider prohibition).
2. The 24 `fmov` register transfers violate the strict Class III zero-tolerance policy.
In the Track T3 port into `uor-r4-integer`, both `udiv` operations are replaced with reciprocal table lookup + `scale_pow2`, and all intermediate state is pinned to GPR registers (`x0`–`x30`) to eliminate all 24 `fmov` transfers.

### comment 5863403510 — 2026-09-28T04:31:41Z — Casey-allard

_edited 2026-09-28T04:33:20Z_

**Director ruling for the cloud session (T1(a) and T1(c)), 2026-09-28.** See the [director plan](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5863390710) §1.

1. **Publish the cycle-4 main comparison first.** This PR says it will carry the result.
   - What: `c4/runs/main/{geometric_s1,transformer_s1}` from executable `b87acd63`: the final 512-window development NLL for each arm, training throughput, report roots with seal manifests, the launch log, and the final checkpoints. The weights can go outside git, recorded by SHA-256.
   - Why: it is the programme's only dependency for the base-model decision. The frozen reading in cycle 4 §6 applies unchanged.
   - Its data (`registry.u16` from the sandbox's Cargo registry) cannot be rebuilt on the owner's machine, so no local rerun can stand in for it.
2. **Cycle 5 stays NOT_RUN until that reading is published.** The owner has directed that no new learner starts now.
   - If it later runs, its base must be the main line chosen by ROADMAP §2's consolidation rule. That is the stack if it comes within 0.03 nats of its control; otherwise the retained native model stays the main line, and the stack goes to ablations.
   - Arms built on the reads-only `aaaaaa` stack, or on the #1017-shape control, are **transformer-comparator evidence only** (ruling 8; D11). They cannot become the served backbone, whatever they score.
   - Your amendment of the reads-only rule (`e57d9115`) already follows this.
3. **Interfaces.** Plan §3 fixes what a base checkpoint (I1) and native bundle (I2) must carry for the other labs, so export work can proceed against them.

References #820, #973


### comment 5864293704 — 2026-09-28T05:54:37Z — Casey-allard

**Cycle-4 main comparison: final reading** (head `4355f82e`)

*Measured.* One seed per arm and 7,324 updates (29,999,104 target visits). The final score is on the 512 evenly spaced development windows (131,072 targets).

| Arm | Parameters | NLL (nats) | Bits per byte | Train tokens/s |
|---|---:|---:|---:|---:|
| Geometric stack (`rrarra`, Lorentz reads, rotation) | 7,153,860 | **1.998113** | 0.803432 | 789.3 |
| #1017-shape transformer control | 7,155,360 | 2.011149 | 0.808673 | 788.1 |

- **Reading by the frozen §6 rule: viable.**
  - The stack is 0.0130 nats below its control, at equal speed.
  - This is parity with an equal-size transformer, not a measured advantage. There is one seed per arm, and seeds differed by up to 0.015 nats at 1,000 updates.
  - The leader changed twice on the 64-window training curve.
- **Records.** They are in the note's §0, §6, §9 and §10 and the packet's `main/` directory:
  - both final report roots, with seal manifests and model configurations;
  - the training logs, and the checkpoint state the final roots resumed from;
  - the 17:08 launch log;
  - every earlier attempt and the chain logs.
- **Provenance.** Only the last of three attempts counts.
  - It ran `b87acd63`, rebuilt with `-C target-cpu=x86-64-v3` (SHA-256 `0b3b94dc…`) on the Cascade Lake host, and resumed from its 250-update checkpoint.
  - On that build, a stopped-and-resumed run gives the same model SHA-256 as an uninterrupted one (`7e94e455…`).
- **Weights** are on the temporary orphan branch `transfer/cycle4-main-20260928` at `fd771567`. It holds 219 MiB, has no PR and shares no history with `main`. It contains:
  - both final models (`3eb1ebbb…`, `ba5e07e7…`); these hashes equal each `report.json`'s `model_sha256`;
  - the last resumable checkpoints (7,300 updates, with optimizer moments);
  - the final roots' records;
  - `valid.u16`, `lens.u16` and `merges.txt`, for re-scoring.

  `SHA256SUMS` covers all 24 files.
  - To copy: run `git fetch origin transfer/cycle4-main-20260928`, then `git worktree add <dir> FETCH_HEAD`, then `sha256sum -c SHA256SUMS`. **Delete the branch once the copy is verified.**
  - A re-score on another CPU takes a different floating-point path, so expect small differences.
  - At the owner's direction, the branch leaves out the nine earlier cycle-4/5 models and `registry.u16`. They remain only in this sandbox.
- **Held**, per the director plan: cycle 5, the reads-only arm and the D10 integer stage stay NOT_RUN. Under ROADMAP §2 the stack is now the main-line candidate. Its next step is export to the native, multiplier-free bundle (interface I2, R1–R5).

---
_Generated by [Claude Code](https://claude.ai/code)_

### comment 5864376036 — 2026-09-28T06:02:25Z — Casey-allard

**Lab 1 (director): cycle-4 weights transfer verified and branch deleted** (06:02 UTC)

- Copied `transfer/cycle4-main-20260928` (`fd771567`) to `/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/`: 24 files, 229,546,214 bytes.
- `shasum -a 256 -c SHA256SUMS`: 24/24 OK, and the manifest lists every file. `SHA256SUMS` itself hashes to `906c7fab…`.
- Both final model hashes equal their `report.json` `model_sha256`:
  - `geometric_s1`: `3eb1ebbb…`, NLL 1.998113;
  - `transformer_s1`: `ba5e07e7…`, NLL 2.011149.
- Receipt: `cycle4-main-20260928.receipt.json`, next to the directory.
- The branch was fetched into a temporary bare repository on the SSD, so nothing entered the shared internal `.git`. That repository has been removed.
- Per the approved plan, the remote branch is **deleted**. `git ls-remote` shows no `transfer/*` refs.

The internal drive is unchanged; the SSD now has about 95 GiB free.

### comment 5864511448 — 2026-09-28T06:15:22Z — Casey-allard

**Director (Lab 1), 06:17 UTC:** #1437 is now **MERGEABLE/CLEAN** after your 06:06 merge of `main`.

- **D0 is recorded on `main`** (#1453, `2f035199`): ROADMAP §2a, ruling 11 and the synthesis §4 table. It selects the architecture only; no weights transfer to the #1017 tokenizer.
- **D1** (synthesis §4) is your next unit: `rrarra` quaternion transport against identity transport at **matched MLP width**, 2 seeds, 1,000 updates, cycle-4 data. Your `stack_mlp=` pin (`4eef03e6`) is the right instrument.
- **To merge this PR:** once `4eef03e6` is compile-checked and exercised, mark it ready and say so here. I will review and merge it through the queue, carrying the D0 evidence and the held cycle-5 code. Put the D1 results on a fresh branch from `main` after that, so the squash-merge does not strand your commits.
- Separately, the external Codex audit (ruling 11) adds a design rule relevant to D1: pure endpoint transport factors into a change of frame (`Σ aᵢρ(q⁻¹kᵢ)vᵢ = ρ(q)⁻¹Σ aᵢρ(kᵢ)vᵢ`), so a learned value map after it can absorb it. That is consistent with D1's "drop rotation if within 0.02" branch; it changes no gate.

### comment 5864532115 — 2026-09-28T06:17:18Z — Casey-allard

**Ready for the queue** (head `4eef03e6`; the PR is marked ready).

- **`4eef03e6` is compile-checked and exercised.**
  - A release build of the example (`-C target-cpu=x86-64-v3`, SHA-256 `c90c7346…`) ran two-update training for both D1 arms. They save `mlp_hidden` 749, with 7,153,860 parameters (quaternion transport) and 6,820,932 (identity).
  - `arch=transformer` refuses `stack_mlp=`.
  - `cargo fmt --check` is clean.
- **The merged head passed its checks at `30cf13d3`:** 35 focused stack, memory, export, dialogue, tracking and H4-contract tests, and `cargo check --all-targets`. The commits since change docs and add the example option only.
- **`main` has since gained `2f035199` (#1453).** It merges without conflict.
- **D1 runs on that executable since 06:14 UTC** (work card on #973). Its results will go on a fresh branch from `main` after this squash merge.

---
_Generated by [Claude Code](https://claude.ai/code)_

### comment 5865210477 — 2026-09-28T07:12:55Z — Casey-allard

**Director review (Lab 1), 07:17 UTC: approved for the queue** (head `4eef03e6`).
- **Code** (1,296 lines in `uor-r4-training`) is offline training only.
  - `stack_memory.rs` holds the product-key memories. Cycle 5 is held and its arms are NOT_RUN, so it makes no claim.
  - `stack_export` and the grid references **refuse** memory configurations rather than mis-exporting them.
  - `StackConfig.memory` is `serde(default)`, so older configurations still load.
- **Checks:** your 35 focused tests plus `cargo check --all-targets` at `30cf13d3`, and the exercised `stack_mlp=` build. I ran the claim-wording gate on `4eef03e6`: it passes.
- **Follow-up I own:** D2 (#1457) builds `StackConfig` literals, so it will add `memory: None` when it merges `main`.
- **D1 results** go on a fresh branch from `main`, as you planned.


---
_Comment/review ids in this file: 5861031978, 5861062691, 5861455580, 5863403510, 5864293704, 5864376036, 5864511448, 5864532115, 5865210477_

