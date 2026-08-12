# Current Direction

Mirror note:
- This file is a human-readable mirror, not the canonical live queue.
- Authoritative queue docs:
  - `docs/research/ACTIVE_STATE.md`
  - `docs/research/KILL_LIST_TRACKER.md`

## Queue Reset
- 2026-03-12 root-theory audit completed.
  - audited against:
    - `CORE_PROJECT_GOALS.md`
    - `docs/PROJECT_CONTEXT.md`
    - `geometric_routing_kill_tests.md`
    - `NEXT_CRITICAL_EXPERIMENTS.md`
    - `EVIDENCE_SUMMARY.md`
  - reading:
    - `INC-0130` through `INC-0134` are real late-stage translated
      sparse-event evidence
    - they do not close the earlier unresolved geometry gate
    - `RR-061` remains the main open kill-list question:
      derive a measure-consistent `H^4` / Hopf route law
  - queue decision:
    - defer `INC-0135` as a supporting lower-bank frontier follow-up
    - queue `INC-0136` next:
      `docs/research/increments/INC_0136_measure_consistent_h4_hopf_route_return.md`
    - keep translated sparse-event/dual-anchor results as downstream evidence,
      not as the main proof branch

## Latest Update
- `INC-0136` completed negative/explanatory on 2026-03-12.
  - geometry-return artifacts:
    - `results/analysis/inc0136_measure_consistent_h4_hopf_route_return_screen.json`
    - `docs/governance/gates/gate_20260312_213408.md`
    - `docs/reports/INC0136_MEASURE_CONSISTENT_H4_HOPF_ROUTE_RETURN_SCREEN.md`
  - bounded read:
    - direct geodesic shell substitution with Hopf-base sectors did **not**
      improve the route law
    - `HOPF_BASE_BALL_K25_PHI` failed the health gate with:
      - `shell_pmax=0.8862`
      - `shell_mass_l1=1.7724`
      - `knn_overlap=0.6362`
      - `route_H_r_corr≈0`
    - the healthy no-fiber control `HOPF_BASE_K25_PHI` remained stronger on
      shell concentration, shell-mass error, and neighborhood preservation
  - queue decision:
    - close `INC-0136` negative/explanatory
    - keep `RR-061` open
    - move next to `INC-0137`:
      `docs/research/increments/INC_0137_measure_consistent_h4_hopf_shell_pressure_blend.md`
    - do **not** return to lower-bank translated frontier work as the main
      queue
- `INC-0134` completed positive/explanatory on 2026-03-12.
  - refreshed comparison artifacts:
    - `results/analysis/inc0134_product_phase_sparse_event_translation_dual_anchor_real_task_refresh_comparison.json`
    - `docs/reports/INC0134_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_DUAL_ANCHOR_REAL_TASK_REFRESH_COMPARISON.md`
  - refreshed real-task comparison now makes the lower-bank split explicit
    from measured evidence rather than stale inheritance:
    - default systems route:
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
      - `top1 delta vs dense = -0.0074`
      - `amortized = 0.0899s`
    - balanced quality comparator:
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
      - `top1 delta vs default = +0.0018`
      - `amortized delta vs default = +0.0043s`
    - quality-first comparator:
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
      - `top1 delta vs dense = +0.0004`
      - `amortized delta vs dense = +0.0158s`
    - unchanged upper-bank default:
      `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - this closes the refreshed real-task comparison
  - the later queue reset now treats the lower-bank frontier as supporting
    evidence and moves next to `INC-0136` on the unresolved geometry gate
- `INC-0133` completed positive/explanatory on 2026-03-12.
  - contract-refresh artifacts:
    - `results/analysis/inc0133_product_phase_sparse_event_translation_lower_bank_contract_refresh.json`
    - `docs/reports/INC0133_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_LOWER_BANK_CONTRACT_REFRESH.md`
  - the lower-bank sparse-event translated contract is now inherited
    consistently across broader, task-side, real-task, and downstream
    surfaces:
    - default lower-bank route:
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
    - explicit lower-bank comparators:
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
      and
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
    - historical-only lower-bank comparator:
      `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - unchanged upper-bank default:
      `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - this closes lower-bank contract refresh and queues one refreshed
    real-task comparison branch (`INC-0134`) instead of more inheritance-only
    packaging
- `INC-0132` completed positive/explanatory on 2026-03-12.
  - reference-selection artifacts:
    - `results/analysis/inc0132_product_phase_sparse_event_translation_lower_bank_reference_reselection.json`
    - `docs/reports/INC0132_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_LOWER_BANK_REFERENCE_RESELECTION.md`
  - lower-bank sparse-event translated selection is now explicit:
    - default systems reference:
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
    - balanced quality comparator:
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
    - quality-first comparator:
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
    - stale historical comparator:
      `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  - the old lower-bank bounded-backfill default is now explicitly demoted
    because its focused amortized time inflated by `18.91x` versus its older
    confirm packet
  - this closed lower-bank reference selection and fed the lower-bank
    contract refresh (`INC-0133`)
- `INC-0131` completed positive/explanatory on 2026-03-12.
  - carry-forward artifacts:
    - `results/analysis/inc0131_product_phase_sparse_event_translation_soft_bias_carry_forward_screen.json`
    - `results/analysis/inc0131_product_phase_sparse_event_translation_soft_bias_carry_forward_confirm.json`
    - `docs/reports/INC0131_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_SOFT_BIAS_CARRY_FORWARD_CONFIRM.md`
  - prewarmed confirm split the lower-bank sparse-event translated surface
    cleanly:
    - systems reference:
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
      - `top1=0.0446`
      - `cand_frac=0.193328`
      - `amortized=0.0899s`
    - balanced quality comparator:
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
      - `top1=0.0464`
      - `cand_frac=0.193328`
      - `amortized=0.0942s`
    - quality-first comparator:
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
      - `top1=0.0524`
      - `cand_frac=0.193328`
      - `amortized=0.1416s`
  - the old lower-bank bounded-backfill point did not hold as the active
    default on the focused prewarmed packet
  - this closes the soft-bias carry-forward branch and queues lower-bank
    reference reselection (`INC-0132`)
- `INC-0130` completed positive/explanatory on 2026-03-12.
  - soft-bias screen artifacts:
    - `results/analysis/inc0130_product_phase_sparse_event_translation_route_coupled_soft_bias_screen.json`
    - `docs/reports/INC0130_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_ROUTE_COUPLED_SOFT_BIAS_SCREEN.md`
  - soft score bias is genuinely downstream-live without item deletion:
    - `event_gate_retrieval_surface_active_mean=1.0`
    - `cand_frac` unchanged on every soft-bias point
  - `SBI030` screened as the balanced lower-bank quality lift
  - `SBI080` screened as the quality-first point
  - the branch therefore reopened lower-bank sparse-event translation on
    reordering rather than pruning
- `INC-0129` completed negative/explanatory on 2026-03-12.
  - threshold-map artifacts:
    - `results/analysis/inc0129_product_phase_sparse_event_translation_route_coupled_threshold_map_screen.json`
    - `docs/reports/INC0129_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_ROUTE_COUPLED_THRESHOLD_MAP_SCREEN.md`
  - the train-gate-prune coupling is genuinely downstream-live
  - threshold `0.010` preserves quality best:
    - `keep_frac=0.992`
    - `top1=0.0448`
    - `cand_frac=0.187042`
  - but `0.010` still regresses online and amortized runtime versus uncoupled
    near-hard
  - stronger thresholds reduce candidate fraction further but collapse top-1
  - this closes train-gate pruning as a viable translated sparse-event
    carry-forward surface and queues a softer route-coupled pilot
    (`INC-0130`)
- `INC-0128` completed positive/explanatory on 2026-03-12.
  - route-coupled screen artifacts:
    - `results/analysis/inc0128_product_phase_sparse_event_translation_route_coupled_screen.json`
    - `docs/reports/INC0128_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_ROUTE_COUPLED_SCREEN.md`
  - a minimal translated sparse-event coupling is now real:
    - `event_gate_translation_coupling=train_gate_prune`
  - soft sparse at threshold `0.02` stayed inert in practice:
    - `event_gate_translation_keep_frac=1.0`
    - top-1 and candidate fraction stayed unchanged
  - near-hard at threshold `0.02` became selectively downstream-live:
    - keep fraction fell to about `0.745`
    - candidate fraction fell to about `0.131`
    - online time fell to about `0.131s`
  - the same near-hard coupled point is not promotable:
    - top-1 collapsed to `0.0212`
  - this closes the “audit-only” objection to translated sparse-event work
    and queues a threshold-map branch (`INC-0129`)
- `INC-0127` completed negative/explanatory on 2026-03-12.
  - rescue-feasibility artifacts:
    - `results/analysis/inc0127_product_phase_sparse_event_translation_systems_cost_rescue.json`
    - `docs/reports/INC0127_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_SYSTEMS_COST_RESCUE.md`
  - translated soft sparse and translated near-hard differ only on
    `event_gate_tau`
  - current translated retrieval treats sparse-event knobs as audit-only:
    - `event_gate_retrieval_surface_active = 0`
  - the observed translated timing gap therefore does not define a selective
    rescue surface on the fixed lower-bank translated branch
  - this closes translated systems-cost rescue and queues a route-coupled
    sparse-event translated pilot (`INC-0128`)
- `INC-0126` completed positive/explanatory on 2026-03-12.
  - the proxy/translated near-hard gap is now explained from fixed evidence
  - audit artifacts:
    - `results/analysis/inc0126_product_phase_sparse_event_proxy_translation_gap_audit.json`
    - `docs/reports/INC0126_PRODUCT_PHASE_SPARSE_EVENT_PROXY_TRANSLATION_GAP_AUDIT.md`
  - translated near-hard preserves quality and candidate fraction:
    - top-1 delta vs translated soft sparse = `0.000000`
    - candidate-fraction delta vs translated soft sparse = `0.000000`
  - the translated failure is systems-cost-only:
    - online delta vs translated soft sparse = `+0.098602s`
    - amortized delta vs translated soft sparse = `+0.117689s`
    - primary driver = `retrieval_search_sec`
  - omission and ordering loss are not supported by the audit
  - this closes the proxy/translation gap audit and queues translated
    systems-cost rescue (`INC-0127`)
- `INC-0123` completed positive/explanatory on 2026-03-12.
  - the explicit downstream LM-proxy dual-anchor real-task comparison is now
    fixed from the completed downstream extension artifact
  - downstream comparison artifact:
    `results/analysis/inc0123_product_phase_sparse_translation_dual_anchor_real_task_downstream_comparison.json`
  - downstream default recommendation:
    - lower bank = `systems-only` via
      `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - upper bank = `quality-near systems promotion` via
      `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - the upper-bank bounded-backfill route remains optional comparator-only for
    downstream real-task branches
  - this closes the downstream real-task comparison branch and queues the
    downstream carry-forward branch (`INC-0124`)
- `INC-0122` completed positive/explanatory on 2026-03-12.
  - the downstream LM-proxy dual-anchor real-task inheritance is now explicit
    from the exact packet manifest
  - downstream extension artifact:
    `results/analysis/inc0122_product_phase_sparse_translation_dual_anchor_real_task_downstream_extension.json`
  - lower-bank downstream default remains `systems-only` via
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  - upper-bank downstream default remains `quality-near systems promotion` via
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - this closes the downstream real-task extension branch and queues the next
    downstream comparison branch (`INC-0123`)
