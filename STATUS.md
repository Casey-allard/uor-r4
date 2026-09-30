# Status

Updated 30 September 2026, following [D18](docs/integration/DECISIONS.md#d18--one-retrieval-question-for-114-october-track-b-cost-and-memory-port-work-parked-with-re-entry-conditions).
- This is a compact navigation view. Live GitHub boards and claims own assignment.
- The [current state](docs/integration/current-state.md) owns measured results and artifacts.
- The [direction review](docs/integration/direction-review-2026-09-30.md) of 30 September sets the plan for 1–14 October.

## The position in one line

The main-line model cannot hold a conversation yet. It cannot copy a value it was just told. D18 spends 1–14 October on that one question, and on whether the answer survives D11 serving.

| Question | Answer now | Evidence |
| --- | --- | --- |
| Can it chat? | **No.** R1, the 7M stack trained on chat-v0 and M-world v1, scores on development phrasings Responsive 0.52, Instruction 0.27 and Relation 0.01 (recall 0/64). Memory requests score 0/10 for every checkpoint | [#1503](https://github.com/UOR-Foundation/uor-r4/pull/1503), [#1492](https://github.com/UOR-Foundation/uor-r4/pull/1492) |
| In-context retrieval? | **Not learned yet.** The A1 pointer head, flock reads, M-world v2.1 (stream pinned) and the sealed English probe are merged as source ([#1548](https://github.com/UOR-Foundation/uor-r4/pull/1548)). They are untrained: the A1 run is D18's first experiment | [#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552) |
| Runtime cost? | **No win measured.** D11 serving has no multiplier, divider or float in its audited path, and equals D10 exactly. It is about 3–5× slower than D10 NEON (*self-reported*), reads 100% of its weights per token, and has no valid J/token. Selected weight access (D5) is unbuilt | [current state](docs/integration/current-state.md) |
| What works | Exact keyed storage (D2 in distribution: 1.000). The trained-in 2I transport snap (+0.011 nats, one seed). QAT Result B on `geometric_s1` passed its gate by 0.002 nats (one seed). Result C kept served NLL but not greedy behaviour | [#1527](https://github.com/UOR-Foundation/uor-r4/pull/1527), [#1506](https://github.com/UOR-Foundation/uor-r4/pull/1506) |
| Track B | **Parked under D18.** [#1518](https://github.com/UOR-Foundation/uor-r4/pull/1518)'s 1e-4 parity gate failed and that record is immutable. A successor host gate is defined but not run | [D18 §6](docs/integration/DECISIONS.md) |

## Labs

| Lab | Board | Current items (D18) | Next gate |
| --- | --- | --- | --- |
| **Claude** | [#1511](https://github.com/UOR-Foundation/uor-r4/issues/1511), [#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552) | D18 delivery; A1 instrument freeze on the #1017 tokenizer; the A1 arms (P, P+ptr, T, C); the §8 count-only tool for the owner | `a1_gate`: development MQAR ≥ 0.9 at distances 16, 64 and 200, and open-relation recall ≥ 0.9 |
| **DeepSeek** (Lab 2) | [#1512](https://github.com/UOR-Foundation/uor-r4/issues/1512) | R1-X on the sealed R1 weights; the `TruncatedPrefix` data unlock; a one-day D5 selected-weight-access design memo; the D11 port of the A1 pointer if it wins | R1-X decision rule (copy-limited or phrasing-limited); `TruncatedPrefix` ≥ 0.02 nats better on two seeds |

Codex, Kimi and Anti-Gravity were removed by the owner on 30 September. Their boards (#1513, #1514, #1515) are historical. Their merged work keeps its stated scope.

## Rules in force

- **Merging.** A recorded review at the exact head (self-review allowed) plus passing compile/tests executed at that head, then the protected merge queue. The delivery-evidence check is advisory. See [#820](https://github.com/UOR-Foundation/uor-r4/issues/820).
- **Checks.** PR compile and test runs use GitHub's free runners. `main` compiles every workspace target again ([#1547](https://github.com/UOR-Foundation/uor-r4/pull/1547)). Known failures are tracked: the three `joint_campaign` tests and the workbench frozen-identity test.
- **Laptop.** 8 cores and 16 GB of unified memory, used for model runs. The runner follows the owner's five rules (#1536): one queue, admission by declared threads and RAM, starts blocked only at critical memory pressure. The SSD workspace is no longer quarantine-mounted.
