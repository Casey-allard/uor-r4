# EVIDENCE_SUMMARY

This document tracks what has been empirically explored so far.

## 1. Routing Experiments

Initial routing experiments explored geometric compatibility routing
versus dense attention baselines.

Observed: - structured routing behavior emerges - compatibility kernels
produce sparse activation

Unknown: - long-range stability - scaling behavior

## 2. Embedding Stability

Experiments evaluated embedding stability within curved spaces.

Observed: - hierarchical datasets embed naturally in negatively curved
space - local neighborhoods remain stable

Risks: - training instability at larger scales

## 3. Sparse Event Routing

Threshold routing experiments tested sparse activation behavior.

Observed: - strong sparsity patterns possible - compute savings in
prototype systems

Unknown: - training convergence - gradient propagation stability

## 4. Phase Transport

Preliminary experiments suggest phase alignment may encode useful
relationships between routed states.

Status: - exploratory - not yet proven necessary

Prime admissibility transport note:

- canonical framing:
  the primary project object is the exact recursive admissibility system
  summarized in `prime_research_summary.md`, with exact state `(b, phi, r)` and
  compressed predictive state `(b, spin_H)`
- canonical theorem-shaped objects:
  universal local-stencil factorization, recursive affine lift law, fiber
  evolution by direct CRT product, symbolic admissibility dynamics, and delayed
  visibility
- canonical implementation priority:
  first internal split horizon, first visible split horizon, and the lag between
  them should now be treated as the next preferred exact-layer target
- the exact finite-depth admissibility state is currently interpreted as a
  layered torus-valued phase-fiber state
- a compressed quotient in `C^2` appears to capture a reusable transport
  backbone across wheel scales
- a shared canonical law trained on `W = 2310, 30030, 510510` generalized to
  unseen `W = 9699690` with only small degradation
- the current conservative framing is `z_{t+1} ≈ A_* z_t + epsilon_t`, where
  `A_*` is the reusable backbone and `epsilon_t` is unresolved local residual
  structure
- this is evidence for reusable compressed transport structure, not evidence of
  exact closure or a prime oracle
- the next bounded question is whether per-layer phase fibers can be stored as
  small grouped complex packets whose composition recovers or explains part of
  the empirical `C^2` backbone
- SU-style language is being treated only as a candidate packet/composition
  formalism for these layer fibers, not as a proved exact symmetry of the full
  admissibility system
- second-stage grouped-packet recovery now tests a small controlled family of
  richer packet/composition maps against the fixed `C^2` target, with the
  outcome to be judged by within-scale recovery and cross-scale transfer rather
  than by expressiveness alone
- current result: richer adjacent-interaction packets slightly improve the
  baseline recovery error, but all tested grouped-packet families still recover
  the `C^2` quotient poorly in absolute terms and do not yet provide convincing
  cross-scale explanatory recovery
- corrective framing: the canonical source of truth is the recursive dynamical
  system summarized in
  `prime_research_summary.md`, where the primary objects are the exact affine
  lift, the exact `(b, phi, r)` state, and finite-horizon spin `spin_H`
- exact-state update: a bounded recursive-system experiment now measures
  finite-horizon predictive refinement directly at the `(b, spin_H)` layer,
  without any quotient target
- current exact result:
  for both `30030 -> 510510` and `510510 -> 9699690`, internal spin refinement
  is active from `H = 1`, while visible predictive-state count growth is
  delayed; for the `17`-layer lift the first visible split occurs at `H = 51`,
  and for the `19`-layer lift no visible predictive-state count increase was
  observed for `H <= 60`
- conservative update: delayed visibility is now directly supported at the
  exact recursive-system layer, and that exact layer should be treated as the
  stable source-of-truth object before any downstream quotient geometry is
  attempted
- threshold-law update:
  a compact exact threshold table across lifts `210 -> 2310 -> 30030 -> 510510
  -> 9699690` and tuplets `twins`, `triplet`, `quadruplet` shows that
  `first_internal_split_H = 1` in every tested case, while visible split ranges
  from immediate to strongly delayed
- currently supported pattern:
  visible lag grows sharply with denser tuplets in the tested lifts; for the
  quadruplet line the visible lag is `0, 20, 50`, and for the `19`-lift the
  visible split was not found for `H <= 60`
