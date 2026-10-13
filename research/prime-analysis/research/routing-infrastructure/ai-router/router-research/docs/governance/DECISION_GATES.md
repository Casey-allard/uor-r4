# Decision Gates

## Auto-proceed gates
Proceed automatically when all are true:
- parser success rate >= 95% for new runs
- no regression > 5% on `test_mse_after` vs current reference route
- wall-clock improves or remains within 3% while quality improves

## Mandatory user-escalation gates
Create a gate note and pause branch promotion when any are true:
- > 10% quality swing in either direction on finalize stage
- > 30% runtime regression on the best-known route
- new route beats baseline by >= 8% quality and >= 15% runtime (major branch opportunity)
- LM transfer branch improves quality/runtime but still shows concentration collapse (`pmax_after >= 0.70` or `eval_shells < 2` or `shell_pmax >= 0.85` or `test_unseen_rate > 0.01`)
- parser/schema failure rate >= 20%
- two leading branches are close enough that noise cannot separate them but they imply different architecture directions
- a result looks promising but the mechanism is unclear enough that the next step would be a guess rather than a hypothesis

## Seed-Wise Transfer Rule
- For multi-seed LM proxy confirm/finalize, mean route-health is not sufficient.
- If `enforce_seed_health=true`, the same shell/bucket/unseen thresholds must hold on every seed in the batch.
- Any seed-wise shell collapse or shell concentration breach blocks promotion even when the route mean still looks healthy.

## Outputs
- Gate notes written to `docs/governance/gates/gate_<timestamp>.md`
- Each note includes recommended path and explicit rationale.
