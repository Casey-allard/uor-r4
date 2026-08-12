# INC-0054: Tangent-Flow Route Law Pilot

## Status
Closed.

## Trigger
`INC-0050` Slice A confirm showed that the tangent surrogate `H^4 + T_xH^4` beats static `H^4` on the main proxy MSE objective while staying slightly faster.

## Hypothesis
The cheapest next architectural step is not a full `H^4 x H^4` rewrite.
It is a route law that lets tangent-flow state influence shell/sector allocation or retrieval locality while keeping the current Hopf geometry as the position branch.

## Minimal Scope
1. Keep position geometry fixed to the current Hopf lead.
2. Inject a low-rank flow term derived from sequential past context change.
3. Test whether flow-aware routing or retrieval narrows neighborhoods more cleanly than the static branch.
4. Do not rewrite the whole router manifold until the pilot proves signal.

## Candidate Mechanisms
- shell bias from flow radial component
- sector tie-break or local band choice from flow angular component
- flow-aware retrieval metric on top of static route buckets

## Acceptance
- beats static dynamic baseline on proxy MSE or translated retrieval efficiency
- does not destroy current route health or alignment

## Artifacts
- Config:
  - `configs/proxy_transfer_inc0054_tangent_flow_route_law_screen.json`
- Analysis:
  - `results/analysis/inc0054_tangent_flow_route_law_screen.json`
- Gate notes:
  - `docs/governance/gates/gate_20260306_124108.md` (invalid first run; baseline failed to emit summary due evaluator bug)
  - `docs/governance/gates/gate_20260306_124322.md` (corrected rerun)

## Result
Corrected 2-seed screen means:
- `STATIC_GLOBAL`: `mse=0.004315002`, `top1=0.02400`, `total=7.674s`, `cand_frac=1.0000`
- `STATIC_BUCKET`: `0.004329264`, `0.02317`, `7.196s`, `0.3408`
- `TXH4_BUCKET_W050`: `0.004320435`, `0.02717`, `7.275s`, `0.3408`
- `TXH4_BUCKET_W025`: `0.004324626`, `0.02583`, `8.029s`, `0.3408`
- `H4XH4_BUCKET_W025`: `0.004318685`, `0.03300`, `8.855s`, `0.3408`

## Reading
- Static Hopf buckets are a real locality structure:
  - candidate fraction dropped to about `0.34`
  - bucket fallback stayed at `0.0`
- That locality cut alone is not enough:
  - `STATIC_BUCKET` was faster than `STATIC_GLOBAL`, but MSE regressed
- Tangent flow partially repaired the same-bucket loss:
  - `TXH4_BUCKET_W050` was the best bucketed runtime branch
  - but it still did not beat `STATIC_GLOBAL` on the main MSE objective
- Product `H^4 x H^4` stayed live because it had the best top-1 under bucketed retrieval:
  - this matches the prior reading that the product branch is more retrieval/discrete-decision oriented than regression oriented

## Decision
- Close `INC-0054` without confirm.
- Keep the result as positive locality evidence, not as a promoted dynamic route law.
- Promote `INC-0055` next:
  - treat the second hyperbolic factor as a retrieval field candidate
  - explicitly consider discrete complex / imaginary route-key storage inside that second field