- unsupported simple laws:
  no small-lag rule is supported, and no law in the new prime `p` alone is
  supported by the current exact table
- visible-threshold predictor update:
  on the current exact table, the strongest tested coarse rank association is
  the mixed scale `p * |A|` (equivalently `p * nu_p(A)` for the tested rows),
  while `p` alone and `|A|` alone are both too weak to stand as a law
- currently plausible exact-layer law class:
  `H_visible_first` appears to depend on both lift prime and local stencil
  burden, with monotone growth in the tested table across both prime and tuplet
  density
- still unearned:
  no closed formula, no theorem in `p` alone, no theorem in `|A|` alone, and no
  claim that `p * |A|` is already the true law
- extension update:
  adding three size-4 collision tuplets that reduce `nu_p(A)` at targeted
  tested primes breaks the earlier `|A|` versus `nu_p(A)` degeneracy
- phase-fiber readability update:
  mapping the known first-splitting classes back into the exact phase-fiber
  chart shows that first splitters are not uniformly distributed in `(b, phi,
  r)`; simple concentration signals in dominant `phi` tuples and base phases are
  strong on the bounded matched-family table
- conservative exact-layer interpretation:
  first visible splitting is partially readable in phase-fiber-scale
  coordinates, but full spin-side first-splitting multiplicity remains the
  stronger exact predictor and no simple local phase-fiber rule is yet earned
- canonical exact-layer synthesis update:
  the current stable reading is that `j -> (b, phi, r)` is a real exact
  lower-complexity chart, while `(b, spin_H)` remains the stronger exact
  predictive state; visible first splitting therefore looks partially geometric
  and partially genuinely dynamical
- spin-minus-phase-fiber residual update:
  treating the question correctly as one of compressive predictive explanation
  rather than raw information subtraction, the strongest small exact dynamical
  residual tested so far is a previous/next return-gap pair
- current exact-layer interpretation:
  return-memory carries more of the residual predictive structure than short
  local suffix memory, but it still captures only part of the spin partition,
  especially on the deeper `30030 -> 510510` family
- consolidated exact-layer reading:
  the strongest current exact-layer picture is now stable: exact static
  phase-fiber-scale structure on `j -> (b, phi, r)`, stronger predictive
  compression in `(b, spin_H)`, canonical visible-threshold law class
  `density + first-splitting event`, and a residual beyond the static chart
  that currently looks more like return-memory than local bit suffix
- return-grammar compression update:
  bounded return-grammar candidates can capture most of the spin-side
  distinctions only by expanding the label space beyond spin itself; the
  residual therefore looks partially compressible, but not yet compressed into
  a small exact object
- current exact-layer classification:
  `(B) partially compressible but still significantly weaker than spin`
- minimal predictive state update:
  the smallest currently viable exact predictive state for practical use is the
  static chart context plus `next_return_gap`; it remains genuinely smaller
  than spin while capturing a substantial but incomplete part of the predictive
  partition
- current routing-state classification:
  `(B) useful but materially weaker than spin`
- routing abstraction update:
  the best current exact-layer routing abstraction is the minimal hybrid state
  `(b, phi, r, next_return_gap)`, which respects coarse chart-first routing and
  delayed refinement while remaining genuinely smaller than full spin
- prototype-spec update:
  the first implementation-facing exact-layer prototype should cache
  `(b, phi, r, next_return_gap)`, route coarsely by chart plus immediate
  return-memory, and promote only unresolved classes that continue collective
  first-splitting
- offline-eval update:
  the bounded offline harness supports `R_min` as a first prototype target:
  it keeps full membership discrimination, preserves about `0.75` of the
  spin-side capture, and remains substantially smaller than spin, with
  promotion still needed on about `31%` of split-partition cases
- mock-module update:
  the smallest implementation-facing step is now specified as an offline module
  that defaults to `R_min`, exposes deterministic state updates, and promotes
  selectively into `R_full` only for unresolved predictive classes
- mock-module implementation update:
  the first executable offline module now exists under the research tree and
  successfully demonstrates initialization, update, routing key generation,
  promotion check, and selective fallback to `R_full` on the bounded summary
  rows
- research-side benchmark validation update:
  across both the tiny guarded trial and the larger exact-row benchmark,
  `base_gap = (b, r, next_return_gap)` remains stable, route-decision
  instability stays at `0.0`, and fallback burden remains the main explicit
  guarded cost rather than a correctness problem
