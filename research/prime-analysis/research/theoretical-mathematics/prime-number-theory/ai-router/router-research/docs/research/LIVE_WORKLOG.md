# Live Worklog

Mirror note:
- This file is an operational mirror, not the canonical queue authority.
- Authoritative queue docs:
  - `docs/research/ACTIVE_STATE.md`
  - `docs/research/KILL_LIST_TRACKER.md`

## Queue Reset
Completed a root-theory audit on 2026-03-12.

Result:
  - the repo has real translated sparse-event evidence, but that lane is now
    treated as downstream support rather than the main unresolved gate
  - `RR-061` remains open as the core measure-consistent `H^4` / Hopf route-law
    problem
  - `INC-0135` is deferred
  - the next increment initially became `INC-0136`:
    measure-consistent `H^4` / Hopf route return

Decision:
  - preserve `INC-0130` through `INC-0134` as valid lower-bank/upper-bank
    translated evidence
  - stop treating lower-bank frontier refinement as the primary next step
  - move next to the earlier unresolved geometry gate

## Latest Update
Completed `INC-0136` on the geometry-return screen.

Result:
  - the new combined geometry-return route was:
    `HOPF_BASE_BALL_K25_PHI` / `sector_mode=phase4d_hopf_base_ball`
  - it failed the route-health gate and did not improve measure consistency:
    - `mse=0.0039117`
    - `total=7.0118s`
    - `shell_mass_l1=1.7724`
    - `hopf_base_mass=1.0284`
    - `shell_pmax=0.8862`
    - `knn_overlap=0.6362`
    - `route_H_r_corr≈0`
  - healthy reference remained:
    `HOPF_BASE_K25_PHI`
    - `mse=0.0039004`
    - `shell_mass_l1=1.0188`
    - `shell_pmax=0.5222`
    - `knn_overlap=0.6738`

Decision:
  - close `INC-0136` negative/explanatory
  - keep `RR-061` open
  - move next to `INC-0137` on a narrower shell-pressure blend correction
  - do not reopen translated lower-bank/frontier work as the main queue

## Current State
- Latest completed increment: `INC-0136` measure-consistent `H^4` / Hopf route return
- Next increment: `INC-0137` measure-consistent `H^4` / Hopf shell-pressure blend
- Transfer control baseline: `R0`
- Routed quality lead: `HOPF_K25_BASE_PHI`
- No-fiber-phase control: `HOPF_BASE_K25_PHI`
- Current coupled-field screen reference: `HOPF_CPX_TRANSPORT_L050_F100`
- Current confirmed product lead: `H4XH4_FIELD_A150`
- Current confirmed product stabilized candidate: `H4XH4_FIELD_A100`
- Current near-hard sparse-event proxy reference:
  `H4XH4_FIELD_A150_EVT_T070_TAU002`
- Current soft sparse-event proxy reference: `H4XH4_FIELD_A150_EVT_T070`
- Current sparse-event translated lower-bank systems reference:
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
  - translated confirm: `top1=0.0446`, `cand_frac=0.193328`,
    `amortized=0.08990s`
- Current sparse-event translated lower-bank balanced quality comparator:
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
  - translated confirm: `top1=0.0464`, `cand_frac=0.193328`,
    `amortized=0.09420s`
- Current sparse-event translated lower-bank quality-first comparator:
  `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
  - translated confirm: `top1=0.0524`, `cand_frac=0.193328`,
    `amortized=0.14159s`
- Historical lower-bank bounded-backfill point now comparator-only:
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  - focused prewarmed confirm: `top1=0.0452`, `cand_frac=0.190150`,
    `amortized=1.99907s`
- Current sparse-event translated upper-bank quality/reference point:
  `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
  - translated confirm: `top1=0.047325`, `cand_frac=0.183764`,
    `amortized=3.53342s`
