# Independent adversarial review record

Reviewer: /root/independent_systems_review. Recorded by root from the returned review; reviewer did not author the proposal. Source inspected: PR1518 exact head1b88c563090329f6d21b8c0105a7a6e0146143f9. No tests or model execution.

Verdict: formulas and counterexample correct at stated seam, with these incorporated clarifications:

- C=query_position+1 is causal prefix length; m is actual deduplicated support, not configured top-k.
- w1=1/(H_m+B)<=1/H_m bounds first SELECTED rank only. Strict when C>m and background positive. Orthogonal background can use repeated directions and is realizable.
- Positive radial invariance requires finite scaled/unscaled norms at or above epsilon; crossing the fallback threshold breaks it. Stored norm diagnostics are not attention inputs in the inspected projection path.
- The equal-weight two-example scalar MSE optimum is(a-b)^2/4 with identical normalized directions and fixed values/support/other inputs. It does not bound the whole learned model or subsequent compensation.

Recommended minimum diagnostics: per row/head C,m,H_m,B,B/H_m, selected mass and first-selected weight; teacher selected mass and attended-output error; a frozen radial pair with fallback flags/F32 tolerance; dense versus recurrent numerator/mass on identical rows to expose cancellation. Keep current gates unchanged and do not infer trained failure from these examples.
