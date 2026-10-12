# Kimi lab: joining review, direction recommendations and the evaluation-integrity line

References #2028 (registration), #2029 (M1), #2030 (M2), #2032 (M4). Lab: Kimi, session `kimi/s1`, 2026-10-12.
No training, no pod and no new model measurement is claimed here: every project number below is quoted from the
named record; external numbers carry their arXiv source. The one new local measurement — the `uor-r4-integer`
test suite and the `joint_campaign` reproduction — is reported in the cycle card on #2029, not here.

> **One line.** A full read of the repository (code, records, decisions, live issues), an external prior-art
> check and the project's own geometry audit say the same thing: the read is the answer path, the served model
> is 0.05 % geometry, and the instruments that would tell a real geometric read from a lucky one — a
> regime-matched recall bench, an equal-budget rule, a forced-copy oracle, an integer-softmax discriminator —
> are the pieces nobody owns. This memo asks the owner to answer the audit's three pending questions, and
> registers the Kimi lab to build and run those instruments.

## 1. What this review covered

Read end to end: `AGENTS.md`, `README.md`, `STATUS.md`, `ROADMAP.md`, `docs/integration/DECISIONS.md` (D0–D22),
`project-track.md`, `current-state.md` (full), `docs/mechanisms/` (all nine briefs), `docs/labs/protocol.md`,
`session-goal.md`, `operations.md`, `audit.md`, the geometry audit, and the lab records of 9–12 October
(read-binding line, addressed-memory/addressed-read line, starling ladder, spin-native weights, context curve,
tokens/s, Codex's whole-path series, the DeepSeek handoff). Source-read: `native_geometric/` module map,
`geometric_stack.rs` (reads, flock, starling, product-key memory, pointer), `stack_memory.rs`,
`stack_export.rs` refusal list, `uor-r4-integer` (both engines). Live GitHub: open PRs (none), issues
#2028–#2037 tails, compute board. External: primary papers listed in §5.

## 2. Where the project is (quoted state, 2026-10-12)

- **M1 (language base).** v5 memory **20/40** (target 34), open reply **37/232** (target 116); served
  **1.20138–1.24402 BPB** on the 29M artifacts against the ≤ 0.90 criterion; the 19.9M chat stack reads
  0.877550 float / 0.886838 served at the pinned 512-window protocol. Powered KEEPs: read-binding supervision
  (wrong-value 47→30 of 120 at 2k; 49→23 at 8k, Fisher χ²=18.4); the Lorentz geometric read ahead of its `l2`
  ablation on all three seeds (wrong-value 49→32 of 120, below the pre-registered per-seed margin).
- **M2 (grounded reply).** Development **437/512**, fresh qualification **0/128** — recorded as panel fitting
  under D22 §4, not a milestone move. Whole-path source connection delivered (#2242); the owner training hold
  stands; the line is at 5/3 with its pivot recorded.
- **M4 (one served model).** Softmax at runtime: **yes**. Best trained softmax-free read: +0.0172 BPB vs the
  0.01 bar. Starling read: provisional KEEP at mqar recall 0.780 (5 seeds) with the caveats recorded on #2032
  (unequal budget against the control; equal-budget row 4 read 0.831 vs 0.991 at ctx 512 and failed at 1536 on
  eviction; readout self-distillation: one seed 0.964, two never learned). Spin-native weights KEEP (icosian
  pairs + 2-bit-mantissa norm at dense parity, ~19–20 bits/quaternion). E8 orbit memory: keep-and-iterate;
  learned keys beat the fixed E8 addresses by 0.023 BPB.
- **Live:** three Claude pods on starling step 3c; DeepSeek's session handed off with its next slice specified
  (`route_sources` wiring, then nine pre-registered fits); Codex delivered #2242 and is blocked on the training
  hold; three `joint_campaign` test failures on main flagged without a repair owner.

## 3. The six-month shape

September built the native learner ladder (D8 rungs 0–4 at frozen scopes) and the A1–A4 attention scaffolds
(0/12 complete answers each); D10 (converted backbone) was adopted and superseded by D11 within a day. October 1:
D18's retrieval question returned outcome D — every arm under 0.5 MQAR at distance 16, including the transformer
control at 2× steps, which read at the time as "no learned retrieval at this scale" and now reads, with the
bench evidence of §5, as a training-regime result. D19 made grounded conversation and durable memory the plan.
D21/D22 (9–10 October) fixed the process failure — 48 merged PRs moving no headline, then five mechanism
closures on tests that could not judge them — and the current rules (a negative closes a configuration, never a
mechanism; ≥2 seeds or ≥200 fresh rows; bars no narrower than the noise; train the mechanism in; fresh transfer
before KEEP) are the right rules. What they cannot yet see is in §4–§5.

## 4. Internal adversarial findings (from the project's own records)

1. **The answer path is the read** ([answer-path](../answer-path-2026-10-11/README.md),
   [head ablation](../head-ablation-2026-10-11/README.md)): facts removed → 0/40; copy head off → −1 row;
   silencing the supervised read → 0/40 with zero wrong values. Pointer-line gains (hit rate 0.377→0.54) were
   read-outs of a ~6 % mixture branch.
2. **The served stack is 0.05 % geometry by arithmetic** ([geometry audit](../../integration/geometry-audit-2026-10-11/README.md)):
   MLP 39.6 %, recurrence projections 31.0 %, reads 16.6 % (40.5 % at 4 K), head 7.4 %, quaternion rotations
   0.05 %, exact addressed machinery 0. The audit's three owner questions (compute ledger; first load-bearing
   mechanism; which family is the project) are **still pending**.
3. **MACs are not time.** The product-key memory's −90 % MACs at the replaced layer never appeared in tokens/s
   (1.65× slower; [tokens-per-second](../tokens-per-second-2026-10-11/README.md)); D11's relative cost *falls*
   with context (1.66×→1.24×; [context curve](../context-curve-2026-10-11/README.md)); a free read would buy at
   most ≈1.7× served rate at 4 K, because MLP + recurrence + head are ~82 % of the arithmetic at ctx 384.
4. **Panel fitting is measured, not suspected:** 437/512 development against 0/128 fresh on the same saved
   model (#2164). D22 §4 already names this; it should be read as the central evidence that development-panel
   gains without a fresh draw prove nothing.
5. **Never run, despite being named:** the forced-copy oracle (the audit's §7 cheapest decisive test); the
   exact addressed memory *as the read* (cycles 18–20 replaced the **MLP**; the erratum is recorded, the
   read-replacement question is open); Lorentz trained from step 0 (every geometric-read result is a fine-tune
   from a flat-trained base); route-holonomy, E1 holonomy lanes, octonion-signed binding, ring-product read —
   all pre-registered, none measured.

## 5. External prior-art check (primary sources; what they say about the core bets)

- **The synthetic bench is out of regime.** Published MQAR (Zoology, arXiv:2312.04927; Based, arXiv:2402.18668)
  uses vocab ≈ 8,192, multiple key–value pairs per sequence, a distance distribution, ~100k training examples,
  and attention solves it at d=64 (>0.9). This project's bench: vocab 512, 64 sequences/seed, 1,800–3,600 steps,
  and the L2-softmax control reads **0.617**. Until the control reproduces the published regime, a
  starling-vs-control comparison (0.780 vs 0.617) is a reading about the bench, not about attention. **This is
  the highest-priority instrument finding of this review.**
- **No published existence proof** of a softmax-free, multiplier-free read matching softmax at scale. Sigmoid
  attention (arXiv:2409.04431) matches softmax at 1B/7B but still uses exp and dense O(n²) scores. MatMul-free
  LM (arXiv:2406.02528) reports ternary attention **fails to converge**; its parity comes from a ternary
  recurrent mixer. Sub-quadratic mixers close the recall gap only with hybrids (small exact local attention
  windows: Based), gating, or the delta rule (DeltaNet arXiv:2406.06484; RWKV-7 arXiv:2503.14456). The project's
  starling/flock/rank lines are in genuinely open territory — which raises the bar on their instruments.
- **The honest discriminator nobody has built:** an integer-only softmax (I-BERT, arXiv:2101.01321 — polynomial
  exp, max error 1.9e-3, GLUE parity). A multiplier-free integer softmax is D11-servable by construction. If it
  matches the table-emulated softmax within ~0.005 BPB, the softmax-free programme's remaining case is cost,
  not quality — and the rank/flock/starling lines become what they actually are: compute mechanisms, to be
  judged on tokens/s and energy at fixed quality, not on beating softmax's quality.
- **Product-key memory:** the literature agrees with the project's measurements. Memory layers are
  bandwidth-bound and need a fused gather (Meta, arXiv:2412.09764: custom kernels 3 TB/s vs <400 GB/s); cutting
  the codebook removes the mechanism's reason to exist; the compute case belongs on the integer/table path —
  exactly where the D11 engine cannot serve it yet.
- **Quaternion/hypercomplex weights:** the published advantage is parameter efficiency (≈¼ parameters at
  comparable quality on speech/vision/NLI), never quality-at-scale; no quaternion LM at scale exists. The
  project's icosian-mantissa parity is therefore genuinely novel at its scope, and its next fair test is at
  **equal bits**, not only equal parameters.
- **D11's cost thesis is conditionally supported:** CPU LUT kernels win on consumer CPUs only with fused
  kernels (T-MAC arXiv:2407.00088: 4× / 60–70 % energy; bitnet.cpp arXiv:2502.11880: 6.25× vs FP16); on GPUs,
  table lookups lose to dequant-matmul (LUT-GEMM 1.75–2.34× slower than BitBLAS). The mechanism of savings is
  **bytes moved**, not the absence of a multiplier — which is what the geometry audit's B5 measured internally.

## 6. Recommendations (owner decisions requested)

- **F1 — Answer the geometry audit's three questions.** Recommended: (i) adopt the compute ledger
  (MACs/token, weights touched, bytes moved, wall at fixed quality, artifact size) as a required field of every
  result PR — the format already exists in the transformerless line's COMPARISON.md; (ii) first load-bearing
  mechanism: **C1, the address-and-copy output path** — the answer-path evidence implicates the emit side, and
  C1 is the D19 priority expressed as arithmetic; (iii) the family question: **the stack is the family**, with
  the native machinery ported into it (the existing trend: VSA codes, product-key memory, E8), and Line B kept
  as the grounding/memory research line under M2/M3.
- **F2 — Re-baseline the synthetic bench before further read claims.** Move mqar-bench to the published regime:
  vocab ≈ 8,192, multiple KV pairs, distance buckets, ≥ 3,000 eval sequences per arm, controls trained to
  convergence with an LR sweep. No architecture claim from a synthetic comparison until the softmax control
  reproduces the published > 0.9 regime. Starling's provisional KEEP stands as a mechanism reading; it is not
  yet evidence about attention.
- **F3 — Build the two missing discriminators before new mechanisms.** (a) the forced-copy oracle
  (supply the true span at serving; answers "is the bottleneck addressing or emission?"); (b) the
  integer-softmax control (separates the normalisation question from the exp question; D11-servable).
- **F4 — A synthetic KEEP must name its language-transfer check before the next ladder rung.** The starling
  line is at step 3c without a language number; ladder depth without a transfer reading is bench-fitting risk.
- **F5 — Standards upgrades.** Equal-budget controls on every cross-architecture comparison (the 0.780-vs-0.617
  at unequal steps must become impossible by construction); each panel's noise posted beside it (v5: ±3 of 40;
  mqar needs its own); tokens/s and an energy proxy beside quality on every result PR (generalising the
  DeepSeek metric direction); `arm=transformer` retained as a **diagnostic-only** control — this reconciles
  D20's matched-control requirement with the 3 October "no transformer baselines" direction (confirmation
  requested; the alternative is retiring the arm, which blinds the bench's only absolute reference).
- **F6 — Keep the process.** D21/D22 counts, pivot cards, TEST FITNESS reviews and the three-hourly audit are
  working. The gap is instruments, not process.
- **F7 — The Kimi lab** (below) owns the instruments in F2–F4.

## 7. The Kimi lab

**Role:** the fourth lab — the evaluation-integrity line. Non-colliding by construction: Claude owns the
starling read, DeepSeek owns the invariant-addressed read, Codex owns the M2 unified path. Kimi owns the
instruments that decide whether any lab's numbers mean what they say, plus the forced-copy oracle (named by the
geometry audit, unblocked, unowned).

**Order of work:** (1) forced-copy oracle, pre-registered on #2029 with TEST FITNESS before compute (CPU-only);
(2) mqar bench re-baseline, pre-registered on #2032 (one pod, coordinated around the live leases on #2037);
(3) integer-softmax discriminator (source, then a served comparison on the frozen 19.9M artifact); (4)
replication of provisional KEEPs at equal budget as they appear. When #2029/#2032 are blocked on the owner,
M5 #2033 measurement-design preparation only (no owner-hardware claims).

**Proposed `/goal` prompt:** added to `docs/labs/session-goal.md` in this PR, in the same contract form as the
other three labs, with peer-audit direction Kimi → Codex (Codex currently has no auditor).

## 8. What this session did not do

No pods, no training, no panel scoring, no edits outside this record and its two companion files. Local work:
the owner's checkout restored to clean `origin/main` with one unpushed DeepSeek commit preserved
(`preserve/deepseek-router-prep-2026-10-12`, disclosed on #2028); a fresh worktree; the `uor-r4-integer` test
suite (284/284 library tests pass at `c4d682cd0`). The three red `joint_campaign` tests on main are reproduced
and diagnosed: an environment-sensitive source binding (`save_checkpoint` records compile-time
`UOR_BUILD_SOURCE_COMMIT`, defaulting to `UNBOUND`; `load_bound_checkpoint` requires a 40-hex digest), confirmed
in both directions (23/23 pass with the variable set) — a maintenance item, not a model result.

## Cost

| | |
|---|---|
| GPU | none; $0 |
| local | one workspace build + focused test runs; ~6 h of source/record/literature reading |

## Next

1. Work card + forced-copy oracle pre-registration on #2029 (TEST FITNESS before compute).
2. mqar re-baseline pre-registration on #2032.
3. Owner answers to §6 F1–F5 where they are owner decisions.