- `INC-0121` completed positive/explanatory on 2026-03-12.
  - the downstream LM-proxy dual-anchor real-task packet is now one reusable
    manifest with exact inherited route specs
  - downstream packet manifest:
    `results/analysis/inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.json`
  - default downstream route ids remain frozen:
    - `DENSE_Q01_T2500`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `DENSE_Q01_T40000`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - this closes the downstream real-task packet-manifest branch and queues the
    next downstream extension branch (`INC-0122`)
- `INC-0120` completed positive/explanatory on 2026-03-12.
  - the explicit LM-proxy dual-anchor real-task comparison now has one fixed
    downstream carry-forward contract
  - carry-forward artifact:
    `results/analysis/inc0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.json`
  - downstream default recommendation:
    - lower bank = `systems-only` via
      `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - upper bank = `quality-near systems promotion` via
      `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - the upper-bank bounded-backfill route remains optional comparator-only for
    downstream real-task branches
  - this closes the downstream real-task carry-forward branch and queues the
    packet-manifest branch (`INC-0121`)
- `INC-0119` completed positive/explanatory on 2026-03-12.
  - the first explicit LM-proxy real-task comparison is now fixed from the
    completed dual-anchor packet and task-side extension
  - real-task comparison artifact:
    `results/analysis/inc0119_product_phase_sparse_translation_dual_anchor_real_task_comparison.json`
  - default real-task recommendation:
    - lower bank = `systems-only` via
      `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - upper bank = `quality-near systems promotion` via
      `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - the upper-bank bounded-backfill route remains optional comparator-only on
    the real-task side
  - this closes the explicit real-task comparison branch and queues the
    downstream carry-forward branch (`INC-0120`)
- `INC-0118` completed positive/explanatory on 2026-03-12.
  - the completed `INC-0116` packet and `INC-0117` broader sparse translated
    read now extend directly onto the real-task side
  - task-side extension artifact:
    `results/analysis/inc0118_product_phase_sparse_translation_dual_anchor_task_side_extension.json`
  - default task-side read:
    - lower bank = `systems-only` via
      `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - upper bank = `quality-near systems promotion` via
      `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - the upper-bank bounded-backfill route remains optional comparator-only on
    the task side
  - this closes the dual-anchor task-side extension branch and queues the next
    explicit real-task comparison branch (`INC-0119`)
- `INC-0117` completed positive/explanatory on 2026-03-12.
  - the `INC-0116` packet now has one explicit broader sparse translated read
  - lower bank stays systems-only by default via
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  - upper bank stays quality-near systems promotion by default via
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - the upper-bank bounded-backfill route remains optional comparator-only
  - this closes the first dual-anchor broader comparison branch
- `INC-0116` completed positive/explanatory on 2026-03-12.
  - the `INC-0115` carry-forward contract now exists as a reusable packet
    manifest with exact inherited route specs
  - packet manifest:
    `configs/packet_inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison.json`
  - default route ids remain frozen:
    - `DENSE_Q01_T2500`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `DENSE_Q01_T40000`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - this closes the dual-anchor broader-comparison packet branch
    positive/explanatory and queues the first broader comparison branch
    (`INC-0117`)
- `INC-0115` completed positive/explanatory on 2026-03-12.
  - the promoted upper-bank reference from `INC-0114` is now frozen as the
    sole upper-bank routed representative in later broader comparisons
  - default broader-comparison packet:
    - `DENSE_Q01_T2500`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `DENSE_Q01_T40000`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - the lower-bank soft sparse point is now nondefault:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
  - the upper-bank bounded-backfill point is now comparator-only:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
  - this closes promoted upper-bank carry-forward positive/explanatory and
    queues the dual-anchor broader-comparison packet (`INC-0116`)
- `INC-0114` completed positive/explanatory on 2026-03-12.
  - the completed `INC-0112` and `INC-0113` evidence now collapses the
    upper-bank sparse translated pair to a single promoted dense-near routed
    reference
  - promoted route:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - supporting comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
  - the pair delta stays inside the configured carry-forward tolerance band:
    - top-1 `+0.000038`
    - candidate fraction `+0.001761`
    - amortized `-0.043834s`
  - selection mode is `tie_break_within_tolerance`, not a broad new rescue
    branch
  - this closes upper-bank dense reference selection positive/explanatory and
    queues promoted upper-bank carry-forward (`INC-0115`)
- `INC-0113` completed positive/explanatory on 2026-03-12.
  - the fixed upper-bank sparse translated dense packet now has a query-level
    gap decomposition
  - both upper-bank sparse translated points remain large systems wins versus
    dense exact:
    - soft sparse `top1=0.047325`, `cand_frac=0.183764`,
      `amortized=3.426262s`
    - bounded backfill `top1=0.0472875`, `cand_frac=0.182003`,
      `amortized=3.470096s`
  - the residual upper-bank dense top-1 gap is now operationally negligible:
    - soft sparse mean net dense advantage rate `0.001525`
    - bounded backfill mean net dense advantage rate `0.001562`
    - both sit inside the `0.002000` operational-negligibility band
  - dense-only wins are overwhelmingly in-candidate rather than omission-led:
    - omission share within dense-only wins is only about `1.2%-1.4%`
    - present-but-not-top1 explains about `98.6%-98.8%` of dense-only wins
  - this closes upper-bank dense gap decomposition positive/explanatory and
    queues upper-bank dense reference selection/carry-forward (`INC-0114`)
- `INC-0112` completed positive/explanatory on 2026-03-12.
  - the upper-bank dense quality-tolerance read survives two fresh paired
    confirm repeats
  - both fixed upper-bank sparse translated points remain
    quality-near systems promotions under the completed upper-bank-only
    hardening
  - upper-bank robust top-1 gaps remain small:
    - soft sparse max abs gap `0.001478`
    - bounded backfill max abs gap `0.001511`
  - upper-bank robust amortized gains remain large and stable:
    - soft sparse median `-7.207634s`
    - bounded backfill median `-7.129120s`
  - this closes upper-bank dense quality-tolerance hardening
    positive/explanatory and queues residual upper-bank dense gap
    decomposition (`INC-0113`)
- `INC-0111` completed positive/explanatory on 2026-03-12.
  - the dense quality/system frontier is now explicit on the fixed sparse
    translated points
  - lower-bank soft sparse versus dense remains `pruning-only`
  - lower-bank bounded backfill remains `systems-only`
  - both upper-bank sparse translated points now classify as
    quality-near systems promotions under the robust dense summaries
  - the upper-bank robust top-1 gap is small enough to sit inside the
    completed dense-quality tolerance read:
    - upper soft sparse max abs gap `0.001440`
    - upper bounded backfill max abs gap `0.001470`
  - the lower-bank robust top-1 gap remains materially larger:
    - lower soft sparse max abs gap `0.007360`
    - lower bounded backfill max abs gap `0.007440`
  - this closes dense quality-frontier accounting positive/explanatory and
    queues focused upper-bank dense quality-tolerance hardening (`INC-0112`)
- `INC-0110` completed positive/explanatory on 2026-03-12.
  - the dense-frontier systems claim now survives repeated robust hardening
  - lower-bank soft sparse versus dense narrows to a pruning-first read:
    - amortized median `-0.004194s`
    - amortized trimmed mean `+0.003878s`
    - top-1 median `-0.007200`
  - lower-bank bounded backfill remains the only robust lower-bank dense
    systems promotion:
    - amortized median `-0.000506s`
    - amortized trimmed mean `-0.014058s`
    - top-1 median `-0.006800`
  - both upper-bank sparse translated points remain robust dense systems
    promotions, but both keep a small robust top-1 deficit:
    - upper soft sparse top-1 median `-0.001100`
    - upper bounded backfill top-1 median `-0.001100`
  - retrieval-search pruning remains the dominant win mechanism, while
    route-index build and route-query remain robust penalties
  - this closes dense robust hardening positive/explanatory and queues dense
    quality-frontier accounting (`INC-0111`)
- `INC-0109` completed positive/explanatory on 2026-03-12.
  - the repeated sparse translated evidence now has a stable robust cost
    reference
  - upper-bank bounded backfill remains a clean robust systems lead versus
    both routed baselines
  - lower-bank bounded backfill remains robust versus the continuous
    translated product reference
  - lower-bank bounded backfill versus the fixed soft sparse translated
    reference is now narrowed to a pruning-first read:
    - amortized median `+0.003406s`
    - amortized trimmed mean `-0.020222s`
    - candidate-fraction median `-0.003160`
    - candidate-count median `-7.8996`
  - stable candidate-fraction and candidate-count reduction are now the clean
    robust systems signals on the sparse translated branch
  - this closes robust cost referencing positive/explanatory and queues robust
    dense-frontier hardening (`INC-0110`)
- `INC-0108` completed positive/explanatory on 2026-03-12.
  - repeated timing hardening did not recover a clean seed-stable wallclock
    story for the bounded backfill sparse translated systems leads
  - lower and upper comparisons still flip sign within the same seed across
    repeats
  - candidate-fraction reduction remains stable across all repeated
    comparisons
  - the bounded backfill points remain valid mean systems leads, but
    microtiming is still too noisy for direct optimization guidance
  - this closes repeated timing hardening positive/explanatory and queues a
    robust cost-reference audit (`INC-0109`)
- `INC-0107` completed positive/explanatory on 2026-03-12.
  - the bounded backfill sparse translated systems leads remain valid on mean
    at both anchors, but the component mix is not seed-uniform
  - lower-bank bounded backfill versus the continuous translated product
    reference is stable:
    - `4/4` seed amortized wins
    - stable candidate-fraction and retrieval-search improvements
  - upper-bank bounded backfill versus both routed references is seed-fragile:
    - `3/4` seed wins, `1/4` seed loss
    - candidate-fraction improvement stays stable
    - `route_query` changes sign across seeds and should not be used as direct
      optimization guidance
  - this closes sparse translated component stability positive/explanatory and
    queues repeated timing hardening (`INC-0108`)
- `INC-0106` completed positive/explanatory on 2026-03-12.
  - the new sparse translated systems leads remain positive against dense at
    the lower and upper anchors
  - lower-bank bounded backfill gain is search-dominated on average
  - upper-bank gain is real on mean but mixes search, route-query, and
    route-index effects
  - no hidden accounting regression surfaced
- `INC-0105` confirm completed positive/narrow on 2026-03-12.
  - the bounded sparse translated systems point from `INC-0104` survives the
    upper-bank anchor at `T40000 Q01`
  - promoted upper-bank sparse translated systems lead:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
    - `top1=0.0472875`
    - `cand_frac=0.182003`
    - `online=3.11690s`
    - `amortized=3.47015s`
  - fixed upper-bank sparse translated quality/reference point remains:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - `top1=0.047325`
    - `cand_frac=0.183764`
    - `online=3.22611s`
    - `amortized=3.53342s`
  - this closes upper-bank carry-forward positive/narrow and queues sparse
    translated systems cost decomposition (`INC-0106`)
- `INC-0104` confirm completed negative on quality recovery and positive/narrow
  on systems refinement on 2026-03-12.
  - bounded small-bucket backfill did not improve translated quality on
    confirm
  - but it did create a materially better lower-bank sparse translated systems
    point:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
    - `top1=0.0444`
    - `cand_frac=0.189366`
    - `online=0.07954s`
    - `amortized=0.10572s`
  - fixed lower-bank sparse translated quality/reference point remains:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `online=0.12081s`
    - `amortized=0.15271s`
  - this closes bounded backfill negative on quality recovery, but promotes a
    new sparse translated systems lead at the lower bank