- angular head-to-head update:
  on the same larger guarded trace set, the existing canonical angular Hopf
  baseline is much more compact than `base_gap` but far too coarse
  predictively; `base_gap + fallback` reaches effective resolved fraction
  `0.9900` versus only `0.1681` for `angular_hopf`, while keeping fallback
  burden bounded and instability at `0.0`
- angular transfer update:
  adding delayed refinement, explicit fallback accounting, and predictive
  partitioning back into the unchanged angular Hopf path repairs its
  predictive-resolution failure, but only by driving fallback step fraction to
  `0.9883`; the remaining advantage of `base_gap` therefore survives the
  discipline transfer and now looks structural rather than merely procedural
- routing decision update:
  the current guarded research-side comparison is now frozen as a policy
  decision: `base_gap + fallback` is the leading routing candidate going
  forward, while canonical angular Hopf is retained as the comparison baseline
- guarded real-signal update:
  on the first replay-based MUDBench signal trial, `base_gap` retains zero
  fallback burden and full effective resolution, while guarded angular still
  needs materially more fallback (`0.5657` average); the main remaining gap is
  that current `base_gap` keying is too fine to show route reuse on the tiny
  real-signal traces
- real-signal reuse-recovery update:
  the zero-reuse issue is now traced mainly to exact base phase on very short
  replay traces rather than to exact `next_return_gap`; bucketing gap alone
  changes nothing, while the first tiny base-phase coarsening recovers reuse
  only by giving back the fallback advantage
- real-signal reuse decision update:
  the original `base_gap` policy remains the correct first real-signal policy
  for now, and the next fair reuse test should move to the paired
  `timing_mode_eval_v1` baseline replay slice rather than to more tiny
  coarsening variants
- larger real-signal trial update:
  the paired `timing_mode_eval_v1` replay slice doubles the replay budget but
  still does not produce natural route reuse for `base_gap`; however, the
  guarded prime-transport policy retains full effective resolution, zero
  instability, and zero fallback burden while guarded angular still needs
  materially more fallback
- final real-signal status update:
  the guarded real-signal result is now frozen: `base_gap` is the correct
  first real-signal policy because its validated win is fallback efficiency at
  full resolution, while natural route reuse remains unearned on the current
  replay-based boundary and should not be forced by more tiny coarsening work
- router-memory-layer update:
  the first structured read/write router-memory prototype is now implemented on
  bounded exact traces; after one warmup pass it achieves full read-hit rate,
  high effective resolution (`0.9808` aggregate), and zero instability, but
  exact branch retrieval remains partial on deeper lifts
- router-memory task-loop update:
  the first bounded stateful task loop now shows the same memory layer can
  carry symbolic state across repeated route-class visits with `1.0` query-hit
  rate on reused addresses and about `0.93` retrieval accuracy overall, while
  promotion burden on deeper lifts remains the main architectural cost
- promoted-query reduction update:
  a first one-key read-side refinement (`phi0` hint) does reduce promoted-query
  burden materially, but it also degrades task retrieval accuracy too much; the
  current router-memory read path should therefore remain unchanged
- router-memory status update:
  the router-memory branch is now frozen as architecturally viable but still
  costly: structured read/write behavior and high carried-state retrieval are
  validated, while deeper-family promoted-query burden remains the main real
  cost and should not be treated as already solved
- larger router-memory experiment update:
  the current unchanged router-memory architecture survives the first richer
  structured-record task family with exact record retrieval on reused
  addresses, zero instability, and the same explicit promoted-query cost; this
  is now strong enough to justify a larger router-native systems experiment
- multi-entity router-memory update:
  the unchanged router-memory architecture now supports bounded multi-entity
  state coordination with exact multi-record retrieval, exact coordinated
  snapshot reads, and zero instability; promoted-query burden remains the main
  explicit architectural cost rather than a coherence risk
- router-memory readiness update:
  the branch is now frozen as architecturally ready for a larger router-native
  systems prototype: coherence is validated across single-record, richer
  record, and multi-entity tasks, while deeper-family promoted-query burden
  remains the main unsolved efficiency cost
- updated discriminator result:
  after the extension, `|A|` alone is more clearly ruled out, `p * |A|` is no
  longer the leading coarse organizer, and `nu_p(A)` alone is also insufficient
