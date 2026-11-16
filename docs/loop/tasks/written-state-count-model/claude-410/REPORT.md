# REPORT: written-state-count-model, claude-410 (a full-support floor for PRLL: count, order-0 and uniform experts mixed into strictly positive Q48 masses)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Nothing was measured on real text** [claude]. This is a host instrument unit. No count model was built, no corpus was read, and no bits-per-byte number is claimed. The weights in the tests are fixtures, not tuned values.
- **The plain `cargo test -p uor-r4-integer` does not compile its integration tests in this worktree** [claude]. They fail with E0432 on unresolved imports of `stack`, `bundle`, `model`, `session` and `uor_r4_tokenizer`. This diff touches none of them. The lib suite is the one that runs here: 189 / 189.
- **A floor weight ≥ 1 does not by itself guarantee a nonzero mass** [claude]. The first REPORT said it did. The fresh review found a counter-example, now tested in T4. At `V = 2^17`, all counts 1 except one 0, with weights `(2^32, 0, 1)`, the floor's share on the unseen target rounds to zero, and the function returns `Err(MassZero)`. It refuses; it never returns a zero. The guarantee holds when `floor * floor(2^48 / V) >= weight sum`. That is so for every weight sum under `2^16` and `V <= 2^32`. Callers that choose weights must stay inside it.
- **Two rounding stages** [claude]. Each expert is rounded to the Q48 grid before mixing, so the result can differ from a one-pass exact mixture by a few `2^-48` units. It is still deterministic and integer.
- **The ORDER first named a roadmap item that does not exist** (`written-state-count-model` has no item page) [claude]. The Item line now points to `m4-lawful-likelihood-bpb`, the item PRLL belongs to. The ORDER is 5,370 B, over the 4,096 B advisory budget.

## What changed

- `crates/uor-r4-integer/src/lawful_likelihood.rs` [claude]:
  - `pub struct FloorMixWeights { count, unigram, floor }`, numerators over their sum;
  - `pub fn floor_mixed_masses(counts, unigram, weights) -> Result<Vec<u64>>`;
  - a private `add_expert`;
  - the error variants `LengthMismatch` and `ScoreOverflow`;
  - four tests, T1–T4.

  `masses_from_scores`, `mle_masses_from_counts`, `bits_q12`, `total_bits_q12` and `bits_per_byte` are unchanged.
- How the masses are built [claude]:
  - Each expert goes through `mle_masses_from_counts`. The uniform floor counts one per candidate.
  - An expert with weight 0 or all-zero counts drops out, so an unseen context backs off to the others.
  - The mixed `u128` score `count*mc + unigram*mu + floor*mf` is refused if it does not fit u64, then goes through `mle_masses_from_counts` again.
  - Nothing is shifted, so the floor is not cancelled (the red team's finding on `masses_from_scores`). Any zero mass is refused.
- `docs/research/notebook/m4-serving.md`: one row [claude].

## Measurements

- **T1 fixture**, worked by hand in the test comment: `V = 4`, counts `[3,1,0,0]`, order-0 `[5,3,1,1]`, weights `(1,2,1)` [claude].
  - Masses `[140737488355328, 77405618595431, 31665934879949, 31665934879948]`, shares 1/2, 0.275, 0.1125 and 0.1125.
  - `total_bits_q12` for targets `[0,1,2]` is 4096 + 7629 + 12911 = **24,636**.
  - `bits_per_byte` over 3 bytes is 24,636 / 12,288.
- **T2**: every target has mass ≥ 1 under three weight settings and an all-unseen context. That includes index 4, which neither the counts nor the order-0 expert saw [claude].
- **T3**: with the order-0 and floor weights at 0, the result equals `mle_masses_from_counts` for count weights 1, 3 and 65,535 [claude].
- **T4** [claude]:
  - counts at u64::MAX are exact;
  - weights at u64::MAX, and a weight sum of `2^16` on V=1, give `ScoreOverflow`;
  - count weight 65,534 with floor 1 on V=8192 keeps every mass ≥ 1;
  - floor 0 with zero-mass targets gives `MassZero`, and so does floor 1 swamped by a count weight of `2^32` on `V = 2^17`;
  - for counts at u64::MAX, the three equal-weight targets lie within one unit of each other and below the heavy one;
  - a length mismatch, an empty vocabulary and all-zero weights are refused.

## What the numbers say

A count model scored through this function gives every target a finite integer bit cost, including targets it never saw [claude]. The worst case per target is set by the floor's share: at most about `-log2(floor / (d * V))` bits [claude]. With count-only weights, the existing MLE behaviour is reproduced exactly [claude]. The tests say nothing about the bits per byte of any real model, or about which weights are good [claude].

## Checks run at the head

- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=local/c410-target ~/.cargo/bin/cargo test -p uor-r4-integer --lib lawful_likelihood`: 13 passed, 0 failed [claude].
- `... cargo test -p uor-r4-integer --lib`: 189 passed, 0 failed (185 on main plus 4) [claude].
- **Negative check** [claude]. With the floor line `add_expert(&mut scores, weights.floor, &vec![1; vocabulary])?;` set aside as a patch, the run gave 10 passed and 3 failed. T2 fails at `lawful_likelihood.rs:488` (`.expect("floor holds")`, `MassZero`), and T1 and T4 fail too. After the restore and a rebuild: 189 / 189.
- After the review fixes, the lawful_likelihood tests gave 13 passed. The negative check was repeated at that head: with the floor line set aside, T1 fails at `:445`, T2 at `:488` and T4 at `:571`, so 10 passed and 3 failed. After the restore: 13 / 13 [claude].
- Fresh review: an Opus reviewer in a fresh context returned APPROVE-WITH-LOW-ITEMS. All three items are fixed in this head: the overclaim above, a test comment, and weak assertions [claude].
- `rustfmt --edition 2021 --check` on the file: clean [claude].
- `UOR_LAB=claude uor-harness loop pr-check --local`: see the PR [claude].

## Skills used

None invoked [claude]. Followed AGENTS.md, the patch-file set-aside for the negative check, and the file's existing conventions.

## Still open

- Choosing the weights on held-out text, and the first real count-model bits per byte (P1) [claude].
- Whether the integration tests' E0432 errors also occur on `main` [claude].
- A roadmap item of its own for the written-state track [claude].

## Plain summary

A count model gives zero probability to a word it never saw after a context, and the project's scorer could not score that word [claude]. This unit adds one integer function. It blends three sources: the model's counts, overall word frequencies, and an even floor across the whole vocabulary [claude]. The blend gives every word a small, positive, exact share, so every word now has a finite cost in bits [claude]. With the floor switched off it gives exactly the old answer, and a hand-worked four-word example matches to the last unit [claude].
