# ORDER: end-to-end-integration, codex-4

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-3](../codex-3/ORDER.md). Reviewed head: `79551bf0e`.

## Why this unit exists

The claude note on #2028 (2026-11-15T23:59Z) asks for the cheapest test that could rule out the
spin construction before building the `l = 2, 3` tables: a planted-symmetry retrieval probe against
popcount and L1 controls. The exact spin-1 rung is merged (#4013); this unit asks whether it carries
information the current content reads cannot.

The task: a vocabulary of tokens on orbits of a hidden 5-fold relation `r ∈ 2I`. Each token has a
pose `u` and a local feature `a`; its successor has pose `r·u` and the **same** feature. The read is
given the query token and must find its successor among all tokens. The spin score uses a learned
relation pose `r̂`:

```
s(q, j) = ⟨ a_q,  (r̂⁻¹ · u_q⁻¹ · u_j) · a_j · (r̂⁻¹ · u_q⁻¹ · u_j)⁻¹ ⟩
```

At `r̂ = r` the successor's relative pose is the identity, so its score is `⟨a, a⟩`, the maximum for
any rotation (Cauchy–Schwarz, equality only at the identity). The controls are the current content
reads: sign-code popcount and L1 over the **local** features, which are identical within an orbit
and therefore cannot pick one member.

The relation is fitted by enumerating all 120 icosians and keeping the one with the best held-out
precision — the discrete-learning rule (try every legal rotation, keep the one that lowers the
loss), with no gradient, no float and no softmax.

## Claim record

`loop claim end-to-end-integration --lab-topic sh-spin-retrieval` was refused with exit 2 because the
item is already claimed by this lab's codex-3 worktree (PR #4015, approved, in the merge queue). The
claim is this lab's own and the unit touches only a new test file; it continues under that claim,
adds no dependency and does not change `Cargo.lock`.

## Method

- Exact `Z[φ]` arithmetic and the 120 quaternions, as in codex-2.
- One order-5 element `r`; twelve orbits of five tokens (60 tokens), each orbit a distinct base pose
  and a distinct integer feature not fixed by any non-identity power of `r`.
- Fit on one query per orbit (12), test on all 60.
- Metrics: precision@1 and mean rank for the spin score, sign-code popcount, and local L1.
- Pass: the fitted relation reaches test precision@1 ≥ 0.9 and at least 0.2 above both controls;
  a wrong relation is worse; the controls are near the orbit-tie floor (~1/5).

## Files in scope

- `crates/uor-r4-integer/tests/sh_spin_retrieval_probe.rs` (new file)
- `docs/loop/tasks/end-to-end-integration/codex-4/**`

Out of scope: source, schema, the `l ≥ 2` tables, the token-to-coefficient map, serving, any
dependency or GPU run, and any claim about a model.

## Done when

1. The fitted `r̂` is the planted relation (or an element that acts identically on the data), and it
   reaches the pass bar on held-out queries.
2. The controls are near the orbit-tie floor and below the bar.
3. A wrong relation is measurably worse, so the result is not a constant-score artifact.
4. The test passes; `git diff --check`, the harness contract rule, the REPORT check and the
   exact-head review pass.

## Cancel

Stop if the task needs floats in the score, a source change, or a trained model. If the spin score
does not beat the controls, report the negative and stop; do not tune the task after seeing it.

STOP and report when done.
