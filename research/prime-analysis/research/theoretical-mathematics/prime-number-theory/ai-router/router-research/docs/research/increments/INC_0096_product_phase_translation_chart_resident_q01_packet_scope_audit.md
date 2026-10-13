# INC-0096: Product Phase Translation Chart-Resident Q01 Packet-Scope Audit

## Status
Confirm completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0095` broadens the chart-resident single-query story more strongly than
expected:
- the focused `Q01` packet now crosses already at `T2500`
- but the earlier `INC-0094` mixed-repeat packet still had `T2500 Q01` slightly
  negative

That means the next honest question is not bank location. It is whether the
lower-bank chart-resident `Q01` claim is stable across packet composition.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- hold cache state to chart-resident / route-ephemeral only:
  - `cache_chart=1`
  - `cache_routes=0`
- keep the lower-bank focus fixed at `T2500`
- use `T40000 Q01` only as a stable high-bank control if needed
- do not reopen bank search, full-warm search, or route-law tuning

## Minimal Scope
1. Carry only:
   - `DENSE`
   - `CHART_H4XH4_FIELD_A150_CPX8`
2. Compare packet shapes at the fixed lower bank:
   - focused `Q01`-only packet
   - mixed-repeat packet containing `Q01/Q02/Q04`
3. Use the same chart-only residency contract on every packet.
4. Treat this as workload-composition audit, not as another hardware frontier
   search.

## Stop Rule
- If `T2500 Q01` survives under both focused and mixed packet shapes on a
  hardened schedule, promote a stable chart-resident lower-bank single-query
  claim and stop packet-scope auditing.
- If focused `Q01` survives but mixed packet still misses, narrow the lower-
  bank chart-resident claim to focused single-query sessions and record the
  workload-mix sensitivity explicitly.

## Resume Note
Resume from the `INC-0096` confirm artifact and treat the next branch as the
reopened sparse / quantized phase-gated shell pilot, with the translated
chart-resident stack kept as the fixed systems reference.

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_prewarm_screen.json`
  - `configs/proxy_transfer_inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_focused_screen.json`
  - `configs/proxy_transfer_inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_screen.json`
  - `configs/proxy_transfer_inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_prewarm_confirm.json`
  - `configs/proxy_transfer_inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_focused_confirm.json`
  - `configs/proxy_transfer_inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_confirm.json`
- Analyses:
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_focused_screen.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_screen.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_focused_confirm.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_confirm.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_screen_compare.json`
  - `results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_confirm_compare.json`
- Reports:
  - `docs/reports/INC0096_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_PACKET_SCOPE_AUDIT_FOCUSED_SCREEN.md`
  - `docs/reports/INC0096_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_PACKET_SCOPE_AUDIT_MIXED_SCREEN.md`
  - `docs/reports/INC0096_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_PACKET_SCOPE_AUDIT_FOCUSED_CONFIRM.md`
  - `docs/reports/INC0096_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_PACKET_SCOPE_AUDIT_MIXED_CONFIRM.md`
  - `docs/reports/INC0096_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_PACKET_SCOPE_AUDIT_SCREEN_COMPARE.md`
  - `docs/reports/INC0096_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_PACKET_SCOPE_AUDIT_CONFIRM_COMPARE.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_092114.md`
  - `docs/governance/gates/gate_20260312_092121.md`
  - `docs/governance/gates/gate_20260312_092137.md`
  - `docs/governance/gates/gate_20260312_092238.md`
  - `docs/governance/gates/gate_20260312_092252.md`
  - `docs/governance/gates/gate_20260312_092334.md`

## Screen Read
- The fresh paired 2-seed screen did not reproduce the old mixed-packet sign
  flip.
- Focused packet:
  - `DENSE_Q01_T2500`: `top1=0.051600`, `amortized=0.1572s`
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044400`,
    `cand_frac=0.189016`, `amortized=0.1347s`
  - margin vs dense: `+0.0226s`
- Mixed packet:
  - `DENSE_Q01_T2500`: `top1=0.051600`, `amortized=0.1215s`
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044400`,
    `cand_frac=0.189016`, `amortized=0.0647s`
  - margin vs dense: `+0.0569s`
- That justified hardening both packets rather than carrying only the
  contradictory one.

## Confirm Read
- The 8-seed hardening confirm closed the packet-scope question positively.
- Focused packet:
  - `DENSE_Q01_T2500`: `top1=0.050300`, `amortized=0.1155s`
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.046300`,
    `cand_frac=0.198723`, `amortized=0.0807s`
  - margin vs dense: `+0.0348s`
- Mixed packet:
  - `DENSE_Q01_T2500`: `top1=0.050300`, `amortized=0.0946s`
  - `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.046300`,
    `cand_frac=0.198723`, `amortized=0.0871s`
  - margin vs dense: `+0.0075s`
- Shared packet-scope invariants:
  - routed `top1` is identical across focused and mixed packets
  - routed `cand_frac` is identical across focused and mixed packets
  - `chart_cache_hit=1.0`
  - `route_cache_hit=0.0`
- The only packet effect that survives hardening is amortized margin size:
  - focused minus mixed margin gap = `+0.0273s`
  - but both packets stay system-positive

## Reading
- The lower-bank chart-resident single-query claim is now stable across packet
  composition below the ideal fully warm case.
- The old `INC-0094` mixed-repeat `T2500 Q01` miss does not survive the paired
  hardening audit.
- Packet composition still affects the size of the systems margin, but not the
  sign of the claim and not the retrieval signal.
- That clears the deferred gate in the route matrix:
  - the translated chart-resident lower-bank single-query story is stable
  - packet-scope auditing can stop
  - the next live branch can reopen sparse / quantized phase-gated shell work

## Decision
- Close `INC-0096` confirm positive/explanatory.
- Promote `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500` as the stable chart-resident
  lower-bank single-query point across both focused and mixed packet shapes.
- Retire packet-scope auditing on this translated lower-bank point.
- Keep the fixed translated chart-resident stack as the hardware-side systems
  reference.
- Move next to reopened sparse / quantized phase-gated shell work (`INC-0097`).
