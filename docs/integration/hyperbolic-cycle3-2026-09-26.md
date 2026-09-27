# Hyperbolic geometry, cycle 3: the Lorentz read in the Rust D8 learner

2026-09-26 · Requested by the owner · References #820 · Branch `claude/blissful-wozniak-girwwq` (kept separate from `main`)

**Status.** An evidence note, not a decision record. It continues [cycle 2](hyperbolic-cycle2-2026-09-26.md), which added an optional Lorentz (hyperbolic) read to the project's Rust D8 learner but had not trained it. This cycle trains it.

Labels: **Measured** (Rust runs in the review sandbox; seeds stated), **Derived**, **Hypothesis**. Every run's settings, final metrics, curves and hashes, including the interrupted attempts, are in [the evidence file](../evidence/hyperbolic-cycle3-runs-2026-09-26.json). Every model here is the project's own `JointModel`, trained by the project's own optimizer and gradient code. Only the data preparation and the run harness are scratch.

## 0. Findings

1. **The hyperbolic (Lorentz) read trains in the project's Rust learner, and it is consistently but only slightly better than the dot-product read.**
   - On this repository's code, over four seeds (2,000 updates at width 128, context 128), the flat-start Lorentz read beat the retained Dot read in 3 of 4 seeds, by 0.018 nats per token on average (0.007 bits per byte).
   - The committed Dot-matched start also beat Dot in 3 of 4 seeds.
   - On WikiText (one seed) the flat-start Lorentz read was ahead by 0.037 nats per token.
   - Training cost per update was the same (§4).
2. **The gain is much smaller than the 600-step pilot suggested.** The pilot's differences of 0.1–0.25 nats came mostly from Dot runs caught at that moment in a failure mode, read dependence. By 2,000 steps those runs had mostly recovered (§3–§4).
3. **Read dependence is the main training risk in this learner, and the initial read temperature controls it.**
   - Every sharp-start run (8 of 8, across the Dot, Dot-matched Lorentz and Euclidean reads) went through a phase with NoRead mass below 0.1, reading memory at almost every position. In 3 of them the model's read-free prediction got worse; in 2, that lasted to the end of the run.
   - No flat-start run (9 of 9) did either.
   - With the committed Dot-matched start, 1 of 4 Lorentz seeds ended read-dependent, 0.20 nats behind Dot.
4. **The Lorentz read needed an initialisation fix, now in the repository.** Cycle 2's score let the NoRead option take 83% of the read mass at the start. The score now has a learned radius δ and a Dot-matched scale β (§2).
5. **Next-token training alone gave the key radius only a weak hierarchy signal.**
   - Within the same token, radius correlates with code-scope depth at 0.08–0.24 for Lorentz keys, against 0.03–0.08 for Dot keys (§5).
   - The strong radius-depth coding of cycle 2 needs an objective that asks for it.
6. **An integer Lorentz read would cost about what the Dot read costs** (§6, derived and checked in scratch): one shared inner product per candidate plus two scalar products and a table lookup. With 24 guard bits the distance error stays below 10⁻⁴.

## 1. Setup

**Model.** The D8 joint learner at reduced scale:
- state width 128, read width 64, context 128 tokens, quaternion transport, full admission;
- batch 16, AdamW at a constant 10⁻³, the project's clipping, one CPU worker per run.

The retained contract is width 256 and context 256. The owner can run this comparison at that scale on TinyStories with the committed example (§7).

**Data.** Two corpora, each with its own 4,096-token byte-level BPE (256 bytes plus 3,840 merges, trained in Rust on the training split only, with exact round trips):

| Corpus | Training tokens | Held-out tokens | Bytes per token |
|---|---:|---:|---:|
| This repository's Rust code (split by file, as in cycle 2) | 7,000,992 | 206,844 | 3.64 |
| WikiText-2 (raw) | 3,044,794 | 316,580 | 3.55 |

**Evaluation.** Fixed, evenly spaced held-out windows, each from a fresh state, as the D8 evaluator does. Reported per window set:
- negative log-likelihood (nats per token) with the read enabled, and bits per byte;
- the same with the read disabled (NoRead);
- the mean NoRead mass.

Final numbers use 256 windows (32,768 targets) in the pilot and 512 windows (65,536 targets) in the full comparison.

## 2. The initialisation problem and its fix

**Measured.** With cycle 2's Lorentz score −β·arcosh(z) and β = 1, the learned NoRead logit took 83% of the read mass at initialisation, against 6% for Dot.

**Derived: why.**
- RMS-normalised inputs through a Glorot map give each query or key coordinate variance 2d/(r+d), so |q|² ≈ 2rd/(r+d) ≈ 85 at d = 128, r = 64.
- Every initial distance therefore sits near arcosh(1 + 85) ≈ 5.15, with a spread across candidates of only about ±0.15.
- Scores near −5 lose to a NoRead logit near 0, and scores that flat carry little information.
- β is a single scalar under Adam, so it moves by at most about the learning rate per step. It grew only from 1.00 to 1.38 in 600 steps.

**Fix (in the repo).** The score is now β·(δ − arcosh z), with a learned radius δ:
- keys closer than δ outscore a zero NoRead logit, so δ is a learned hyperbolic ball around the query;
- δ starts at δ₀ = arcosh(1 + 2rd/(r+d)), the distance between independent initial rows;
- β starts at β₀ = sinh(δ₀)/√r.

**Derived.** Near z₀ = cosh δ₀ and for fixed norms, arcosh z ≈ δ₀ − ⟨q,k⟩/sinh δ₀. So this initial score is ⟨q,k⟩/√r + const plus a per-key radius term: the Lorentz read starts as a first-order copy of the Dot read.

