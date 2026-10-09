# UOR-R4 Geometric Language Model - roadmap

Updated 9 October 2026. **Pre-alpha.** Useful conversation, broad reasoning, coding, frontier
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
| M1 | [Language base](https://github.com/UOR-Foundation/uor-r4/issues/2029) (Claude, DeepSeek) | in progress | A trained geometric language model good enough to carry chat and memory; PROPOSED language-quality threshold on the saved model | 214M base dev NLL 2.073; 19.9M chat stack 0.933 BPB served |
| M2 | [Grounded reply from exact memory](https://github.com/UOR-Foundation/uor-r4/issues/2030) (Codex) | in progress | Replies emitted from the exact store by learned geometric operators; PROPOSED completion threshold | 8 of 512 complete; gate 9 of 15 |
| M3 | [Durable conversation memory](https://github.com/UOR-Foundation/uor-r4/issues/2031) | in progress | Memory persists across sessions and answers stay grounded | evaluator and session delivered; not qualified |
| M4 | [One served model, D11 and CLI](https://github.com/UOR-Foundation/uor-r4/issues/2032) | not started | One model served multiplier-free through the shipped CLI; PROPOSED parity threshold | D11 engine bit-exact; CLI cannot serve the stack yet |
| M5 | [Laptop cost, D5](https://github.com/UOR-Foundation/uor-r4/issues/2033) | not started | Measured J/token, RSS and tokens/s on the M1 with selected parameter access | none |
| M6 | [Reasoning and coding](https://github.com/UOR-Foundation/uor-r4/issues/2034) | not started | Executable reasoning and coding on the same model | exact arithmetic is the visible gap |
| M7 | [API, WASM, Pages Studio](https://github.com/UOR-Foundation/uor-r4/issues/2035) | not started | The model reachable through API, WASM and the Studio | local API only |
| M8 | [Alpha release](https://github.com/UOR-Foundation/uor-r4/issues/2036) | not started | Owner-declared alpha | none |

Standing direction (owner, 3 October): no transformer baselines; mechanisms are compared with matched geometric alternatives and with the previous best geometric model.

Standing issues: compute board [#2037](https://github.com/UOR-Foundation/uor-r4/issues/2037); open bugs [#1542](https://github.com/UOR-Foundation/uor-r4/issues/1542), [#1476](https://github.com/UOR-Foundation/uor-r4/issues/1476),
[#1718](https://github.com/UOR-Foundation/uor-r4/issues/1718), [#1738](https://github.com/UOR-Foundation/uor-r4/issues/1738).

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
