# REPORT: infra-loop-harness, kimi-37 (record remaining answered premise probes)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8.

References #2904

## What did not work, first

- PLAN continued offering three finite probes after they had already answered their questions. Repeating them adds no information.
- These are finite-control/instrument results only: no trunk, durable-conversation, reasoning, oracle or milestone claim follows. Every registered/panel experiment and hold remains pending.
- `docs/integration/DECISIONS.md` is an owner path. The PR stays draft until Casey approves its exact text in this chat and the ORDER records it with an ISO-UTC time.

## Cancel probe

At `f1f55c0c1`, normalization, exact memory and reasoning-instrument rows remained open despite the recorded 13/13, 4/4 and 3/3 answers. The PM status records stand.

## What changed

- PLAN marks only those three premise rows `closed`, with the exact answers and pending experiments named.
- D62–D64 append evidence, non-claims, alternatives and reopening cost. No other premise, bar, hold, milestone or item status changes.

## Measurements

- Normalization: 13/13 at `5844d911c`, #2842 `6381825466`; registered trunk test remains pending.
- Exact-memory arithmetic: 4/4 at `9aaaf13bb`, #2833 `6382217138`; address-oracle checkpoint/FIT/dispatch/approval remains pending.
- Reasoning-ablation instrument: 3/3 at `9aaaf13bb`, #2866 `6382231845`; frozen-panel causal question remains pending.

No GPU work or model evaluation ran.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, and `git diff --check`.

## Skills used

`uor-research-architect` for exact claim scope and no-repeat decisions; `superpowers:verification-before-completion` for final delivery checks.

## Still open

Casey's owner-path approval in this chat, then exact-head review and merge. All registered model/panel experiments, checkpoints, FIT and holds remain unchanged.

## Plain summary

The plan stops offering three answered tests again. It records only finite evidence and leaves every actual model question open.
