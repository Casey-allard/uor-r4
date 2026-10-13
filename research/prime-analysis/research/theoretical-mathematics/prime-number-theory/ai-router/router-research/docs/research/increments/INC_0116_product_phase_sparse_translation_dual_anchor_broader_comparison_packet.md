# INC-0116: Product Phase Sparse Translation Dual-Anchor Broader Comparison Packet

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0115` closed the carry-forward contract for the fixed sparse translated
dense stack:
- lower-bank default routed point:
  `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
- upper-bank promoted routed point:
  `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- upper-bank bounded-backfill point remains comparator-only

The next honest question is whether later broader hardware-side or task-side
evaluation can now use a single dual-anchor default packet instead of reopening
old lower-bank or upper-bank route forks.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the lower-bank dense read frozen as systems-only
- keep the promoted upper-bank soft sparse point as the sole default
  upper-bank routed representative
- keep the upper-bank bounded-backfill point available only as an explicit
  comparator
- do not reopen the old upper-bank pair or the lower-bank pruning-only point
  unless a new branch finds a real regression against the default packet

## Minimal Scope
1. Use the `INC-0115` carry-forward contract as the default broader comparison
   packet.
2. Record the exact default route ids that later task-side or hardware-side
   branches must inherit.
3. Only reintroduce nondefault comparators when the later branch explicitly
   needs them.

## Acceptance
- future broader comparison work defaults to the dual-anchor packet from
  `INC-0115`
- the old upper-bank pair and lower-bank pruning-only point no longer reappear
  silently in later packets

## Evidence
- Analyses:
  - `results/analysis/inc0115_product_phase_sparse_translation_promoted_upper_bank_carry_forward.json`
  - `results/analysis/inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.json`
- Packet manifest:
  - `configs/packet_inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison.json`
- Report:
  - `docs/reports/INC0116_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_BROADER_COMPARISON_PACKET.md`

## Reading
- The `INC-0115` contract now resolves to one reusable dual-anchor packet
  artifact rather than a narrative-only handoff:
  - packet id:
    `inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet`
  - mode:
    `dual_anchor_default_packet`
- Default inherited route ids are now frozen in one place:
  - lower bank:
    - `DENSE_Q01_T2500`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500`
  - upper bank:
    - `DENSE_Q01_T40000`
    - `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
- The packet manifest now records the exact resolved route args inherited from
  the source configs:
  - lower-bank source:
    `configs/proxy_transfer_inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json`
  - upper-bank source:
    `configs/proxy_transfer_inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json`
- Nondefault routes are now explicitly separated:
  - optional comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000`
  - excluded-by-default lower-bank route:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`
- This closes the branch positive/explanatory:
  - later broader comparison work now has an exact packet manifest to inherit
  - the old route forks no longer need to be reconstructed ad hoc

## Resume Note
Resume from the completed `INC-0116` packet manifest. The next branch should
use that exact dual-anchor packet and only add comparators when the broader
comparison question explicitly requires them.
