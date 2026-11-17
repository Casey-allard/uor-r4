# ORDER — written-state-count-model (claude-418)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-418. Task: `written-state-count-model`, P0.5 (R\*), part 1 of 3: the adjusted counts
that interpolated modified Kneser–Ney needs, on the GPU counter of claude-412 and claude-415.

## Goal

R\* is the sealed count reference that bar Q is measured against (0.80 × R\*). It is
interpolated modified Kneser–Ney with three discounts per order and full N1/N2/N3+ type counts,
unpruned. Its lower orders use *continuation* counts: how many distinct tokens precede a gram,
not how often the gram occurs. The counter has raw counts, per-context follower census and raw
count-of-counts. It has no continuation counts.

This unit adds the **adjusted counts** and their census, exactly, on the counter's own keys.
The next unit, claude-419, is the scorer. claude-420 is the sheet, the FIT and the sealed run.

## The rule (deterministic; integers only)

Orders use the counter's numbering: an order-k gram is k context tokens plus the target, and N is
the highest order R\* uses.

1. **Adjusted count `a(g)`** of a counted gram `g` of order k:
   - `k = N` (the highest order): `a(g) = c(g)`, the raw count;
   - `k < N`, and the oldest token of `g` is BOS (id 0, allowed only in the oldest slot): `a(g) = c(g)`;
   - `k < N` otherwise: `a(g) = N1+(•g)`, the number of distinct tokens `v` such that the
     order-(k+1) gram `v g` is counted under the BOS rule of claude-415. Each counted order-(k+1)
     triple whose `PrimeRouteKey::back_off(key)` and `next` equal `g`'s is one such `v`.
   - The BOS case follows the standard treatment of the start symbol: a gram that starts a
     document has no admitted left extension, so its count stands in.
2. **Adjusted census per context** `h` of order k: `total = Σ_w a(h w)`, `followers` (unchanged),
   and `n1`, `n2`, `n3_plus`, the followers whose adjusted count is exactly 1, exactly 2, and 3
   or more.
3. **Adjusted count-of-counts per order**: the number of order-k grams with `a(g)` exactly 1, 2,
   3 and 4. They are the inputs of the modified-KN discounts
   `D_i = i − (i+1)·Y·n_{i+1}/n_i` with `Y = n1/(n1 + 2·n2)`, which claude-419 computes.
4. Every counted gram with `k < N` and no BOS in its oldest slot has `a(g) ≥ 1`. The run refuses
   a gram whose adjusted count would be 0. That cannot happen, since an admitted gram `g` with
   `j − k − 1 ≥ 0` and no BOS in its oldest slot has an admitted left extension.
   - Grams at stream position `j = k` have no token before them. They can only begin with the
     stream's first token, which is BOS by the UORT layout, so they fall under the BOS case.
   - The run checks `tokens[0] == BOS`.

## Partitions

- Orders `k ≥ 1` and `k + 1` both partition on `t[j−1]`, and `back_off` keeps that slot. So an
  order-k partition and the order-(k+1) partition over the same token range hold exactly the
  grams one needs.
- A plan built from order k's histogram fits both orders: order k admits at least the targets of
  order k + 1.
- Order 0 partitions on `t[j]`. Its adjusted counts (4,096 at most) are summed over all order-1
  partitions.

## Scope

- `crates/uor-r4-discrete/src/ngram_count.rs` and the device and oracle files beside it:
  - an `NgramAdjusted` assembler that takes an order-k partition and the order-(k+1) partition
    over the same token range, and gives the adjusted triples, the adjusted contexts and the
    adjusted count-of-counts, each with its SHA-256;
  - an order-0 path;
  - a device entry that counts orders k and k + 1 under one plan;
  - an independent host oracle that computes `a(g)` from the tokens by its definition: a set of
    distinct left tokens per gram, not via `back_off` on triples.
- Tests on the host:
  - a hand fixture whose adjusted counts are worked out in the test's comments, including a BOS
    case and a gram with two distinct left extensions seen three times;
  - assembler = oracle on random streams with BOS planted, at several budgets;
  - the zero-adjusted-count refusal.
- Tests under CUDA (`--features cuda`, on wick):
  - device = oracle on a synthetic stream;
  - on the real D87 TRAIN stream of claude-415 (`UOR_NGRAM_STREAM`, ignored by default), device =
    oracle for orders 0..3 over a slice and stable SHA values across two budgets and two runs.
- The probe reports the adjusted count-of-counts and timings for orders 0..8 on the real stream.

This unit builds an instrument and runs finite kernel tests and a counting probe. It scores no
model and evaluates nothing, so it needs no pre-registration sheet (§4.5).

## Gates

- G1: the host tests of `uor-r4-discrete` pass, the new ones included.
- G2: set-aside negatives, each passing again when restored:
  - with the BOS case set aside (`a(g) = N1+` for every `k < N`), the BOS fixture test fails;
  - with the distinct-`v` grouping replaced by a sum of raw counts, the hand fixture fails.
- G3: `uor-harness loop pr-check --local` at the committed head reads `verdict: PASS`.
- G4: on wick, the CUDA tests pass, and the real-stream test gives device = oracle with SHA
  values equal across budgets and runs. The CUDA sources are touched before the build (the
  stale-cubin trap of claude-415).
- G5: no corpus text, no token stream and no binary is committed.
