# Route Matrix

## Stage policy
- Screen: 1-2 seeds
- Confirm: 2 seeds
- Finalize: 4 seeds

## Current Queue Override
- Current primary RR: **Publication phase — Angular Manifold Routing / sparse-routing-MoE replacement (no active RR assigned; experimental research complete).**
  All prior experimental RRs (including RR-068) are frozen/inactive and preserved as historical record.
- Current primary INC: **Publication** — all increments closed. Paper skeleton and publication packet complete. INC-0171 KEEP (2026-03-26).

<!-- ═══ FROZEN RESEARCH HISTORY — all entries below are closed/inactive ═══ -->
- 2026-03-26 INC-0171 KEEP: End-to-End LM Integration. HOPF/BASELINE PPL ratio 1.081 (confirmed). eff_ratio 1.39× at convergence. Stage 7: COMPLETE. Publication-ready.
- 2026-03-14 INC-0170 KEEP: Large-K Angular Capacity Test. eff_ratio 2.6–2.8× at K=600–5000. Full-range alpha=0.657.
- 2026-03-14 INC-0169 KEEP: Canonical law freeze. eff_buckets=2.957×K^0.572 (TRANS ORIG). Norm-invariant. Shell hierarchy excluded. INC-0170 proposed.
- 2026-03-14 INC-0168 KEEP: Norm-geometry diagnostic. α=0.572 TRANS ORIG norm-invariant. Routing is purely angular.
- 2026-03-14 INC-0167 KEEP: Scaling mechanism diagnostic. √K scaling from sector discretization, shells structurally inaccessible.
- 2026-03-14 INC-0166 KEEP: Architecture law freeze + K=100 boundary audit.
- 2026-03-14 INC-0164 KEEP: Scaling-law consistency. Predicted vs measured ratios within 1-11% (TRANS), 1-6% (BASE). 13/14 criteria pass.
- 2026-03-14 INC-0163 KEEP: Matched-progress compute efficiency. ORIG 1.7-2.2x fewer eff buckets (TRANS), 1.9-2.4x faster convergence.
- 2026-03-13 INC-0154 REFINE: Event-gate efficiency routing-agnostic (delta <0.1pp).
- 2026-03-14 INC-0153 REFINE: Per-sample spectral-gate r > 0.3 but geometry-agnostic (delta 2.7–5.1pp).
- 2026-03-14 INC-0152 REFINE: Gate saturated. Spectral roughness ↔ error confirmed.
- 2026-03-13 INC-0151 KEEP: 4-seed finalize confirmed. Stage 5 PARTIAL-PASS (strong).
- 2026-03-13 INC-0149 KEEP: Task-signal smoothness confirmed (screen). error_indicator +109%, true_margin +28–134%.
- 2026-03-13 INC-0147 REFINE: Raw alpha (λ=0) rel_diff=66.7% vs full transport (λ=1) at 68.7%.
  Fiber alpha confirmed (+20.6pp over base). Transport correction not needed.
- 2026-03-13 INC-0145 KEEP: HOPF_FULL rel_diff=40.7% (stable: 38.6%, 42.7%). HOPF_BASE=31.2%. HOPF_TRANS=28.1% (variable, K=25 bin dilution). Stage 4 → PARTIAL-PASS.
- 2026-03-13 INC-0144 KEEP: Hopf fixed (phase4d_hopf_base, 4D) vs K-means adaptive (100D). HOPF rel_diff=31.2%
  (stable: 31.8%, 30.6%). KMEANS rel_diff=3.1% (variable). Stage 3 → PARTIAL-PASS.
- 2026-03-13 INC-0143 KEEP: 4-seed finalize of PPMI-SVD discrimination. rel_diff=38.5% mean across
  4 seeds (range 30.6%–54.6%). All seeds pass. Stage 2 closed PARTIAL-PASS.
- 2026-03-13 INC-0142 KEEP: PPMI-SVD semantic embeddings confirm H^4 Hopf routing discrimination. rel_diff=31.2%, z≈4.2, 2-seed confirm. ORIG > COL_PERM > GAUSSIAN correct ordering.
  Stage 2 geometry NOT falsified. Stage 2 → PARTIAL-PASS pending finalize.
- 2026-03-13 INC-0141 KILL: routing with optimal dims (46,117,62,78) — max within-pair correlation |corr|=0.479 — gives OPT_ORIG pmax=0.379 vs OPT_COL_PERM pmax=0.388 (wrong direction), rel_diff=0.025.
  Pre-screen TV=0.109 signal did not survive routing. Mathematical proof: chi_u and delta are scale-invariant; hash embedding is isotropic by construction.
  Stage 2 is proxy-task-blocked on wikitext2 hash embedding. Next test: semantically structured embeddings.
- 2026-03-13 INC-0140 KILL: angular sector routing (phase4d_hopf_base, learn_so8=0) indistinguishable from col-perm on L2-normalized embeddings.
  Forensic audit confirms genuine kill: col-perm changes 66% of sector assignments but sector SIZE distribution is near-invariant (TV=0.009).
  Root cause: within-pair Hopf correlations near-zero on L2-normalized embeddings (corr≈−0.04 for dims 0,2; −0.02 for dims 4,6).
  All fixed-geometry routing paths on L2-normalized embeddings now exhausted (INC-0136–0141).
  Root cause confirmed: hash embedding lacks semantic angular structure regardless of dim choice.