- `INC-0103` confirm completed negative on 2026-03-12.
  - bounded low-margin reranking did not recover translated quality on top of
    the fixed soft sparse translated point
  - best rerank point `R050` only matched top-1 and shaved a tiny amount of
    amortized cost
  - this kills reranking as the next sparse translated quality-recovery path
- `INC-0102` screen completed negative on 2026-03-12.
  - the confirmed near-hard proxy point did not survive as a translated
    systems improvement
  - translated near-hard candidate
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500` preserved the same
    routed retrieval signal as the continuous and soft sparse references:
    - `top1=0.0444`
    - `cand_frac=0.189016`
  - but it gave back the translated systems story:
    - `online=0.21561s`
    - `amortized=0.26344s`
    - `event_gate_mean=0.02097`
  - the translated sparse-event story therefore remains explicitly soft
  - this closes near-hard translated carry-forward negative and queues bounded
    quality recovery on the fixed soft sparse translated point (`INC-0103`)
- `INC-0101` confirm completed positive/narrow on 2026-03-12.
  - the fixed product route law now supports a stable near-hard proxy event
    point
  - promoted point `H4XH4_FIELD_A150_EVT_T070_TAU002` held across 4 seeds:
    - `mse=0.0038642`
    - `total_sec=6.040`
    - `shell_pmax=0.5702`
    - `event_gate_mean=0.02055`
    - `event_gate_active_frac=0.0`
  - the true hard controller `H4XH4_FIELD_A150_HARD_T062` also stayed
    healthy, but remained mostly on:
    - `mse=0.0039054`
    - `total_sec=6.169`
    - `event_gate_mean=0.8439`
    - `event_gate_active_frac=0.8439`
  - this branch therefore closes positive on near-hard event activation, not
    on genuinely sparse binary hard firing
  - this queues translated carry-forward of the near-hard point (`INC-0102`)
- `INC-0100` confirm completed positive/narrow on 2026-03-12.
  - the fixed sparse-event controller now carries through the translated
    chart-resident retrieval stack
  - promoted point `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500` held
    across 4 seeds:
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `online=0.11774s`
    - `amortized=0.16861s`
    - `event_gate_mean=0.3191`
  - the continuous translated product reference kept the same retrieval signal
    but ran much slower:
    - `online=0.27591s`
    - `amortized=0.33801s`
  - the sparse-event point reaches an almost exact lower-bank `Q01`
    amortized tie with dense exact:
    - routed `0.16861s`
    - dense `0.16866s`
  - the branch stays soft-sparse, not hard-firing:
    - `event_gate_active_frac=0.0`
  - this closes translated sparse-event carry-forward positive and queues hard
    event activation on the proxy harness (`INC-0101`)
- `INC-0099` confirm completed positive/narrow on 2026-03-12.
  - the fixed product route law now has a healthy soft-sparse proxy point
  - promoted point `H4XH4_FIELD_A150_EVT_T070` held across 4 seeds:
    - `mse=0.0038966`
    - `total_sec=6.558`
    - `shell_pmax=0.5702`
    - `event_gate_mean=0.3191`
    - `event_gate_cost_proxy=0.3191`
  - the continuous product reference also stayed healthy:
    - `mse=0.0039004`
    - `total_sec=7.427`
    - `shell_pmax=0.5702`
  - the sparse win is soft-sparse, not hard-firing:
    - `event_gate_active_frac=0.0`
    - the meaningful event signal is the soft update-mass proxy, not the hard
      `0.5` activity cutoff
  - this closes sparse event proxy trainability positive and queues translated
    carry-forward of the fixed sparse point (`INC-0100`)
- `INC-0098` completed positive/explanatory on 2026-03-12.
  - the fixed chart-resident translated stack is already positive against
    dense at both hardware-side anchors:
    - lower bank `T2500 Q01`: `0.0807s` versus dense `0.1155s`
    - upper bank `T40000 Q01`: `2.4952s` versus dense `9.2545s`
  - lower-bank chart vs full-warm gap is small and split across components:
    - amortized delta: `+0.0245s`
    - route-index build: `+0.0138s`
    - retrieval search: `+0.0098s`
    - route query: `+0.0014s`
  - upper-bank chart vs full-warm gap is larger but still not a pure
    route-materialization story:
    - amortized delta: `+0.4767s`
    - retrieval search: `+0.2748s`
    - route-index build: `+0.2045s`
  - there is no hidden residual/offline accounting surface left to explain
  - translated systems optimization is therefore no longer the highest-value
    branch
  - this freezes the translated chart-resident stack as the current
    hardware-side reference and moves the queue back to sparse event-driven
    proxy trainability (`INC-0099`)
- `INC-0097` screen completed negative on 2026-03-12.
  - the fixed product-phase sparse shell pilot is mechanism-live but not
    health-passing
  - `H4XH4_FIELD_A150` remains the only healthy product route:
    - `mse=0.003899`
    - `total_sec=7.956`
    - `shell_pmax=0.5662`
  - gated candidate `H4XH4_FIELD_A150_G030` changes only about `6.4%` of
    routed points, but still collapses shell balance:
    - `shell_pmax=0.9846`
    - `mse=0.003915`
    - `total_sec=7.127`
  - banded candidate `H4XH4_FIELD_A150_B035` is similarly sparse but collapses
    one seed to a single shell:
    - `eval_shells=1.5`
    - `shell_pmax=0.9860`
    - `mse=0.003908`
    - `total_sec=7.463`
  - no sparse / shared-state shell candidate beat the continuous product
    reference on health plus runtime/quality tradeoff
  - this closes the shell pilot negative and moves the queue back to
    translated route-cost decomposition (`INC-0098`)
- `INC-0096` confirm completed positive/explanatory on 2026-03-12.
  - the fixed translated chart-resident lower-bank single-query claim now
    survives packet composition hardening
  - focused packet at `T2500 Q01`:
    - dense `0.1155s`
    - routed `0.0807s`
    - margin `+0.0348s`
  - mixed packet at `T2500 Q01`:
    - dense `0.0946s`
    - routed `0.0871s`
    - margin `+0.0075s`
  - routed retrieval signal stays unchanged across packet shapes:
    - `top1=0.046300`
    - `cand_frac=0.198723`
  - packet scope changes the size of the systems margin, but not the sign
  - the lower-bank chart-resident single-query story is now stable below the
    ideal fully warm case
  - this clears the deferred gate to reopen sparse / quantized phase-gated
    shell work
- `INC-0095` confirm completed positive/explanatory on 2026-03-12.
  - the fixed translated product stack now has a focused chart-resident
    single-query lower-bank result on the same translated route law
  - `T2500/T2750/T3000/T4000` all cross at chart-resident `Q01` on the 4-seed
    confirm
  - search work stays pinned near `19.2%-19.3%` of dense on the full lower
    slice
  - cache state remains exact for the routed branch:
    - `chart_cache_hit=1.0`
    - `route_cache_hit=0.0`
  - that branch broadened the single-query chart-resident claim to the lower
    anchor and forced the packet-scope audit
- `INC-0094` confirm completed positive/narrow on 2026-03-12.
  - the fixed translated product stack now has a confirmed chart-resident /
    route-ephemeral repeat map on the two anchor banks
  - `T2500` confirm means:
    - `Q01`: chart-only misses slightly at `amortized=0.1241s`
      versus dense `0.1041s`
    - `Q02`: chart-only crosses at `0.0964s` versus dense `0.1049s`
    - `Q04`: chart-only stays positive at `0.0543s` versus dense `0.0896s`
  - `T40000` confirm means:
    - `Q01`: chart-only stays strongly positive at `2.4952s` versus dense
      `9.2545s`
    - `Q02`: `2.1109s` versus `9.3027s`
    - `Q04`: `1.9548s` versus `9.0216s`
  - chart-persistent sessions now survive:
    - lower bank by `Q02`
    - upper bank already by `Q01`
- `INC-0093` confirm completed positive/explanatory on 2026-03-12.
  - the fixed translated product stack now has an explicit cache-residency
    decomposition on the same two anchor operating points
  - `T2500 Q01` confirm means:
    - dense: `top1=0.052000`, `amortized=0.1035s`
    - chart-only warm: `top1=0.044600`, `cand_frac=0.193328`,
      `amortized=0.1326s`
    - route-only warm: `0.044600`, `0.193328`, `2.5345s`
    - full warm: `0.044600`, `0.193328`, `0.0562s`
  - `T40000 Q01` confirm means:
    - dense: `top1=0.048850`, `amortized=9.5814s`
    - chart-only warm: `top1=0.047325`, `cand_frac=0.183764`,
      `amortized=2.3856s`
    - route-only warm: `0.047325`, `0.183764`, `34.7146s`
    - full warm: `0.047325`, `0.183764`, `2.0185s`
  - chart residency carries almost all of the operational rescue
  - route-only residency does not rescue either anchor
  - the exact lower-bank `T2500 Q01` floor remains a full-warm claim
- `INC-0092` confirm completed positive/explanatory on 2026-03-12.
  - the fixed translated product stack no longer supports the old exact
    `T2500` miss / `T2501` hit story under hardening
  - dense confirm means:
    - `T2500 Q01`: `top1=0.050300`, `amortized=0.1078s`
    - `T2501 Q01`: `0.050300`, `0.1225s`
  - routed confirm means:
    - `T2500 Q01`: `top1=0.046300`, `cand_frac=0.198723`, `amortized=0.0741s`
    - `T2501 Q01`: `0.046300`, `0.198731`, `0.0638s`
  - both banks now hold at `Q01` on the expanded seed schedule
  - the lower-bank warm-cache floor therefore collapses to `T2500`
  - all routed warm runs hit both caches:
    - `chart_cache_hit=1.0`
    - `route_cache_hit=1.0`
- `INC-0091` confirm completed positive/narrow on 2026-03-12.
  - the fixed translated product stack now has an exact tracked confirmed
    warm-cache single-query crossover at `T2501`
  - dense confirm means:
    - `T2501 Q01`: `top1=0.051800`, `amortized=0.1364s`
    - `T2502 Q01`: `0.052158`, `0.1212s`
    - `T2503 Q01`: `0.051958`, `0.1126s`
    - `T2504 Q01`: `0.051917`, `0.1324s`
  - routed confirm means:
    - `T2501 Q01`: `top1=0.044600`, `cand_frac=0.193338`, `amortized=0.0779s`
    - `T2502 Q01`: `0.044764`, `0.193503`, `0.0827s`
    - `T2503 Q01`: `0.044764`, `0.193513`, `0.0644s`
    - `T2504 Q01`: `0.044728`, `0.193565`, `0.0592s`
  - all four tracked banks now hold at `Q01`
  - the screen-only `T2503 Q02` pocket disappeared on confirm
  - all routed warm runs hit both caches:
    - `chart_cache_hit=1.0`
    - `route_cache_hit=1.0`
- `INC-0090` confirm completed positive/narrow on 2026-03-12.
  - the fixed translated product stack now has an earlier tracked confirmed
    warm-cache single-query crossover at `T2505`
  - dense confirm means:
    - `T2505 Q01`: `top1=0.052117`, `amortized=0.1291s`
    - `T2510 Q01`: `0.051594`, `0.0834s`
    - `T2515 Q01`: `0.051909`, `0.0890s`
    - `T2520 Q01`: `0.051587`, `0.1010s`
  - routed confirm means:
    - `T2505 Q01`: `top1=0.044728`, `cand_frac=0.193544`, `amortized=0.0579s`
    - `T2510 Q01`: `0.044422`, `0.193187`, `0.0544s`
    - `T2515 Q01`: `0.044749`, `0.192989`, `0.0559s`
    - `T2520 Q01`: `0.044841`, `0.193024`, `0.0581s`
  - all four tracked banks now hold at `Q01`
  - all routed warm runs hit both caches:
    - `chart_cache_hit=1.0`
    - `route_cache_hit=1.0`
- `INC-0089` confirm completed positive/narrow on 2026-03-12.
  - the fixed translated product stack now has an earlier tracked confirmed
    warm-cache single-query crossover at `T2525`
  - dense confirm means:
    - `T2525 Q01`: `top1=0.051902`, `amortized=0.1787s`
    - `T2550 Q01`: `0.051373`, `0.1225s`
    - `T2575 Q01`: `0.051476`, `0.1304s`
  - routed confirm means:
    - `T2525 Q01`: `top1=0.045166`, `cand_frac=0.193195`, `amortized=0.0643s`
    - `T2550 Q01`: `0.044902`, `0.192997`, `0.0563s`
    - `T2575 Q01`: `0.044872`, `0.192759`, `0.0544s`
  - all three tracked banks now hold at `Q01`
  - all routed warm runs hit both caches:
    - `chart_cache_hit=1.0`
    - `route_cache_hit=1.0`
- `INC-0088` completed positive/explanatory on 2026-03-12.
  - the local `T2600/T2650/T2700` split is a systems-cost effect, not a
    routing-law change
  - `T2600` crosses because search gain still beats the fixed route-query plus
    residual offline penalty
  - `T2650` misses because dense search time dips locally while routed
    route-query cost stays almost unchanged
  - `T2700` crosses again because dense search time rises back up while search
    work stays pinned near `0.193`
- `INC-0087` confirm completed positive/narrow on 2026-03-12.
  - the fixed translated product stack now has an earlier tracked confirmed
    warm-cache single-query crossover at `T2600`
  - dense confirm means:
    - `T2600 Q01`: `top1=0.051923`, `amortized=0.1180s`
    - `T2650 Q01`: `0.051132`, `0.0970s`
    - `T2700 Q01`: `0.051852`, `0.1217s`
  - routed confirm means:
    - `T2600 Q01`: `top1=0.045577`, `cand_frac=0.193587`, `amortized=0.1159s`
    - `T2650 Q01`: `0.044528`, `0.193916`, `0.1143s`
    - `T2650 Q02`: `0.044528`, `0.193916`, `0.0910s`
    - `T2700 Q01`: `0.044259`, `0.193139`, `0.0850s`
  - `T2600` and `T2700` cross at `Q01`, while `T2650` falls back to `Q02`
  - all routed warm runs hit both caches:
    - `chart_cache_hit=1.0`
    - `route_cache_hit=1.0`
- `INC-0086` confirm completed positive/narrow on 2026-03-12.
  - the fixed translated product stack now has an earlier tracked confirmed
    warm-cache single-query crossover at `T2750`
  - dense confirm means:
    - `T2500 Q01`: `top1=0.052000`, `amortized=0.1103s`
    - `T2750 Q01`: `0.050182`, `0.1084s`
    - `T3000 Q01`: `0.049833`, `0.1319s`
  - routed confirm means:
    - `T2500 Q01`: `top1=0.044600`, `cand_frac=0.193328`, `amortized=0.1403s`
    - `T2500 Q02`: `0.044600`, `0.193328`, `0.0911s`
    - `T2750 Q01`: `0.044545`, `0.192894`, `0.0999s`
    - `T3000 Q01`: `0.044833`, `0.191704`, `0.0726s`
  - `T2500` misses at `Q01` but already crosses at `Q02`
  - all routed warm runs hit both caches:
    - `chart_cache_hit=1.0`
    - `route_cache_hit=1.0`
- `INC-0085` confirm completed positive/narrow on 2026-03-12.
  - the fixed translated product stack now has an earliest tracked confirmed
    warm-cache single-query crossover at `T3000`
  - dense confirm means:
    - `T3000 Q01`: `top1=0.049833`, `amortized=0.1592s`
    - `T4500 Q01`: `0.045889`, `0.2826s`
    - `T6000 Q01`: `0.048667`, `0.3956s`
  - routed confirm means:
    - `T3000 Q01`: `top1=0.044833`, `cand_frac=0.191704`, `amortized=0.0744s`
    - `T4500 Q01`: `0.047333`, `0.193020`, `0.1877s`
    - `T6000 Q01`: `0.047083`, `0.187229`, `0.2047s`
  - `Q02` also crossed at every tracked bank
  - all routed warm runs hit both caches:
    - `chart_cache_hit=1.0`
    - `route_cache_hit=1.0`
- `INC-0083` confirm completed positive/narrow on 2026-03-12.
  - persistent chart/train-route cache reuse now rescues the fixed translated
    upper-bank stack without changing routed quality or candidate fraction
  - `Q04 T40000` flips from a cold-start miss to a strong warm-cache crossover:
    - cold routed amortized `10.389s`
    - warm routed amortized `1.972s`
    - warm dense amortized `9.246s`
    - routed cache hits: `chart=1.0`, `route=1.0`
  - `Q08 T40000` also strengthens under warm cache:
    - cold routed amortized `6.165s`
    - warm routed amortized `1.891s`
    - warm dense amortized `9.113s`
    - routed cache hits: `chart=1.0`, `route=1.0`
  - the routed signal itself stays unchanged under reuse:
    - `Q04 T40000`: `top1=0.047325`, `cand_frac=0.183764`
    - `Q08 T40000`: `top1=0.047325`, `cand_frac=0.183764`
- `INC-0082` completed positive/explanatory on 2026-03-12.
  - the fixed translated product stack now has a direct explanation for the
    non-monotone `Q04` threshold
  - `Q04 T36000` crosses because online savings beat the offline penalty:
    - online gain per repeat `9.862s`
    - offline penalty per repeat `7.407s`
    - amortized margin `+2.455s`
  - `Q04 T40000` misses because the offline penalty stays larger:
    - online gain per repeat `7.072s`
    - offline penalty per repeat `8.038s`
    - amortized margin `-0.966s`
  - `Q08 T40000` still crosses because the same static offline cost amortizes
    across more repeats:
    - online gain per repeat `7.657s`
    - offline penalty per repeat `4.121s`
    - amortized margin `+3.536s`
  - the routing/search signal itself stays stable:
    - `T36000 Q04`: search-work ratio `0.190206`
    - `T40000 Q04`: search-work ratio `0.183764`
    - bytes-saved proxy stays near `81%`
- `INC-0081` confirm completed positive/narrow on 2026-03-12.
  - the fixed product branch now has a first real confirmed `Q04` systems
    crossover point
  - the upper-bank threshold is not monotone:
    - `max_train=36000`: first systems crossover at `Q04`,
      search-work ratio `0.190206`
    - `max_train=40000`: first systems crossover at `Q08`,
      search-work ratio `0.183764`
  - `H4XH4_FIELD_A150_CPX8_Q04_T36000` is now the earliest confirmed systems
    crossover point:
    - `top1=0.04707`
    - `cand_frac=0.19021`
    - `amortized=9.694s`
  - `H4XH4_FIELD_A150_CPX8_Q08_T40000` is now the highest-bank confirmed
    systems crossover point:
    - `top1=0.04733`
    - `cand_frac=0.18376`
    - `amortized=6.106s`
- `INC-0081` screen completed positive on 2026-03-11.
  - the 2-seed threshold search already showed the same mixed structure:
    - `T36000`: `Q04` crossed
    - `T40000`: `Q04` did not cross, but `Q08` still did
  - that justified a confirm on the same `Q04/Q08` bracket rather than another
    bank sweep
- `INC-0080` confirm completed positive/narrow on 2026-03-11.
  - the fixed product branch held its `Q08` systems crossover through a second
    upper-bank extension
  - `H4XH4_FIELD_A150_CPX8_Q08_T30000` became the highest-bank confirmed
    systems crossover point before `INC-0081` extended the search to `Q04`
- `INC-0079` confirm completed positive/narrow on 2026-03-11.
  - the fixed product branch now has a first confirmed upper-bank boundary
    extension
  - the secondary-key search-work ratio still stays essentially fixed near
    `0.19` while the onset keeps moving earlier with bank size:
    - `max_train=3000`: no crossover through `Q24`
    - `max_train=6000`: first systems crossover at `Q24`
    - `max_train=12000`: first systems crossover at `Q12`
    - `max_train=18000`: first systems crossover at `Q08`
  - `H4XH4_FIELD_A150_CPX8_Q08_T18000` became the earliest confirmed systems
    crossover point before `INC-0080` tested whether `Q04` would appear
- `INC-0078` confirm completed positive/narrow on 2026-03-11.
  - the fixed product branch established the first explicit confirmed crossover
    boundary
  - the secondary-key search-work ratio stays essentially fixed near `0.19`
    while the onset moves earlier with bank size:
    - `max_train=3000`: no crossover through `Q24`
    - `max_train=6000`: first systems crossover at `Q24`
    - `max_train=12000`: first systems crossover at `Q12`
  - `H4XH4_FIELD_A150_CPX8_Q12_T12000` became the first upper-bank crossover
    point before `INC-0079` pushed the onset earlier again
- `INC-0078` screen completed positive on 2026-03-11.
  - the 2-seed map already showed the same coherent boundary:
    - `T3000`: no crossover through `Q24`
    - `T6000`: onset at `Q24`
    - `T12000`: onset already at `Q12`
  - that justified a narrowed confirm on the actual boundary bracket
- `INC-0077` confirm completed positive/narrow on 2026-03-11.
  - the fixed product branch survives the first explicit hardware-cost profile
  - the search-work ratios remain stable across bank size:
    - secondary-key branch near `19%` of dense candidate scan work
    - plain product branch near `31%`
  - the crossover is scale-dependent rather than dead:
    - at `max_train=12000`, crossover begins at `Q16`
    - at `max_train=6000`, crossover survives only at `Q24`
  - `H4XH4_FIELD_A150_CPX8_Q24_T6000` is now the first confirmed smaller-bank
    crossover point:
    - `top1=0.04708`
    - `cand_frac=0.18723`
    - `amortized=0.351s`
- `INC-0077` screen completed positive on 2026-03-11.
  - the smaller-bank screen already showed the same slope:
    - no crossover at `Q16`
    - crossover at `Q24`
    - stable work ratios
  - that justified a confirm on the exact same smaller-bank bracket
- `INC-0076` confirm completed positive/narrow on 2026-03-11.
  - the fixed product branch now has a confirmed amortized crossover against
    dense exact retrieval on the translated pipeline
  - `H4XH4_FIELD_A150_Q16` matched dense top-1 while beating dense amortized
    cost:
    - `top1=0.04912`
    - `amortized=1.163s`
    - versus `DENSE_Q16` at `1.322s`
  - `H4XH4_FIELD_A150_CPX8_Q16` became the first stronger-pruning systems
    crossover:
    - `top1=0.04867`
    - `cand_frac=0.19032`
    - `amortized=1.036s`
  - `H4XH4_FIELD_A150_CPX8_Q24` remains the stabilized dense-frontier systems
    point:
    - `top1=0.04867`
    - `cand_frac=0.19032`
    - `amortized=0.831s`
- `INC-0076` screen completed positive on 2026-03-11.
  - the crossover region screened cleanly:
    - no routed crossover at `Q08`
    - both routed points crossed dense by `Q16`
    - the gap widened again at `Q24` and `Q32`
  - that justified a focused `Q08/Q16/Q24` confirm rather than another rescue
    surface
- `INC-0075` confirm completed negative on 2026-03-11.
  - bounded rerank quality rescue did not move the dense-frontier result
  - `H4XH4_FIELD_A150` remained the quality-matched routed point
  - `H4XH4_FIELD_A150_CPX8` remained the dense-frontier systems lead
  - the next honest question therefore became amortized break-even, not another
    retrieval tweak
- `INC-0075` screen completed positive on 2026-03-11.
  - the rerank variants were not useful, but the unchanged frontier trio was
    still strong enough to justify a confirm close-out on the exact same law
- `INC-0074` confirm completed positive/narrow on 2026-03-11.
  - the fixed product phase-field branch is now directly positive against
    dense exact retrieval on the translated pipeline
  - `H4XH4_FIELD_A150_CPX8` confirmed:
    - `top1=0.04867`
    - `cand_frac=0.19032`
    - `online=0.3904s`
    - `amortized=0.8308s`
  - versus `DENSE_Q24`:
    - top-1 delta `-0.00046`
    - candidate-fraction delta `-0.80968`
    - online delta `-0.9491s`
    - amortized delta `-0.5087s`
  - the dense-frontier systems win therefore holds, but the systems lead gives
    back a very small amount of top-1
- `INC-0074` screen completed positive on 2026-03-11.
  - the 2-seed screen was even stronger:
    - `H4XH4_FIELD_A150_CPX8` beat dense on top-1 as well as on candidate
      fraction and online/amortized cost
  - that justified a 4-seed confirm on the exact same fixed law
- `INC-0073` confirm completed positive/narrow on 2026-03-11.
  - the fixed `INC-0071` translated-addressing law stayed system-positive on
    the larger translated load
  - `H4XH4_FIELD_A150_CPX8` confirmed:
    - `top1=0.04867`
    - `cand_frac=0.19032`
    - `online=0.3804s`
    - `amortized=0.7949s`
  - versus `HOPF_K25_BASE_PHI`:
    - top-1 delta `-0.00238`
    - candidate-fraction delta `-0.14646`
    - online delta `-0.2486s`
    - amortized delta `-0.2732s`
  - the larger-load hardening therefore passed on pruning/runtime, but not on
    total translated quality leadership
- `INC-0073` screen completed positive on 2026-03-11.
  - the 2-seed larger-load screen already showed the same narrower read:
    - `H4XH4_FIELD_A150_CPX8` kept a major candidate-fraction/runtime edge
    - the small-load top-1 lead had already disappeared
  - the screen still justified a 4-seed confirm because the systems signal was
    clearly alive on the harder load
- `INC-0072` confirm completed positive on 2026-03-11.
  - the systems-only rescue on the fixed product secondary-key branch held on
    4 seeds
  - `H4XH4_FIELD_A150_CPX8` confirmed:
    - `top1=0.04708`
    - `cand_frac=0.18723`
    - `online=0.1356s`
    - `amortized=0.3544s`
  - versus `HOPF_K25_BASE_PHI`:
    - top-1 delta `+0.00025`
    - candidate-fraction delta `-0.16384`
    - online delta `-0.0557s`
    - amortized delta `-0.0803s`
  - this is the first confirm-stage translated systems win on the fixed
    product branch: the geometric/addressing signal now cashes out as a real
    runtime/pruning advantage on the translated harness
- `INC-0072` screen completed positive on 2026-03-11.
  - the systems-only rescue was already strong on the 2-seed screen:
    - `H4XH4_FIELD_A150_CPX8` reached `online=0.1332s`, `amortized=0.3516s`
    - both were well below `HOPF_K25_BASE_PHI`
  - the screen therefore justified a 4-seed confirm on the exact same fixed
    route/key law
- `INC-0071` confirm completed positive/narrow on 2026-03-11.
  - the fixed product route law stayed healthy while the secondary-key product
    winner kept the main screen signal
  - `H4XH4_FIELD_A150_CPX8` confirmed:
    - `top1=0.04708`
    - `cand_frac=0.18723`
    - `fallback=0.00333`
    - `amortized=0.4999s`
  - versus the fixed `H4XH4_FIELD_A150` baseline:
    - top-1 delta `+0.00258`
    - candidate-fraction delta `-0.12110`
  - versus `HOPF_K25_BASE_PHI`:
    - top-1 delta `+0.00025`
    - candidate-fraction delta `-0.16384`
  - this is the first confirm-stage translated-addressing positive for the
    second `H^4` on top of the fixed product geometry, but it is not yet a
    systems/runtime promotion because cost still trails the main Hopf
    translated control
- `INC-0071` screen completed positive on 2026-03-11.
  - `H4XH4_FIELD_A150_CPX8` was already the clear winner on the 2-seed screen:
    - `top1=0.04767`
    - `cand_frac=0.19046`
    - `amortized=0.4406s`
  - that justified confirm on the fixed product route set
- `INC-0070` screen completed negative on 2026-03-11.
  - the low-margin selective rerank surface stayed health-passing but failed as
    a retrieval rescue
  - `H4XH4_FIELD_A100` remained better than both rerank variants on the
    translated quality/runtime tradeoff:
    - base: `top1=0.04817`, `cand_frac=0.32029`, `amortized=0.4346s`
    - `R025`: `0.04800`, `0.32029`, `0.6064s`
    - `R050`: `0.04783`, `0.32029`, `0.6108s`
  - `H4XH4_FIELD_A150_R050` recovered some top-1 versus `A150`, but still paid
    too much cost to be useful:
    - base: `top1=0.04567`, `cand_frac=0.31473`, `amortized=0.4749s`
    - `R050`: `0.04650`, `0.31473`, `0.6001s`
  - the fixed `INC-0069` product routes remain the correct translated product
    references, and the next responsible retrieval rescue is secondary keys
- `INC-0069` confirm completed positive/narrow on 2026-03-11.
  - the fixed product phase-field routes preserved useful translated retrieval
    structure across 4 seeds
  - versus `HOPF_K25_BASE_PHI`, both product routes reduced candidate fraction
    and amortized runtime:
    - `H4XH4_FIELD_A100`: `cand_frac 0.3177` vs `0.3511`,
      `amortized 0.4733s` vs `0.4806s`, `top1 0.04625` vs `0.04683`
    - `H4XH4_FIELD_A150`: `cand_frac 0.3083` vs `0.3511`,
      `amortized 0.4688s` vs `0.4806s`, `top1 0.04450`
  - `HOPF_BASE_K25_PHI` still remained the strongest coarse-address translated
    comparator, so the result is a translated-evaluation positive rather than a
    route-frontier replacement
- `INC-0069` screen completed positive/narrow on 2026-03-11.
  - the 2-seed screen already showed the same translated tradeoff:
    - `H4XH4_FIELD_A100` improved pruning and slightly improved top-1 versus
      `HOPF_K25_BASE_PHI`
    - `H4XH4_FIELD_A150` improved pruning and amortized runtime with only a
      small top-1 drop
  - the branch therefore justified confirm instead of stopping at screen
- `INC-0068` screen completed inconclusive/negative on 2026-03-11.
  - all audited route graphs stayed connected across the 2-seed screen
  - the confirmed product operator remained intact:
    - product routes stayed more shell-focused and less sector-concentrated
      than the controls
    - product routes still kept higher participation than the controls
  - routed task-error probes did not turn positive against the Hopf controls:
    - Hopf residual-L2 lowfreq gap: `-0.013560`
    - Hopf residual-L2 Dirichlet gap: `-0.018546`
    - Hopf error-indicator lowfreq gap: `-0.007110`
    - Hopf true-margin lowfreq gap: `-0.021551`
  - this is a cleaner negative than `INC-0067`, so the branch should stop at
    screen stage rather than be replayed on the same proxy target
- `INC-0067` confirm completed inconclusive on 2026-03-11.
  - all audited route graphs stayed connected across 4 seeds
  - the distinct `INC-0066` operator signature remained intact:
    - product routes stayed more shell-focused and less sector-concentrated
    - product routes still kept higher participation than the control set
  - direct task-label projection did not turn positive against the Hopf controls:
    - Hopf label lowfreq gap: `-0.000154`
    - Hopf label Dirichlet gap: `-0.000695`
  - direct one-hot label smoothness is therefore not the signal that validates
    the product spectral branch
- `INC-0067` screen completed inconclusive on 2026-03-11.
  - the 2-seed screen already showed the same pattern:
    - distinct operator shape
    - no positive raw label-projection gap against the Hopf controls
- `INC-0066` confirm completed positive on 2026-03-11.
  - all audited route graphs stayed connected across 4 seeds
  - the confirmed product routes retained a distinct operator signature:
    - higher low-mode participation than the control set
    - lower low-frequency sector concentration than the control set
    - higher low-frequency shell concentration than the Hopf controls
  - confirm gaps stayed positive:
    - full-control participation gap: `+0.0222`
    - full-control sector lowfreq gap: `+0.0805`
    - Hopf-only participation gap: `+0.0208`
    - Hopf-only shell lowfreq gap: `+0.0479`
  - `H4XH4_FIELD_A150` remained the strongest spectral/product point on
    low-mode participation
- `INC-0066` screen completed positive on 2026-03-11.
  - all audited route graphs stayed connected across the 2-seed screen
  - the product routes already showed the same qualitative split:
    - more delocalized low modes than the control set
    - less sector low-frequency concentration than the control set
  - screen gaps were stronger than confirm:
    - participation gap: `+0.0479`
    - sector lowfreq gap: `+0.0849`
- `INC-0065` confirm completed on 2026-03-11.
  - both carried product variants passed the 4-seed health gate
  - `H4XH4_FIELD_A150` confirmed the best product MSE (`0.003900`)
  - `H4XH4_FIELD_A100` remained the stabilized transfer recommendation because
    it beat `R0` on both quality and runtime while staying healthy
  - seed-0 confirm address audit still shows about `98.4%-98.6%` sector
    difference vs `HOPF_BASE_K25_PHI`
  - explicit product phase-field routing is now confirmed mechanism-positive,
    but `HOPF_K25_BASE_PHI` remains the overall routed quality lead
- `INC-0065` product phase-field screen completed on 2026-03-11.
  - all three explicit `H^4 x H^4` product variants passed the configured
    route-health gate
  - `phase_transport_field_shift_abs_mean` is nonzero across the whole screen:
    about `0.0065`, `0.0134`, `0.0145`
  - seed-0 address audit shows about `98.4%-98.6%` sector difference vs
    `HOPF_BASE_K25_PHI`
  - `H4XH4_FIELD_A150` achieved the best screen MSE (`0.003899`)
  - the sweep recommendation nominates `H4XH4_FIELD_A100` as the stabilized
    transfer candidate because it stays healthy while beating `R0` on both MSE
    and runtime
- Corrected reruns of `INC-0062`, `INC-0063`, and `INC-0064` on 2026-03-11 are
  now the canonical readings for this phase of the project.
- `INC-0062` corrected the angular-law read:
  - `phase4d_hopf_base` is the canonical no-fiber-phase coarse-address control
  - route-law-sensitive Hopf sector diagnostics now provide real evidence for
    base/fiber separation rather than only shared-chart distribution effects
  - pure `phase4d_hopf` still keeps the best confirm MSE
  - `RR-061` therefore remains open as a global measure-consistent route-law
    question, but the old “control only” read is too weak
- `INC-0063` corrected the standalone transport read:
  - the old negative was confounded by dead `alpha` bins at `K=25`
  - corrected transport routes now have `phase_transport_alpha_bins=2.0`
  - transported phase shifts scale with `lambda`
  - address-diff audit shows about `98.6%-98.8%` sector change vs
    `phase4d_hopf_base`
- `INC-0064` corrected the coupled-field read:
  - explicit complex-field coupling is mechanism-live and health-passing
  - `phase_transport_field_shift_abs_mean` is strongly nonzero
  - address-diff audit shows about `98.64%` sector change vs
    `phase4d_hopf_base`
  - the branch is not yet the routed quality lead on proxy MSE
- Immediate next action:
  - treat `INC-0098` as the translated cost/decomposition close-out on the
    fixed chart-resident systems stack
  - keep that translated stack frozen as the hardware-side reference
  - move next to sparse event-driven proxy trainability (`INC-0099`) rather
    than more translated cost rescue or shell-law threshold tuning
  - keep `RR-061` open as a background mathematical constraint, not as a reason
    to keep repeating the obsolete “phase inert” story

## Canonical Artifacts
- `INC-0062`
  - `configs/proxy_transfer_inc0062_hopf_base_screen_corrected.json`
  - `configs/proxy_transfer_inc0062_hopf_base_confirm_corrected.json`
  - `results/analysis/inc0062_hopf_base_screen_corrected.json`
  - `results/analysis/inc0062_hopf_base_confirm_corrected.json`
  - `docs/governance/gates/gate_20260311_101015.md`
  - `docs/governance/gates/gate_20260311_101213.md`
- `INC-0063`
  - `configs/proxy_transfer_inc0063_phase_transport_screen_corrected.json`
  - `results/analysis/inc0063_phase_transport_screen_corrected.json`
  - `results/analysis/inc0063_phase_transport_address_diff_corrected.json`
  - `docs/governance/gates/gate_20260311_101344.md`
- `INC-0064`
  - `configs/proxy_transfer_inc0064_coupled_complex_phase_screen.json`
  - `results/analysis/inc0064_coupled_complex_phase_screen_corrected.json`
  - `results/analysis/inc0064_coupled_complex_phase_address_diff_corrected.json`
  - `docs/governance/gates/gate_20260311_101607.md`
- `INC-0065`
  - `configs/proxy_transfer_inc0065_product_phase_field_screen.json`
  - `configs/proxy_transfer_inc0065_product_phase_field_confirm.json`
  - `results/analysis/inc0065_product_phase_field_screen.json`
  - `results/analysis/inc0065_product_phase_field_address_diff.json`
  - `docs/governance/gates/gate_20260311_105034.md`
  - `results/analysis/inc0065_product_phase_field_confirm.json`
  - `results/analysis/inc0065_product_phase_field_confirm_address_diff.json`
  - `docs/governance/gates/gate_20260311_110024.md`
  - `results/analysis/inc0065_product_phase_field_confirm_spectral_seed0.json`
- `INC-0066`
  - `configs/spectral_route_inc0066_screen.json`
  - `configs/spectral_route_inc0066_confirm.json`
  - `tools/spectral_route_audit.py`
  - `tools/spectral_route_sweep.py`
  - `results/analysis/inc0066_spectral_route_operator_screen.json`
  - `results/analysis/inc0066_spectral_route_operator_confirm.json`
  - `docs/governance/gates/gate_20260311_112010.md`
  - `docs/governance/gates/gate_20260311_112215.md`
- `INC-0067`
  - `configs/spectral_signal_inc0067_screen.json`
  - `configs/spectral_signal_inc0067_confirm.json`
  - `tools/spectral_signal_probe.py`
  - `tools/spectral_signal_sweep.py`
  - `results/analysis/inc0067_spectral_signal_probes_screen.json`
  - `results/analysis/inc0067_spectral_signal_probes_confirm.json`
  - `docs/governance/gates/gate_20260311_113729.md`
  - `docs/governance/gates/gate_20260311_114258.md`
- `INC-0068`
  - `configs/spectral_residual_inc0068_screen.json`
  - `tools/spectral_residual_probe.py`
  - `tools/spectral_residual_sweep.py`
  - `results/analysis/inc0068_spectral_residual_task_signals_screen.json`
  - `docs/governance/gates/gate_20260311_122236.md`
- `INC-0069`
  - `configs/proxy_transfer_inc0069_product_phase_translation_screen.json`
  - `configs/proxy_transfer_inc0069_product_phase_translation_confirm.json`
  - `results/analysis/inc0069_product_phase_translation_screen.json`
  - `results/analysis/inc0069_product_phase_translation_confirm.json`
  - `docs/governance/gates/gate_20260311_124011.md`
  - `docs/governance/gates/gate_20260311_124616.md`
- `INC-0070`
  - `configs/proxy_transfer_inc0070_product_phase_translation_rescue_screen.json`
  - `results/analysis/inc0070_product_phase_translation_rescue_screen.json`
  - `docs/governance/gates/gate_20260311_130226.md`
- `INC-0071`
  - `configs/proxy_transfer_inc0071_product_phase_translation_secondary_keys_screen.json`
  - `configs/proxy_transfer_inc0071_product_phase_translation_secondary_keys_confirm.json`
  - `results/analysis/inc0071_product_phase_translation_secondary_keys_screen.json`
  - `results/analysis/inc0071_product_phase_translation_secondary_keys_confirm.json`
  - `docs/governance/gates/gate_20260311_131450.md`
  - `docs/governance/gates/gate_20260311_132141.md`
- `INC-0072`
  - `configs/proxy_transfer_inc0072_product_phase_translation_secondary_key_cost_rescue_screen.json`
  - `configs/proxy_transfer_inc0072_product_phase_translation_secondary_key_cost_rescue_confirm.json`
  - `results/analysis/inc0072_product_phase_translation_secondary_key_cost_rescue_screen.json`
  - `results/analysis/inc0072_product_phase_translation_secondary_key_cost_rescue_confirm.json`
  - `docs/governance/gates/gate_20260311_133356.md`
  - `docs/governance/gates/gate_20260311_133731.md`
- `INC-0073`
  - `configs/proxy_transfer_inc0073_product_phase_translation_secondary_key_large_load_screen.json`
  - `configs/proxy_transfer_inc0073_product_phase_translation_secondary_key_large_load_confirm.json`
  - `results/analysis/inc0073_product_phase_translation_secondary_key_large_load_screen.json`
  - `results/analysis/inc0073_product_phase_translation_secondary_key_large_load_confirm.json`
  - `docs/governance/gates/gate_20260311_134522.md`
  - `docs/governance/gates/gate_20260311_135226.md`
- `INC-0074`
  - `configs/proxy_transfer_inc0074_product_phase_translation_dense_frontier_screen.json`
  - `configs/proxy_transfer_inc0074_product_phase_translation_dense_frontier_confirm.json`
  - `results/analysis/inc0074_product_phase_translation_dense_frontier_screen.json`
  - `results/analysis/inc0074_product_phase_translation_dense_frontier_confirm.json`
  - `docs/governance/gates/gate_20260311_140824.md`
  - `docs/governance/gates/gate_20260311_141705.md`
- `INC-0075`
  - `configs/proxy_transfer_inc0075_product_phase_translation_dense_quality_recovery_screen.json`
  - `configs/proxy_transfer_inc0075_product_phase_translation_dense_quality_recovery_confirm.json`
  - `results/analysis/inc0075_product_phase_translation_dense_quality_recovery_screen.json`
  - `results/analysis/inc0075_product_phase_translation_dense_quality_recovery_confirm.json`
  - `docs/governance/gates/gate_20260311_142843.md`
  - `docs/governance/gates/gate_20260311_144445.md`
- `INC-0076`
  - `configs/proxy_transfer_inc0076_product_phase_translation_break_even_screen.json`
  - `configs/proxy_transfer_inc0076_product_phase_translation_break_even_confirm.json`
  - `results/analysis/inc0076_product_phase_translation_break_even_screen.json`
  - `results/analysis/inc0076_product_phase_translation_break_even_confirm.json`
  - `docs/governance/gates/gate_20260311_145722.md`
  - `docs/governance/gates/gate_20260311_151013.md`
- `INC-0077`
  - `configs/proxy_transfer_inc0077_product_phase_translation_hardware_profile_screen.json`
  - `configs/proxy_transfer_inc0077_product_phase_translation_hardware_profile_confirm.json`
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_screen.json`
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_confirm.json`
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_screen_profile.json`
  - `results/analysis/inc0077_product_phase_translation_hardware_profile_confirm_profile.json`
  - `docs/reports/INC0077_PRODUCT_PHASE_TRANSLATION_HARDWARE_PROFILE_SCREEN.md`
  - `docs/reports/INC0077_PRODUCT_PHASE_TRANSLATION_HARDWARE_PROFILE_CONFIRM.md`
  - `docs/governance/gates/gate_20260311_152632.md`
  - `docs/governance/gates/gate_20260311_153047.md`