- remaining plausible class:
  visible threshold appears to depend on lift prime plus finer local stencil
  arrangement information, not just on raw size or forbidden-count alone
- arrangement-statistics update:
  among the tested simple arrangement-sensitive features, `gap_max` and
  `gap_span` are the strongest, and both materially outperform forbidden-count
  alone
- current limit:
  no tested arrangement statistic beats the lift prime as the strongest coarse
  organizer, so arrangement currently looks like a secondary refinement factor
  rather than the main law by itself
- arrangement-isolation update:
  after fixing both `p` and `nu_p(A)` in small exact families, residual visible
  threshold variation remains real, so arrangement is now isolated as a genuine
  secondary driver
- current best isolated candidates:
  `gap_entropy` is the strongest residual correlator in the tiny fixed-family
  analysis, while `gap_max` remains the cleanest simple arrangement-sensitive
  explanatory candidate
- current limit:
  parent admissible density still outperforms the tested arrangement statistics
  on the fixed-family residual table, so no simple arrangement law is yet
  established
- interaction-statistics update:
  a first bounded family of parent-return / forbidden-stencil interaction
  statistics was tested, comparing parent return-gap residues mod `p` to the
  lift-prime forbidden set
- current result:
  none of the tested interaction proxies beats the current coarse organizers;
  the strongest interaction score reaches only Spearman `0.1741`, while
  `new_prime`, parent admissible density, and `gap_max` remain substantially
  stronger
- conservative reading:
  parent-grammar / new-stencil interaction is still a plausible law class, but
  the first global residue-alignment summaries are too weak to serve as the
  leading explanatory object
- next exact-layer direction:
  test which parent return-gap classes are split first under lift, rather than
  relying on globally averaged interaction scores
- first-splitting-event update:
  event-level statistics built from the pre-threshold predictive classes
  `(b, spin_{H_visible_first-1})` substantially outperform the earlier global
  interaction averages
- current exact result:
  `num_first_splitting_classes` reaches Spearman `0.8204` and
  `total_extra_child_classes_from_first_split` reaches `0.7982`, versus only
  `0.1741` for the best earlier global interaction summary
- current reading:
  visible threshold now looks more like a first-splitting-event object than a
  global residue-alignment object, but parent admissible density remains the
  strongest single control at Spearman `-0.8594`
- next exact-layer direction:
  condition on parent density and test whether first-splitting multiplicity
  still explains residual threshold variation
- density-conditioned update:
  after partitioning the current exact rows into three coarse parent-density
  bands, first-splitting-event multiplicity still carries strong residual
  signal
- current exact result:
  within-band residual Spearman scores are `0.8144` for
  `num_first_splitting_classes`, `0.7879` for
  `total_extra_child_classes_from_first_split`, and `0.7776` for
  `fraction_first_splitting_classes`
- current best exact-layer reading:
  visible threshold is now best described as a density-plus-first-splitting
  event law class, not a density-only law and not a global interaction-average
  law
- current limit:
  the conditioned bands are still small, so this narrows the law class but does
  not yet justify a closed formula
- tight density-matched update:
  on a single exact `2310 -> 30030` family with six size-4 tuplets all at
  exactly the same parent density `0.05454545454545454`, first-splitting-event
  statistics remain very strong while `gap_max` becomes weak
- matched-family result:
  `num_first_splitting_classes`, `total_extra_child_classes_from_first_split`,
  and `fraction_first_splitting_classes` all reach Spearman `0.9710`, while
  `gap_max` falls to `0.2538`
- current best exact-layer law class:
  `density + first-splitting event`
- current limit:
  this matched-family result does not rule out all future arrangement
  corrections, but it does not justify elevating `gap_max` to the leading
  correction term on the current exact evidence
- second matched-family update:
  a second exact density-matched slice on `30030 -> 510510` reproduces the same
  pattern more strongly
- replicated matched-family result:
  `num_first_splitting_classes`,
  `total_extra_child_classes_from_first_split`, and
  `fraction_first_splitting_classes` all achieve Spearman `1.0`, while
  `gap_max` is only `0.3162`
- current best exact-layer law class:
  robustly `density + first-splitting event` across two tightly matched lift
  families
- current limit:
  this is still a bounded family-by-family result, not a closed theorem for the
  full recursive system
