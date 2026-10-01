# Status

Updated 1 October 2026: [D18](docs/integration/DECISIONS.md#d18--one-retrieval-question-for-114-october-track-b-cost-and-memory-port-work-parked-with-re-entry-conditions) reached outcome D, and the owner confirmed it.
- This is a compact navigation view. Live GitHub boards and claims own assignment.
- The [current state](docs/integration/current-state.md) owns measured results and artifacts.
- The [direction review](docs/integration/direction-review-2026-09-30.md) of 30 September set the plan for 1–14 October.

## The position in one line

The main-line model cannot hold a conversation yet.
- **Measured:** no trained read at about 2M parameters binds a query to its key in context. Geometric and transformer reads were tested, with and without a pointer head.
- **Next:** retrieval moves to an exact log of the conversation plus a prime sieve (D18 outcome D), starting with a design memo.

| Question | Answer now | Evidence |
| --- | --- | --- |
| Can it chat? | **No.** R1 (7M, trained on chat-v0 and M-world v1), development phrasings: Responsive 0.52, Instruction 0.27, Relation 0.01 (recall 0/64). Memory requests score 0/10 for every checkpoint | [#1503](https://github.com/UOR-Foundation/uor-r4/pull/1503), [#1492](https://github.com/UOR-Foundation/uor-r4/pull/1492) |
| In-context retrieval? | **Not learned (D18 outcome D, 1 October).** A1, development cell, one seed per arm:<br>• Arms without a pointer (Lorentz, trained flock, transformer at 2× steps) score MQAR 1/109.<br>• Pointer arms score 0.28–0.44 MQAR, about 1/N, matching the untrained "most recent value" rule (0.36). They copy recency, not the queried key.<br>• No arm reaches 0.5 at distance 16, so the pre-registered kill applies | [#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552) |
| Does §8 fit the window? | **No.** The sealed panel needs up to 317 positions with a 32-token reply budget (owner-run count, panel `220cbdbe…`). A 384-position window is merged as training source only ([#1557](https://github.com/UOR-Foundation/uor-r4/pull/1557)); no model is trained at 384 | [#1554](https://github.com/UOR-Foundation/uor-r4/pull/1554), [#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552) |
| Runtime cost? | **No win measured.** D11 serving has no multiplier, divider or float in its audited path, and equals D10 exactly. It is about 3–5× slower than D10 NEON (*self-reported*), reads 100% of its weights per token, and has no valid J/token. Selected weight access (D5) is unbuilt | [current state](docs/integration/current-state.md) |
| What works | Exact keyed storage (D2 in distribution: 1.000). The trained-in 2I transport snap (+0.011 nats, one seed). QAT Result B on `geometric_s1` passed its gate by 0.002 nats (one seed). Result C kept served NLL but not greedy behaviour | [#1527](https://github.com/UOR-Foundation/uor-r4/pull/1527), [#1506](https://github.com/UOR-Foundation/uor-r4/pull/1506) |
| Track B | **Parked under D18.** [#1518](https://github.com/UOR-Foundation/uor-r4/pull/1518)'s 1e-4 parity gate failed and that record is immutable. A successor host gate is defined but not run | [D18 §6](docs/integration/DECISIONS.md) |

## Labs

| Lab | Board | Current items | Next gate |
| --- | --- | --- | --- |
| **Claude** | [#1511](https://github.com/UOR-Foundation/uor-r4/issues/1511), [#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552) | The exact-log plus prime-sieve retrieval design memo (D18 outcome D, started 1 October) | The memo, then the owner's choice of its first experiment |
| **DeepSeek** (Lab 2) | [#1512](https://github.com/UOR-Foundation/uor-r4/issues/1512) | The one-day D5 selected-weight-access design memo. A1 training is stopped, and the D11 pointer port does not apply (it was for outcome A only) | The memo |

Codex, Kimi and Anti-Gravity were removed by the owner on 30 September. Their boards (#1513, #1514, #1515) are historical. Their merged work keeps its stated scope.

## Rules in force

- **Merging.** A recorded review at the exact head (self-review allowed) plus passing compile/tests executed at that head, then the protected merge queue. The delivery-evidence check is advisory. See [#820](https://github.com/UOR-Foundation/uor-r4/issues/820).
- **Checks.** PR compile and test runs use GitHub's free runners. `main` compiles every workspace target ([#1547](https://github.com/UOR-Foundation/uor-r4/pull/1547)).
  - The three `joint_campaign` tests pass when the build binds `UOR_BUILD_SOURCE_COMMIT`, as the evidence runs now do. Unbound, a checkpoint records `UNBOUND` and its loader refuses it by design.
  - The workbench frozen-identity test is a known failure.
- **Laptop.** 8 cores and 16 GB of unified memory, used for model runs. One heavy job holds `/Volumes/UOR-Workspace/locks/model-slot.json`. A run's wall time is sized from its rate measured under the current load; the wall is a stop, not a budget.
