# ORDER: execute the bounded public CPU rank-margin pilot, deepseek-2

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `374c7bdd3`.

## Goal

Execute the already merged FIT sheet `docs/harness/prep/cpu-training-pilot-public-rank-margin-2026-11-02.md` without changing its bar, then report the measured positive or negative and its CPU substrate, cost and unresolved capability gaps.

## Scope

- `local/deepseek/cpu-pilot/**` for ignored raw run outputs and trained artifacts.
- `docs/loop/tasks/cpu-training-pilot/deepseek-2/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No serving edit, code edit, GPU run or paid compute. No public or sealed panel claim beyond the vocabulary in the FIT sheet.

## Pre-run gates

- A fresh non-author pre-run review confirms the FIT sheet identity, the clean source head, the built binary and toolchain, the exact public corpus/artifact hashes, the two candidate commands, identity capture, disk/RAM capacity and stop rule.
- Focused CPU corpus/checkpoint controls and exact public FBS parity pass at this source head.
- The captured source, binary, Rust/Cargo versions, OEM hardware, `uname`, `lscpu`, RAM and filesystem identity are written under the run root before any timing probe.

## Execution

- Run a separate 16-step timing probe for each artifact seed from fresh directories and estimate full 256-step cost.
- If both projections are under one CPU hour, run the FIT zero-update baseline and 256-step arm for both seeds with evaluation stride 2, evaluation cap 1024 and checkpoint interval 64.
- Preserve exact JSON outputs, logs, checkpoints and final artifacts under `local/deepseek/cpu-pilot/`.
- Apply the FIT three-standard-deviation, no-trade, admission and exact-export rules per seed without changing them after observing results.

## Done when

- Both seeds have baseline, probe and either completed 256-step or declared-stop evidence.
- The REPORT records actual context-tokens/s, target visits/s, mean step time, wall time, peak RSS, accepted updates, admission/top-1/top-8, exact export, checkpoint/final-artifact identities and the pre-registered verdict.
- The notebook and roadmap distinguish this public internal-development result from complete replies, genuinely fresh phrasing and public-run restart, which remain `NOT_RUN`.
- Full relevant checks, fresh exact-head review, PR delivery and fresh-main verification are recorded.

## Stop

Apply the FIT stop rule. Do not shorten a healthy arm, do not tune after seeing a miss and do not run a third seed. A miss is retained as a configuration negative.