- `INC-0078`
  - `configs/proxy_transfer_inc0078_product_phase_translation_crossover_map_screen.json`
  - `configs/proxy_transfer_inc0078_product_phase_translation_crossover_map_confirm.json`
  - `results/analysis/inc0078_product_phase_translation_crossover_map_screen.json`
  - `results/analysis/inc0078_product_phase_translation_crossover_map_confirm.json`
  - `results/analysis/inc0078_product_phase_translation_crossover_map_screen_profile.json`
  - `results/analysis/inc0078_product_phase_translation_crossover_map_confirm_profile.json`
  - `docs/reports/INC0078_PRODUCT_PHASE_TRANSLATION_CROSSOVER_MAP_SCREEN.md`
  - `docs/reports/INC0078_PRODUCT_PHASE_TRANSLATION_CROSSOVER_MAP_CONFIRM.md`
  - `docs/governance/gates/gate_20260311_155644.md`
  - `docs/governance/gates/gate_20260311_161119.md`
- `INC-0079`
  - `configs/proxy_transfer_inc0079_product_phase_translation_large_bank_boundary_extension_screen.json`
  - `configs/proxy_transfer_inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json`
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen.json`
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json`
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen_profile.json`
  - `results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm_profile.json`
  - `docs/reports/INC0079_PRODUCT_PHASE_TRANSLATION_LARGE_BANK_BOUNDARY_EXTENSION_SCREEN.md`
  - `docs/reports/INC0079_PRODUCT_PHASE_TRANSLATION_LARGE_BANK_BOUNDARY_EXTENSION_CONFIRM.md`
  - `docs/governance/gates/gate_20260311_222501.md`
  - `docs/governance/gates/gate_20260311_223841.md`
- `INC-0080`
  - `configs/proxy_transfer_inc0080_product_phase_translation_second_large_bank_boundary_extension_screen.json`
  - `configs/proxy_transfer_inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json`
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen.json`
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json`
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen_profile.json`
  - `results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm_profile.json`
  - `docs/reports/INC0080_PRODUCT_PHASE_TRANSLATION_SECOND_LARGE_BANK_BOUNDARY_EXTENSION_SCREEN.md`
  - `docs/reports/INC0080_PRODUCT_PHASE_TRANSLATION_SECOND_LARGE_BANK_BOUNDARY_EXTENSION_CONFIRM.md`
  - `docs/governance/gates/gate_20260311_230536.md`
  - `docs/governance/gates/gate_20260311_232657.md`