- 2026-03-13 INC-0138 REFINE: geometry-only shell activation confirmed stable 2-shell structure;
  real vs Gaussian separation strong; real vs col-perm indistinguishable at shell level (norm-driven).
- 2026-03-12 root-theory audit:
  - treat the translated sparse-event lower-bank and dual-anchor line as
    supporting downstream evidence
  - do not continue from `INC-0135` as the main next branch
  - restore the queue to the earlier unresolved geometry gate:
    `RR-061` / `INC-0136_measure_consistent_h4_hopf_route_return.md`
  - translated retrieval remains an evaluation harness, not the primary proof
    surface
- 2026-03-12 `INC-0136` geometry-return screen result:
  - direct geodesic shell substitution via `phase4d_hopf_base_ball` failed the
    route-health gate
  - it worsened shell concentration and neighborhood preservation versus
    `HOPF_BASE_K25_PHI`
  - move next to:
    `RR-061` / Stage 3 INC — Hopf Angular Correctness (now unblocked by Stage 2 closure)

## Routes
- `R0`: `sector_mode=kmeans`, `scale_mode=radial`, `time_pressure_lambda=0.0`
  - Hypothesis: strongest control baseline.
  - Current status: standing transfer control, but collapsed on shells.
  - Kill: only replaced as control if a better control definition is needed.

- `R5`: `sector_mode=phase4d`, `phase4_dims=i,j,k,l`
  - Hypothesis: 4D polar structuring improves long-tail routing.
  - Current status: best-known synthetic route family via `R5B`.
  - Kill: no confirm-stage gain.

- `R8`: `sector_mode=phase4d_adaptive`
  - Hypothesis: time-expanded phi-balanced widening prevents angular collapse while preserving the quality/runtime benefit of `phase4d`.
  - Current status:
    - fixed-controller branch exists as historical comparator (`D30_FIXED_SG16`)
    - continuous `phi_ratio` branch exists as historical comparator (`PHI_D32_L120`, `PHI_D30_L120`)
    - `PHI_PHI_PHI v1` is the current lead family:
      - normalized artifact label: `PHI3_K25_D36_L065`
      - historical artifact label: `PHILOG_D36_L065`
      - superseded inside the routed family by the phase-coupled `INC-0024` branch
    - `PHASE_K25_C035` is the current routed family lead candidate:
      - beats `PHI3_K25_D36_L065` on confirm-stage quality and runtime
      - still misses the strict runtime gate vs `R0` by a small margin
      - keep it as the routed family lead candidate, not the runtime lead
    - `PHI3_K20_D36_L065` is the active compression comparator:
      - won screen
      - lost 4-seed confirm to `K25`
  - Kill: fails larger-subset control, cannot maintain shell activation, survives only on a narrow and unstable timing edge, or fails seed-wise health review.

- `R6`: `sector_mode=complex2`, `complex_dims=i,j`
  - Hypothesis: complex-plane routing may help as a local discriminator even if it is weak globally.
  - Current status: global use underperformed; local use remains research-only.
  - Kill: unstable or no efficiency improvement.

- `R9`: `sector_mode=phase4d_complex_local`
  - Hypothesis: use `phase4d_adaptive` for coarse routing and local complex zoom for neighborhood refinement.
  - Current status:
    - `INC-0020` rescued the branch with local convergence and `local_min_k=2`
    - `HYB4_M2_T010_C005` is the healthiest hybrid local-quality branch
    - branch is still quality-oriented, not hardware-efficient
  - Kill: any branch that regresses back into unseen-route explosion or cannot justify its runtime cost.

- `R10`: `sector_mode=phase4d_hopf`
  - Hypothesis: direct shell-capacity coupling from capped `H^4` growth plus Hopf-aware pair-bin allocation fixes the remaining global geometry mismatch.
  - Current status:
    - `HOPF_K25_BASE_IT40_P2_STATIC` is the current operational routed lead
    - `HOPF_K25_BASE_IT60_P4_STATIC` is now the historical static reference
    - `HOPF_K25_BASE_IT60_P4` remains the dynamic quality reference
    - cheap routed frontier: `chart_iters=40`, `so8_candidates=2`, `scale_candidates=2`
    - static training-route reuse is now part of the active systems stack for this family
    - still compressed to only about `4` effective sectors
    - cheap static variant now beats cheap `R0` on both quality and runtime
  - Kill: if the phi/Fibonacci lattice cannot widen this branch without losing the quality gain.

- `R11`: `sector_mode=phase4d_hopf_fib_rung`
  - Hypothesis: recurrence-constrained `phi^2` rung forcing can widen Hopf without losing the core quality signal.
  - Current status:
    - `INC-0031` widened Hopf to about `10.5` sectors and `20` buckets
    - active `chi` usage stayed at `2` bins
    - quality stayed close to Hopf and slightly better than `R0`
    - runtime regressed too sharply to promote
    - `INC-0032` threshold gating reduced cost modestly but still left the family far slower than Hopf and `R0`
    - keep as a geometry candidate family, not an operational route
  - Kill: replaced as the main widened-Hopf candidate by `R12`.

