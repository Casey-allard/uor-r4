# ORDER — written-state-count-model (claude-419)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-419, P0.5 (R\*), part 2 of 4: the integer interpolated modified Kneser–Ney scorer
on the host, over claude-418's adjusted counts. claude-420 builds the split streams, filtered
counts and the GPU scorer; claude-421 is the sheet, FIT and sealed run.

## The rule (integers only; M = 2^48)

Order k has k context tokens; N is the highest order. For each order, count-of-counts n1..n4
come from claude-418 (adjusted below N, raw at N).

1. **Discounts per order**, quantized once: with S = n1 + 2·n2,
   `Dq_i = floor(M · (i·n_i·S − (i+1)·n1·n_{i+1}) / (n_i·S))` for i = 1, 2, 3.
   Refuse unless `0 ≤ Dq_i ≤ i·M` and n1..n4 > 0.
2. **Mass of target w after context h of order k** (h: the k tokens before w), with
   `a = a(hw)`, `A = Σ_v a(hv)` and `n1(h), n2(h), n3+(h)` from the context record:
   - `D(a) = Dq_min(a,3)` for a ≥ 1, and `D(0) = 0`;
   - `γ = floor((Dq_1·n1(h) + Dq_2·n2(h) + Dq_3·n3+(h)) / A)`;
   - `P_k = floor((a·M − D(a)) / A) + floor(γ · P_{k−1} / M)`;
   - if h is unseen (A = 0) or order k does not admit the target under claude-415's BOS
     rule, `P_k = P_{k−1}`;
   - `P_{−1} = M / 4096`, the uniform base.
3. The scored mass is `P_N`; bits come from `lawful_likelihood::bits_q12`.

## Lawfulness

In exact arithmetic the masses sum to M for every context: the discounted terms sum to
`A·M − Σ_v D(a(hv))`, and γ's numerator equals that deficit. Only the floors remove mass. The
scorer therefore gives a **sub-distribution**: every `P_N ≤` the exact value. Each level loses
less than 3 units: the discounted term's floor, γ's floor scaled by `P_{k−1}/M ≤ 1`, and the
outer floor. So the per-target deficit is below `3·(N + 1)` units of 2^-48. The unit proves this
on fixtures by summing all 4,096 masses.

## Scope

- `crates/uor-r4-discrete/src/kn_score.rs` (host; no CUDA in this unit):
  - `KnDiscounts::from_count_of_counts`;
  - a `KnTables` view over each order's adjusted triples and context records, using binary
    search on the counter's canonical order;
  - `score_target(tokens, j) -> u64` (Q48);
  - `total_bits_q12` over a stream.
- An independent CPU oracle in `kn_score_oracle.rs`. It counts with `oracle_count` and
  `oracle_adjusted`, holds `BTreeMap` tables, and applies the rule above from the token stream.
- Host tests:
  - a hand fixture worked out in comments;
  - discount refusals;
  - Σ of all 4,096 masses in `[M − 3(N+1)·4096, M]` for every context of a seeded stream;
  - scorer = oracle on seeded streams (orders 1..4);
  - order 0 = unigram KN against uniform;
  - an unseen context falls back to order k − 1.

No GPU run, no evaluation and no milestone claim, so no pre-registration sheet (§4.5).

## Gates

- G1: the host tests of `uor-r4-discrete` pass, the new ones included.
- G2: set-aside negatives, each failing, then restored:
  - γ set to 0, so the masses stop summing (the sum test fails);
  - discounts taken from the raw count-of-counts below N (scorer ≠ oracle);
  - no BOS-rule fallback (the hand fixture fails).
- G3: `uor-harness loop pr-check --local` reads `verdict: PASS`.
- G4: no corpus text, token stream or binary is committed.
