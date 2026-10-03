# Status

Updated October 3, 2026. **Pre-alpha: useful conversation, broad reasoning,
frontier capability and lower complete-path energy remain unestablished.**

The owner-adopted [active plan](docs/integration/project-track.md) prioritizes
**grounded conversation and durable memory**, then the same model's broader
language, coding/reasoning and efficient D11/D5 laptop execution.
[Current state](docs/integration/current-state.md) and live [#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552)
own results. This page is navigation, not an independent results ledger. See [ROADMAP](ROADMAP.md).

**[Owner direction, 3 October](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5971681179):** no transformer baselines. Mechanisms are compared
with matched geometric alternatives and with the previous best geometric model.

| Boundary | Current evidence and remaining work |
|---|---|
| Geometric development model | Completed matched 8M text-loss comparisons retain the geometric emitter; flat L2 wins the three measured splits. These floating training/reference models are separate from complete native serving. [Loaded results and identities](docs/history/current-state-2026-10-03-overnight.md#delivered-overnight-results-and-geometric-chat-blocker--october-3). |
| Grounded response and durable memory | Saved compiler → exact store → generated-history emitter is exercised. The selected payload can be delivered correctly while the answer follows a distractor. Same-process save/reload continuation is measured; useful chat and broad scope/version coverage remain unfinished. [Source-consumption result](docs/integration/geometric-chat-source-binding-2026-10-03.md). |
| Geometric attention | Persistent integer attention, strict q4 context/value/potential/NoRead and connected capture/age components are merged. Component replay is fidelity; mixed capture learning is retained. Next: typed selected-record occurrence→actual-BPE consumer with connected answer credit and measured cost. |
| Data and query conditioning | Surface-form breadth improves its synthetic statement-prefix panel; question-side transfer remains unresolved. Distance recipe is a retained negative confounded by reduced binding practice. Neither establishes a unique capacity or addressing cause. [Current evidence](docs/history/current-state-2026-10-03-overnight.md#deepseek-research-and-next-integration). |
| Native serving and efficiency | Scoped integer opcode/parity results remain. Current grounded generation retains a floating development path; bounded complete native chat, selected parameter access and full-path energy savings remain open. |
| Scale ladder (Claude lab report, #820) | 20M geometric model (512 wide, 8 heads, 8 layers, `rrarrarr`; quaternion rotation, flat L2 read, context 384; 300M tokens of TinyStories, TinyDialogues and chat-v0 weighted 0.6/0.15/0.25): TinyStories validation NLL 1.3882 (seed 1), 1.3858 (seed 2). At 20M, learning rate 0.001 beat 0.002 by about 0.045 nats at steps 5k and 7.5k; 0.0005 was better still (dev NLL 1.8173 / 1.7294 at steps 5k / 7.5k; the learning-rate checks stopped at 660 s). 29M (576 wide, 8 heads, 10 layers; 434M tokens) finished at TinyStories validation NLL 1.2857 (learning rate 5e-4) and 1.2989 (learning rate 1e-3). A ~96M geometric run (two GPUs, 1.5B tokens) started at 5:35 PM ET, 3 October. At 8M, quaternion lanes beat U(1) lanes and the flat L2 read beat the Lorentz read. These are language-modelling losses on one data mix, not conversation results. |
| Chat | **Not achieved.** `chat-8m-a` (exact store plus prime-atom log-recall sieve): 959 of 1,075 D19 grounded-session turns; 21 of 232 panel replies acceptable under the qwen2.5:7b judge, against 8 for its deranged control. The 20M chat fine-tune with M-world and chat-v0 data: 28 acceptable and 40 relevant on the same 232-reply panel (relevant p=0.003 against `chat-8m-a`), session 960 of 1,075. The 29M chat fine-tune (learning rate 5e-4) scores 972 of 1,075 on the D19 session, the best so far (store off: 865; MQAR 108 → 1). Authored development panels, not general chat. |
| Compute | Training on a rented Runpod 2×RTX 4090 pod; grading on the M1. Optional `cuda` feature for offline training merged in [PR #1649](https://github.com/UOR-Foundation/uor-r4/pull/1649) at `dc28b495` (18 device-parity tests passing at `c17411b8`). Hosted GitHub runners are reserved for `main`'s required checks. |
| Track B | Parked. #1518's original full-model parity failure stands; no automatic retry or transformer serving follows. |

Read the [evidence review](docs/integration/grounded-memory-evidence-2026-10-01.md)
for scope, hashes and primary result links. D18's A1 outcome D is preserved;
old A1 training does not resume. The best E4 route (v16) reports 36/52 open relations, 2/17 closed and 106/109
MQAR, against 48/52 with the oracle recall line and 27/52 for the best trained
read; its extraction drops fixed-vocabulary values. v17 (trunk features in the
table) is a recorded negative at 33/52. These are diagnostics, not promotion.

Provenance: the Geometric development model, Grounded response, Geometric attention,
Data and query conditioning, Native serving and Track B rows are from
[#1644](https://github.com/UOR-Foundation/uor-r4/pull/1644) (merged at `de0b9d10`, 3 October).
The Scale ladder, Chat and Compute rows are the Claude lab's 3 October reports on #820,
copied rather than re-measured here.

## Coordination

- Claude: #1511 / #1552, current compiler/emitter integration.
- DeepSeek (its own harness; OpenCode was removed 3 October): #1512, current data/read
  research and shared interfaces.
- Codex: #1552 / #1512, geometric source-consumer integration and protected delivery
  (#1644, #1650 merged; #1652 open, 3 October).
- Support lab (Antigravity): documentation, issue hygiene and cleanup only; no model code.
  Kimi and the older boards remain historical.

Live GitHub claims and actual process handles determine activity and ownership.
Use the [shared protocol](docs/labs/protocol.md), [host guide](docs/labs/operations.md)
and prospective resource work card before compute. Current FIFO admission uses
aggregate host limits; small bounded checks need no heavy-job reservation.
Preserve live runs' reservations. No paid compute or destruction of unique
material is authorized.

Protected PRs require recorded exact-head review and actual scoped checks.
One CI job publishes the five historical acknowledgement names (3 October 2026); dropping them from the ruleset awaits an org admin. The historical
acknowledgement jobs are not compile/test evidence;
the delivery-evidence check remains advisory under the owner's current rule.
