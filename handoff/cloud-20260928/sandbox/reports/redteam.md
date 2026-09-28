# Red-team report on REVIEW_DRAFT.md (22:49 regeneration)

Every defect below was re-checked after the regeneration and is still present. §6 placeholders are treated as pending. Scratch code and output: `exp/redteam/rt_checks.py`, `rt_checks_output.txt`.

**Spot checks (31).** Confirmed: math.rs loop, 4.4×, KVAR, INC-0171/0168, AGENTS.md:32/122, echo CI, recovery doc, HQMQ (+0.745 vs +1.118), Allen-Zhu–Li, P–S U(1), 2404.08819 assumptions, E6 outputs. Wrong or overstated: nine items below.

## CRITICAL

**C1. §8.4, §8.5, §0.6 overstate what geometry replaces.**
> "Geometry replaces every matmul in roles 1–2."

§9.3's time-mix is decayed linear attention: S_t = r_t S_{t−1} + k̃_t v_tᵀ, read by S_tᵀq̃_t.
- Both write and read are d_k×d_v activation×activation products. At S scale that is about 0.59M per token (12 layers × 6 heads × 64×64 × 2) [DERIVED], roughly 6× the 97k software products that §3.4 condemns.
- The quaternion frame only rotates q and k.
- Arch F8: these products "need 4-bit or ternary keys/queries … or the quarter-square LUT". Ternary Q/K has already failed to converge (§3.6).

**Fix:** "Geometry replaces the state-transition matrix and supplies relative transport. The linear-attention write/read stay bilinear integer products (≈0.6M per token at S scale), which are D0-b-legal only with ≤4-bit k/q/v (untested) or the §11.2 ruling." Make §9.3 conditional on §11.2.

**C2. B3 rests on a misapplied cost figure** (§9.1, §9.5, §11.3).
> "the *only M1-feasible route* … (LoLCATs converted 1B–8B models with 40M tokens…)"

[LITERATURE 2410.10254]:
- LoLCATs keeps a 64-token sliding-window *softmax* attention in every layer (Table 7).
- Without the window, Llama-3-8B stays at chance MMLU (23.8) even with attention transfer (Table 5).
- With the window, MMLU still drops from 66.6 to 52.8 (8B) and from 31.9 to 27.3 (1B).

Pure recurrent conversion is MOHAWK's regime: 3B tokens for 1.3B, i.e. months on an M1 at 360M [DERIVED].
- SmolLM2's 49,152-token vocabulary alone makes a 47M-parameter head [SOURCE models/smollm2-360m-instruct.json].
- Ternary QAT of the transferred MLPs is uncosted.

**Fix:** "Replacing attention entirely has no cheap precedent. The 40M-token figure applies only to windowed-softmax hybrids, which need their own ruling. Pure conversion needs billions of tokens. Usefulness at 135–360M is a hypothesis."

**C3. §11.1 A is over-supported.**
> "All three independent specialist agents … (physics, architecture, verification, audit, literature) flagged decision 11.1 without being prompted by each other."

This names five agents, and the shared briefing *instructed* every agent: "NOTE a possible tension: … D5 … Flag this tension where relevant" [SOURCE CONTEXT.md:42-44].

Three further problems:
- **Only support is a learned router.** "32-expert MoE … 1.3×" used a *learned* top-1 router (tutel) [LITERATURE 2404.05405 App. D], which is exactly what A forbids.
- **Fixed-gate routing.** "A state is quantized to a lattice/polytope cell, and that address selects table rows" is, for a learned state, top-1 routing with a frozen geometric gate. The owner may call that MoE.
- **Arch dissent omitted.** The arch report says D5's energy rationale "is weak for cache-resident models", and a 5.7 MB model fits in L2. §11.4 makes that the M1 ceiling.

Also: D5 is "Ratified by owner instruction" [SOURCE DECISIONS.md:241].

**Fix:**
- Delete the independence claim.
- Describe A as "top-1 access with a fixed codebook over learned states".
- Scope the sparse-access lever to models larger than cache (B3, or ≳30M at 4-bit).

## MAJOR

**M1. The physics headline misreads the evidence** (§0.4, §5.1).
> "In every configuration modelled … at most about 0.4%" and "Both saved energy by *finishing sooner, having moved fewer bits*, not through cheaper arithmetic."

- The physics report gives ≤0.4% only for DRAM-bound configurations, and ≤3.4% for cache-resident ternary.
- T-MAC compared the *same* 4- and 2-bit models against llama.cpp. Its LUT kernels remove dequantize-and-multiply work, cutting energy 20.6–61.2% at equal bits [LITERATURE 2407.00088 §5.4].
- bitnet.cpp's energy method is undescribed (physics F5).

**Fix:** "Multipliers are ≤0.4% of the DRAM-bound floor. Bytes and instructions decide energy. T-MAC's multiplier-free LUT kernel cut measured energy 21–61% at equal bits; the repo's shift-add emulation adds instructions instead." This steelmans D0-b's LUT half.

