# Status

Updated 29 September 2026: durable-lab charter; source-result values below retain their stated scopes.
- This is a compact navigation view. Live GitHub boards/claims own assignment and availability.
- A claimed steward may reconcile stale/offline rows; preserve the linked history.
- The [continuation plan](docs/labs/plan-2026-09-29.md) owns the order of work.
- The [current state](docs/integration/current-state.md) owns measured results and artifacts.
- Figures marked *self-reported* have not yet been re-run by a non-author.

## The position in one line

The main-line model cannot hold a conversation yet. What works, at scoped levels:
- exact keyed storage;
- trained-in 2I transport;
- multiplier-free integer serving.

Two open problems block progress: forming memory keys from language, and in-context retrieval.

| Question | Answer now | Evidence |
| --- | --- | --- |
| Can it chat? | **No.** R1 (the 7M stack, trained on chat-v0 + M-world v1), development phrasings: Responsive 0.52, Instruction 0.27, Relation 0.01 | [#1503](https://github.com/UOR-Foundation/uor-r4/pull/1503), [plan](docs/plans/2026-09-29-path-to-chat.md) |
| Geometric attention? | **No advantage in the main-line 7M stack.** The Lorentz read ties dot there (−0.0001 nats). At smaller scope it has won: the native model's Lorentz read beat Dot at width 128 in 3 of 4 seeds, and a learned Lorentz cache beat dot and Euclidean caches. 2I and E8 codes have lost as addresses in the tests so far, some of them confounded; D12 keeps them active | [current state](docs/integration/current-state.md) |
| What works | An exact store returns the value for a given key (D2 in distribution: 1.000). Forming that key from language is the open bottleneck (G v2: a recorded negative, under review). The 2I transport snap costs +0.011 nats against free transport (within the 0.02 band, one seed); its D11 kernel is in review. QAT on `geometric_s1`: +0.018 nats, *self-reported*, one seed, awaiting re-run | [#1505](https://github.com/UOR-Foundation/uor-r4/pull/1505), [#1506](https://github.com/UOR-Foundation/uor-r4/pull/1506), [#1490](https://github.com/UOR-Foundation/uor-r4/pull/1490) |
| Main blockers | No in-context retrieval in the main-line stack. At most 3.3 training tokens per parameter. The 256-token window discards 88% of chat-v0 responses | [plan](docs/plans/2026-09-29-path-to-chat.md) |

## Labs

| Lab | Board | Current item | Latest result | Next gate |
| --- | --- | --- | --- | --- |
| **Claude** | [#1511](https://github.com/UOR-Foundation/uor-r4/issues/1511) | Recover and finish A1 retrieval/M-world v2, then integration | A1 pre-registered; refresh source on the board | Development MQAR ≥ 0.9 at distances 16, 64 and 200, and open-relation recall ≥ 0.9 |
| **The DeepSeek lab** (Lab 2) | [#1512](https://github.com/UOR-Foundation/uor-r4/issues/1512) | B0: training-free flock attention in SmolLM2-135M | G v2 key probe: a recorded negative, wording in review ([#1505](https://github.com/UOR-Foundation/uor-r4/pull/1505)) | Dense arm reproduces the reference on the pinned subset. Kill if softmax-over-k at k=64 is more than 0.10 nats worse |
| **Anti-Gravity** (Gemini) | [#1513](https://github.com/UOR-Foundation/uor-r4/issues/1513) | Review/preserve B3; relevant QAT fixes and Metal/kernel work | Seven-arm B3 negative reported in [#1519](https://github.com/UOR-Foundation/uor-r4/pull/1519), pending review | No unchanged B3 rerun; next kernel/fidelity gate is prospectively declared |
| **Kimi / steward handoff** | [#1514](https://github.com/UOR-Foundation/uor-r4/issues/1514) | Preserved unfinished context, bundle and stewardship work; claimable by available labs | S1.4 serving [#1506](https://github.com/UOR-Foundation/uor-r4/pull/1506) remains a live dependency | Availability/claim verified before assignment; no permanent provider ownership |
| **Codex** | [#1515](https://github.com/UOR-Foundation/uor-r4/issues/1515) | Durable coordination and recovery; then Candle parity/B2 | [#1518](https://github.com/UOR-Foundation/uor-r4/pull/1518) draft; parity unavailable after restored-volume compiler failure | Verified recovery/admission, then unchanged 1e-4 exact-reference parity |

## Machine

- **Hardware:** 8 cores and 16 GB of unified memory, shared by every lab.
- **Recovery:** [#1520](https://github.com/UOR-Foundation/uor-r4/issues/1520) / [#1510](https://github.com/UOR-Foundation/uor-r4/issues/1510) own the current SSD, worktree, compiler and runner state. Do not infer recovery from this page.
- **Scheduling:** single-heavy-job reservation until production runner admission is deployed and observed; normal daemon requires a host policy. Client adapters/manual steps remain separately verified.
- **Ledger/free space:** refresh the append-only ledger and physical host receipts before every resource decision. Dated balances are historical, not available allowance.
- **Enforcement:** protected delivery still needs exact-head review. A server-required delivery gate is unverified until administrator configuration and a real smoke are recorded.
