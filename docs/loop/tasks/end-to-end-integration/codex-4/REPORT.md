# REPORT: end-to-end-integration, codex-4

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic sh-spin-retrieval` was refused with exit 2 because
  the item is still claimed by this lab's codex-3 worktree (PR #4015, approved, in the merge queue).
  The claim is this lab's own; the unit adds only a test and docs, no dependency and no `Cargo.lock`
  change.
- **The exact `Z[φ]` comparator used the derived lexicographic coefficient order, not the real
  order.** A correct exact sign test would square cross-products near `2^111` and overflow `i128` at
  these magnitudes, so ordering now uses an f64 value of the exact score; the test asserts the
  minimum target/runner-up margin is **0.191**, far above f64 noise. Score equality remains exact.
- **The first retrieval construction was wrong.** Laying tokens on orbits `r^k · b` makes the
  relation between a token and its successor `b⁻¹ r b`, not `r`, so one learned relation cannot fit
  more than one orbit. The quaternion order also broke the wrap: for a rotation of order 5 the
  lift has `q⁵ = ±1`, and the orbit is ten quaternions, not five. The first run was discarded; the
  probe now uses shared poses `1, r, …, r⁴` (features distinguish the tokens) and tests the
  **rotation** order, accepting `q⁵ = ±1`.
- **The task is easy by design.** At the true relation the target's relative pose is the identity,
  so its score is `⟨a, a⟩`, the maximum any rotation can reach; precision 1.0 is therefore expected
  and is not the interesting number. What the probe tests is that the mechanism can carry the
  relation at all, and that the content controls cannot: their floor is one of the five tokens that
  share a feature.
- **The controls are content-only.** Popcount and L1 read the local feature and ignore the pose, so
  they tie within a feature orbit; popcount sits below the 1/5 floor (0.133) because four features
  share sign codes with four others, and L1 is exactly at it (0.200). A pose-aware read could solve
  this task; the claim is only about the reads the model actually has.
- Synthetic: 60 tokens, one hidden 2I relation, equal-norm integer features. No real text, no
  model, no token-level claim, and no `l ≥ 2` component.
- The fit enumerates 120 icosians (cheap, discrete, no gradient). It says nothing about fitting at
  scale or with a continuous relation outside 2I.
- **Ordering, not scoring, uses f64.** The scores are exact `Z[φ]` rationals; only their comparison
  is floating, and the 0.191 margin is the evidence that this cannot change the answer here.

## What changed

- `crates/uor-r4-integer/tests/sh_spin_probe.rs` gains one test,
  `a_planted_five_fold_relation_is_learned_and_beats_the_content_controls`, plus the helpers it
  needs (relation-adjusted pose, rotation order, equal-norm features, ranking). The existing three
  tests are unchanged.

Size: about 260 lines of test and the ORDER/REPORT.

## Measurements

- The best of the 120 relations on the 12 fit queries acts **identically** to the planted one on
  every token (exact score equality), and reaches **precision@1 1.000** on the 48 held-out queries
  (1.000 on all 60; planted 1.000).
- Controls: sign-code popcount **0.133**, local L1 **0.200** — at or below the 1/5 orbit-tie floor,
  far below the bar.
- A relation that differs on the data reaches **0.000** on the fit queries, so the result is not a
  constant-score artifact.
- Mean rank of the target **0.00**; the smallest target/runner-up gap **0.191**, so the f64 ordering
  is unambiguous.
- Runtime 0.13 s in the test profile; no exponent, no gradient.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --test sh_spin_probe` — `4` passed (the three earlier tests
  and the retrieval).
- `rustfmt --check` on the file — clean.
- `cargo clippy --locked --no-deps -p uor-r4-integer --test sh_spin_probe` — no warning in the file.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at the
  head.

## Skills used

- `solution-integrity`: the first construction was rejected for a real reason (the relation was
  conjugated by the base pose), and the rebuilt task's easy positive is stated as easy rather than
  sold as a capability result.
- `scope-guard`: one test file and docs; no source, schema, dependency or mechanism status.
- `test-freeze`: the file gains a test and changes no existing assertion; the commit carries a
  `TEST-CHANGE:` line.

## Still open

- The `l = 2, 3` spherical-harmonic ladder; the spin-1 rung and this retrieval probe are the only
  evidence so far.
- A harder retrieval task: a relation outside 2I, noisy features, and a query relation that must be
  inferred from context rather than fitted globally.
- The token-to-coefficient map and the served `Read` arm, with a fixed-width integer score.
- The real-state lane objective (codex-17 on the c387 states) — the bottleneck the claude note
  names; the states are not local to this checkout.

## Plain summary

A hidden five-fold rotation can be recovered exactly by trying all 120 icosians, and the spin score
then finds each token's successor with precision 1.0, while the model's content reads (popcount,
L1) sit at their 1/5 floor and a wrong relation scores zero. That is the cheapest evidence that the
spin representation carries a relation the current reads cannot — on a designed, easy task, with no
model claim.