**M2. §11.2 is not neutral.**
> "D0-b's goal is no floating point and weights of 4 bits or fewer."

D0-b itself says it "preserves the actual objective — no multiplier, tiny RAM, no GPU, local, measured energy … The multiplier constraint stays" [SOURCE DECISIONS.md:113-114]. Option A therefore changes an owner-stated objective.

Option B's quarter-square LUT has two problems:
- It reads a 1 MiB table. By the draft's own Horowitz figures (≈100 pJ per 64-bit read from a 1 MB cache vs ≈3 pJ per multiply), it is a speed fix and likely energy-negative.
- It covers only ≤16-bit operands, but the mixture products are 48×15-bit [verify F2].

**M3. The integer-quaternion codebook is inconsistent and costlier than stated** (§8.4).
> "The 1/√N norm correction is folded into the separately stored radius" vs "r ∈ {1 − 2^−k}, computed as h − (h >> k)".

Once 1/|p| is folded in, the radius is r/|p|, not 1−2^−k.
- Only 15 of 157 codewords have rational |p| [MEASURED].
- The only unit quaternions with dyadic coordinates are the 24 Hurwitz units, by a sum-of-four-squares argument mod 8 [DERIVED]. Any finer shift-add codebook therefore needs a non-dyadic normaliser.

Measured in 16-bit fixed point against float64, approximating r/|p| with 3 signed powers of two [MEASURED]:
- An isometric lane's norm ends at 1.52× after T=256 and 827× after T=4096.
- Six terms are needed for ≤0.7% drift.
- A timescale-256 memory lane errs 47% with 4 terms and 0.8% with 6.

Separately, "7.15°" is an SO(3) angle (quaternion angle 3.58°), while 2I is quoted at 36° as a quaternion angle.

**Fix:** "Each fine codeword needs a constant multiply by r/|p|: about 6 shift-adds per coordinate for ≤1% drift at 4k tokens, i.e. ~36 operations per lane-step versus 16 hardware multiplies." Use one angle convention throughout.

**M4. The finite-subgroup claim is false; "rediscovered 2I" is forced** (§8.4, §0.5).
> "exactness for unbounded length with non-abelian structure forces 2I-level coarseness."

Binary dihedral groups are non-abelian and arbitrarily fine: Dic₆₀ has 3° elements [MEASURED].
- **Correct statement:** "non-solvable structure forces 2I."
- **Rediscovery is forced.** By the same classification, *any* exact A5 solution in an SU(2) lane is conjugate to 2I.
- **The closure is not exact.** At tolerances tighter than 0.05 the learned closure is not finite (>400 elements; `e6_closure_output.txt`).

**Fix:** "Training found near-exact A5 homomorphisms (≈0.2° from 2I, as any exact solution must be); snapping makes them exact."

**M5. The S5 claim and the controls are rigged toward a win** (§9.1 B1, §9.3).
- S5 cannot be tracked by SU(2)_L lanes; the math report itself says "S5 is not reachable" (§2.2).
- The gates only require beating diagonal controls, which provably fail.
- "neither win … nor tie" lets geometry survive on a tie.
- The math report's own falsifier is "the snapped 2I arm loses to DeltaProduct₂ or gathers at equal serving cost."

**Fix:**
- Drop S5, or add permutation-gather lanes.
- Gate against DeltaProduct n_h=2/4 and against gathers at equal serving cost.
- Kill the geometric claim unless 2I beats the best non-diagonal control.

**M6. The tracking lanes contradict the time-mix design** (§9.3).

Tracking lanes have "Input path: none", yet they sit inside S_t = r_t S_{t−1} + k̃_t v_tᵀ, which is additive by construction. They must be separate group-index lanes (h_t = q_t h_{t−1}) with an index-embedding readout.

Also, 2609.18966 (single author, repository-native pre-registration) states that "'delete W_b' is a probe … not a training recipe."

**M7. The exposure share is overstated** (§3.2, §7.2).
> "Most of the 0.5-nat gap to #1017 is therefore *data/compute exposure*."

The audit's slope (0.15–0.16 nats per e-fold) puts the learner at ≈1.84 at 150M tokens, so exposure explains about half the gap [DERIVED].

A same-population comparison exists and should be used: #1014 sealed scores 2.127 against the learners' 2.092/2.117 on the identical 249,856 targets [SOURCE 1014 doc:19-20; arch F1]. The draft instead compares dev 2.131 with tail 2.085.

**M8. The process audit is unfair in places** (§7.3).
> "Median open-to-merge time is 1.8 minutes, with no reviews" and "the ceremony now adds little *independent* assurance."

