# INC-0102 Screen

- Configs:
  - `configs/proxy_transfer_inc0102_product_phase_near_hard_event_translation_prewarm_screen.json`
  - `configs/proxy_transfer_inc0102_product_phase_near_hard_event_translation_screen.json`
- Analyses:
  - `results/analysis/inc0102_product_phase_near_hard_event_translation_prewarm_screen.json`
  - `results/analysis/inc0102_product_phase_near_hard_event_translation_screen.json`
- Gate notes:
  - `docs/governance/gates/gate_20260312_115408.md`
  - `docs/governance/gates/gate_20260312_115418.md`

## Read
- The translated near-hard candidate preserved the same routed retrieval signal
  as the continuous and soft sparse references:
  - `top1=0.0444`
  - `cand_frac=0.189016`
- But it lost the translated systems tradeoff:
  - continuous translated product:
    - `online=0.08139s`
    - `amortized=0.11841s`
  - translated soft sparse reference:
    - `online=0.11701s`
    - `amortized=0.14575s`
  - translated near-hard candidate:
    - `online=0.21561s`
    - `amortized=0.26344s`
    - `event_gate_mean=0.02097`

## Decision
- Close the branch negative at screen stage.
- Keep translated sparse-event claims explicitly soft.
- Return the queue to bounded quality recovery on the fixed soft sparse
  translated point.
