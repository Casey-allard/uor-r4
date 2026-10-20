# First-principles review of UOR-R4: can it reach a geometric language model on an M1?

2026-09-25 · Requested by the owner · References #820

**Status.** This is an evidence document, not a decision record. It changes no D-entry, gate, artifact or active work card. Owner decisions are requested in §11.

**Method.** Eight specialist agents ran in parallel:
- mathematics;
- physics and energy;
- CS/ML architecture;
- literature and novelty;
- programme audit;
- code verification;
- a quantization experiment;
- a state-tracking experiment.

The lead reviewer ran three side experiments and wrote the synthesis. An adversarial red-team agent then spot-checked 31 numbers and citations. It found 3 critical, 11 major and 14 minor defects in the first draft, and all of them are corrected here.

**Evidence labels.** Every claim carries one of:
- **Source**: repository file and line;
- **Measured**: run during this review;
- **Literature**: paper retrieved during this review, cited by arXiv id;
- **Derived**: shown here;
- **Hypothesis**: not yet tested.

The repository was read-only throughout the review. Appendix B lists the evidence.

**A note on the agents' instructions.** The shared briefing asked every agent to flag the tension between D5 (per-token parameter sparsity) and the owner's "no sparse routing". So when several agents raise it, that is *not* independent corroboration.

## 0. Bottom line

1. **A breakthrough is possible, but not the one the programme is organised around.**
   - **A frontier-quality model trained from scratch on an M1 is not reachable.** Even assuming well-used compute, which is not yet measured (§3.3), one M1-week trains roughly a 20–40M-parameter model.
   - **A "geometric predictive advantage" on natural-text loss is supported neither by theory nor by any measurement.** That includes this review's own WikiText-2 runs (§6.3).
   - **Three outcomes are realistic and falsifiable (§9.1):**
     - **B1:** exact, cheaply served non-abelian state inside a real language model, judged against the *strongest* non-diagonal controls;
     - **B2:** J/token measured on the M1 at matched quality;
     - **B3:** conversion of an open model. This one is expensive: a fully attention-free conversion took billions of tokens in the literature.
2. **The project is partly off course (§7).**
   - The engineering has been back on course since D8: a correct Rust autodiff trainer, a matched control, learned 4-bit rounding, and bit-exact integer serving.
   - The geometry has left the model. The active crates contain no prime, zeta, Hopf, hyperbolic, icosian, E8 or lattice code.
   - The only geometric element is a quaternion lane rotation. It is fed by a dense matmul, and it lost by 0.025 nats (one seed) to a control that is *itself* a quaternion map (H(v)H(e0)x = v·x·v).
   - The founding routing thesis never had a valid test.
   - The pace of decisions (11 in 7 days; 3 direction changes in 14 hours) outran the evidence.
3. **The original idea was sound, and it has been published by others.**
   - TurboQuant, PolarQuant and QJL all *keep* the radius; none of them ablates it.
   - The 4-D, radius-preserving quaternion quantizer was published by MIT/IBM in May 2026 (HQMQ). On Llama-3-8B KV caches at about 3 bits it beats a TurboQuant-style baseline.
   - 4-D is a weak block size for compression; 8-D, 24-D and trellis codes win.
   - The idea's best homes are:
     - compressing event and KV memory with a *quantized* radius;
     - more deeply, **dynamics**, where the radius is the retention (decay) factor and the direction is a non-commuting rotation (§8.1).
4. **The physics premise needs restating.** Bytes moved and instructions executed decide energy; multiplier circuits barely register.
   - In DRAM-bound serving, multipliers account for at most about 0.4% of the energy floor.
   - Multiplier-free *lookup-table* kernels can still save energy by executing fewer instructions: T-MAC measured 21–61% less energy at equal bit width.
   - The current integer session does the opposite: it *emulates* multiplication in software. That makes it 4.4× slower than hardware multiply on x86, with bit-identical output.
   - No joule has ever been measured in this repository (§5).
