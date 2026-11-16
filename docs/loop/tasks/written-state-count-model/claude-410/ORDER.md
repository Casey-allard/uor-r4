# ORDER: written-state-count-model, claude-410 (a full-support floor for PRLL: count, order-0 and uniform experts mixed into strictly positive Q48 masses)

Lab: claude. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (track: written-state-count-model, no item of its own yet).
Base: `origin/main` `ef1f4a641`.

## Why this unit

The project is moving to a written-state count model, and its bits per byte are scored by the potential-ratio lawful likelihood (PRLL) in `crates/uor-r4-integer/src/lawful_likelihood.rs`, under the owner-approved sheet `docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md` and the design note `docs/labs/m4-lawful-likelihood-bpb/README.md` (owner answer: a linear ratio).

The red team found two problems with scoring a count model through PRLL today:

- `masses_from_scores` shifts every score by the minimum (`phi = s - min(s) + 1`), so a uniform floor added to the scores cancels out and does nothing.
- `mle_masses_from_counts` gives zero mass to a target the counts never saw, and `bits_q12` refuses a zero mass, so a count model cannot score an unseen target at all.

The fix, DECISION §4 P0 item 5: a full-support floor expert, scored through the existing no-shift `mle_masses_from_counts`, with every score at least 1. This unit builds that one function and its tests. It is a finite host unit: no model, no corpus, no run. Under §4.5 it needs no sheet.

## Native-space items (D85)

1. **State schema.** None added. The inputs are the count model's own integer counts (`&[u64]`), an order-0 count vector of the same length and three integer weights. The output is the existing Q48 mass vector that `sampling` and `bits_q12` take.
2. **Shared operators.** The one ratio operator already in the file, `mle_masses_from_counts`, is used three times: once per expert and once for the mixture. No second normaliser is written.
3. **Replay oracle.** The function is pure integer and deterministic: the same inputs give the same masses bit for bit, with ties keyed by the lowest token id (the existing largest-remainder rule). The hand-computed fixture (test T1) is the oracle.
4. **Forward, gradient, update.** Scoring only. Nothing is trained; no gradient or update path changes.
5. **Serialization.** None. The weights are caller constants; the masses are not stored.

## Design

`floor_mixed_masses(counts, unigram, weights) -> Result<Vec<u64>>`, with `weights` three integer numerators (`count`, `unigram`, `floor`) over the common denominator `count + unigram + floor`.

1. Each expert becomes exact Q48 masses through `mle_masses_from_counts`: the counts, the order-0 counts, and the uniform floor (one count per candidate). An expert whose weight is 0, or whose counts are all 0 (an unseen context), contributes nothing.
2. The mixed score of candidate `i` is `count * mc[i] + unigram * mu[i] + floor * mf[i]`, in `u128`, refused if it does not fit the `u64` that `mle_masses_from_counts` takes. The scores sum to `d' * 2^48` with `d'` the sum of the contributing weights, so the second pass divides by `d'` exactly: there is no shift, and the floor's share is never cancelled.
3. The mixed scores go through `mle_masses_from_counts` once more, giving masses that sum to `2^48` exactly.
4. Every mass is checked to be at least 1; a zero mass is refused (`MassZero`), so an `Ok` result always gives every target a finite `bits_q12`. With `floor >= 1` the floor expert gives every candidate at least `floor(2^48 / V)` units, so the refusal happens only if the floor's share is below one Q48 unit.

Integer only: add, multiply, one exact `u128` ratio per candidate per pass, and the existing largest-remainder split. No float, no exponential, no log table, no normalised exponential (CHARTER §3.2), no matmul.

## Tests

- **T1.** Hand-computed fixture: `V = 4`, counts `[3, 1, 0, 0]`, order-0 counts `[5, 3, 1, 1]`, floor weight 1, weights `(1, 2, 1)`. The exact integer masses and the exact `total_bits_q12` for a 3-target sequence, worked by hand in a comment.
- **T2.** Every mass is at least 1 for every target, including the count-0 targets, and each has a finite `bits_q12`.
- **T3.** With the floor and order-0 weights 0 and full-support counts, the result equals `mle_masses_from_counts(counts)` exactly.
- **T4.** Counts near `u64::MAX` and weights near `u64::MAX` return `Ok` (exact) or `Err` as documented, never a panic.
- **Negative check.** With the floor term set aside as a patch, T2 fails; restored, it passes.

## Files in scope

- `crates/uor-r4-integer/src/lawful_likelihood.rs`: the new function, its weights type, two error variants and the tests;
- `docs/loop/tasks/written-state-count-model/claude-410/**`;
- one row in `docs/research/notebook/m4-serving.md`.

## Stop list

- `masses_from_scores`, `mle_masses_from_counts`, `bits_q12`, `total_bits_q12` and `bits_per_byte` are unchanged in behaviour; their tests are unchanged.
- No count model, no corpus read, no bits-per-byte measurement on real text, no GPU.
- No backoff hierarchy beyond the three experts, no learned weights.

## Done when

1. T1 to T4 pass in `cargo test -p uor-r4-integer lawful_likelihood`, and the full `uor-r4-integer` suite has no new failure against `origin/main`.
2. The negative check fails with the floor set aside and passes after the restore and a rebuild.
3. The REPORT gives the fixture numbers and what they do not show.
4. `loop pr-check --local` passes, or its output is in the REPORT.
