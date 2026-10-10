# UOR-R4 Geometric Language Model - roadmap

Updated 10 October 2026. **Pre-alpha.** Useful conversation, broad reasoning, coding, frontier
capability and lower complete-path energy are not established.

Milestones M1-M8 below, with the standing serving rules in section 0. Live status and claims:
tracker [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028); each milestone is its own issue. Results:
[current state](docs/integration/current-state.md). Owner decisions:
[DECISIONS](docs/integration/DECISIONS.md). Long-form plan and history:
[canonical plan](docs/integration/project-track.md). The 29 September roadmap is preserved in
[docs/history/roadmap-2026-09-29-snapshot.md](docs/history/roadmap-2026-09-29-snapshot.md).

## Milestones

Two lines meet at M4: line A (language base, M1) and line B (grounded reply from exact memory,
M2 and M3). Thresholds marked PROPOSED on M1, M2 and M4 await owner confirmation; a milestone
closes only when its acceptance is met on the saved model.

| | Milestone | Status | Goal and acceptance summary | Latest result |
|---|---|---|---|---|
| M1 | [Language base](https://github.com/UOR-Foundation/uor-r4/issues/2029) (Claude, DeepSeek) | in progress | A trained geometric language model good enough to carry chat and memory; PROPOSED language-quality threshold on the saved model | 214M base dev NLL 2.073; 19.9M chat stack 0.933 BPB served at 64 windows (0.877550 float / 0.886838 served matched at 512 windows — the figure is protocol-dependent, [pinned protocol](docs/labs/criterion2-protocol-pin-2026-10-09/README.md)) |
| M2 | [Grounded reply from exact memory](https://github.com/UOR-Foundation/uor-r4/issues/2030) (Codex) | in progress | Replies emitted from the exact store by learned geometric operators; PROPOSED completion threshold | [Cross-state continuation](docs/labs/m2-cross-resume-2026-10-10/README.md) improves saved complete replies **22→145/512**: 126 gains, 3 losses,19 retained; six of the original eight remain. KEEP, target 256 then fresh 40% still unmet. Only the 115,200-coefficient field learns; Source48/Generate64 upstream stays frozen. Four additional full-panel passes, fresh Adam, sole saved endpoint; count 0/3. Length8 remains weak at 2/128. Next: registered continuation from saved 145 after protected delivery and cleanup. The ordinary24/96 recipe and constructor line remain closed; separate historical conditional gate9/15 was not rerun. |
| M3 | [Durable conversation memory](https://github.com/UOR-Foundation/uor-r4/issues/2031) | in progress | Memory persists across sessions and answers stay grounded | evaluator and session delivered; not qualified |
| M4 | [One served model, D11 and CLI](https://github.com/UOR-Foundation/uor-r4/issues/2032) | in progress | One model served multiplier-free through the shipped CLI, with no softmax-shaped normalization at runtime (item 0); PROPOSED parity threshold | D11 engine bit-exact; `uor-chat --stack` serves the stack (#2050); read layers still use a table-emulated softmax |
| M5 | [Laptop cost, D5](https://github.com/UOR-Foundation/uor-r4/issues/2033) | not started | Measured J/token, RSS and tokens/s on the M1 with selected parameter access | none |
| M6 | [Reasoning and coding](https://github.com/UOR-Foundation/uor-r4/issues/2034) | not started | Executable reasoning and coding on the same model | exact arithmetic is the visible gap |
| M7 | [API, WASM, Pages Studio](https://github.com/UOR-Foundation/uor-r4/issues/2035) | not started | The model reachable through API, WASM and the Studio | local API only |
| M8 | [Alpha release](https://github.com/UOR-Foundation/uor-r4/issues/2036) | not started | Owner-declared alpha | none |

Standing direction (owner, 3 October): no transformer baselines; mechanisms are compared with matched geometric alternatives and with the previous best geometric model.

Standing issues: compute board [#2037](https://github.com/UOR-Foundation/uor-r4/issues/2037); open bugs [#1542](https://github.com/UOR-Foundation/uor-r4/issues/1542), [#1476](https://github.com/UOR-Foundation/uor-r4/issues/1476),
[#1718](https://github.com/UOR-Foundation/uor-r4/issues/1718), [#1738](https://github.com/UOR-Foundation/uor-r4/issues/1738).

## Experiments in flight

| Experiment | Milestone | Status | Question |
| --- | --- | --- | --- |
| Native VSA retraining (4 arms × 2 seeds) | M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) | **done: KEEP** ([record](docs/labs/vsa-native-test-2026-10-09/README.md)) | Trained VSA (fixed codes) improves held-out BPB by 0.014–0.023. Icosian-root codes don't: they collapse token identity. Mode 2 (root + per-token residual) is worse than fixed codes on all 4 cells: not KEEP |
| Softmax-free reads: soft (A), flock rank (B), B + prime-route copy (C), Hamming-rank (D) | M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) | pre-registered | Can served reads drop the table-emulated softmax with no loss? |
| Route-holonomy read | M1 #2029 | pre-registered | Can the angle of h_j⁻¹·h_t rank earlier positions, order-aware and softmax-free? |
| Exact icosian holonomy lanes (E1) | M1 #2029 | pre-registered | Does an exact 2I group product beside the r-layer help, beyond a shuffled-geometry control? |
| Octonion-signed binding, then transport | M1 #2029 | pre-registered | Does a Fano-signed XOR keep order and grouping that plain XOR loses? |
| Shared `BitCode` primitive | M4 #2032 | planned (engineering) | One Hamming/popcount type for the native learner, R4G1 and the integer engine |

Pictures of the pre-registered mechanisms are in [docs/geometry.md § 11](docs/geometry.md#11-pre-registered-mechanisms-not-yet-measured); none of them has a measured result yet.

## 0. Mission and hard runtime rules

**North star: geometric intelligence.** The goal is a working geometric language model whose
runtime uses integer, fixed-point and geometric operations only: no transformer, no matrix
multiplication and no floating point. Training, data preparation, analysis and offline tooling
may use floats and matrix products. None of those may leak into the serving path.

The owner reaffirmed this target on 2026-09-27: "Keep the native, multiplier-free serving
target" ([#820](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5852795279)).

The rules below apply to every lab. A serving change that breaks any of R1–R4 does not enter
the mission path.

| Rule | Meaning | Evidence required |
|---|---|---|
| **R1: no float** | No f32/f64 instruction and no libm call in any served symbol. | An instruction audit of the release binary (`scripts/serving_multiplier_check.py`, `scripts/audit_zero_matmul_serving.py`). |
| **R2: no multiplier** | No integer multiply or divide instruction in served kernels ([D0-b](docs/integration/DECISIONS.md#d0-b--what-no-matmul-at-serving-means-adopted)). Products of runtime values use product or quarter-square tables, or exact geometric structure (signed permutations, ℤ[φ] shift-add). D10's hardware-multiplier exception is not adopted by any lab. | The same audit. |
| **R3: no matmul** | End state: no dense per-token access to a learned weight store ([D5](docs/integration/DECISIONS.md#d5--per-token-parameter-sparsity-is-the-terminal-serving-invariant)). In the interim, ≤4-bit maps executed as adds, shifts and table reads are permitted as labeled stepping stones. Each must be reported with its dense per-token parameter reads, and each must be owned by a track that is replacing it with addressed or sparse access. | Per-token parameter reads in every serving report. |
| **R4: no transformer** | No served backbone whose primary mechanism is stacked dense all-pairs attention plus MLP. A model counts as a transformer when most of its token-mixing layers are dense all-pairs reads. Converted open-weight transformers (D10's SmolLM2) may serve only as comparators or offline teachers. Recurrence-primary hybrids (most token mixing recurrent) are interim, and their reads move to bounded geometric or exact addressing (D5/D6). | An architecture statement in the PR. |
| **R5: honest cost** | An integer path is not an efficiency result. Energy may be claimed only when measured on the M1 in J/token. Bytes touched per token set the energy floor, not multiplier count ([first-principles review §5](docs/integration/first-principles-review-2026-09-25.md)). | Measured J/token, RSS and tokens/s. |