- `R12`: `sector_mode=phase4d_hopf_fib_band`
  - Hypothesis: shared-state `phi^2` banding can preserve widened Hopf geometry while cutting the runtime cost of ungated rung forcing.
  - Current status:
    - `INC-0033` kept the widened Hopf signal (`10.5` sectors) and cut runtime sharply relative to `R11`
    - `CTRL-0003` kept the widened quality signal on 4 seeds
    - `INC-0040` reduced schedule rescued runtime
    - `INC-0041` showed the widened efficient lead does not hold under larger load
    - `HOPF_PHI2_BAND_IT40_P2_STATIC` is now the widened cheap routed lead
    - `HOPF_PHI2_BAND_IT48_P3_STATIC` and `HOPF_PHI2_BAND_IT60_P4_STATIC` are historical static references
    - `HOPF_PHI2_BAND_IT60_P4` remains the dynamic widened reference
    - `chi` concentration remained severe
    - keep as the widened efficient route family
  - Kill: if a stronger global-alignment branch replaces it as the better widened Hopf candidate.

- `R13`: `sector_mode=phase4d_hopf_blend`
  - Hypothesis: a Hopf-anchored blended capacity law can widen only where the local geometry is under-allocated, avoiding the global cost of `phi^2` overlays.
  - Current status:
    - `INC-0034` completed
    - the branch widened Hopf to about `8-9` sectors
    - it reduced `chi` concentration relative to `R12`
    - it did not beat `HOPF_K25_BASE` on quality
    - it did not recover the runtime story vs `R0`
    - keep only as a historical negative result
  - Kill: already killed as the primary next branch.

- `R14`: `sector_mode=phase4d_hopf_ball`
  - Hypothesis: keep Hopf angular routing intact, but anchor shells to original-ball geodesic radius to recover global radial structure.
  - Current status:
    - `INC-0035` Slice B completed
    - improved proxy MSE relative to `HOPF_K25_BASE`
    - worsened shell concentration and Poincare alignment
    - keep only as a negative control for shell-only global repair
  - Kill: already killed as the primary next branch.

- `R15`: `sector_mode=phase4d_hopf_iso`
  - Hypothesis: route shells and sectors from a shared rotation-only near-isometric chart coordinate to recover global Poincare alignment.
  - Current status:
    - `INC-0036` completed
    - recovered exact Poincare alignment on the fast screen
    - remained compressed at about `4` sectors / `8` buckets
    - slower than both `HOPF_K25_BASE` and `R0`
    - keep only as a mathematical diagnostic and as a base for the next isometric-band branch
  - Kill: already killed as a standalone promotion branch.

- `R16`: `sector_mode=phase4d_hopf_fib_band_iso`
  - Hypothesis: exact alignment plus shared-state widening can coexist in one route family.
  - Current status:
    - `INC-0037` completed
    - exact Poincare alignment and widened sectors were both recovered
    - runtime regressed too sharply to promote
    - keep only as a mathematical diagnostic
  - Kill: already killed as an operational promotion branch.

- `R17`: `sector_mode=phase4d_hopf_fib_band_bound`
  - Hypothesis: partial chart scale can preserve part of the alignment win without paying the full isometric cost.
  - Current status:
    - `INC-0038` completed
    - bounded scale produced a real alignment/runtime interpolation
    - no bounded point passed the operational gate
    - keep only as a diagnostic family
  - Kill: already killed as an active promotion branch.

- `R18`: `sector_mode=phase4d_hopf_base`
  - Hypothesis: coarse routing should live on the Hopf base while the common
    fiber phase remains excluded from the coarse address.
  - Current status:
    - corrected `INC-0062` screen and confirm completed on 2026-03-11
    - `HOPF_BASE_K25_PHI` is the canonical no-fiber-phase coarse-address control
    - corrected Hopf sector diagnostics show uniquely low within-sector `chi`
      spread and a strong base/fiber separation signature
    - pure Hopf still keeps the best confirm MSE
  - Kill: only if a future route law provides the same coarse-address clarity
    while dominating it on both quality and route health.

- `R19`: `sector_mode=phase4d_hopf_transport`
  - Hypothesis: a geometry-induced transported fiber phase should move addresses
    materially relative to the no-phase Hopf-base control.
  - Current status:
    - corrected `INC-0063` screen completed on 2026-03-11
    - the old inert negative was invalidated by dead `alpha` bins at `K=25`
    - corrected variants now have `phase_transport_alpha_bins=2.0`
    - corrected address audit shows about `98.6%-98.8%` sector difference vs
      `HOPF_BASE_K25_PHI`
    - branch is mechanism-positive but not yet the routed quality lead
  - Kill: only if a corrected confirm still cannot convert the live mechanism
    into a stable advantage.

- `R20`: `sector_mode=phase4d_hopf_transport_complex`
  - Hypothesis: coupling the field term into transported phase should produce a
    stronger and more attributable phase mechanism than base-only transport.
  - Current status:
    - corrected `INC-0064` screen completed on 2026-03-11
    - field-shift metrics are strongly nonzero
    - corrected address audit shows about `98.64%` sector difference vs
      `HOPF_BASE_K25_PHI`
    - branch passes the route-health gate
    - same-chart coupling still trails Hopf-base and pure Hopf on proxy MSE
  - Kill: only if the explicit product-field follow-up fails to improve on this
    surrogate.