5. **Geometry has one theorem-backed job it can do cheaply: exact non-abelian state transport.**
   - With transitions in the binary icosahedral group 2I (the 600-cell), one 4-D lane can track the A5 word problem.
   - The A5 word problem is NC¹-complete. Diagonal SSMs of the Mamba class provably cannot track it at arbitrary length, assuming TC⁰ ≠ NC¹, fixed depth and log precision.
   - **This review measured it** (§6.2):
     - Quaternion lanes learned A5 and extrapolated to 16× the training length.
     - Snapped to 2I and served as a 120-state integer automaton (one byte of state and two table reads per token), they stayed at 100% to length 4,096, where the float32 originals drifted to 0.62–0.74.
     - Every commutative model stayed at chance.
   - **Honest limits:**
     - Two learned reflections (DeltaProduct-style) match the capability, so the quaternion/2I edge is *exact, cheap serving*, not unique power.
     - That training lands on 2I is mathematically forced: any exact A5 representation in SU(2) is conjugate to 2I.
     - S5 is provably out of reach for rotation-only lanes.
     - Learning is bimodal and fragile.
     - The repo's own near-identity parameterisation (q = normalize(e0 + 0.1·raw)) never learns it.
     - Natural-text perplexity barely moves.
6. **The correct theory (§8).**
   - **What geometry replaces.** The *state-transition* matrix (rotation times radial decay) and relative transport (a data-dependent, non-commutative "quaternionic RoPE").
   - **Attention and recall.** Their products can be designed away with codebook-keyed lookup-table scores and exact addressed memory, or kept as integer products, which requires the D0-b ruling in §11.2.
   - **Per-token projections** become low-bit additive maps.
   - **Knowledge storage stays in parameters,** at about 2 bits per parameter, measured with ≥8-bit weights.
   - **Energy floor.** It is set by the bytes of parameters touched per token. For models larger than the cache, *sparse access* is therefore the dominant energy lever. That conflicts with the owner's "no sparse routing", which is the key decision in §11.1.
7. **Next steps, in order (§9).**
   - **Hours to days:**
     - cool down the existing checkpoints;
     - remove the serving overheads;
     - measure the first joules with `macmon`;
     - add the missing *commutative* and *no-transport* control arms plus an A5 probe to the existing learner.
   - **Weeks:** choose between an *incremental* path (the D8 learner with a throughput fix and 2I tracking lanes) and a *parallel linear-recurrence re-base*, using a throughput benchmark and matched-token pilots with 3 seeds.
   - **Retire:**
     - local selector and admission tuning at T = 256;
     - zeta and prime mechanisms in predictive paths;
     - authored 32-prompt panels as architecture gates;
     - n = 1 comparisons.

## 1. What exists today (verified facts)

| Item | Fact | Label |
|---|---|---|
| Accepted model | One-layer GRU-like recurrence (d=256) with 64 quaternion lanes; one soft read head over all ≤255 earlier events, with age bias and a NoRead slot; pointer-sentinel copy mixture; tied 4,096-token BPE vocabulary; TinyStories data | Source joint_model.rs, config.rs |
| Size | 1,678,466 parameters; 1,048,576 (62.5%) embedding; **629,890 non-embedding** | Derived, 3 agents agree |
| Quality (full 249,856-target development population) | Quaternion 2.117, ordinary 2.092. Same population, 7.16M transformer #1014 after 30M tokens: 2.127 (sealed). After 150M tokens (#1017): about 1.57. Comparison tail: quaternion 2.110, control 2.085, 5-gram plus cache 2.392 | Source |
| Training | Rust/Candle F32; constant LR 1e-3 with no cooldown; about 30M target visits per arm; 1,362 tokens/s per arm, about 14 GFLOP/s | Source, Derived |
| Serving | Standalone integer session; ≤4-bit weights; bit-exact with training evaluation; about 97k software-emulated products per token; reads all 1,672,704 four-bit weights every token (62.7% of reads are the output head) | Source, Measured |
| Tests | Integer 24/24, tokenizer 3/3, training 68/68 on x86; no correctness bug found. The 2I group-table property was confirmed by independent rebuilds; the repo's own core `group_table` test was not run here (the core compile took more than 20 minutes) | Measured |
| Geometry in the forward path | The quaternion lane rotation only | Source, grep |
| Code | Active path 21,841 lines, 3.3% of the 666,682 in `crates/`; `native_geometric` (186k lines) is unused by the D8 path | Measured |
| Energy | Never measured | Source |