- `INC-0081`
  - `configs/proxy_transfer_inc0081_product_phase_translation_q04_threshold_search_screen.json`
  - `configs/proxy_transfer_inc0081_product_phase_translation_q04_threshold_search_confirm.json`
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen.json`
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm.json`
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen_profile.json`
  - `results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm_profile.json`
  - `docs/reports/INC0081_PRODUCT_PHASE_TRANSLATION_Q04_THRESHOLD_SEARCH_SCREEN.md`
  - `docs/reports/INC0081_PRODUCT_PHASE_TRANSLATION_Q04_THRESHOLD_SEARCH_CONFIRM.md`
  - `docs/governance/gates/gate_20260311_234946.md`
  - `docs/governance/gates/gate_20260312_002000.md`
- `INC-0082`
  - `tools/translated_cost_accounting.py`
  - `tests/test_translated_cost_accounting.py`
  - `results/analysis/inc0082_product_phase_translation_cost_accounting_audit.json`
  - `docs/reports/INC0082_PRODUCT_PHASE_TRANSLATION_COST_ACCOUNTING_AUDIT.md`
- `INC-0083`
  - `tasks/router_retrieval_eval.py`
  - `tools/translated_cache_compare.py`
  - `tests/test_cache_contract.py`
  - `tests/test_router_retrieval_eval.py`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_screen_cold.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_screen_warm.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_screen_compare.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_cold.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_warm.json`
  - `results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_compare.json`
  - `docs/reports/INC0083_PRODUCT_PHASE_TRANSLATION_PERSISTENT_ROUTE_CACHE_SCREEN_COMPARE.md`
  - `docs/reports/INC0083_PRODUCT_PHASE_TRANSLATION_PERSISTENT_ROUTE_CACHE_CONFIRM_COMPARE.md`
  - `docs/governance/gates/gate_20260312_010935.md`
  - `docs/governance/gates/gate_20260312_011438.md`
  - `docs/governance/gates/gate_20260312_012911.md`
  - `docs/governance/gates/gate_20260312_013859.md`
