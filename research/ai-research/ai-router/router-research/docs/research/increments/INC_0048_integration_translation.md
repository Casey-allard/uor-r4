# INC-0048: Integration Translation

## Status
In progress.

## Trigger
`INC-0047` showed that the cheap routed frontier survives near-full-proxy scale.

## Objective
Translate the current routed frontier into a more model-like integration path so the project stops proving itself only inside the proxy harness.

## Chosen First Path
- Routed retrieval cache over token-memory items.

## Minimal Scope
1. Keep the current routed winners fixed:
   - `HOPF_K25_BASE_IT40_P2_STATIC`
   - `HOPF_PHI2_BAND_IT40_P2_STATIC`
2. Translate them into routed retrieval preselection over LM proxy token-memory items.
3. Use exact dense retrieval as the control.
4. Build only the smallest viable harness that can compare routed vs control on that path.
5. Preserve the same structured logging, analysis, and gate-note contract.

## Decision Rule
- Keep the translation path only if it preserves a meaningful fraction of the current routed systems advantage while staying coherent with the project’s geometric mechanism.
- If the translation path fails immediately for engineering reasons, tighten the harness rather than reopening geometry.

## Planned Artifacts
- Design note:
  - `docs/research/INTEGRATION_TRANSLATION_PLAN.md`
- Config or task harness:
  - `tasks/router_retrieval_eval.py`
  - `configs/proxy_transfer_inc0048_retrieval_translation_screen.json`
- Analysis:
  - `results/analysis/inc0048_*.json`
- Gate:
  - `docs/governance/gates/gate_*.md`

## Active Next Command
- `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0048_retrieval_translation_screen.json`