- `R21`: `sector_mode=phase4d_hopf_product_phase`
  - Hypothesis: an explicit asymmetric `H^4 x H^4` split should preserve route
    health while letting the second factor drive attributable phase motion on
    the first.
  - Current status:
    - `INC-0065` confirm completed on 2026-03-11
    - both carried product candidates passed the 4-seed route-health gate
    - `phase_transport_field_shift_abs_mean` is nonzero across the branch
    - seed-0 confirm address audit shows about `98.4%-98.6%` sector difference vs
      `HOPF_BASE_K25_PHI`
    - `H4XH4_FIELD_A150` has the best confirmed product MSE
    - `H4XH4_FIELD_A100` is the stabilized-candidate recommendation from the
      sweep analysis
    - pure Hopf still holds the overall routed quality lead
    - `INC-0066` confirm also shows a distinct spectral signature on this
      branch:
      - higher low-mode participation than the control set
      - lower low-frequency sector concentration than the control set
    - `INC-0067` and `INC-0068` then showed that the same confirmed branch does
      not beat the Hopf controls on direct label probes or routed task-error
      probes on the proxy target
    - `INC-0069` confirm then showed that the same fixed product routes do
      preserve translated retrieval signal:
      - versus `HOPF_K25_BASE_PHI`, both product routes cut candidate fraction
        and slightly cut amortized runtime across 4 seeds
      - `HOPF_BASE_K25_PHI` still remained the strongest coarse-address
        translated comparator
    - `INC-0070` screen then killed the first translated rescue surface:
      - low-margin reranking stayed healthy but gave back too much runtime
      - no rerank variant beat its corresponding fixed product baseline on the
        translated tradeoff
    - `INC-0071` confirm then promoted the second `H^4` to a translated
      addressing result:
      - `H4XH4_FIELD_A150_CPX8` improved top-1 over the fixed product
        baselines and over `HOPF_K25_BASE_PHI`
      - candidate fraction dropped to about `0.187`
      - runtime still trailed the main Hopf translated control
    - `INC-0072` confirm then converted that same fixed route/key law into a
      translated systems win:
      - `H4XH4_FIELD_A150_CPX8` still improved top-1 over `HOPF_K25_BASE_PHI`
      - online and amortized runtime both moved below the main Hopf translated
        control
    - `INC-0073` confirm then hardened that same systems result under larger
      translated load:
      - `H4XH4_FIELD_A150_CPX8` kept a large pruning advantage
      - online and amortized runtime both stayed below the Hopf translated
        controls
      - the smaller-load top-1 edge narrowed away at the harder load
    - `INC-0074` confirm then carried the same fixed law directly against
      dense exact retrieval:
      - `H4XH4_FIELD_A150_CPX8` kept a major pruning advantage vs dense
      - online and amortized runtime both stayed far below dense
      - the dense-frontier systems lead gave back only a very small amount of
        top-1
    - `INC-0075` confirm then killed bounded quality rescue on that same
      frontier:
      - the rerank variants were not worth carrying
      - `H4XH4_FIELD_A150` stayed the quality-matched routed point
      - `H4XH4_FIELD_A150_CPX8` stayed the systems lead
    - `INC-0076` confirm then established repeated-query break-even on the same
      fixed law:
      - no routed crossover survived at `Q08`
      - `H4XH4_FIELD_A150_Q16` matched dense top-1 while beating dense on
        amortized cost
      - `H4XH4_FIELD_A150_CPX8_Q16` became the first stronger-pruning systems
        crossover point
      - `H4XH4_FIELD_A150_CPX8_Q24` remained the stabilized systems point
    - current read:
      - the branch is mechanism-positive and operator-positive
      - it is not task-signal-positive on proxy regression
      - it is translated-retrieval-positive and now translated-addressing-
        positive under secondary keys
      - it is now also translated-systems-positive on the fixed route and key
        law
      - larger-load hardening now also passes
      - dense-frontier pressure now also passes
      - bounded quality rescue is now dead
      - the branch now has a confirmed amortized crossover against dense exact
        retrieval
      - `INC-0077` then showed that the crossover survives the first explicit
        hardware-cost profile:
        - search-work ratios stay stable across bank size
        - at `max_train=12000`, crossover begins at `Q16`
        - at `max_train=6000`, crossover survives only at `Q24`
        - `H4XH4_FIELD_A150_CPX8_Q24_T6000` is the first smaller-bank
          crossover point
      - `INC-0078` then confirmed the crossover boundary on the fixed law:
        - `max_train=3000`: no crossover through `Q24`
        - `max_train=6000`: first systems crossover at `Q24`
        - `max_train=12000`: first systems crossover at `Q12`
        - the secondary-key search-work ratio stays pinned near `0.19`
      - `INC-0079` then confirmed the first larger-bank extension:
        - `max_train=18000`: first systems crossover already at `Q08`
        - the secondary-key search-work ratio still stays pinned near `0.19`
      - `INC-0080` then held the onset at `Q08` through `24000` and `30000`
      - `INC-0081` then found the first real `Q04` crossover at `36000`, but
        the upper-bank onset stayed non-monotone because `40000` still began
        at `Q08`
      - `INC-0082` then explained that split from static offline route-build
        cost composition rather than from route collapse
      - `INC-0083` then operationally rescued the explained split with
        persistent cache reuse:
        - `Q04 T40000` cold routed amortized `10.389s` became warm `1.972s`
        - `Q08 T40000` cold routed amortized `6.165s` became warm `1.891s`
        - routed top-1 and candidate fraction stayed unchanged under reuse
      - `INC-0084` then confirmed that warm-cache onset already reaches `Q01`
        on the fixed `T40000` bank:
        - `Q01` warm routed amortized `2.204s` vs dense `9.536s`
        - routed top-1 and candidate fraction stayed unchanged across the
          full `Q01/Q02/Q04/Q08` bracket
      - `INC-0085` then confirmed that the same fixed stack already reaches
        tracked warm-cache `Q01` crossover by `T3000`:
        - `Q01 T3000` warm routed amortized `0.074s` vs dense `0.159s`
        - search work stayed near `0.192` of dense
      - `INC-0086` then refined that onset below `T3000`:
        - `T2500` still misses at `Q01`, but crosses at `Q02`
        - `T2750` is now the earliest tracked confirmed warm-cache `Q01`
          crossover point
        - search work stayed pinned near `0.193` of dense across the bracket
      - `INC-0087` then moved the earliest tracked confirmed warm-cache `Q01`
        onset to `T2600`, but exposed a local non-monotone pocket at `T2650`
      - `INC-0088` then explained that pocket from local route-query versus
        dense-search cost balance rather than from route degeneration
      - `INC-0089` then re-centered the bracket and moved the earliest tracked
        confirmed warm-cache `Q01` onset again to `T2525`
      - `INC-0090` then closed the `2505/2510/2515/2520` bracket positive and
        moved the earliest tracked confirmed warm-cache `Q01` onset again to
        `T2505`
      - `INC-0091` then closed the final integer `2501/2502/2503/2504`
        bracket positive and moved the earliest tracked confirmed warm-cache
        `Q01` onset again to `T2501`
      - `INC-0092` then hardened that bracket and overturned the old exact
        separation:
        - `T2500` also survives at `Q01` on the expanded seed schedule
        - the lower-bank warm-cache floor therefore collapses to `T2500`
      - `INC-0093` then decomposed the cache-residency story:
        - chart-only warm preserves the upper-bank `T40000 Q01` systems win
        - route-only warm stays negative at both anchor operating points
        - the exact lower-bank `T2500 Q01` floor remains a full-warm claim
      - `INC-0094` then strengthens the operational read:
        - chart-resident `T2500` now crosses by `Q02`
        - chart-resident `T40000` already crosses by `Q01`
        - the remaining gap is chart-resident `T2500 Q01`
      - `INC-0095` then broadens the focused single-query read:
        - chart-resident `T2500 Q01` now crosses on the focused packet
        - the remaining issue is packet-scope sensitivity versus the older
          mixed-repeat `INC-0094` read
      - `INC-0096` then hardens that packet-scope question positively:
        - focused and mixed packets both stay positive at `T2500 Q01`
        - packet scope changes the margin size, not the sign
      - `INC-0097` then reopens sparse shell work and kills it at screen:
        - gated and banded shell controllers are mechanism-live
        - both variants fail shell health by collapsing shell balance
        - no sparse candidate beats the continuous product reference on the
          screen contract
      - `INC-0098` then closes translated cost decomposition:
        - the chart-resident stack is already positive against dense at both
          fixed anchors
        - the remaining chart-versus-full-warm gap is split between
          route-index build and retrieval-search overhead
        - no hidden residual accounting surface remains
      - `INC-0099` then closes sparse event proxy trainability positive:
        - `H4XH4_FIELD_A150_EVT_T070` survives 4 seeds as a healthy soft-sparse
          controller
        - `event_gate_mean≈0.319` while shell balance stays matched to the
          continuous product reference
        - the branch is positive on soft update mass, not yet on hard firing
      - `INC-0100` then closes translated sparse-event carry-forward positive:
        - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500` preserves the
          translated routed signal from the continuous product branch
        - online runtime drops from about `0.2759s` to `0.1177s`
        - amortized runtime drops from about `0.3380s` to `0.1686s`
        - the lower-bank `Q01` dense-amortized comparison is now a knife-edge
          tie rather than a routed miss
        - the branch stays soft-sparse with `event_gate_active_frac=0.0`
      - `INC-0101` then closes hard-event proxy activation positive/narrow:
        - `H4XH4_FIELD_A150_EVT_T070_TAU002` survives 4 seeds as the
          healthiest near-hard proxy point
        - `event_gate_mean≈0.0205` while shell balance stays matched to the
          continuous product reference
        - true hard thresholding stays healthy only in a mostly-on regime
          (`event_gate_active_frac≈0.844`)
        - the branch is therefore positive on near-hard activation, not on a
          genuinely sparse binary hard controller
      - `INC-0102` then closes translated near-hard carry-forward negative at
        screen:
        - the near-hard translated point preserves the same retrieval signal
          as the continuous and soft sparse translated references
        - but it becomes slower than both translated references
        - translated sparse-event claims therefore remain explicitly soft
      - `INC-0103` then closes bounded rerank recovery negative at confirm:
        - low-margin reranking does not recover translated quality on the
          fixed soft sparse translated point
        - the best rerank point only matches top-1 and trims amortized cost
          slightly
      - `INC-0104` then closes bounded backfill recovery negative on quality
        recovery but positive on lower-bank systems refinement:
        - bounded small-bucket backfill does not improve confirm-stage top-1
        - but `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
          becomes the best lower-bank sparse translated systems point
      - `INC-0105` then carries that systems point to the upper bank:
        - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
          improves candidate fraction and runtime over both routed references
        - the quality delta is negligible and slightly negative, so the point
          remains a systems lead rather than a quality lead
      - `INC-0106` then explains the new sparse translated systems leads:
        - lower-bank bounded backfill gain is search-dominated on average
        - upper-bank mean gain is real but mixes retrieval-search,
          route-query, and route-index effects
        - no hidden accounting regression surface remains
      - `INC-0107` then hardens the same comparisons seed by seed:
        - lower-bank backfill versus the continuous translated product
          reference is stable at `4/4` seed wins
        - upper-bank comparisons remain `3/4` mean-positive rather than
          seed-uniform
        - candidate-fraction reduction is stable across seeds
        - `route_query` changes sign across seeds, so it is not safe
          optimization guidance yet
      - `INC-0108` then repeats the same lower/upper packets on fresh warmed
        chart caches:
        - repeated wallclock timing still flips sign within seed on many
          comparisons
        - candidate-fraction reduction stays stable across every repeated
          comparison
        - the bounded backfill systems leads remain valid on mean, but
          microtiming remains too noisy for direct optimization guidance
      - `INC-0109` then converts that repeated evidence into a robust
        cost-reference read:
        - upper-bank bounded backfill remains a clean robust systems lead
          against both routed baselines
        - lower-bank bounded backfill remains robust against the continuous
          translated product reference
        - lower-bank bounded backfill versus the fixed soft sparse translated
          reference narrows to a pruning-first read rather than a clean robust
          wallclock promotion
      - `INC-0110` then hardens the dense-frontier claim on those same fixed
        sparse translated points:
        - lower-bank soft sparse versus dense narrows to a pruning-first read
        - lower-bank bounded backfill remains the only robust lower-bank dense
          systems promotion
        - both upper-bank sparse translated points remain robust dense systems
          promotions
        - every dense comparison still carries a robust top-1 deficit versus
          dense exact
      - `INC-0111` then resolves the dense quality/system frontier on those
        same fixed sparse translated points:
        - lower-bank soft sparse = pruning-only
        - lower-bank bounded backfill = systems-only
        - both upper-bank sparse translated points = quality-near systems
          promotions
      - `INC-0112` then hardens that upper-bank near-frontier read with two
        fresh paired repeats:
        - both fixed upper-bank sparse translated points remain
          quality-near systems promotions
        - the upper-bank top-1 gap stays small but robustly negative
      - `INC-0113` then decomposes that residual upper-bank dense top-1 gap:
        - the net dense advantage rate stays inside the `0.002` operational
          band on both fixed upper-bank sparse translated points
        - omission explains only about `1.2%-1.4%` of dense-only wins
        - dense-only wins are overwhelmingly present-but-not-top1 rather than
          candidate omission
        - the upper-bank dense gap is no longer worth a rescue queue
      - `INC-0114` then selects the single upper-bank dense-near carry-forward
        reference:
        - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000` is promoted as the
          explicit upper-bank dense-near routed reference
        - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000` drops to a
          supporting comparator
        - no further upper-bank dense rescue remains queued
      - `INC-0115` then freezes the broader-comparison carry-forward contract:
        - default broader packet uses the lower-bank systems-only routed point
          plus the promoted upper-bank dense-near routed reference
        - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500` becomes explicitly
          nondefault at the lower bank
        - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000` becomes
          explicitly comparator-only at the upper bank
      - `INC-0116` then turns that contract into a reusable packet manifest:
        - the default dual-anchor packet now exists as one reusable artifact
        - later branches can inherit exact route specs instead of rebuilding
          the packet from older configs by hand
      - `INC-0117` then turns that packet into the default broader sparse
        translated comparison read:
        - lower bank stays systems-only by default
        - upper bank stays quality-near systems promotion by default
        - the upper-bank bounded-backfill route remains optional comparator-only
      - `INC-0118` then carries the same packet and broader read onto the
        real-task side:
        - `docs/reports/REAL_TASK_COMPARISON.md` now has an explicit dual-anchor
          task-side extension contract
        - lower bank stays systems-only by default
        - upper bank stays quality-near systems promotion by default
      - `INC-0119` then converts that task-side extension into one explicit
        LM-proxy real-task comparison:
        - lower bank stays the systems-only default
        - upper bank stays the promoted quality-near systems default
        - the upper-bank bounded-backfill route remains optional comparator-only
      - `INC-0120` then freezes that explicit real-task comparison as the
        downstream carry-forward contract:
        - lower bank remains the systems-only downstream default
        - upper bank remains the promoted downstream real-task default
        - the upper-bank bounded-backfill route remains optional comparator-only
      - `INC-0121` then turns that downstream carry-forward contract into one
        reusable downstream packet manifest:
        - the downstream LM-proxy real-task packet now exists as one reusable
          artifact
        - later downstream branches can inherit exact route specs instead of
          reconstructing the packet from older comparison artifacts
      - `INC-0122` then carries that downstream packet manifest into one
        explicit downstream extension artifact:
        - lower bank remains the systems-only downstream default
        - upper bank remains the quality-near systems-promotion downstream
          default
        - optional comparator reintroduction is now explicit rather than
          implicit
      - `INC-0123` then turns that downstream extension into one explicit
        downstream LM-proxy real-task comparison:
        - lower bank remains the systems-only downstream default
        - upper bank remains the quality-near systems-promotion downstream
          default
        - the upper-bank bounded-backfill route remains optional comparator-only
      - `INC-0124` then freezes that explicit downstream comparison as the
        downstream carry-forward contract:
        - lower bank remains the systems-only downstream default
        - upper bank remains the promoted downstream real-task default
        - the upper-bank bounded-backfill route remains optional comparator-only
  - Kill: only if the restrained translated retrieval rescue surfaces
    (`INC-0070`, `INC-0071`, `INC-0072`) fail to convert the
    geometric/addressing signal into a useful systems tradeoff, or if the new
    systems win collapses under larger-load hardening.