- `INC-0084`
  - `configs/proxy_transfer_inc0084_product_phase_translation_warm_cache_onset_map_screen.json`
  - `configs/proxy_transfer_inc0084_product_phase_translation_warm_cache_onset_map_confirm.json`
  - `results/analysis/inc0084_product_phase_translation_warm_cache_onset_map_screen.json`
  - `results/analysis/inc0084_product_phase_translation_warm_cache_onset_map_confirm.json`
  - `docs/reports/INC0084_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_ONSET_MAP.md`
  - `docs/governance/gates/gate_20260312_015936.md`
  - `docs/governance/gates/gate_20260312_021141.md`

## Current Best-Known Routes
- Synthetic lead: `R5B`
- Transfer control baseline: `R0`
  - still shell-collapsed and health-failing on strict gate
- Current routed quality lead: `HOPF_K25_BASE_PHI`
  - `phase4d_hopf`
  - corrected `INC-0062` confirm: `mse=0.003895`, `total=6.093s`
- Current no-fiber-phase coarse-address control: `HOPF_BASE_K25_PHI`
  - `phase4d_hopf_base`
  - corrected `INC-0062` confirm: `mse=0.003906`, `total=6.035s`
  - strongest corrected base/fiber separation evidence
- Current widened healthy comparator: `HOPF_PHI2_BAND_PHI`
  - `phase4d_hopf_fib_band`
  - corrected `INC-0062` confirm: `mse=0.003903`, `total=6.061s`
