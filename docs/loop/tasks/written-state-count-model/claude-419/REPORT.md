# REPORT: written-state-count-model, claude-419 (the integer modified Kneser–Ney scorer on the host)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **No bits in this unit** [claude]:
  - The ORDER asked for `total_bits_q12` through `lawful_likelihood::bits_q12`. That function is unreachable from `uor-r4-discrete`'s tests. `uor-r4-integer` is only a dev-dependency here, and the dev-dependency `uor-r4-cpu-train` turns on the integer crate's `serving-wasm` feature, which compiles `lawful_likelihood` out. Making it a normal dependency fails the same way in the test build.
  - So the scorer returns exact Q48 masses (`score_target`, `scored_masses`). claude-420's instrument takes `bits_q12` of them by linking `uor-r4-integer` directly, without `serving-wasm`.
  - A second log2 was not written here. Two implementations could drift apart.
- **The ORDER's third negative could not fail as written** [claude]:
  - "No BOS-rule fallback, the hand fixture fails" assumed a separate BOS check. But the counter never writes a context with BOS outside its oldest slot (claude-415), so such a context has no record and backs off anyway.
  - Read literally for a BOS target, the rule would break the ORDER's own lawfulness claim. The target's mass would stay at M/4096 instead of following the γ chain.
  - The scorer therefore gives a BOS target the γ chain (a = 0 at every order), and `score_target` refuses BOS. The negative was moved to where it can fail: a BOS target that backs off makes the sum test fail, with sum > M.
  - An unseen context giving 0 instead of backing off was added as a fourth negative.
- **The first three test streams were not text-like** [claude]:
  - They gave order spectra with n4 ≥ n3 or with n1..n4 = 0 at order 0. The scorer refused them correctly: for example D3 < 0 at n1..n4 = [1728, 23, 15, 19], and D2 < 0 at [1487, 19, 14, 12].
  - Real D87 spectra fall smoothly. claude-418's order 3 is 4,132,730 / 413,614 / 122,848 / 56,120, which gives valid discounts.
  - The final test stream has a heavy-headed unigram (a product of four uniforms), BOS one draw in 61, and one draw in 9 copying 2..9 earlier tokens, as text repeats phrases. Every order 0..4 then falls. On a real slice a negative discount stays a refusal, never a clamp.
- **The delegated builder stopped at its context mark before writing code** [claude]. Its groundwork (the hand-fixture counts, the overflow bounds, the redundant BOS check) was used. I wrote the code myself.

## What changed

- `crates/uor-r4-discrete/src/kn_score.rs` [claude]:
  - `KnDiscounts::from_count_of_counts`: Chen–Goodman in checked i128, floored once to Q48. It refuses a zero n_i, overflow, and any D outside [0, i·M].
  - `KnTables::new(highest, orders)`: one counter output per order 0..=N. It refuses a wrong count of orders, an order mismatch, non-canonical order, a key of another order, and a context total of 0.
  - `score_target(tokens, j) -> u64` (Q48) and `scored_masses(tokens)`. The interpolation runs in u128 from `P_{-1} = M/4096`, with two binary searches per order.
- `crates/uor-r4-discrete/src/kn_score_oracle.rs` (test only) [claude]:
  - It counts from the token stream into `BTreeMap`s keyed by the context tokens themselves: raw counts, left-token sets, adjusted counts, per-context A/n1/n2/n3+, and its own count-of-counts and discounts.
  - It shares only `ngram_admitted` with the code under test.
- Two `mod` lines in `lib.rs`. No dependency change, no CUDA, no binary [claude].

## Measurements

Host only, on the laptop [claude]:

- **Count spectra of the test streams** (6,000 tokens, n1..n4):
  - N = 3: order 0 [157, 80, 48, 38], order 1 [2193, 329, 99, 59], order 2 [3516, 309, 65, 7], order 3 [3545, 584, 134, 40].
