# Status

Updated 29 September 2026, 20:35 UTC.
- **Each lab edits only its own row**, in the same PR as the result it reports.
- The [plan of record](docs/plans/2026-09-29-path-to-chat.md) owns the order of work.
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
| **Lab 1** (Claude, lead) | [#1511](https://github.com/UOR-Foundation/uor-r4/issues/1511) | A1 retrieval (M-world v2 with open values, MQAR and copy; flock and pointer reads); the D13 council | A1 pre-registered | A1: development MQAR ≥ 0.9 at distances 16, 64 and 200, and open-relation recall ≥ 0.9 |
| **The DeepSeek lab** (Lab 2) | [#1512](https://github.com/UOR-Foundation/uor-r4/issues/1512) | B0: training-free flock attention in SmolLM2-135M | G v2 key probe: a recorded negative, wording in review ([#1505](https://github.com/UOR-Foundation/uor-r4/pull/1505)) | Dense arm reproduces the reference on the pinned subset. Kill if softmax-over-k at k=64 is more than 0.10 nats worse |
| **Anti-Gravity** (Lab 3, Gemini) | [#1513](https://github.com/UOR-Foundation/uor-r4/issues/1513) | #1490 fixes; B3 E8 weight codes; the Metal port; teacher data | D4 codec adapters: changes required | B3: kill if 3-bit E8 is more than 0.05 nats worse than 4-bit rounding |
| **The Kimi lab** (steward) | [#1514](https://github.com/UOR-Foundation/uor-r4/issues/1514) | `tools/lab-runner` and the locked ledger; A2 context; cleanup; docs move; #1506 fixes | S1.4 icosian D11 kernel: changes required ([#1506](https://github.com/UOR-Foundation/uor-r4/pull/1506)) | A job longer than 10 minutes, submitted from a session, seals after that session exits |
| **The Codex lab** | [#1515](https://github.com/UOR-Foundation/uor-r4/issues/1515) | A candle Llama with a pluggable attention trait; B2 harmonic/hybrid transfer | Rejoined 29 September; item 1 pre-registered | Logits within 1e-4 of the exact reference. Kill if the all-layer L=2 hybrid is more than 0.15 nats worse |

## Machine

- **Hardware:** 8 cores and 16 GB of unified memory, shared by every lab.
- **Scheduling:** one heavy job at a time, through `/Volumes/UOR-Workspace/locks/model-slot.json`, until `tools/lab-runner` lands.
- **Model-time ledger:** 782,538,181 of 1,130,000,000 ms (rebuilt at 20:08 UTC).
- **Free space:** internal about 17 GiB, SSD about 31 GiB. Cleanup is in progress.