- canonical threshold-law consolidation:
  the exact-layer threshold result is now consolidated into one durable note,
  with the best current law class stated explicitly as
  `density + first-splitting event`
- canonical reading:
  internal split is immediate, visible split is delayed, density alone is
  insufficient, simple burden/arrangement alone are insufficient, and
  arrangement is secondary on the current tightly matched evidence
- exact-layer synthesis update:
  the canonical stack and the canonical threshold-law result are now tied
  together in one compact synthesis note, including a minimal routing
  interpretation that remains downstream of the exact recursive system
- mock-router trace update:
  the offline mock router now matches exact per-position transport transitions
  on bounded traces, but literal `R_min` route keys are too fine-grained and
  behave like pointwise addresses rather than reusable routing classes
- grouping-key update:
  a bounded exact-layer comparison of coarser `R_min` routing partitions shows
  that `base_gap = (b, r, next_return_gap)` is the best current first routing
  key; it is much coarser than literal `R_min`, still smaller than the average
  full-spin partition, and triggers promotion at a meaningful intermediate rate
- base-gap prototype update:
  the first full offline prototype using stored `R_min`, `base_gap` routing,
  and selective fallback to `R_full` achieves near-complete post-promotion
  resolution (`0.9925`) while keeping promotion well below the pathological
  `gap_only` regime; this is now the best current offline routing prototype
- base-gap refinement update:
  adding the first phi coordinate to `base_gap` does cut promotion materially,
  but only by making the routing partition much finer; on current bounded
  evidence this is not a better enough tradeoff to replace `base_gap` as the
  first prototype target
- base-gap routing-loop update:
  when reused as a cached routing policy over the bounded traces, `base_gap`
  remains stable, reuses classes heavily, and reaches effective resolved
  fraction `0.9813` with zero route-decision instability; this is now enough
  to justify a guarded non-runtime integration experiment
- guarded adapter update:
  the `base_gap` policy is now packaged into a small research-side adapter
  boundary with initialize/step/route/promotion calls, and a tiny
  benchmark-like demo confirms that the same reuse/promotion pattern survives
  outside the pure evaluation scripts
- benchmark-wrapper update:
  the guarded adapter is now wrapped around the existing bounded trace-building
  path used by the research evaluations, yielding the first research-only
  benchmark boundary for the selected policy while preserving the same bounded
  reuse/promotion behavior
- guarded benchmark-harness update:
  on the same bounded traces, `base_gap` now sits in the intended middle
  position between the address-like static control and the overfallback-heavy
  `gap_only` control; this is enough to justify the next guarded
  benchmark-side integration step, but not live seam work
- fallback-burden update:
  promoted `base_gap` burden is moderately structured overall but becomes
  diffuse on the deeper lift family, so current evidence does not support one
  obvious cheap refinement before guarded benchmark-side integration
- guarded hookup-plan update:
  the smallest future benchmark-side hookup point is now specified explicitly
  as the existing research-only wrapper around the bounded trace-building path,
  with fixed policy inputs, fixed controls, and mandatory fallback-accounting
  metrics
- research-side readiness update:
  the current `base_gap` policy is now consolidated as ready for a research-side
  integration trial through the existing wrapper boundary, with fallback burden
  retained as the main remaining guarded risk
- research-side trial update:
  the smallest guarded research-side integration trial has now been executed
  through the existing wrapper boundary and reproduces the prior guarded
  benchmark result without drift
- larger research-side benchmark update:
  on the next larger reproducible exact-row trace set, `base_gap` remains
  stable, stays between the two controls, and shows lower promotion burden
  than in the smaller guarded trial
- router-memory workspace update:
  the unchanged router-memory architecture now maintains bounded workspace-style
  local and shared state coherently, with exact local/shared retrieval,
  exact joint snapshots, and `0.0` instability; promoted querying remains the
  main explicit architectural cost rather than a correctness problem
- router-memory agent-loop update:
  the unchanged workspace memory remains exact and stable under a first bounded
  control loop, but action correctness is only moderate; the branch has now
  separated memory-substrate validation from control-policy quality
- router-memory context-packet update:
  a minimal structured decision packet improves bounded control quality
  materially without changing route reuse, promoted-query burden, or zero-
  instability memory behavior
- router-memory packet-control update:
  a stronger bounded ranking policy improves control quality again on the same
  unchanged packet surface, confirming that the next remaining headroom is in
  controller quality rather than memory or routing structure