- Current sparse-event translated upper-bank systems lead:
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
  - translated confirm: `top1=0.0472875`, `cand_frac=0.182003`,
    `amortized=3.47015s`
- Promoted upper-bank dense-near routed reference:
  `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Default broader-comparison packet:
  - `DENSE_Q01_T2500`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
  - `DENSE_Q01_T40000`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Current broader/task-side default read:
  - lower bank = `systems-only` via
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
  - lower-bank explicit comparators =
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500` and
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
  - upper bank = `quality-near systems promotion` via
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Current explicit real-task comparison recommendation:
  - lower bank:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
    - carry as systems-only default
  - upper bank:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - carry as promoted real-task default
- Current explicit downstream real-task carry-forward:
  - lower bank:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
    - carry as systems-only default
    - explicit comparators:
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`,
      `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
  - upper bank:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - carry as promoted real-task default
- Current downstream real-task packet manifest:
  - `DENSE_Q01_T2500`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
  - `DENSE_Q01_T40000`
  - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Current downstream real-task extension read:
  - lower bank = `systems-only` via
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
  - upper bank = `quality-near systems promotion` via
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- Current explicit downstream real-task comparison recommendation:
  - lower bank:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
    - carry as systems-only default
  - upper bank:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - carry as promoted real-task default
- Current explicit downstream real-task carry-forward:
  - lower bank:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
    - carry as systems-only default
  - upper bank:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
    - carry as promoted real-task default
- Comparator-only upper-bank route:
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
- Nondefault lower-bank pruning/quality route:
  `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
- Current confirmed spectral participation lead: `H4XH4_FIELD_A150`
- Current confirmed spectral sector reference: `HOPF_K25_BASE_PHI`
- Current direct-label probe reference: `HOPF_PHI2_BAND_PHI`
- Current residual/task-error probe reference: `HOPF_K25_BASE_PHI`
- Current translated retrieval top-1 routed reference: `HOPF_K25_BASE_PHI`
- Current translated retrieval coarse-address efficiency reference: `HOPF_BASE_K25_PHI`
- Current translated dense-frontier systems lead: `H4XH4_FIELD_A150_CPX8`
  - dense confirm: `top1=0.04867`, `cand_frac=0.19032`,
    `online=0.3904s`, `amortized=0.8308s`
- Current translated dense-frontier quality-matched routed point: `H4XH4_FIELD_A150`
  - dense confirm: `top1=0.04912`, `cand_frac=0.31475`,
    `amortized=0.9447s`
- Current translated break-even quality-matched point: `H4XH4_FIELD_A150_Q16`
  - break-even confirm: `top1=0.04912`, `cand_frac=0.31475`,
    `amortized=1.163s`
- Current translated break-even systems crossover point: `H4XH4_FIELD_A150_CPX8_Q16`
  - break-even confirm: `top1=0.04867`, `cand_frac=0.19032`,
    `amortized=1.036s`
- Current translated stabilized break-even systems point: `H4XH4_FIELD_A150_CPX8_Q24`
  - break-even confirm: `top1=0.04867`, `cand_frac=0.19032`,
    `amortized=0.831s`
- Current translated smaller-bank crossover point: `H4XH4_FIELD_A150_CPX8_Q24_T6000`
  - hardware-profile confirm: `top1=0.04708`, `cand_frac=0.18723`,
    `amortized=0.3507s`
- Current translated earliest confirmed systems crossover point: `H4XH4_FIELD_A150_CPX8_Q04_T36000`
  - threshold confirm: `top1=0.04707`, `cand_frac=0.19021`,
    `amortized=9.6944s`
- Current translated highest-bank confirmed systems crossover point: `H4XH4_FIELD_A150_CPX8_Q08_T40000`
  - threshold confirm: `top1=0.04733`, `cand_frac=0.18376`,
    `amortized=6.1057s`
- Current translated warm-cache rescued upper-bank crossover point: `H4XH4_FIELD_A150_CPX8_Q04_T40000`
  - cache confirm: `top1=0.04733`, `cand_frac=0.18376`,
    `amortized=1.9724s`
- Current translated warm-cache single-query crossover point: `H4XH4_FIELD_A150_CPX8_Q01_T2500`
  - warm-cache hardening confirm: `top1=0.04630`, `cand_frac=0.19872`,
    `amortized=0.0741s`
- Current translated warm-cache earliest any-repeat crossover point: `H4XH4_FIELD_A150_CPX8_Q01_T2500`
  - warm-cache hardening confirm: `top1=0.04630`, `cand_frac=0.19872`,
    `amortized=0.0741s`
- Current translated warm-cache stabilized upper-bank systems point: `H4XH4_FIELD_A150_CPX8_Q08_T40000`
  - cache confirm: `top1=0.04733`, `cand_frac=0.18376`,
    `amortized=1.8680s`
- Current translated chart-resident upper-bank systems point:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000`
  - chart-resident confirm: `top1=0.04733`, `cand_frac=0.18376`,
    `amortized=2.4952s`