- **Sum test**, per history, over all 4,096 targets:

  | N | Histories | Largest deficit (units of 2^-48) | Bound 3(N+1)·4,096 |
  |---|---|---|---|
  | 1 | 493 | 4,225 | 24,576 |
  | 2 | 2,872 | 6,964 | 36,864 |
  | 3 | 4,102 | 9,414 | 49,152 |
  | 4 | 4,664 | 11,554 | 61,440 |

- **Scorer = oracle**: every non-BOS target of three 6,000-token streams at N = 1, 2, 3 and 4 is equal, one by one and through `scored_masses`.

## What the numbers say

- At the worst history the floors cost about half a unit of 2^-48 per target per order, about a sixth of the bound [claude]. The sub-distribution loses under 10^-10 of probability per history, which is far below any BPB digit that bar Q reads.
- The hand fixture fixes the rule's meaning on 12 tokens [claude]:
  - a document start keeps raw counts;
  - (7, BOS) backs off to order 1;
  - an unseen context and an unseen word take the γ chain.
  The scorer equals the oracle on streams whose every order has counts 1..4.
- Nothing here is a BPB, and nothing here claims bar Q [claude].

## Checks run at the head

- G1 [claude]:
  - `cargo test --release -p uor-r4-discrete --lib -- kn_score ngram_count`: 27 passed, 0 failed (6 scorer tests, 21 counter tests), after main with claude-418 was merged in;
  - `cargo clippy -p uor-r4-discrete --lib --tests`: no finding in the new files.
- G2: set-aside negatives, each failing, then restored and rebuilt [claude]:
  - γ forced to 0: the hand fixture, the sum test and the oracle test fail.
  - A BOS target backing off: the sum test fails at history `[]`, with sum 281,526,425,064,523 > M.
  - An unseen context giving 0: the hand fixture, the sum test and the oracle test fail.
  - Raw count-of-counts below N in the oracle test's tables: the oracle test fails at seed 419a, N = 1, target 1, on a mass mismatch rather than a refusal.
  - Suggested by the review, a ≥ 3 mapped to Dq2: the N = 0 test, the sum test and the oracle test fail.
  - Restored and rebuilt: 27 of 27.
- G3: `UOR_LAB=claude uor-harness loop pr-check --local`: see the PR [claude].
- G4: no corpus text, no token stream and no binary is committed [claude].
- Fresh review [claude]: one reviewer, given the ORDER and the diff only, returned APPROVE-WITH-LOW and found no correctness, overflow or lawfulness defect. It found four LOWs:
  - The ORDER's test "order 0 against uniform" was missing. Fixed: `kn_score_order_zero_against_uniform`, by hand, where a = 4 takes Dq3.
  - The sum test stopped at N = 3. Fixed: it now runs N = 1..4.
  - The a ≥ 3 branch was pinned only by the oracle. Fixed by the N = 0 test, and its set-aside fails (above).
  - `KnTables::new` trusts each digest's count-of-counts and context records. Not changed: every table comes from the counter's assembler, which census-checks those fields against the triples before `finish`. A caller that bypasses the assembler is outside this unit.

## Skills used

None invoked [claude]. I followed AGENTS.md and the native-space rule: integer masses only, no float, no squashing, no matmul.

## Still open

- claude-420 [claude]:
  - the split TRAIN / DEV-tune / DEV-test UORT streams from claude-411's id lists, with byte counts;
  - counts filtered to DEV-test contexts;
  - a GPU scorer equal to this host scorer;
  - an instrument that takes `bits_q12` of the masses, plus order-2 BPB equal to a CPU oracle on a small slice.
- claude-421: the sheet, FIT, N tuned on DEV-tune, and the sealed DEV-test run on all targets and the novel subset [claude].
- A real-slice discount check: the D87 spectra of every order under the chosen N must give D in [0, i]. If one does not, that is a refusal to report, not a clamp [claude].

## Plain summary

The reference model for bar Q needs probabilities from the n-gram counts [claude]. This unit turns claude-418's counts into probabilities using integers only. It blends each context length with the shorter ones and sets aside a little mass for unseen words. For every history the probabilities add up to one, to within a ten-billionth. An independent second implementation gives the same answer on every token. The next unit runs the same scorer on the GPU over the real corpus split.