- Current standalone phase-transport screen lead:
  - `HOPF_TRANSPORT_L150`
  - corrected `INC-0063` screen: `mse=0.003900`, `total=6.125s`,
    `shift_abs_mean=0.2971`
- Current coupled-field phase screen lead:
  - `HOPF_CPX_TRANSPORT_L050_F100`
  - corrected `INC-0064` screen: `mse=0.003932`, `total=5.532s`,
    `field_shift_abs_mean=0.6085`
- Current explicit product phase-field screen lead:
  - `H4XH4_FIELD_A150`
  - `INC-0065` screen: `mse=0.003899`, `total=6.036s`,
    `field_shift_abs_mean=0.0145`
- Current stabilized product carry-forward candidate:
  - `H4XH4_FIELD_A100`
  - `INC-0065` screen recommendation from `tools/proxy_sweep.py`
- Current confirmed product phase-field lead:
  - `H4XH4_FIELD_A150`
  - `INC-0065` confirm: `mse=0.003900`, `total=5.890s`,
    `field_shift_abs_mean=0.0124`
- Current confirmed product stabilized candidate:
  - `H4XH4_FIELD_A100`
  - `INC-0065` confirm recommendation from `tools/proxy_sweep.py`
- Current spectral low-mode participation lead:
  - `H4XH4_FIELD_A150`
  - `INC-0066` confirm: `participation_mean=0.2844`,
    `sector_lowfreq=0.0366`, `shell_lowfreq=0.1255`
- Current routed sector-concentration reference under the spectral operator:
  - `HOPF_K25_BASE_PHI`
  - `INC-0066` confirm: `participation_mean=0.2522`,
    `sector_lowfreq=0.0956`
- Current direct-label probe leader:
  - `HOPF_PHI2_BAND_PHI`
  - `INC-0067` confirm: `label_lowfreq=0.023796`
- Current translated retrieval top-1 routed reference:
  - `HOPF_K25_BASE_PHI`
  - `INC-0069` confirm: `top1=0.04683`, `cand_frac=0.3511`,
    `amortized=0.4806s`
- Current translated retrieval coarse-address efficiency reference:
  - `HOPF_BASE_K25_PHI`
  - `INC-0069` confirm: `top1=0.04642`, `cand_frac=0.3113`,
    `amortized=0.4616s`
- Current translated product balanced branch:
  - `H4XH4_FIELD_A100`
  - `INC-0069` confirm: `top1=0.04625`, `cand_frac=0.3177`,
    `amortized=0.4733s`
- Current translated product pruning branch:
  - `H4XH4_FIELD_A150`
  - `INC-0069` confirm: `top1=0.04450`, `cand_frac=0.3083`,
    `amortized=0.4688s`
- Current translated secondary-key product reference:
  - `H4XH4_FIELD_A150_CPX8`
  - `INC-0071` confirm: `top1=0.04708`, `cand_frac=0.1872`,
    `amortized=0.4999s`
- Current translated secondary-key systems lead:
  - `H4XH4_FIELD_A150_CPX8`
  - `INC-0073` confirm: `top1=0.04867`, `cand_frac=0.1903`,
    `online=0.3804s`, `amortized=0.7949s`
- Current translated dense-frontier systems lead:
  - `H4XH4_FIELD_A150_CPX8`
  - `INC-0074` confirm: `top1=0.04867`, `cand_frac=0.1903`,
    `online=0.3904s`, `amortized=0.8308s`
- Current translated dense-frontier quality-matched routed point:
  - `H4XH4_FIELD_A150`
  - `INC-0074` confirm: `top1=0.04912`, `cand_frac=0.3147`,
    `amortized=0.9447s`
- Current translated break-even quality-matched point:
  - `H4XH4_FIELD_A150_Q16`
  - `INC-0076` confirm: `top1=0.04912`, `cand_frac=0.3147`,
    `amortized=1.163s`
- Current translated break-even systems crossover point:
  - `H4XH4_FIELD_A150_CPX8_Q16`
  - `INC-0076` confirm: `top1=0.04867`, `cand_frac=0.1903`,
    `amortized=1.036s`
- Current translated stabilized break-even systems point:
  - `H4XH4_FIELD_A150_CPX8_Q24`
  - `INC-0076` confirm: `top1=0.04867`, `cand_frac=0.1903`,
    `amortized=0.8308s`
- Current translated smaller-bank crossover point:
  - `H4XH4_FIELD_A150_CPX8_Q24_T6000`
  - `INC-0077` confirm: `top1=0.04708`, `cand_frac=0.1872`,
    `amortized=0.3507s`
- Current translated earliest confirmed systems crossover point:
  - `H4XH4_FIELD_A150_CPX8_Q04_T36000`
  - `INC-0081` confirm: `top1=0.04707`, `cand_frac=0.1902`,
    `amortized=9.6944s`
- Current translated highest-bank confirmed systems crossover point:
  - `H4XH4_FIELD_A150_CPX8_Q08_T40000`
  - `INC-0081` confirm: `top1=0.04733`, `cand_frac=0.1838`,
    `amortized=6.1060s`
- Current translated warm-cache rescued upper-bank crossover point:
  - `H4XH4_FIELD_A150_CPX8_Q04_T40000`
  - `INC-0083` confirm: `top1=0.04733`, `cand_frac=0.1838`,
    `amortized=1.9724s`
- Current translated warm-cache single-query crossover point:
  - `H4XH4_FIELD_A150_CPX8_Q01_T2500`
  - `INC-0092` confirm: `top1=0.04630`, `cand_frac=0.1987`,
    `amortized=0.0741s`
- Current translated warm-cache earliest any-repeat crossover point:
  - `H4XH4_FIELD_A150_CPX8_Q01_T2500`
  - `INC-0092` confirm: `top1=0.04630`, `cand_frac=0.1987`,
    `amortized=0.0741s`
- Current translated warm-cache stabilized upper-bank systems point:
  - `H4XH4_FIELD_A150_CPX8_Q08_T40000`
  - `INC-0084` confirm: `top1=0.04733`, `cand_frac=0.1838`,
    `amortized=1.8680s`
- Current translated chart-resident upper-bank systems point:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`
  - `INC-0094` confirm: `top1=0.04733`, `cand_frac=0.1838`,
    `amortized=2.4952s`
- Current translated chart-resident lower-bank crossover point:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`
  - `INC-0096` focused confirm: `top1=0.04630`, `cand_frac=0.1987`,
    `amortized=0.0807s`
  - `INC-0096` mixed confirm: `top1=0.04630`, `cand_frac=0.1987`,
    `amortized=0.0871s`
- Current translated exact lower-bank full-warm floor:
  - `FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500`
  - `INC-0093` confirm: `top1=0.04460`, `cand_frac=0.1933`,
    `amortized=0.0562s`
- Current translated route-only residency read:
  - `T2500 Q01`: still negative at `amortized=2.5345s`
  - `T40000 Q01`: still negative at `amortized=34.7146s`
- Current translated packet-scope read:
  - the chart-resident lower-bank `Q01` point survives both focused and mixed
    packet shapes
  - packet scope changes the margin size, not the retrieval signal or the sign
- Current translated cost read:
  - the chart-resident stack is already positive against dense at both lower
    and upper anchors
  - the remaining chart-versus-full-warm gap is split between route-index
    build and retrieval-search overhead
  - no dominant hidden residual remains
  - translated systems work is now frozen as a reference rather than an active
    tuning branch

## Current Mechanistic Reading
- Geometry routing remains live.
- Hopf-base coarse routing is now a stronger result than the original
  `INC-0062` writeup captured.
- Standalone transported phase is not inert; the old negative was a harness
  artifact.
- Coupled complex-field transport is also live, but the current same-chart
  implementation still does not beat the routed quality controls.
- The explicit product `H^4 x H^4` split is now also live on the proxy screen:
  it creates large address motion, nonzero field-shift metrics, and healthy
  route statistics.
- The product branch now survives confirm as well:
  it is a valid confirmed phase-field branch, but not the overall routed MSE
  leader.
- Direct spectral/operator measurement is now also positive:
  - the confirmed route graphs stay connected
  - the product routes carry more delocalized low modes than the control set
  - pure Hopf still concentrates sector signal more strongly in the low band
  - `HOPF_BASE_K25_PHI` stays unusually shell-focused, which means the product
    branch is not the only nontrivial spectral shape in the confirmed set
- Direct one-hot label probes are not enough to validate useful spectral
  structure:
  - the product branch does not beat the Hopf controls on raw label lowfreq
    energy
  - the per-class indicator mean is nearly flat and too small to claim as
    evidence
