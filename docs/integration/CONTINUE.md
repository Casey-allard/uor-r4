# Continue UOR-R4 geometric model research

## Active: competing-source geometric read

The [result](competitive-reader-result-2026-09-20.md) and [receipt](../evidence/native_geometric_competitive_reader_2026-09-20.txt) report one complete constructive task from base `37bf2bd1`, sealed at `.uor-models/realtext-prior-2026-09-20/competitive-reader-1`. The principal review of PR #1319 remains its interpretation layer.

`read_step` + `predict_next` are one **target-free shared causal inference path** (evaluation, generation, interventions, timing); `relation_index` is the single relation encoding used by training, inference, export and reload, with an independent `RLR2` loader exercised per arm (0 parity failures). On a construction where several plausible same-class sources compete for the query key: local 12.6347, exact 10.1377, learned categorical 10.1513, relational H4 **9.9582**, query-blind 10.2716 bits (hard-action CE); paired relational minus exact **-2.4956 bits [-3.7642, -1.3010]**. The query-blind arm is worse than exact and the trained categorical arm matches it, so the gain is query- and geometry-dependent on this panel.

**Natural text regresses +1.2908 bits/token (6.8002 -> 8.0910) against a declared +0.05 tolerance**, so `positive = false`. Named causes: the strength is still global (`score = v(c) + sb[a]` factorises) and the descriptor is a static single-token code with no natural-text fitting.

**Next task:** (1) one contextual strength interaction over causal candidate/query/local-score observations only (never coverage or correctness); (2) a source-separated natural-text fit; then re-measure the same construction and text panels. Do not widen the ring or candidate bound, sweep widths, or reopen the frozen S attribution. CPQK remains required only before a fit that uses QueryTrainer.



Refresh origin/main, AGENTS.md, canonical plan/current state, live #973/#820, artifact identities, ledger and free space. PR #1319 is merged as `bfa13a91` with verified head/merge tree equality. Its [principal review](relational-reader-review-2026-09-20.md) corrects the interpretation of the [original result](relational-reader-result-2026-09-20.md).

**One next task:** [competitive query-dependent geometric reader](deepseek-competitive-reader-step-2026-09-20.md). Incorporate the specific evaluator/search/runtime corrections into one shared target-free current-prefix inference path, then learn to distinguish competing plausible sources using the query. Include actual generated counterfactuals, hard-action loss and bounded natural-text regression. A parity-only repeat followed by immediate two-hop/derived-output work is superseded.

Retain the useful synthetic H4-versus-exact result, original `positive=false` and negative artifacts. Categorical group-attribution evidence is invalid; source-bank membership can solve the intended pair task. No fresh natural-text quality was measured. Current inventory of `relational-reader-1` has unlisted `summarize.py`; preserve it and use new claimed roots for corrections.

The [canonical five stages](project-track.md#structural-memory-and-geometric-representation-follow-up) remain query-dependent access, structural persistence, dependent reads/composition, broader language and executed Rust, and qualified scale/delivery. Reduced H4/Hopf/spin relations are active; S7/E8/harmonic state is available for a demonstrated representation or retention limit.

Live ledger at review: 182138565/191300000 ms, about 152.7 minutes remaining. Refresh before executing; retain prior charges once. Record complete projections and any standing-authorized extension before use, keep the 128 MiB margin, preserve owner checkouts and unique research, and deliver through a protected PR. No paid/external compute is authorized; broad capability issues remain open.