- Current translated chart-resident lower-bank crossover point:
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`
  - packet-scope hardening focused confirm: `top1=0.04630`,
    `cand_frac=0.19872`, `amortized=0.0807s`
  - packet-scope hardening mixed confirm: `top1=0.04630`,
    `cand_frac=0.19872`, `amortized=0.0871s`
- Current translated exact lower-bank full-warm floor:
  - `FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500`
  - cache-residency confirm: `top1=0.04460`, `cand_frac=0.19333`,
    `amortized=0.0562s`
- Current translated route-only residency read:
  - `T2500 Q01`: negative
  - `T40000 Q01`: negative
- Current translated packet-scope read:
  - the chart-resident lower-bank `Q01` point survives both focused and mixed
    packet composition
  - packet accounting is now stable enough to stop as an active branch
- Current sparse shell pilot read:
  - gated and banded shell control are mechanism-live but both fail shell
    health on the 2-seed proxy screen
- Current sparse event proxy read:
  - `H4XH4_FIELD_A150_EVT_T070_TAU002` survives 4 seeds as the hardened
    near-hard controller on the fixed product route law
  - `event_gate_mean≈0.020` and `event_gate_active_frac=0.0` make it the
    clean sparse-quality winner under the harder proxy load
  - `H4XH4_FIELD_A150_EVT_T070` remains the healthy softer sparse comparator
    with `event_gate_mean≈0.319` and `event_gate_active_frac=0.0`
  - `H4XH4_FIELD_A150_HARD_T062` still behaves as a mostly-on controller with
    `event_gate_active_frac≈0.840`
- Current sparse event proxy/translation gap read:
  - translated near-hard preserves top-1 and candidate fraction
  - omission and ordering-loss explanations are not supported
  - translated failure is systems-cost-only
  - primary driver is retrieval search, with route-index build as the second
    contributor
- Current sparse event translated rescue-feasibility read:
  - translated soft sparse and translated near-hard differ only on
    `event_gate_tau`
  - current translated retrieval treats sparse-event knobs as audit-only
  - the translated rescue reopen is therefore closed until sparse-event
    behavior is coupled into the translated route or retrieval surface
- Current sparse event translated route-coupled read:
  - train-gate pruning now makes translated sparse-event behavior genuinely
    downstream-live
  - soft sparse at threshold `0.02` stays inert with `keep_frac=1.0`
  - near-hard at threshold `0.02` cuts the translated bank to about `74.5%`
    and lowers candidate fraction to about `0.131`
  - the same near-hard point collapses top-1 and is not a carry-forward route
- Current sparse event translated threshold-map read:
  - threshold `0.010` is the best quality-preserving train-gate-prune point
  - it still regresses online and amortized runtime versus uncoupled near-hard
  - stronger prune thresholds create a monotone candidate-fraction win /
    quality-collapse tradeoff
  - no current train-gate-prune threshold is promotable
- Current sparse event translated soft-bias read:
  - soft score bias is now confirm-stage downstream-live without item deletion
  - `SBI030` is the balanced lower-bank quality lift
  - `SBI080` is the quality-first lower-bank comparator
  - uncoupled near-hard remains the lower-bank sparse-event translated systems
    reference
- Current translated cost read:
  - chart-resident translated routing is already robust against dense at both
    fixed anchors
  - the remaining chart-versus-full-warm gap is shared between route-index
    build and retrieval-search cost
  - soft sparse carry-forward now survives on the lower-bank translated stack
  - near-hard proxy activation is now closed positive/narrow
  - near-hard translated carry-forward is now closed negative at screen
  - `INC-0103` rerank recovery is now closed negative at confirm
  - `INC-0104` bounded backfill recovery is now closed negative on quality
    recovery but positive on lower-bank systems refinement
  - `INC-0105` upper-bank carry-forward is now closed positive/narrow
  - `INC-0106` sparse translated systems cost decomposition is now closed
    positive/explanatory
  - `INC-0107` sparse translated component stability audit is now closed
    positive/explanatory
  - `INC-0108` repeated timing hardening is now closed positive/explanatory
  - `INC-0109` robust cost referencing is now closed positive/explanatory:
    - upper-bank bounded backfill remains a clean robust systems lead versus
      both routed baselines
    - lower-bank bounded backfill remains robust versus the continuous
      translated product reference
    - lower-bank bounded backfill versus the fixed soft sparse translated
      reference is now a pruning-first read rather than a clean robust
      wallclock promotion
  - `INC-0110` dense-frontier hardening is now closed positive/explanatory:
    - lower-bank bounded backfill remains the only robust lower-bank dense
      systems promotion
    - both upper-bank sparse translated points remain robust dense systems
      promotions
    - every dense comparison still carries a robust top-1 deficit versus
      dense exact
  - `INC-0111` dense quality-frontier accounting is now closed
    positive/explanatory:
    - lower-bank soft sparse = pruning-only
    - lower-bank bounded backfill = systems-only
    - both upper-bank sparse translated points = quality-near systems
      promotions
  - `INC-0112` upper-bank dense quality-tolerance hardening is now closed
    positive/explanatory:
    - both upper-bank sparse translated points remain quality-near systems
      promotions
    - the upper-bank top-1 gap stays small but robustly negative
  - `INC-0117` dual-anchor broader comparison is now closed
    positive/explanatory
  - `INC-0118` dual-anchor task-side extension is now closed
    positive/explanatory
  - `INC-0119` explicit dual-anchor real-task comparison is now closed
    positive/explanatory
  - `INC-0120` downstream dual-anchor real-task carry-forward is now closed
    positive/explanatory
  - `INC-0121` downstream dual-anchor real-task packet manifest is now closed
    positive/explanatory
  - `INC-0122` downstream dual-anchor real-task extension is now closed
    positive/explanatory
  - `INC-0123` downstream dual-anchor real-task comparison is now closed
    positive/explanatory
  - `INC-0124` downstream dual-anchor real-task carry-forward is now closed
    positive/explanatory
  - `INC-0125` sparse-event proxy trainability hardening is now closed
    positive/explanatory
  - `INC-0126` sparse-event proxy/translation gap audit is now closed
    positive/explanatory
  - `INC-0127` translated systems-cost rescue is now closed
    negative/explanatory
  - next tracked branch is `INC-0128` route-coupled sparse-event translated
    pilot

## If Session Gets Cut
1. Read:
  - `docs/SESSION_BOOTSTRAP.md`
  - `docs/PROJECT_CONTEXT.md`
  - `docs/routes/ROUTE_MATRIX.md`
  - `docs/DECISIONS.md`
  - `docs/research/CURRENT_DIRECTION.md`
  - `docs/research/HANDOFF_CURRENT.md`
  - `docs/research/increments/INC_0074_product_phase_translation_dense_frontier.md`
  - `docs/research/increments/INC_0075_product_phase_translation_dense_quality_recovery.md`
  - `docs/research/increments/INC_0076_product_phase_translation_break_even.md`
  - `docs/research/increments/INC_0077_product_phase_translation_hardware_profile.md`
  - `docs/research/increments/INC_0078_product_phase_translation_crossover_map.md`
  - `docs/research/increments/INC_0079_product_phase_translation_large_bank_boundary_extension.md`
  - `docs/research/increments/INC_0080_product_phase_translation_second_large_bank_boundary_extension.md`
  - `docs/research/increments/INC_0081_product_phase_translation_q04_threshold_search.md`
  - `docs/research/increments/INC_0082_product_phase_translation_cost_accounting_audit.md`
  - `docs/research/increments/INC_0083_product_phase_translation_persistent_route_cache.md`
  - `docs/research/increments/INC_0084_product_phase_translation_warm_cache_onset_map.md`
  - `docs/research/increments/INC_0085_product_phase_translation_warm_cache_q01_bank_boundary.md`
  - `docs/research/increments/INC_0086_product_phase_translation_warm_cache_q01_lower_boundary_refine.md`
  - `docs/research/increments/INC_0087_product_phase_translation_warm_cache_q01_threshold_refine.md`
  - `docs/research/increments/INC_0088_product_phase_translation_warm_cache_q01_local_cost_audit.md`
  - `docs/research/increments/INC_0089_product_phase_translation_warm_cache_q01_2500_2600_refine.md`
  - `docs/research/increments/INC_0090_product_phase_translation_warm_cache_q01_2500_2525_refine.md`
  - `docs/research/increments/INC_0091_product_phase_translation_warm_cache_q01_2500_2505_refine.md`
  - `docs/research/increments/INC_0092_product_phase_translation_warm_cache_q01_floor_hardening.md`
  - `docs/research/increments/INC_0093_product_phase_translation_cache_residency_mix.md`
  - `docs/research/increments/INC_0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map.md`
  - `docs/research/increments/INC_0095_product_phase_translation_chart_resident_q01_bank_boundary.md`
  - `docs/research/increments/INC_0096_product_phase_translation_chart_resident_q01_packet_scope_audit.md`
2. Inspect:
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_focused_confirm.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_confirm.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_confirm_compare.json`
  - `docs/reports/INC0096_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_PACKET_SCOPE_AUDIT_CONFIRM_COMPARE.md`
  - `docs/research/increments/INC_0097_product_phase_sparse_gated_shell_pilot.md`
  - `results/analysis/inc0097_product_phase_sparse_gated_shell_screen.json`
  - `docs/reports/INC0097_PRODUCT_PHASE_SPARSE_GATED_SHELL_SCREEN.md`
  - `docs/research/increments/INC_0098_product_phase_translation_chart_resident_route_cost_decomposition.md`
  - `results/analysis/inc0098_product_phase_translation_chart_resident_route_cost_decomposition.json`
  - `docs/reports/INC0098_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_ROUTE_COST_DECOMPOSITION.md`
  - `docs/research/increments/INC_0099_product_phase_sparse_event_proxy_pilot.md`
  - `results/analysis/inc0099_product_phase_sparse_event_proxy_confirm.json`
  - `docs/reports/INC0099_PRODUCT_PHASE_SPARSE_EVENT_PROXY_CONFIRM.md`
  - `docs/research/increments/INC_0100_product_phase_sparse_event_translation_pilot.md`
  - `docs/research/increments/INC_0101_product_phase_hard_event_proxy_pilot.md`
  - `docs/research/increments/INC_0102_product_phase_near_hard_event_translation_pilot.md`
  - `docs/research/increments/INC_0103_product_phase_soft_sparse_translation_quality_recovery.md`
3. Resume with `INC-0128`.