- Reviews were checked on one PR (#1389), and §7.1 itself credits reviews.
- **Counter-example 1.** The reader-utility review withdrew a false "representation-limit" claim after finding a fit/serve threshold mismatch [SOURCE reader-utility-review-2026-09-21.md:7].
- **Counter-example 2.** The precision factorial localized the gap to parameter quantization and led directly to the accepted learned rounding. It used 183 s of model time in a 53-min cycle that the audit counts as "≤10%" [SOURCE precision-factorial-result; d8_cycles.csv].
- Echo CI predates Sep 8 and is declared [AGENTS.md:146].
- Budget raises follow the owner's standing authorization [AGENTS.md:161].

**Fix:** state these facts. Measure decisions per wall-hour, not model-time share.

**M9. Zeta evidence is held to a double standard** (§2.2, §4.2, §5.4).

The 91/96 vs 86/96 figure is an ablation without retraining. Its Wilson intervals overlap: [0.88, 0.98] vs [0.82, 0.94] [DERIVED]. The same row shows geometry-disabled at 66/96 and H4-disabled at 82/96 [SOURCE native_geometric_recovery_973.md:309-311]. Rely on the math measurements, or quote the whole row.

**M10. The CI timing claim contradicts verify F1** (§10.2).
> "under 2 minutes, as measured by the verification agent"

Verify F1 measured a training-crate build of 8 m 45 s plus 14 s of tests.

**M11. Missing alternatives.**
- **Incremental path.** Keep the D8 learner, which already ties #1014 at equal tokens with 9.5× fewer non-embedding parameters. Fix its throughput (chunked or truncated BPTT, larger batch), add 2I tracking lanes, and compare that against the re-base.
  - The lead's completed runs already show diagonal 1.869 vs quaternion 1.881 BPB (seed 0; `exp/lead/full_*_s0.json`).
  - §9.3 must not outrun §6.3.
- **Exact factored or class-based output softmax.** It cuts the 62.7% output-head reads without a learned router (arch rec. 7).
- **The existing integer softmax read.** It costs 5% of compute, and removing it costs +0.48 nats. It should be a first-class option.

## MINOR

1. §0.5 should add "unless TC⁰=NC¹, fixed depth, log precision" [2404.08819 Cor. 4.7].
2. §3.5 "below typical run-to-run variation": Kaplan gives about 0.02 for seeds (§4.2). Say "indistinguishable without seeds".
3. §8.2: Allen-Zhu–Li did not test QAT; they only suggest it "may be necessary".
4. §3.4 "6.5×" is, in the source's words, "a cross-run indication, not a newly controlled F32" measurement.
5. §5.5, §10.1: macmon only "*may* remove" the blockers; macOS 26 support is open (physics §6).
6. Golden gates (§8.4):
   - P–S prove optimal *almost*-covering *and* big holes (≈N^{−3/4}), so §6.4's holes are predicted.
   - ½ℤ[φ] coordinates hold only after scaling by √(7+5φ).
   - "exact per step" conflicts with "must round".
7. §9.3 "1–2B tokens of TinyStories": the pinned corpus is 2.23 GB of 2.72M stories [SOURCE], about 0.5–0.6B tokens [HYPOTHESIS]. Say 2–4 epochs.
8. KD from #1017 (1.574) into a student targeting 1.0–1.3: the teacher is weaker than the target. Use it early only.
9. §11.1 B:
   - "4.5–5.7 MB" corresponds to 18–23M parameters, not 40M.
   - "Forbid any per-token selection" would also forbid embedding lookup.
10. §9.2 cooldown: state which decision it changes (D9); §9.6 dismisses the analogous card.
11. Stale PENDING markers at §2.2, §2.5, §5.4.
12. 1,672,704 weights, not 1,672,960; harmonize the slowdown ranges; "Dead code" → "unused by D8".
13. "1e-13" is float64. In float32 I measured ≤9e-6 relative error and |U_t| drift ≤6e-6 at T=16,384. That is fine, but say so.
14. §0.1 "20–40M" rests on an unmeasured throughput of 0.3 TFLOP/s.

## What the draft gets right (keep)

- **Engineering.** The engineering verdict is accurate: bug-free D8 path, bit-exact serving, 4.4× software-multiply penalty.
- **Quantization lineage.** The TurboQuant/PolarQuant/QJL premise correction is correct, and the HQMQ details are exact.
- **Mathematics.**
  - The associative scan and the frame decomposition are correct (≤3e-14 in float64, my check).
  - The no-go theorem is correct.
  - §8.4's NC¹/TC⁰ statement is properly conditional.
  - E6 is now reported accurately.
- **Verdicts.** The zeta, prime and Hopf verdicts and the Householder-control critique are sound.
- **Measurement protocol.** "Measure joules before claiming them" is right.
- **"Partly off course"** is fair on the substance. The process part needs M8.
- **Breakthrough verdict.** Right on frontier and zeta; about right on B1 with fair controls; too optimistic on B3.