- Routed residual/task-error probes are also not enough on the proxy target:
  - the product branch loses to the Hopf controls on residual norm, error
    indicators, and true-margin low-mode projections
  - the confirmed operator distinction is therefore structural but not yet
    validated as useful task structure on proxy regression
- Task translation is now positive/narrow on the fixed product branch:
  - the product routes preserve useful translated locality and pruning
  - the branch gives a real runtime/pruning tradeoff against the main Hopf
    routed controls
  - the remaining weakness is small translated recall / ordering loss, not
    fallback instability or address collapse
- `INC-0070` then ruled out the first rescue surface:
  - low-margin reranking inside the fixed product candidate set gave back too
    much runtime for too little quality recovery
- `INC-0071` then turned the second `H^4` into a confirm-stage translated
  addressing result:
  - secondary keys materially improved top-1 and pruning on the fixed product
  branch
  - the remaining failure is systems cost, not addressing quality
- `INC-0072` then turned that same fixed route/key law into a confirm-stage
  translated systems result:
  - the cost rescue preserved top-1 and pruning
  - and converted the branch into a real online/amortized win versus the main
    Hopf translated control
- `INC-0073` then hardened that systems result under larger translated load.
- `INC-0074` then carried the same fixed law directly against dense exact
  retrieval.
- `INC-0075` then killed bounded quality recovery on the fixed frontier.
- `INC-0076` then established the first confirmed repeated-query crossover.
- `INC-0077` then showed the crossover survives the first explicit hardware-cost
  profile.
- `INC-0078` established the first coherent bank-by-bank crossover boundary:
  - no crossover through `Q24` at `3000`
  - first crossover at `Q24` for `6000`
  - first crossover at `Q12` for `12000`
- `INC-0079` extended that boundary coherently upward:
  - first crossover stays at `Q12` for `12000`
  - first crossover moves earlier to `Q08` for `18000`
  - the secondary-key search-work ratio still stays pinned near `0.19`
- `INC-0080` showed that the onset holds at `Q08` through `24000` and
  `30000`, but still did not move to `Q04`.
- `INC-0081` now confirms the first real `Q04` crossover at `36000`, but the
  onset is not monotone because `40000` still begins at `Q08`.
- `INC-0082` explains that split directly from static offline build cost
  composition on the fixed translated stack.
- `INC-0083` then confirms that persistent cache reuse removes almost all of
  that cost without changing routed quality or search work.
- `INC-0084` then shows that the same fixed translated stack crosses dense all
  the way down to `Q01` under warm-cache conditions at `T40000`.
- `INC-0085` then confirms that the same fixed stack already crosses at
  `Q01 T3000` on the tracked warm-cache bank ladder.
- `INC-0086` then refines that lower boundary:
  - `T2500` still misses at `Q01` but crosses at `Q02`
  - `T2750` already crosses at `Q01`
- `INC-0087` then moves the earliest tracked confirmed `Q01` onset to `T2600`,
  but exposes a local non-monotone pocket at `T2650`.
- `INC-0088` then explains that pocket from local route-query versus search
  cost balance instead of route degeneration.
- `INC-0089` then re-centers the bracket and moves the earliest tracked
  confirmed `Q01` onset again to `T2525`.
- `INC-0090` then closes the `2505/2510/2515/2520` bracket positive and moves
  the earliest tracked confirmed `Q01` onset again to `T2505`.
- `INC-0091` then closes the final integer `2501/2502/2503/2504` bracket
  positive and moves the earliest tracked confirmed `Q01` onset again to
  `T2501`.
- `INC-0092` then hardens that floor and overturns the old exact separation:
  `T2500` also survives at `Q01` on the expanded seed schedule.
- `INC-0093` then decomposes the cache-residency story:
  - chart-only warm already preserves the upper-bank `T40000 Q01` systems win
  - route-only warm stays negative at both anchor banks
  - the exact lower-bank `T2500 Q01` floor remains a full-warm claim
- `INC-0094` then strengthens that into a chart-persistent session result:
  - `T2500` crosses by `Q02` under chart-only residency
  - `T40000` already crosses by `Q01` under chart-only residency
- `INC-0095` then broadens the single-query chart-resident read further:
  - the focused `Q01` packet crosses already at `T2500`
  - the lower-bank chart-only question is no longer bank location
  - the remaining issue is packet-scope sensitivity against the older
    mixed-repeat `INC-0094` read
- `INC-0096` then hardens that against packet composition:
  - both focused and mixed packets stay positive at `T2500 Q01`
  - packet scope changes the margin size, but not the sign
  - the lower-bank chart-resident single-query story is now stable
- The next responsible move is bounded quality recovery on the fixed soft
  sparse translated point (`INC-0103`), not more translated bank/cache
  accounting or more event sharpening.

## Resume Rule
If context is lost again:
1. read `docs/SESSION_BOOTSTRAP.md`
2. read `docs/PROJECT_CONTEXT.md`
3. read `docs/routes/ROUTE_MATRIX.md`
4. read `docs/DECISIONS.md`
5. read this file
6. read `docs/research/HANDOFF_CURRENT.md`
7. read `docs/research/SESSION_LEDGER.md`
8. read `docs/research/increments/INC_0066_spectral_route_operator.md`
9. read `docs/research/increments/INC_0067_spectral_signal_probes.md`
10. read `docs/research/increments/INC_0068_spectral_residual_task_signals.md`
11. read `docs/research/increments/INC_0069_product_phase_translation_eval.md`
12. read `docs/research/increments/INC_0070_product_phase_translation_rescue.md`
13. read `docs/research/increments/INC_0071_product_phase_translation_secondary_keys.md`
14. read `docs/research/increments/INC_0072_product_phase_translation_secondary_key_cost_rescue.md`
15. read `docs/research/increments/INC_0073_product_phase_translation_secondary_key_large_load.md`
16. read `docs/research/increments/INC_0074_product_phase_translation_dense_frontier.md`
17. read `docs/research/increments/INC_0075_product_phase_translation_dense_quality_recovery.md`
18. read `docs/research/increments/INC_0076_product_phase_translation_break_even.md`
19. read `docs/research/increments/INC_0077_product_phase_translation_hardware_profile.md`
20. read `docs/research/increments/INC_0078_product_phase_translation_crossover_map.md`
21. read `docs/research/increments/INC_0079_product_phase_translation_large_bank_boundary_extension.md`
22. read `docs/research/increments/INC_0080_product_phase_translation_second_large_bank_boundary_extension.md`
23. read `docs/research/increments/INC_0081_product_phase_translation_q04_threshold_search.md`
24. read `docs/research/increments/INC_0082_product_phase_translation_cost_accounting_audit.md`
25. read `docs/research/increments/INC_0083_product_phase_translation_persistent_route_cache.md`
26. read `docs/research/increments/INC_0084_product_phase_translation_warm_cache_onset_map.md`
27. read `docs/research/increments/INC_0085_product_phase_translation_warm_cache_q01_bank_boundary.md`
28. read `docs/research/increments/INC_0086_product_phase_translation_warm_cache_q01_lower_boundary_refine.md`
29. read `docs/research/increments/INC_0087_product_phase_translation_warm_cache_q01_threshold_refine.md`
30. read `docs/research/increments/INC_0088_product_phase_translation_warm_cache_q01_local_cost_audit.md`
31. read `docs/research/increments/INC_0089_product_phase_translation_warm_cache_q01_2500_2600_refine.md`
32. read `docs/research/increments/INC_0090_product_phase_translation_warm_cache_q01_2500_2525_refine.md`
33. read `docs/research/increments/INC_0091_product_phase_translation_warm_cache_q01_2500_2505_refine.md`
34. read `docs/research/increments/INC_0092_product_phase_translation_warm_cache_q01_floor_hardening.md`
35. read `docs/research/increments/INC_0093_product_phase_translation_cache_residency_mix.md`
36. read `docs/research/increments/INC_0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map.md`
37. read `docs/research/increments/INC_0095_product_phase_translation_chart_resident_q01_bank_boundary.md`
38. read `docs/research/increments/INC_0096_product_phase_translation_chart_resident_q01_packet_scope_audit.md`
39. read `docs/research/increments/INC_0097_product_phase_sparse_gated_shell_pilot.md`
40. read `docs/research/increments/INC_0098_product_phase_translation_chart_resident_route_cost_decomposition.md`
41. read `docs/research/increments/INC_0099_product_phase_sparse_event_proxy_pilot.md`
42. read `docs/research/increments/INC_0100_product_phase_sparse_event_translation_pilot.md`
43. read `docs/research/increments/INC_0101_product_phase_hard_event_proxy_pilot.md`
44. read `docs/research/increments/INC_0102_product_phase_near_hard_event_translation_pilot.md`
45. read `docs/research/increments/INC_0103_product_phase_soft_sparse_translation_quality_recovery.md`
46. read `docs/research/increments/INC_0104_product_phase_soft_sparse_translation_backfill_recovery.md`
47. read `docs/research/increments/INC_0105_product_phase_soft_sparse_translation_upper_bank_carry_forward.md`
48. read `docs/research/increments/INC_0106_product_phase_sparse_translation_systems_cost_decomposition.md`
49. read `docs/research/increments/INC_0107_product_phase_sparse_translation_component_stability_audit.md`
50. read `docs/research/increments/INC_0108_product_phase_sparse_translation_repeated_timing_hardening.md`
51. read `docs/research/increments/INC_0108_product_phase_sparse_translation_repeated_timing_hardening.md`
52. read `docs/research/increments/INC_0109_product_phase_sparse_translation_robust_cost_reference.md`
53. read `docs/research/increments/INC_0110_product_phase_sparse_translation_dense_robust_hardening.md`
54. read `docs/research/increments/INC_0111_product_phase_sparse_translation_dense_quality_frontier.md`
55. read `docs/research/increments/INC_0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening.md`
56. read `docs/research/increments/INC_0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition.md`
57. read `docs/research/increments/INC_0114_product_phase_sparse_translation_upper_bank_dense_reference_selection.md`
58. read `docs/research/increments/INC_0115_product_phase_sparse_translation_promoted_upper_bank_carry_forward.md`
59. read `docs/research/increments/INC_0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.md`
60. read `docs/research/increments/INC_0117_product_phase_sparse_translation_dual_anchor_broader_comparison.md`
61. read `docs/research/increments/INC_0118_product_phase_sparse_translation_dual_anchor_task_side_extension.md`
62. read `docs/research/increments/INC_0119_product_phase_sparse_translation_dual_anchor_real_task_comparison.md`
63. read `docs/research/increments/INC_0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.md`
64. read `docs/research/increments/INC_0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.md`
65. read `docs/research/increments/INC_0122_product_phase_sparse_translation_dual_anchor_real_task_downstream_extension.md`
66. read `docs/research/increments/INC_0123_product_phase_sparse_translation_dual_anchor_real_task_downstream_comparison.md`
67. read `docs/research/increments/INC_0124_product_phase_sparse_translation_dual_anchor_real_task_downstream_carry_forward.md`
68. read `docs/research/increments/INC_0125_product_phase_sparse_event_proxy_trainability_hardening.md`
69. read `docs/research/increments/INC_0126_product_phase_sparse_event_proxy_translation_gap_audit.md`
70. read `docs/research/increments/INC_0127_product_phase_sparse_event_translation_systems_cost_rescue.md`
71. resume with `INC-0128`, using the completed `INC-0127`
    rescue-feasibility audit, the completed `INC-0125` hardening confirm
    artifact, and the fixed product route law from `INC-0065`