## Current Transfer Frontier
- control baseline = `R0`
- `R0` remains shell-collapsed and health-failing on strict confirm
- operational routed lead under larger load = `HOPF_K25_BASE_IT40_P2_STATIC`
- hardware-efficiency routed lead under larger load = `HOPF_PHI2_BAND_IT40_P2_STATIC`
- translated retrieval control = `DENSE`
- translated retrieval amortized lead candidate = `HOPF_RET_P1_Q24`
  - candidate fraction about `0.3511`
  - beat matched dense on the 2-seed screen
  - lost on the 4-seed confirm
  - keep only as translated evaluation evidence, not an operational lead
- translated widened retrieval comparator = `HOPF_PHI2_RET_P1_Q24`
  - stronger pruning but slower amortized systems result
- historical static references = `HOPF_PHI2_BAND_IT48_P3_STATIC`, `HOPF_K25_BASE_IT60_P4_STATIC`
- dynamic quality reference under larger load = `HOPF_K25_BASE_IT60_P4`
- widened dynamic reference under larger load = `HOPF_PHI2_BAND_IT60_P4`
- historical widened reference = `HOPF_PHI2_BAND`
- widened routed-family comparator = `PHASE_K25_C035`
- coarse family reference = `PHI_PHI_PHI v1` (`PHI3_K25_D36_L065` / `PHILOG_D36_L065`)
- compression comparator = `PHI3_K20_D36_L065`
- historical continuous-phi comparator = `PHI_D32_L120`
- routed quality-first comparator = `PHI_D30_L120`
- fixed-controller comparator = `D30_FIXED_SG16`
- hybrid local-quality comparator = `HYB4_M2_T010_C005`
- corrected no-fiber-phase control = `HOPF_BASE_K25_PHI`
- corrected standalone phase-transport reference = `HOPF_TRANSPORT_L150`
- corrected coupled-field transport reference = `HOPF_CPX_TRANSPORT_L050_F100`
- explicit product phase-field screen lead = `H4XH4_FIELD_A150`
- explicit product stabilized candidate = `H4XH4_FIELD_A100`
- confirmed product phase-field lead = `H4XH4_FIELD_A150`
- confirmed product stabilized candidate = `H4XH4_FIELD_A100`
- near-hard sparse-event proxy reference = `H4XH4_FIELD_A150_EVT_T070_TAU002`
- soft sparse-event proxy reference = `H4XH4_FIELD_A150_EVT_T070`
- sparse-event translated lower-bank systems reference =
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
- sparse-event translated lower-bank balanced quality comparator =
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
- sparse-event translated lower-bank quality-first comparator =
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
- historical sparse-event translated lower-bank bounded-backfill point =
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- sparse-event translated upper-bank quality/reference point =
  `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- sparse-event translated upper-bank systems lead =
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- sparse-event translated route-coupled pilot read =
  - soft sparse + prune at threshold `0.02`: inert (`keep_frac=1.0`)
  - near-hard + prune at threshold `0.02`: downstream-live but non-promotable
    (`keep_frac≈0.745`, `cand_frac≈0.131`, `top1≈0.0212`)
- sparse-event translated route-coupled threshold-map read =
  - threshold `0.010` best preserves quality:
    `keep_frac≈0.992`, `cand_frac≈0.187`, `top1≈0.0448`
  - but `0.010` still regresses runtime versus uncoupled near-hard
  - thresholds `0.015-0.022` become progressively more selective and
    progressively less usable on quality
- sparse-event translated soft-bias carry-forward read =
  - soft score bias is confirm-stage downstream-live without deletion
  - `SBI030` is the balanced lower-bank quality lift:
    `top1=0.0464`, `amortized≈0.0942s`
  - `SBI080` is the quality-first lower-bank comparator:
    `top1=0.0524`, `amortized≈0.1416s`
  - uncoupled near-hard remains the lower-bank systems reference:
    `top1=0.0446`, `amortized≈0.0899s`
  - the old lower-bank bounded-backfill point does not hold on the focused
    prewarmed packet and now needs explicit reselection
- promoted upper-bank dense-near routed reference =
  `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- default broader-comparison lower-bank routed point =
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
- default broader-comparison lower-bank balanced quality comparator =
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
- default broader-comparison lower-bank quality-first comparator =
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
- historical lower-bank sparse translated comparator =
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- default broader-comparison upper-bank routed point =
  `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- comparator-only upper-bank sparse translated route =
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- active lower-bank contract refresh artifact =
  `results/analysis/inc0133_product_phase_sparse_event_translation_lower_bank_contract_refresh.json`
- default broader/task-side read =
  - lower bank: `systems-only`
  - upper bank: `quality-near systems promotion`
- refreshed explicit real-task lower-bank read =
  - default:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
    - `systems-only`
  - balanced comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
  - quality-first comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
- default explicit real-task recommendation =
  - lower bank: carry as systems-only default
  - upper bank: carry as promoted real-task default
- downstream real-task carry-forward =
  - lower bank: systems-only default
  - upper bank: promoted real-task default
- downstream real-task packet manifest =
  `results/analysis/inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.json`
- translated break-even quality-matched point = `H4XH4_FIELD_A150_Q16`
- translated break-even systems crossover point = `H4XH4_FIELD_A150_CPX8_Q16`
- translated stabilized break-even systems point = `H4XH4_FIELD_A150_CPX8_Q24`
- translated smaller-bank crossover point = `H4XH4_FIELD_A150_CPX8_Q24_T6000`
- translated earliest confirmed systems crossover point = `H4XH4_FIELD_A150_CPX8_Q04_T36000`
- translated highest-bank confirmed systems crossover point = `H4XH4_FIELD_A150_CPX8_Q08_T40000`
- translated threshold split explanation = static offline route-build cost
  composition on the fixed translated stack
- translated warm-cache rescued upper-bank crossover point =
  `H4XH4_FIELD_A150_CPX8_Q04_T40000`
- translated warm-cache single-query crossover point =
  `H4XH4_FIELD_A150_CPX8_Q01_T2500`
- translated warm-cache earliest any-repeat crossover point =
  `H4XH4_FIELD_A150_CPX8_Q01_T2500`
- translated warm-cache stabilized upper-bank systems point =
  `H4XH4_FIELD_A150_CPX8_Q08_T40000`
- translated chart-resident upper-bank systems point =
  `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`
- translated chart-resident lower-bank crossover point =
  `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`
- translated exact lower-bank full-warm floor =
  `FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500`
- translated route-only residency read = negative at both `T2500 Q01` and
  `T40000 Q01`

## Next Live Branches
1. keep `INC-0125` as the hardened sparse-event proxy trainability result:
   - near-hard proxy reference =
     `H4XH4_FIELD_A150_EVT_T070_TAU002`
   - soft sparse proxy comparator =
     `H4XH4_FIELD_A150_EVT_T070`
   - hard gate remains comparator-only =
     `H4XH4_FIELD_A150_HARD_T062`
2. treat `INC-0126` as closing the proxy/translation sparse-event gap
   positively/explanatorily:
   - translated near-hard preserves quality and candidate fraction
   - translated failure is systems-cost-only
   - primary driver = retrieval search
3. close `INC-0127` negative/explanatory:
   - translated soft sparse and translated near-hard differ only on
     `event_gate_tau`
   - sparse-event knobs are audit-only on the current translated surface
   - the old translated systems-cost rescue queue is therefore invalid
4. close `INC-0128` positive/explanatory:
   - translated sparse-event behavior is no longer audit-only once train-gate
     pruning is wired into the translated train bank
   - soft sparse at threshold `0.02` stayed inert
   - near-hard at threshold `0.02` pruned materially but collapsed top-1
5. close `INC-0129` negative/explanatory:
   - no train-gate-prune threshold window is promotable
   - tiny pruning preserves quality but does not improve runtime
   - useful pruning collapses translated quality
6. close `INC-0130` positive/explanatory:
   - soft score bias is the first non-omission translated sparse-event
     coupling that stays downstream-live
7. close `INC-0131` positive/explanatory:
   - uncoupled near-hard = lower-bank systems reference
   - `SBI030` = balanced lower-bank quality comparator
   - `SBI080` = quality-first lower-bank comparator
   - the old lower-bank bounded-backfill default now requires reselection
8. close `INC-0132` positive/explanatory:
   - uncoupled near-hard = explicit lower-bank default
   - `SBI030` = balanced lower-bank quality comparator
   - `SBI080` = quality-first lower-bank comparator
   - the old lower-bank bounded-backfill default is now stale historical-only
9. close `INC-0133` positive/explanatory:
   - apply the `INC-0132` lower-bank selection once across broader,
     task-side, and downstream contract surfaces
   - keep `TAU002` as the only default lower-bank route
   - keep `SBI030` and `SBI080` explicit but nondefault
   - keep the old lower-bank bounded-backfill route historical-only
10. close `INC-0134` positive/explanatory:
    - refreshed real-task comparison reaffirms `TAU002` as the lower-bank
      systems-only default
    - `SBI030` remains the balanced lower-bank quality comparator
    - `SBI080` remains the quality-first lower-bank comparator
    - upper-bank default remains unchanged
11. defer `INC-0135` as a supporting translated lower-bank frontier follow-up
12. move next to `INC-0136` on the unresolved measure-consistent
    `H^4` / Hopf route-law gate
13. keep bank, packet, cache, and event-threshold mapping frozen unless a later
    branch exposes a new dense-quality regression outside the completed
    operational-negligibility band
14. revisit explicit precomputed Poincare-ball route coordinates only if a later
    implementation branch proves route materialization is again the binding cost