- attractor-identity update:
  a minimal controller-side convergence anchor improves bounded control quality
  materially without hurting retrieval or stability, but it does not reduce the
  underlying promoted-query burden
- router-memory control-stack decision:
  the current best bounded stack is now fixed as the unchanged router-memory
  substrate plus the unchanged packet surface plus attractor identity as the
  primary controller surface
- router-memory coordination update:
  the current best stack remains coherent on the first harder four-entity
  reassignment and dependency loop, with exact retrieval and zero instability;
  the main remaining challenge is full-loop coordination quality, not substrate
  coherence
- router-native demo-package update:
  the current branch is now frozen into an internal demo checkpoint with one
  overview note, one manifest, the current best stack, the canonical notes and
  drivers, the open costs, and the next experiment fixed for continued work
- coordination-policy improvement update:
  a controller-only conflict-resolution pass on the unchanged packet plus
  attractor stack improves reassignment / handoff quality to exact and nudges
  full-loop coordination quality upward, while retrieval, route reuse,
  promoted-query burden, and instability all remain unchanged
- coordination-lookahead update:
  a small one-step replanning controller on the same unchanged stack preserves
  exact retrieval and zero instability but does not improve aggregate
  coordination quality over the improved local policy, so this branch should
  not spend more time on tiny lookahead variants of the same action surface
- coordination-conflict-arbitration update:
  an explicit shared conflict / dependency arbitration controller on the same
  unchanged stack also fails to beat the improved local policy and matches the
  rejected lookahead aggregate, while retrieval, promoted-query burden, route
  reuse, and instability remain fixed
- coordination-frame update:
  a small explicit coordination-episode layer on the same unchanged stack is
  the first controller variant beyond the improved local policy that nudges
  the harder full-loop coordination metric upward, while all substrate-side
  metrics remain fixed
- geometry-native-controller update:
  a first explicit math-native controller signal derived from routing geometry
  preserves the best coordination-frame result exactly, but does not improve it
  further, so architecture math is now at parity rather than yet ahead
- phase-fiber-geometry update:
  a richer phase-fiber-aware basin tag derived from exact routing geometry
  preserves retrieval and stability but degrades coordination quality
  materially, showing that stronger geometry exposure can oversteer the
  controller if introduced too directly
- geometry-native-sequence-model update:
  on a bounded phase-fiber-controlled sequence task where token meaning depends
  on the evolving exact routing state, a geometry-native model with explicit
  geometric transport plus routed memory and a tiny learned readout reaches
  `0.592881917953` test accuracy and `0.885010242462` query accuracy, beating
  a tiny causal transformer baseline at `0.430338531733` and
  `0.503764569759` respectively while using far fewer parameters
- geometry-native-sequence-model-v2 update:
  on a stronger bounded compositional stack-rewrite task that is less directly
  aligned to the phase-fiber variables, the geometry-native model reaches
  exact bounded-task accuracy `1.0` and exact query accuracy `1.0`, while the
  tiny transformer baseline reaches `0.790161132812` and `0.770186364651`,
  strengthening the thesis in the bounded experimental sense
- geometry-native-sequence-model-v3 update:
  on a more language-like bounded discourse/reference task with overloaded
  tokens whose interpretation depends on evolving latent role state, the
  geometry-native model reaches `0.998567700386` test accuracy and
  `0.991875946522` query accuracy, while the tiny transformer baseline reaches
  `0.695442736149` and `0.679468214512`, further strengthening the thesis in
  the bounded contextual-sequence setting
- geometry-native-sequence-model-v4 update:
  on the first held-out structural-shift test inside the discourse-style
  family, where training excludes one explicit speaker-reference query mode and
  test restores it, the geometry-native model keeps `0.996438443661` held-out
  test accuracy and `0.979522168636` held-out query accuracy, while the tiny
  transformer baseline drops to `0.620404422283` and `0.453924924135`,
  strengthening the thesis beyond same-distribution bounded competence
- geometry-native-sequence-model-v5 update:
  on the first multi-axis held-out generalization test inside the discourse
  family, where training excludes the removed speaker-reference query mode,
  keeps shorter contexts, and limits style shifts while test restores all of
  them together, the geometry-native model keeps `0.996558785439` held-out
  test accuracy and `0.985419213772` held-out query accuracy, while the tiny
  transformer baseline drops to `0.559709846973` and `0.469015806913`