**Measured check** (numpy, the model's initial distributions; 4 draws of 16 queries × 200 keys).

| Initial score, across candidates | Width 128 | Width 256 |
|---|---:|---:|
| Dot spread | 1.32 | 1.57 |
| Lorentz spread, β = 1 | 0.15 | 0.15 |
| Lorentz spread, β₀ | 1.64 | 1.96 |
| Correlation of Lorentz (β₀) with Dot | 0.81 | 0.82 |
| Mean distance minus δ₀ | −0.015 | −0.026 |

The Rust test `lorentz_initial_read_matches_dot_scale` asserts these relations on the model's own initial parameters.

## 3. Pilot: 600 steps on code, and what causes the gain

The 2,000-step comparison (§4) supersedes these conclusions where they differ.

**Measured** (600 steps, 1.2 million target visits per run; 256 held-out windows).

| Read | Seed | NLL, read on | Bits/byte | NLL, NoRead | NoRead mass | β at end |
|---|---:|---:|---:|---:|---:|---:|
| Dot | 1 | 4.590 | 1.859 | 7.417 | 0.009 | — |
| Dot | 2 | 4.411 | 1.786 | 5.167 | 0.297 | — |
| Dot, flat start: learned scale from 0.12, so its initial scores are as flat as the β = 1 Lorentz read (control) | 1 | 4.528 | 1.834 | 5.318 | 0.313 | 0.16 |
| Dot, flat start (control) | 2 | 4.302 | 1.742 | 4.957 | 0.340 | 0.17 |
| Euclidean distance β(δ − ‖q − k‖), Dot-matched start (control) | 1 | 4.440 | 1.798 | 5.299 | 0.104 | 1.66 |
| Lorentz, no offset, β = 1 (cycle 2) | 1 | 4.481 | 1.815 | 5.257 | 0.602 | 1.38 |
| Lorentz + offset, β = 1 | 1 | 4.383 | 1.775 | 5.171 | 0.558 | 1.74 |
| Lorentz + offset, β = 1 | 2 | 4.337 | 1.756 | 5.117 | 0.589 | 1.84 |
| **Lorentz + offset, Dot-matched β₀ (committed)** | 1 | **4.377** | **1.772** | 5.261 | 0.075 | 8.92 |

**Reading.**
- **Seed 1 of Dot became dependent on its read; seed 2 did not.** In seed 1 the NoRead mass fell to 0.009, and the likelihood with the read disabled got worse as training went on (7.01 → 7.42 nats after step 100) while the read-enabled likelihood improved. Seed 2 kept NoRead near 0.3 and ended 0.18 nats better. §4 calls this state *read dependence*. It is not simply copying: only 42% of held-out code targets occur earlier in their 128-token window (34% for WikiText), and disabling the read also removes the read value from every recurrent update.
- **Runs with the same seed share initial arrays and training windows, so the comparisons are paired.**
  - Lorentz with the offset beat Dot in both seeds, by 0.21 and 0.07 nats.
  - It was also the most consistent arm: 4.34–4.38 across three runs and two initialisations, against 4.41–4.59 for Dot and 4.30–4.53 for the flat-start Dot control.
- **The flat start alone also helps Dot** (by 0.06 and 0.11 nats), so part of the pilot's gain is the initial read temperature. Whether the geometry adds more is unresolved at this length: Lorentz beat the flat-start control by 0.15 in seed 1 and lost by 0.035 in seed 2.
- **A flat-space distance read got most of the way in seed 1.** The Euclidean control (same Dot-matched start) ended at 4.440, between Dot (4.590) and Lorentz (4.38). In this one seed, distance scoring gave about 70% of the gain over Dot and curvature the remaining 0.06 nats. Its baseline, Dot seed 1, was the run that had become read-dependent.
- **The two Lorentz initialisations ended equal** (4.38 and 4.38 in seed 1). The committed Dot-matched start reads like Dot (NoRead 0.075); the β = 1 start reads about half the time (NoRead 0.56–0.59).
- **Cost:** 3.0–3.3 s per update for every arm; the Lorentz read costs no measurable training time here.

## 4. Full comparison: 2,000 steps

**Design.** Four arms on code, seeds 1 and 2, 2,000 updates each (4.1 million target visits):
- Dot (the retained read);
- Lorentz with the committed Dot-matched start;
- Lorentz with β₀ = 1 (flat start);
- Dot with a learned scale starting at 0.12 (flat start, the control).

Runs with the same seed share initial arrays, training windows and the 512 evaluation windows (65,536 targets). Each run used one CPU worker, with four to eight runs sharing four cores. Models are saved for §5.

**Decision rules**, written at 05:15 UTC before any 2,000-step final existed:
1. A geometry "beats" another only if the paired final NLL difference has the same sign in both seeds and averages more than 0.03 nats. The pilot showed seed effects of 0.1–0.2 nats, so smaller or mixed differences are reported as unresolved.
2. The Lorentz comparisons are made at equal starts: Dot-matched Lorentz against Dot, and flat Lorentz against flat Dot.
3. The committed Lorentz initialisation switches to the flat start (β₀ = 1) if flat Lorentz beats Dot-matched Lorentz under rule 1. Otherwise it stays.
4. The integer arcosh serving path (§6) is built only if some Lorentz arm beats its equal-start Dot control under rule 1.

**Incident.** The container restarted at 06:51 UTC and killed every process. The seed-1 Dot and Dot-matched Lorentz runs had just finished and recorded their final reports, but the example then failed to save their models: it wrote into a directory it had not created, now fixed in `e6be6e3`. The other six runs died without finals: the seed-2 pair at step 1,750 and the four flat-start runs at step 500. The flat-start runs were restarted from scratch at 06:55, and the example gained checkpoints with exact resume (`be6be3b`). The interrupted report roots are kept.

**Measured, final** (512 windows, 65,536 held-out code targets):

| Arm | Seed | NLL, read on | Bits/byte | NLL, NoRead | NoRead mass |
|---|---:|---:|---:|---:|---:|
| Dot | 1 | **3.246** | **1.308** | 3.890 | 0.477 |
| Lorentz, Dot-matched start | 1 | 3.450 | 1.390 | 8.375 | 0.036 |

**Measured, development curve to step 1,750** (32 windows; seed 2 has no final):

| Step | Dot, seed 2 | Lorentz Dot-matched, seed 2 |
|---:|---:|---:|
| 500 | 4.794 (NoRead 0.28) | 4.847 (0.014) |
| 1,000 | 3.882 (0.40) | 3.892 (0.39) |
| 1,500 | 3.524 (0.47) | 3.506 (0.58) |
| 1,750 | 3.422 (0.53) | 3.395 (0.60) |

**Measured, final: flat starts** (relaunched after the restart; same seeds, windows and evaluation):

| Arm | Seed | NLL, read on | Bits/byte | NLL, NoRead | NoRead mass | β, δ at end |
|---|---:|---:|---:|---:|---:|---:|
| **Lorentz, flat start (β₀ = 1)** | 1 | **3.236** | **1.304** | 3.888 | 0.650 | 3.58, 4.96 |
| **Lorentz, flat start** | 2 | **3.246** | **1.307** | 3.905 | 0.672 | 3.49, 4.95 |
| Dot, flat start (control) | 1 | 3.333 | 1.343 | 3.985 | 0.508 | 0.23, — |
| Dot, flat start (control) | 2 | 3.251 | 1.310 | 3.889 | 0.564 | 0.25, — |

**Applying the rules.**
- **Rule 1, flat Lorentz against flat Dot.** Lorentz is ahead in both seeds, by 0.097 and 0.006 nats; the mean is 0.051. The rule is met, but the seed-2 margin alone is within evaluation noise.
- **Rule 1, Dot-matched Lorentz against Dot.** Lorentz is behind by 0.204 in seed 1 (its seed-2 final was lost). The rule cannot be met.
- **Rule 3.** Settled by the seed-2 reruns below: the seeds disagree, so the committed start stays.
- **Rule 4.** Its condition is met by the first comparison, so the integer serving path may be built. With gains this small, it waits on the full-scale test (§9).

**Reading.**
- **The flat-start Lorentz read was the best and the most consistent arm:** 3.236 and 3.246, a seed spread of 0.010, against 0.082 for the flat-start Dot control.
- **No flat-start run became dependent on its read.** Across nine flat-start runs (five Lorentz, four Dot; pilot and full), the NoRead mass never fell below 0.19, or below 0.21 for Lorentz.
- **Every sharp-start run passed through a low-NoRead phase.** There are eight: Dot, Dot-matched Lorentz and Euclidean, counting the two seed-2 runs killed at step 1,750. Every one had NoRead mass below 0.1 at some evaluation.
  - In three, the read-free likelihood got worse during that phase. This is what §3 calls read dependence.
  - In two it lasted to the end of the run: the pilot's Dot seed 1 at 600 steps, and the full comparison's Dot-matched Lorentz seed 1 at 2,000 steps.
- **Its margin over plain Dot is small once Dot recovers:** 0.010 nats in seed 1. Most of the pilot's large differences came from Dot runs caught in read dependence at step 600. With more training, read dependence either resolved (Dot seed 1 here) or persisted (Dot-matched Lorentz seed 1).
- **For Dot, the flat start traded speed for safety.** It kept all four flat-Dot runs out of read dependence, but did not reliably make Dot better at 2,000 steps. In seed 1 it ended 0.087 behind the sharp start (3.333 against 3.246), which had recovered; in seed 2 it was 0.009 ahead (3.251 against 3.260).

**Measured, final: the seed-2 reruns and WikiText** (checkpointed; none needed resuming):

| Arm | Corpus | Seed | NLL, read on | Bits/byte | NLL, NoRead | NoRead mass |
|---|---|---:|---:|---:|---:|---:|
| Dot | code | 2 | 3.260 | 1.313 | 3.902 | 0.568 |
| Lorentz, Dot-matched start | code | 2 | **3.234** | **1.303** | 3.911 | 0.616 |
| Dot | WikiText | 1 | 4.429 | 1.805 | 4.663 | 0.746 |
| **Lorentz, flat start** | WikiText | 1 | **4.392** | **1.790** | 4.636 | 0.788 |

**All two-seed code finals** (nats per token):

| Arm | Seed 1 | Seed 2 | Mean |
|---|---:|---:|---:|
| Dot | 3.246 | 3.260 | 3.253 |
| Lorentz, Dot-matched start (committed) | 3.450 | 3.234 | 3.342 |
| Lorentz, flat start | 3.236 | 3.246 | 3.241 |
| Dot, flat start | 3.333 | 3.251 | 3.292 |

**Rules on two seeds.**
- **Dot-matched Lorentz against Dot:** +0.204 and −0.026 nats. The signs are mixed, so the comparison is unresolved.
- **Flat Lorentz against flat Dot:** −0.097 and −0.006, so flat Lorentz beats flat Dot (rule 1).
- **Rule 3, flat against Dot-matched Lorentz:** −0.214 and +0.011. The signs are mixed, so the committed start stays. The Dot-matched start produced both the best single run (3.234) and the one failure (3.450).
- **Flat Lorentz against Dot** (descriptive, unequal starts): −0.010 and −0.015, a consistent direction but below the 0.03 bar.
- **WikiText** (one seed): flat Lorentz is ahead of Dot by 0.037 nats (0.015 bits per byte).

**Extension to four seeds**, rules written at 09:20 UTC before seeds 3 and 4 were launched. Seeds 1 and 2 left two questions unresolved: which Lorentz start to commit, and whether Lorentz beats the retained Dot read. Seeds 3 and 4 run for Dot, Dot-matched Lorentz and flat Lorentz, with the same settings, checkpoints and 512 evaluation windows.

- **R3b, the committed start.** Switch the default to the flat start if either:
  - flat Lorentz has the lower final NLL in at least 3 of the 4 seeds; or
  - the Dot-matched start ends a run read-dependent (final NoRead mass below 0.1) in seed 3 or seed 4.
- **R1b, Lorentz against Dot.** Compare flat Lorentz with Dot:
  - it "beats" Dot if it is better in at least 3 of 4 seeds and the mean difference exceeds 0.03 nats;
  - it is "consistently but slightly better" if it is better in at least 3 of 4 seeds with a smaller mean;
  - otherwise the comparison is unresolved.

A second container restart, at about 10:49 UTC, stopped all six runs. Every one had a checkpoint at step 1,500, and each resumed in a new report root that records its parent.

**Measured, final: four seeds** (512 windows, 65,536 held-out code targets; nats per token):

| Arm | Seed 1 | Seed 2 | Seed 3 | Seed 4 | Mean |
|---|---:|---:|---:|---:|---:|
| Dot (retained read) | 3.246 | 3.260 | 3.272 | 3.219 | 3.249 |
| Lorentz, Dot-matched start (committed) | 3.450 | 3.234 | 3.218 | **3.215** | 3.279 |
| Lorentz, flat start (`lorentz_start=flat`) | 3.236 | 3.246 | 3.222 | 3.222 | **3.231** |

The seed-3 and seed-4 finals ended with NoRead mass 0.56–0.70 in every arm, so none of these runs finished read-dependent.

**Rules R3b and R1b.**
- **R3b, the committed start: it stays Dot-matched.** Flat Lorentz had the lower NLL in only one seed (seed 1; −0.214), and lost in seeds 2–4 by 0.011, 0.004 and 0.007. The Dot-matched start did not end read-dependent in seed 3 or 4.
- **R1b, Lorentz against Dot: consistently but slightly better.**
  - Flat Lorentz is ahead in 3 of 4 seeds (by 0.010, 0.015 and 0.051; behind by 0.004 in seed 4), with a mean gain of 0.018 nats (0.007 bits per byte), below the 0.03 bar.
  - The Dot-matched start is also ahead in 3 of 4 seeds (by 0.026, 0.054 and 0.003). Its seed-1 failure makes its mean 0.030 worse.
- **The trade-off between the two starts:**
  - The Dot-matched start is marginally better when it trains normally.
  - The flat start never became read-dependent in five runs. It is kept available as the example's `lorentz_start=flat` option, which reproduces the flat runs' initial state exactly.

## 5. What the Lorentz read learned

**Question.** In cycle 2, hyperbolic keys encoded depth in their radius when the task required it. Does a language-trained Lorentz read do the same on code?

**Measured** (saved models: both flat-start pairs and the seed-2 sharp-start pair; 800 held-out windows, 102,400 positions; key radius = arcsinh‖k‖ for every model; depth = unmatched code braces before the token, ignoring comments, strings and character literals):

| Model | Spearman ρ, radius vs depth | After removing each token id's mean radius | Median within-token ρ (50 most frequent tokens) |
|---|---:|---:|---:|
| Lorentz, flat start, seed 1 | +0.174 | +0.148 | +0.128 |
| Lorentz, flat start, seed 2 | +0.177 | +0.138 | +0.082 |
| Dot, flat start, seed 1 | +0.108 | +0.139 | +0.075 |
| Dot, flat start, seed 2 | +0.163 | +0.158 | +0.046 |
| Lorentz, Dot-matched start, seed 2 | +0.189 | +0.188 | +0.235 |
| Dot, seed 2 | +0.070 | +0.105 | +0.032 |

**Reading.**
- **The depth signal is weak in every model.** Once token identity is removed, the correlations are 0.10–0.19. The mean radius rises about 0.1 from depth 0 to depth 1 and changes little after that.
- **It is consistently stronger in the Lorentz keys.** Within the same token, the correlation between radius and depth is larger for Lorentz than for Dot in all three paired comparisons: 0.128 against 0.075, 0.082 against 0.046, and 0.235 against 0.032. A token deeper in scope gets a slightly larger radius in the Lorentz models, as a hierarchy would require, but only slightly.
- In cycle 2 the task made the hierarchy necessary and the radius carried it strongly. Here nothing in the next-token loss asks for it.
- **Hypothesis.** An objective that does ask for it, such as a JEPA-style latent prediction measured with the hyperbolic distance (§9), is what would make the radius carry structure.

**How the models used their read** (same positions; copy weight is g·(1 − NoRead), where g is the copy gate; a target counts as copyable when it occurs earlier in its window):

| Model | Read mass | Copy gate | Copy weight | On copyable targets | On the rest |
|---|---:|---:|---:|---:|---:|
| Lorentz, flat, seed 1 | 0.350 | 0.570 | 0.255 | 0.416 | 0.134 |
| Lorentz, flat, seed 2 | 0.327 | 0.547 | 0.223 | 0.367 | 0.115 |
| Dot, flat, seed 1 | 0.494 | 0.472 | 0.249 | 0.376 | 0.154 |
| Dot, flat, seed 2 | 0.440 | 0.430 | 0.204 | 0.314 | 0.122 |

The Lorentz models read less often but copy more selectively: 3.1–3.2 times more copy weight on copyable targets than on the rest, against 2.4–2.6 for Dot.

## 6. The integer Lorentz read

**Derived and checked in scratch.** The integer runtime stores queries and keys as Q8 codes, and the Dot read already computes ⟨q,k⟩ exactly in i128. The Lorentz read adds:
- |k|² and k₀ = √(1 + |k|²), computed once when the key is written and stored beside it;
- q₀ once per query;
- per candidate, z = q₀k₀ − ⟨q,k⟩ (one product and a shift), one table lookup for arcosh, and β·(δ − d) (one product).

That is 2 scalar products and a lookup on top of the 64 products the Dot score already costs per candidate.

**Precision.** Near q ≈ k, z − 1 is a small difference of large numbers. Measured over 4,000 pairs of Q8 codes against float64 on the same codes:

| Computation of z | Maximum distance error |
|---|---:|
| q₀, k₀ with 16 fractional bits | 1.9 × 10⁻² |
| **q₀, k₀ with 24 fractional bits (i128)** | **8.7 × 10⁻⁵** |
| Stable form z − 1 = ½(‖q − k‖² − (‖q‖² − ‖k‖²)²/(q₀ + k₀)²), 16 bits | 5.5 × 10⁻⁴ |

Scores are quantised to Q8 steps of 0.0039, so 24 guard bits are enough.

**Table.** arcosh(1 + u) is indexed by the leading-bit position of u plus 10 mantissa bits; u below 2⁻³⁰ maps to zero. That is about 52K entries, sealed like the existing exp, tanh and sigmoid tables.

**Not implemented yet.** The quantisation-aware training path, packed export and runtime kernel for a Lorentz model. Today the integer runtime refuses Lorentz manifests with a typed error. *(Implemented later the same day; see [§11](#11-addendum-the-integer-lorentz-read).)*

## 7. Changes in the repository

Commits `9df1afa`, `2ce0ab8`, `e6be6e3`, `be6be3b` and the commit adding this note, on this branch:
- **`JointConfig` shapes** (`crates/uor-r4-integer/src/config.rs`): a Lorentz model has a second scalar, `read.lorentz_offset`. Dot models keep their exact parameter inventory, JSON, hashes and contracts.
- **The Lorentz score** (`crates/uor-r4-training/src/joint_model.rs`) is exp(`read.lorentz_log_beta`)·(`read.lorentz_offset` − arcosh z), in both the full and the bounded-admission read. Both scalars are initialised by `lorentz_initial_offset` and `lorentz_initial_log_beta` (§2), and the checkpoint contract declares the score and both initialisations. Neither initialiser draws from the random stream, so every other initial array is unchanged.
- **Tests.** The five cycle-2 Lorentz tests now cover the offset: inventory, gradients, the F64 score reference and the checkpoint round trip. A new test checks that the initial Lorentz read matches the Dot read's scale and ranking (§2). The Dot identity test is unchanged and passes.
- **The `joint-read-geometry` example** (`crates/uor-r4-training/examples/`) trains the joint learner from scratch on u16 token files for one geometry and reports Read and NoRead likelihood, NoRead mass and bits per byte in a claimed, sealed report root. Every 2,000-step run in §4 used it. It reads the owner's TinyStories `train.u16`/`dev.u16` as they are.
- **Example options added after the first runs:**
  - `read_dropout=P` trains round(P·batch) windows of every batch with the read disabled, in one loss weighted by window count. It is a direct test of read dependence (§4). With P = 0 a fixed run reproduces the earlier result bit for bit.
  - `checkpoint_every` and `resume` keep an atomically replaced checkpoint (model, AdamW moments, progress) and continue in a new report root. A run killed with SIGKILL and resumed reproduces the uninterrupted run's final likelihoods and every development evaluation bit for bit.
  - The model directory is now created before saving (the bug in §4's incident).
  - `lorentz_start=flat` starts the Lorentz scale at β = 1 instead of the committed Dot-matched value. A zero-step run reproduces the flat runs' initial evaluation exactly.
- **Checks run:**
  - 6 focused training tests and integer 25/25;
  - `cargo fmt --check`, and clippy clean on the touched files;
  - the example exercised end to end: a sealed report, refusal to reuse a report root, and refusal of an unknown argument before any directory is claimed.

The scratch harness, the BPE tool, the controls (flat-start Dot, Euclidean) and the analysis scripts are not committed.

## 8. Resources

All work ran on the review sandbox's four CPU cores and 15 GB of RAM. It incurred no external or accelerator cost.

| Item | Amount |
|---|---|
| Pilot training and evaluation | 9 runs, 4.86 process-hours (one thread each), about 6.5 core-hours of machine time |
| 2,000-step runs | 16 finished runs, about 27.5 core-hours of machine time |
| Lost to container restarts (included above) | about 6.5 core-hours at the first restart, before checkpoints existed; under 1.5 at the second, when all six runs resumed from step-1,500 checkpoints |
| Builds, tests, BPE, analyses | about 1.5 core-hours |
| Total | about 36 core-hours over 8 hours of wall time, 03:17–11:20 UTC, with the four cores fully used |
| Peak memory | 0.8–1.0 GB per training process; at most 8 GB in total |
| New scratch storage | build cache 8.2 → 12 GB; data 22 MB; run roots 22 MB, plus 2.7 MB per saved model; analysis outputs 4.7 MB |
| Repository | code and example changes only; no data, models or build products are committed |

Training cost per update was the same for both geometries: about 3.0–3.3 s per update, single-threaded, at width 128, context 128, batch 16.

## 9. What this changes

1. **Keep the Lorentz read as a tested option, not yet a replacement.**
   - It is at least as good as Dot, slightly better on average, costs the same to train, and has cycle 2's advantage on hierarchies.
   - The evidence is at reduced scale, on code and one seed of prose.
2. **The decisive test is the owner's full-scale setting.** Run the committed example on the retained TinyStories tokens at width 256 and context 256, two seeds for each of `geometry=dot`, `geometry=lorentz` and `geometry=lorentz lorentz_start=flat`. For example:

   ```text
   cargo run --release -p uor-r4-training --features cpu-accelerate --example joint-read-geometry -- \
     train=TRAIN.u16 valid=DEV.u16 out=NEW_ROOT geometry=lorentz seed=1 \
     width=256 context=256 batch=16 steps=7324 shards=2 \
     eval_every=250 checkpoint_every=250 save_model=true
   ```

   `TRAIN.u16` is one retained training store, or both concatenated. A stopped run continues with `resume=OLD_ROOT/checkpoint`.
3. **Watch the NoRead mass early in any run, whatever the geometry.** If it falls below 0.1 while the read-free likelihood gets worse, the run has become read-dependent. A flat start prevented this in every run here. `read_dropout` is available as a more direct remedy but has not been measured.
4. **Next engineering, if the Lorentz read is adopted: the integer serving path** (§6). It needs quantisation-aware training for the two scalars, packed export, the sealed arcosh table, the runtime kernel and parity tests against the F32 path. *(Built later the same day; see [§11](#11-addendum-the-integer-lorentz-read).)*
5. **Next research.**
   - A hyperbolic JEPA objective: predict a latent summary of the coming text from the recurrent state, and score the prediction with the Lorentz distance, to make the key radius carry structure (§5). The predictor exists only during training, so serving cost is unchanged.
   - Cycle 2's hyperbolic admission test at 4K–16K candidates, which this cycle did not reach.

## 10. Addendum: context 256

Added later on 2026-09-26. **Measured**, reduced scale: code corpus, width 128, two seeds.

**Question.** Does the flat-start Lorentz read's advantage over the retained Dot read grow when the context doubles?

**Design.** The same example and corpus as §4. The same 2,000 updates at the same tokens per update: batch 8 × context 256 here, against batch 16 × context 128 in §4. Arms: `geometry=dot` and `geometry=lorentz lorentz_start=flat`, seeds 1 and 2. The final evaluation is 512 windows (131,072 held-out targets).
- A container restart stopped all four runs at step 750. They resumed exactly from their step-750 checkpoints; the interrupted roots are kept.
- They were also paused for about 17 minutes in total for unrelated benchmarks.

**Final NLL** (nats per token):

| Seed | Dot | flat Lorentz | Lorentz − Dot | same seed at context 128 |
|---|---:|---:|---:|---:|
| 1 | 3.1694 | 3.1186 | −0.051 | −0.010 |
| 2 | 3.1851 | 3.1238 | −0.061 | −0.015 |

- The advantage is about four times the context-128 value for the same seeds; the four-seed mean at context 128 was −0.018.
- Both seeds agree and the mean, 0.056, exceeds rule 1's 0.03-nat threshold. So under rule 1, flat Lorentz beats the *retained* Dot read at context 256.
- The equal-start control of rule 2, flat Dot, was not run at this context.
- Dot seed 2 became read-dependent: NoRead mass 0.004, read-free NLL 5.89. Dot seed 1 did not (NoRead mass 0.41). Neither Lorentz run did (0.58–0.59).

**Where the gain is** (Lorentz − Dot NLL, 95% bootstrap intervals over 800 windows of the probe dump; negative favours Lorentz):

| Tokens | Share | Seed 1 | Seed 2 |
|---|---:|---:|---:|
| not copyable | 48% | −0.021 [−0.028, −0.014] | −0.080 [−0.087, −0.073] |
| copy, same scope | 36% | −0.064 [−0.072, −0.056] | −0.023 [−0.033, −0.014] |
| copy, enclosing scope | 7% | −0.053 [−0.081, −0.028] | −0.056 [−0.085, −0.027] |
| copy distance 1–4 | 6% | −0.055 [−0.073, −0.038] | +0.041 [+0.023, +0.060] |
| copy distance 17–64 | 20% | −0.078 [−0.090, −0.068] | −0.053 [−0.064, −0.041] |
| copy distance 65–255 | 9% | −0.086 [−0.105, −0.068] | −0.088 [−0.106, −0.069] |
| slope per doubling of copy distance | | −0.012 [−0.015, −0.008] | −0.023 [−0.027, −0.020] |

- **The largest and most consistent gain is on copies 65–255 tokens back**, mostly beyond the context-128 window.
- The gain grows with copy distance in both seeds, as it did at context 128.
- The enclosing-scope gain in the table does not control for copy distance.
- *Controlled.* Regressing the per-token difference on log copy distance plus scope category (same scope as the baseline; bootstrap over windows) keeps the distance effect: −0.012 [−0.016, −0.008] and −0.022 [−0.026, −0.018] per doubling.
- At equal distance, the enclosing-scope effect points in opposite directions in the two seeds: +0.023 [+0.003, +0.043] (less Lorentz advantage than same-scope copies) and −0.018 [−0.047, +0.010].
- So, as at context 128, the gain is a copy-distance effect, not a scope-structure effect.
- The gain on non-copyable tokens differs fourfold between the seeds. Seed 2's is the larger, and its Dot run is the read-dependent one.

**Read geometry of the trained Lorentz reads** (probe dump).
- *Radius.* Keys sit at hyperbolic radius about 3.5 (queries 3.8), a strongly curved regime. Within a window the key radius varies little (std 0.23).
- *Radius versus direction.* Replacing each key's radius by the window mean, keeping its direction, moves the read distribution by 0.09–0.10 total variation. Top-1 agreement is 76–77%.
- *Scores and cosine.* The cosine to the query explains 79–81% of the Lorentz score variance, against 98–99% for Dot. At this radius the hyperbolic distance is not a function of the angle alone.

**What changes.**
- The Lorentz read becomes the stronger candidate for the native model's read at longer context.
- Rule 4 (build the integer arcosh path) was already met in §4, by flat Lorentz against flat Dot at context 128. This addendum adds weight to that decision; it is not a new trigger.
- The full-scale test of §9.2, on TinyStories at width and context 256, is still the decisive one.

**Cost.**
- Four runs of about 11,000 s of training updates each (about 12.2 core-hours in total), with 4–8 processes sharing four cores.
- The probe dumps and analyses took a few minutes.
- The analyses are scratch scripts; the numbers are in the [evidence file](../evidence/hyperbolic-cycle3-runs-2026-09-26.json) under `context_256_addendum`.

## 11. Addendum: the integer Lorentz read

Added later on 2026-09-26 (cycle 3b). **Measured** unless labelled. The numbers are in the [evidence file](../evidence/hyperbolic-cycle3-runs-2026-09-26.json) under `cycle_3b_integer_read`.

**What was built.** Commits `4183830`, `d0b417a` and `f92cf80`, and the commit adding this section.
- **Kernel** ([`crates/uor-r4-integer/src/lorentz.rs`](../../crates/uor-r4-integer/src/lorentz.rs)).
  - Queries and keys are Q8 codes. With Q = |q|², K = |k|² and D = ⟨q,k⟩ in code units, P = (2¹⁶+Q)(2¹⁶+K) and M = 2¹⁶+D, the argument is z − 1 = (√P − M)/2¹⁶.
  - P − M² = 2¹⁶|q − k|² + (QK − D²) is an exact non-negative integer. So for M > 0 the kernel evaluates (P − M²)/(√P + M), with no cancellation: one floor square root with 24 guard bits and one rounded division give z − 1 at Q32.
  - It applies the training clamp: z ≥ 1 + 2⁻²⁰, the F32 value of 1 + 10⁻⁶.
  - arcosh(1 + u) comes from a sealed Q24 table. Argument codes below 2¹⁰ are read directly, then 1,024 linearly interpolated points per octave up to 2⁶⁴: 56,321 entries, 225 KB.
  - The scale β = exp(log β) is computed once at load: ln 2 range reduction and a Taylor series at Q60, rounded to Q32.
  - The score β(δ − d) is rounded to Q40. Age is added, then the sum is rounded and clipped to the Q8 score interface.
  - Every product goes through the runtime's software shift-and-add arithmetic, like the rest of this prototype runtime.
  - Unlike the §6 sketch, z − 1 comes from exact integers rather than from rounded q₀ and k₀.
- **Contract.** A Lorentz model binds `packed_numerical_contract(Lorentz)`: the retained packed contract with a quantized Lorentz read declaration, one definition shared by training and the runtime. The Dot contract, the retained table files, Dot model identities and the Dot arithmetic are unchanged.
- **Training side.**
  - The two scalars quantize as signed 16-bit codes with one frozen scale each, like the other additive parameters.
  - *Precision, stated explicitly.* A Lorentz model thus carries two learned signed 16-bit scalars (log β and δ) beside its ≤4-bit weight maps, and the kernel derives a Q32 scale from log β at load. They are not 4-bit parameters. Whether D10's weight-map rule admits them is part of the owner clarification requested by the fourth lab.
  - Quantization, packed export and `load_hard` accept Lorentz models.
  - The table export adds `arcosh.json` and `arcosh.bin` next to the retained tables. A Lorentz model refuses a table root without them; older roots still serve Dot models.
- **Shapes.** The runtime now also serves width-128 models: normalization divides by the state width. The width-256 arithmetic is unchanged; the parity test reproduces its earlier numbers exactly.
- **Tools.**
  - The `joint-integer-parity` example packs a checkpoint. A float checkpoint gets post-training scales; a quantization-aware one keeps its own. It then scores the same windows with the float weights, the packed F32 emulator and the integer runtime, read on and off.
  - `joint-read-geometry` gains `init=MODEL_DIR` (fine-tune a saved float model) and `quantize_ramp=N` (quantization-aware training on the packed format's frozen scales).
  - [`scripts/native-lorentz-m1.sh`](../../scripts/native-lorentz-m1.sh) runs §9.2's full-scale comparison through to integer serving on an Apple-silicon Mac.
  - With `TOKENIZER=`, it also packs each fine-tuned model as a development serving bundle (`uor-r4-integer pack-development`) and generates text with the integer runtime. The bundle states that it has no accepted parent; `pack` for accepted artifacts is unchanged.

**Kernel accuracy** (focused tests in `uor-r4-integer` and `joint_integer_tables`).
- The integer arcosh is within 2·10⁻⁷ of the function across every octave, including between grid points.
- z − 1 is within 2⁻³¹ + 10⁻¹³(1 + |q|² + |k|²) of an F64 evaluation. The test covers 400 random, near-parallel and opposite vector pairs.
- The load-time scale is within one Q32 unit of exp over the range the calibration can produce.
- Untrained packed models at widths 256 and 128 run in the integer runtime within 0.01 of their F32 emulator on every probability and state coordinate. The cases are Dot, and Lorentz with the Dot-matched, flat and trained scalars.
- At width 256 the Dot-matched initial Lorentz scale reads almost only the oldest key: read entropy 0.01 nats. So the flat and trained-scale cases, at 1.7–3.3 nats, are the ones that exercise spread reads.

**Trained models, post-training quantization.**
- *Models and quantization.* The four context-256 models of §10 (width 128). Each is packed with frozen dyadic scales calibrated once on its float weights, without fine-tuning.
- *Evaluation.* 64 evenly spaced windows of 256 tokens (16,384 targets) of the development split that §4 and §10 also evaluated. This is a development measurement, not a final holdout.
- *Table.* Read enabled; NLL in nats per token on the same windows.

| Model | Float | Integer | Quantization cost | Integer − emulator | Mean TV | Top-1 agrees with emulator | Read effect, float / integer |
|---|---:|---:|---:|---:|---:|---:|---:|
| Dot s1 | 3.2676 | 3.3546 | +0.087 | −0.00006 | 0.0017 | 99.68% | 0.823 / 0.812 |
| Dot s2 | 3.3037 | 3.3704 | +0.067 | −0.00001 | 0.0017 | 99.77% | 2.581 / 2.488 |
| flat Lorentz s1 | 3.2276 | 3.3037 | +0.076 | −0.00005 | 0.0019 | 99.75% | 0.824 / 0.828 |
| flat Lorentz s2 | 3.2342 | 3.3167 | +0.083 | −0.00004 | 0.0021 | 99.73% | 0.804 / 0.798 |

Dot s2 is the read-dependent run of §10.

- **The integer runtime matches the packed F32 emulator to 0.0001 nats per token for both geometries**, with the read on and off.
- **The Lorentz advantage survives 4-bit integer serving.** Integer Lorentz − Dot is −0.051 (seed 1) and −0.054 (seed 2); in float on these windows it is −0.040 and −0.070.
- Post-training 4-bit quantization costs 0.07–0.09 nats per token, 0.027–0.036 bits per byte.
- **The largest single deviation slightly exceeds the retained 0.01 engineering limit in 4 of 8 model/mode cells.** The maximum is taken over 16,384 positions × 4,096 tokens and every state coordinate. It reaches 0.0120 on a probability and 0.0146 on a state coordinate.
  - It occurs for Dot as well, and with the read disabled. So it comes from the shared recurrent and output path, not from the Lorentz kernel.
  - The limit was set for the accepted width-256 model; these models are width 128.

**Cost of the read.** Idle machine, one thread, 4 windows per run, two repetitions.
- An integer step with the read over up to 255 keys takes 4.2–4.3 ms for Dot and 4.3–4.6 ms for Lorentz.
- With the read disabled it takes 2.6–2.9 ms.
- The difference between the geometries is within the run-to-run spread.
- This measures the prototype runtime's software-multiply arithmetic, not an optimized serving speed.

**Quantization-aware fine-tuning.**
- *Design.* Each §10 model was fine-tuned for 300 updates (batch 8 × context 256, AdamW at learning rate 3·10⁻⁴) in two arms:
  - with `quantize_ramp=100`: the packed format's frozen scales, calibrated once on the starting weights, with straight-through fake quantization ramped to full strength over 100 updates;
  - a float control with the same updates, windows and optimizer.
- *Evaluation.* The final evaluation covers the same 512 windows (131,072 targets) as §10's. The integer column is `joint-integer-parity` on those windows.
- *Table.* Read enabled; NLL in nats per token.

| Model | §10 float | Float fine-tune | Quantization-aware, integer | Integer − float fine-tune | Integer − emulator |
|---|---:|---:|---:|---:|---:|
| Dot s1 | 3.1694 | 3.0953 | 3.1303 | +0.035 | +0.00001 |
| Dot s2 | 3.1851 | 3.0963 | 3.1306 | +0.034 | −0.00001 |
| flat Lorentz s1 | 3.1186 | 3.0359 | 3.0680 | +0.032 | −0.00001 |
| flat Lorentz s2 | 3.1238 | 3.0374 | 3.0713 | +0.034 | −0.00001 |

- **After quantization-aware fine-tuning, 4-bit integer serving costs 0.032–0.035 nats per token** against a float model fine-tuned equally long: 0.013–0.014 bits per byte, down from 0.07–0.09 nats after post-training quantization.
  - Every integer model is better than its §10 float parent (by 0.039–0.055 nats), because it trained 300 more updates.
  - At update 100, when the ramp completes, the quantization-aware models already matched or beat their parents on the 64-window development evaluation.
- **The Lorentz advantage is unchanged by 4-bit integer serving.** Integer Lorentz − Dot is −0.062 and −0.059 nats. The equal float fine-tunes give −0.060 and −0.059, and §10 gave −0.051 and −0.061.
- **Integer against emulator.**
  - NLL within 0.00001 nats; mean total variation 0.0017–0.0027; top-1 agreement 99.5–99.7%.
  - The read-enabled NoRead mass is identical to four decimals.
  - The read effect is kept: 0.865–0.873 nats for the healthy runs and 2.94 for read-dependent Dot s2.
- **The largest single deviation**, over 131,072 positions, 4,096 tokens and every state coordinate, exceeds 0.01 in every model.
  - With the read enabled, it is 0.012–0.015 (probability) and 0.012–0.014 (state) for Dot, and 0.014–0.017 and 0.009–0.018 for Lorentz. The two geometries are of the same size, and two seeds cannot separate them.
  - Dot s2 reaches 0.031 and 0.034 with the read *disabled*, in the read-dependent model's weak read-free path.
  - These maxima grow with the number of positions scored: the 64-window runs above stayed at 0.012 and 0.015.
  - The mean total variation stays below 0.003.

**What changes.**
- The native model's hyperbolic read has an integer serving path. Trained models run in it at parity with their F32 emulator, at the Dot read's cost.
- The Lorentz advantage over Dot measured in §4 and §10 carries through 4-bit weights and integer arithmetic unchanged, after quantization-aware fine-tuning.
- Quantization-aware fine-tuning is the way to package a trained native model. A few hundred updates bring the 4-bit loss to about 0.03 nats; post-training quantization alone costs 0.07–0.09.
- §9.2's full-scale test can now end in integer serving: [`scripts/native-lorentz-m1.sh`](../../scripts/native-lorentz-m1.sh) runs it on the owner's M1.
- The retained 0.01 limit on the largest single deviation does not hold at this width and evaluation length, for either geometry. It is a limit on one position out of 131,072; a limit on the mean, or on a high quantile, would suit a long evaluation better.
- *Not established.* Speed or energy on an M1; behaviour at width 256 or on TinyStories; a final holdout; any language capability beyond these development scores.

**Cost.**
- About 4.2 core-hours on the review sandbox's four cores; no external or accelerator cost.
  - Fine-tunes: 8 runs, 2.3 core-hours, four at a time. A quantization-aware update took about 3.4 s and a float one 2.6 s at width 128, batch 8, context 256.
  - Parity runs: 1.4 core-hours. They took 140–150 s each at 64 windows and 1,130–1,160 s each at 512 windows.
  - Builds, tests, pilots and timing: about half a core-hour.
- New scratch storage: 54 MB, including the packed models, tables and report roots. The repository gained one Rust module, one example, one script and these notes.
