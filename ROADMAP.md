# UOR-R4 Geometric Language Model — roadmap

Updated 3 October 2026 (Eastern Time). **Pre-alpha.** Useful conversation, broad reasoning,
coding, frontier capability and lower complete-path energy are not established.

This file is navigation and the standing serving rules. Results live in
[current state](docs/integration/current-state.md); ordered responsibilities in the
[canonical plan](docs/integration/project-track.md); owner decisions in
[DECISIONS](docs/integration/DECISIONS.md); live status and claims on
[#820](https://github.com/UOR-Foundation/uor-r4/issues/820). The previous roadmap text
(lab assignments, leadership packet, dead-path register, director log) is preserved
verbatim in [docs/history/roadmap-2026-09-29-snapshot.md](docs/history/roadmap-2026-09-29-snapshot.md).

## Direction — owner, 3 October 2026

- **No transformer baselines.** Comparisons stay inside the geometric design: a mechanism
  against a matched geometric alternative, and each model against the previous best geometric
  model. Earlier transformer results keep their recorded scope; the transformer-gap kill rule
  is withdrawn (see the [3 October comment on #820](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5971681179)).
- **Grounded conversation and durable memory first** (D19), then the same model's broader
  language, coding/reasoning and efficient D11/D5 laptop execution. The integration owner is
  [#973](https://github.com/UOR-Foundation/uor-r4/issues/973); the active work card is
  [#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552); geometry and semantic
  addressing continue on [#1512](https://github.com/UOR-Foundation/uor-r4/issues/1512).

## Where the geometric stack stands (reported by the Claude lab, 3 October)

These figures are the lab's reports on #820; each names its model and data. None is a
conversation, coding or efficiency result.

| Rung | Configuration | Reported outcome |
|---|---|---|
| 8M | width 288, 6 heads, 6 layers, pattern `rrarra` | Done. Controls at this size: quaternion lanes beat U(1) lanes; the flat L2 read beat the Lorentz read. |
| 20M | width 512, 8 heads, 8 layers, `rrarrarr`, quaternion rotation, flat L2 read, context 384; 300M tokens of TinyStories, TinyDialogues and chat-v0 weighted 0.6/0.15/0.25 | Final TinyStories validation NLL **1.3882** (seed 1) and **1.3858** (seed 2). Learning rate 0.001 beat the 0.002 default by about 0.045 nats at steps 5k and 7.5k; 0.003 was worse; 0.0005 was better still (dev NLL 1.8173 / 1.7294 at steps 5k / 7.5k; the learning-rate checks stopped at 660 s). |
| 29M | width 576, 8 heads, 10 layers; 434M tokens | Finished at TinyStories validation NLL **1.2857** (learning rate 5e-4) and 1.2989 (learning rate 1e-3). |
| ~96M | geometric run on two GPUs, 1.5B tokens | Started 5:35 PM ET, 3 October; no result yet. |

**Chat is not achieved.** The `chat-8m-a` D19 grounded session (exact store plus prime-atom
log-recall sieve) answers 959 of 1,075 turns. On the 232-reply panel judged by
qwen2.5:7b, 21 replies are acceptable against 8 for the deranged control. The 20M chat
fine-tune with M-world and chat-v0 data scores 28 acceptable and 40 relevant on the same panel
(relevant p=0.003 against `chat-8m-a`), with session 960 of 1,075. The 29M chat fine-tune
(learning rate 5e-4) scores 972 of 1,075 on the D19 session, the best so far (store off: 865;
MQAR 108 → 1). These are authored development panels, not general chat.

## Order of work

learned saved compiler/store/emitter ([#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552),
[#1508](https://github.com/UOR-Foundation/uor-r4/issues/1508), #973) → durable grounded
conversation (#962, #954) → faithful integer serving (#964) → useful language → executable
reasoning/coding (#955, #1088) → API/WASM/Studio and release (#1172, #1173, #965). Selected
access and cost ([#963](https://github.com/UOR-Foundation/uor-r4/issues/963)) share that path.
Track B conversion (#1509) is parked; its original failed parity stands.

## Compute and delivery

- **Training** runs on a rented Runpod 2×RTX 4090 pod, administered by the Claude lab. The
  optional `cuda` feature for offline training merged in
  [PR #1649](https://github.com/UOR-Foundation/uor-r4/pull/1649) at `dc28b495` (18 device-parity
  tests passing at `c17411b8`). **Grading** runs on the M1 laptop.
- **GitHub hosted runners are reserved for `main`'s required PR and merge-queue checks** (owner
  direction, 3 October). Do not push `codex/ci/*` branches; run exact-head checks locally or on the
  pod and record head SHA, clean tree, command and exit codes on the PR.
- Protected PRs require recorded exact-head review and actual scoped checks; queue status names
  are acknowledgements, not compile/test evidence. No paid compute beyond the authorized pod, and
  no destruction of unique material.

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