- geometry-native-sequence-model-v6 update:
  on the first cross-family transfer test, where training uses the original
  discourse/reference family and test uses a related delegation-style family
  with changed latent role semantics, the geometry-native model reaches
  `0.994750976562` transfer accuracy and `0.971504330635` transfer query
  accuracy, while the tiny transformer baseline reaches `0.634033203125` and
  `0.661365151405`
- geometry-native-sequence-model-v7 update:
  on the first reduced-schema-alignment transfer test, where the target family
  is projected back into the old schema only through a lossy proxy, the prior
  large geometry-native advantage collapses to near parity overall
  (`0.479204952717` vs `0.48046875`), though the geometry-native model still
  retains a small edge on the transfer query metric
- geometry-native-chart-realignment update:
  a first small geometry-native chart realignment rule recovers a meaningful
  part of the v7 reduced-alignment loss, lifting transfer accuracy from
  `0.479204952717` to `0.568933844566` and transfer query accuracy from
  `0.544108569622` to `0.624918460846`, and reopens a clear gap over the tiny
  transformer baseline, though it does not restore the clean shared-schema v6
  regime
- geometry-native-multichart update:
  a bounded three-chart selector using internal predictive coherence remains
  better than the v7 collapse and better than the tiny transformer baseline,
  but it does not beat the simpler v8 single-rule chart realignment, so v8
  remains the best reduced-alignment recovery result so far
- geometry-native-chart-calibration update:
  a short prefix-calibrated chart selector still improves on the unrecovered
  v7 reduced-alignment transfer case and still beats the tiny transformer
  baseline, but it does not beat the simpler v8 single-rule chart realignment
  and therefore does not strengthen the adaptive-geometry result beyond v8
- geometry-native-divisibility-bridge update:
  a bounded arithmetic transition layer built from prime-coded factors,
  semiprime bridge states, and small divisibility hubs materially outperforms
  the earlier chart-based recovery line, lifting reduced-alignment transfer
  accuracy to `0.883501827717` and transfer query accuracy to
  `0.969032287598`, which strongly supports the need for explicit transition
  infrastructure rather than chart selection alone
- geometry-native-divisibility-bridge-v12 update:
  under a stronger reduced-alignment mismatch, the divisibility bridge weakens
  substantially to `0.552269339561` transfer accuracy and
  `0.584205031395` transfer query accuracy; it still beats the unrecovered v7
  case and the tiny transformer baseline, but it no longer beats v8, so the
  transition idea remains promising while fixed bridge robustness remains open
- geometry-native-bridge-calibration update:
  a short support-window bridge-family selector recovers part of the v12 loss,
  improving stronger-mismatch transfer accuracy to `0.593377947807` and
  transfer query accuracy to `0.633928596973`; it still remains well below the
  v11 peak, but it now beats both v12 and the earlier v8 chart recovery on the
  stronger setting
- geometry-native-bridge-switching update:
  allowing one bounded mid-sequence bridge re-choice improves the stronger-
  mismatch line again to `0.606119811535` transfer accuracy and
  `0.660416662693` transfer query accuracy, which supports local temporal
  bridge recalibration over sequence-level calibration alone, though the result
  still remains well below the v11 peak
- geometry-native-event-switch update:
  a one-shot event-triggered bridge switch based on local coherence drop still
  beats the fixed v12 bridge and the tiny transformer baseline, but it does
  not beat the simpler fixed midpoint switch from v14, so the switch-trigger
  problem remains open
- geometry-native-gcd-bridge-revision update:
  a bounded factor-overlap bridge revision using GCD-style retained-versus-
  replaced factors remains competitive and beats the fixed v12 bridge and the
  tiny transformer baseline, but it does not beat the current best v14
  midpoint switch, so factor-level revision remains promising without yet
  becoming the strongest recovery mechanism
- geometry-native-conflict-revision update:
  a query/binding-conflict-triggered factor revision improves on weaker
  adaptive variants such as the generic event-triggered switch and the fixed
  v12 bridge, but it still does not beat the current best v14 midpoint switch,
  so structural contradiction looks useful without yet becoming the decisive
  trigger
