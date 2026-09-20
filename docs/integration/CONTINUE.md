# Continue UOR-R4 geometric model research

## Active: learned contextual geometric relational reader — promising near miss

The [relational-reader result](relational-reader-result-2026-09-20.md), its [design note](relational-reader-design-2026-09-20.md) and [receipt](../evidence/native_geometric_relational_reader_2026-09-20.txt) report one complete constructive task from base `647483ab`, sealed at `.uor-models/realtext-prior-2026-09-20/relational-reader-1`.

A **learned** 120-valued descriptor `Q(x_{i-1})` (initialised from the frozen S write slots) forms the directed relative element `inverse(q)*k`; a learned 120-entry rank table and a learned action set `{NoRead} u {(candidate, strength)}` (0.0625/1/8 nats) are fitted against the **actual one-step action loss** of the observed next token. Four arms share one causal pool, one action set, one dose and one objective.

| arm | covered read precision | E[action loss] nats | real CE delta bits |
| --- | ---: | ---: | ---: |
| exact-recurrence (no relation) | 0.755 | -0.807 | -1.082 |
| categorical control (arbitrary code) | 0.709 | -0.722 | -0.995 |
| **relational (learned descriptor)** | **0.873** | **-1.024** | **-1.500** |

Paired by sequence: precision **+0.0987 [+0.0418, +0.1603]**; expected action loss **-0.2172 [-0.2976, -0.1342]** nats. The predeclared precision bar (0.15) is missed while the loss criterion passes decisively, so the label is **not positive** and the candidate is retained, not promoted. The **nongeometric categorical control is worse than no relation at all**, which is the evidence that the group structure rather than a larger table carries the gain.

**Next task:** (1) re-run the same frozen panels with the corrected parity control and a **prospective practical margin stated on the calibrated expected action loss** rather than raw precision, recording the new criterion before scoring and leaving the original criterion's outcome unchanged; then (2) carry the learned relation into **one dependent two-hop read with a learned scheduler**, including a derived output absent from all source payloads when claiming movement beyond copying.

Do **not** widen the ring or the candidate bound, sweep widths/precision/corpus, reopen the frozen S attribution, or start a reset-only campaign. CPQK production continuation remains required **before any fit that uses `QueryTrainer`** and is not used here.


Refresh origin/main, AGENTS.md, canonical plan/current state, live issues and resources. PR #1314 delivered a constructed selection gain and natural-text harm; PR #1317 audited it. The owner's completed handoff is that same run, with no successor model result. [Correcting audit](occurrence-reader-audit-2026-09-20.md).

**One next task:** [learn a contextual geometric reader](deepseek-relational-reader-step-2026-09-20.md). The owner explicitly reconfirmed the full mathematical bridges. Learn causal query/source descriptions and directed H4 relations over exact BPE memory. A repaired/calibrated PR #1314 reader is the comparator, with bounded source/NoRead/copy-strength decisions in both arms. Do not execute the superseded memory-utility-only draft as a separate campaign. Current exact-key support may require one declared causal proposal correction; compare ranking on shared pools and charge complete access work.

The verified prior result includes five correct raw reads out of 88, five correct out of ten covered positions, and 737/515 read/NoRead training labels. Negative supervision was not absent. Preserve the source/ranking and output-integration distinction, exact payloads and frozen local predictor. The [canonical sequence](project-track.md#structural-memory-and-geometric-representation-follow-up) is contextual relational access -> structural persistence -> learned dependent reads/composition -> broad language and executed Rust -> complete machine cost and API/WASM/Studio delivery. Finite Hopf/spin relations are tested through the reduced H4 mechanism now; full S7/E8/harmonic state remains a targeted comparison when needed.

The last reviewed ledger is 181238565/191300000 ms; refresh the live JSON before work. Preserve prior charges once, owner checkouts and all unique material. Record complete projections and any standing-authorized local extension before use; retain the 128 MiB storage margin and whole-machine free-space reserve. No paid/external compute is authorized. Use the actual fitter's continuation, focused causal/generated checks and protected PR delivery. No broad capability issue closes from this component.