- geometry-native-microrepair update:
  a very small conflict-centered repair window remains better than the fixed
  v12 bridge and the tiny transformer baseline, but it weakens relative to the
  broader factor-revision line and does not beat the current best v14 midpoint
  switch, suggesting the broken region is larger than a tiny local cluster
- geometry-native-mesorepair update:
  a moderate contradiction-centered repair region improves on the micro-window
  variant but still does not beat the current best v14 midpoint switch, so
  repair scale alone does not explain the remaining stronger-mismatch gap
- geometry-native-grownrepair update:
  contradiction-grown regional repair becomes the strongest factor-repair
  variant so far at `0.604437589645` transfer accuracy, but it still does not
  beat the current best v14 midpoint switch and remains notably below it on
  query accuracy, suggesting handcrafted boundary discovery is helping but not
  sufficient
- geometry-native-schema-induction update:
  a bounded regional schema selector becomes the first semi-learned stronger-
  mismatch mechanism to beat the handcrafted repair line and set the best
  query accuracy so far at `0.686405777931`, but it still misses the v14
  midpoint switch slightly on overall accuracy, suggesting light schema
  induction helps more than more repair heuristics
- geometry-native-hybrid-regime update:
  a bounded two-region hybrid that combines coarse regional segmentation with
  local schema induction becomes the first stronger-mismatch mechanism to beat
  both the v14 midpoint-switch accuracy leader and the v21 query-accuracy
  leader, strongly supporting adaptive regional geometry over either idea
  alone
- geometry-native-hybrid-boundary update:
  a lightly induced split boundary improves the v22 hybrid slightly on both
  overall accuracy and query accuracy, suggesting the remaining gains now come
  from better regime placement more than from richer local schema logic
- geometry-native-hybrid-boundary-v24 update:
  a contradiction-aware boundary score improves the hybrid line again on
  overall accuracy, but it gives back some query accuracy relative to v23,
  suggesting the architecture is nearing its bounded optimum and the
  remaining question is objective tradeoff in boundary placement
- geometry-native-hybrid-boundary-v25 update:
  a bounded Pareto / two-objective boundary scorer fails to improve the hybrid
  tradeoff and lands below both v23 and v24, which is strong evidence that
  the current two-region hybrid is near its bounded local optimum on this
  family
- geometry-native-regime-field update:
  a very small learned regional boundary/regime field beats the handcrafted
  hybrid family on both the best-overall and best-query reference points,
  showing that learned regional structure can add real value while leaving the
  geometry-native computation core intact
- geometry-native-regime-field-v27 update:
  the learned regional-field hybrid retains a clear advantage over the tiny
  transformer under a stronger shifted family, but the gain weakens sharply
  relative to v26, showing that the architecture direction survives while the
  current tiny field is not yet robust cross-family
- geometry-native-regime-field-v28 update:
  a slightly richer learned regional field recovers a bit of the v27 overall
  accuracy loss but does not recover query accuracy, suggesting the next
  bottleneck is field structure rather than just a small increase in scorer
  capacity
- geometry-native-regime-field-v29 update:
  a shallow structured field over ordered split candidates improves on both
  v27 and v28 on the shifted family, which supports field topology as a more
  important lever than scorer width alone
- geometry-native-regime-field-v30 update:
  an explicit contiguous chart field improves query accuracy again on the
  shifted family but gives back overall accuracy relative to v29, suggesting
  that more explicit regional chart structure is helpful but not yet enough to
  recover the lost cross-family margin
- geometry-native-regime-field-v31 update:
  a small global learned chart field on the shifted family fails to improve on
  the stronger recent regional/structured variants, suggesting broader scope
  alone is not sufficient without a better global field design
- geometry-native-regime-field-v32 update:
  a bounded multiscale chart field recovers some overall accuracy relative to
  the weaker global-field variant, but it still does not beat the best
  shifted-family structured-field results, suggesting this field branch may be
  nearing its bounded limit on the shifted family
- geometry-native-regime-field-v33 update:
  a sparse region-interaction field does not beat the better shifted-family
  regional/structured field results, suggesting the missing piece is not
  captured by this small routing-style coordination fabric

## 5. Hardware Implications

Prototype results suggest possible compute savings if sparse routing
scales.

However: - routing overhead must remain small - memory locality must be
preserved

------------------------------------------------------------------------

Current evidence is promising but insufficient to confirm the central
hypothesis.
