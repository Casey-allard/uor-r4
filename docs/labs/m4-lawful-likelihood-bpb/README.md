# A lawful likelihood for bits-per-byte scoring — design note (deepseek-8 → deepseek-11)

Status: **design only.** Card 1 is **answered by the owner**: Casey answered in Claude's chat at about 04:15Z on 22 October 2026, recorded verbatim by Claude on #2028 at 04:17Z as "Yes, a linear ratio (Recommended)" — CHARTER §3.2 bans exponential and squashing transforms, and dividing non-negative potentials by their sum is lawful, and the item itself asks for an exact sum over the alphabet. The construction below is therefore lawful under the owner's reading. Card 2 (`-log2(mass)` as a measurement) stands as the note reads it. No code is changed; the item stays `ready`. Posted on #2861 before any code, as item step 2 requires.

## The problem

The current integer model does produce Q48 masses — but only through an integer softmax over an exp table: `softmax_into` (`crates/uor-r4-integer/src/model.rs:4541`) reads `tables.exp` and normalizes, and the step path calls it at `model.rs:3811`, storing `session.last_probabilities`. That is the banned normalized-exponential form (CHARTER §3.2). What the model does **not** produce is a **lawful** probability. The Q48 sampling contract (`sampling.rs`, `PROBABILITY_ONE = 1<<48`) accepts masses summing exactly to one; `docs/research/open-questions.md` records "A lawful likelihood is still unbuilt."

## What the project already has

- **Q24 integer potentials**: `geometric_potential.rs` scores candidates with signed Q24 table reads and exact integer addition on the fixed signed H4 geometry; `MAX_ABS_SCORE < 2^39` (asserted there).
- **A Q48 mass contract**: `sampling.rs` requires masses summing exactly to `1<<48`; greedy, categorical and Min-P selection consume it.
- **An existing read-weight normalization**: the Flock / rank-table read weights kept sources by rank, `w_r ∝ 1/(r+1)`, or by a learned per-head table (`archive/mechanisms/flock-rank-reads-2026-10-20.md`). The integer implementation ships `RAW_RANK_WEIGHTS_Q16`, `RECIPROCAL_Q32` and `rank_table_q31` (`crates/uor-r4-integer/src/stack/flock.rs:382-470`), Q31 rank tables in artifact schema `uor-r4.lut-stack/3`, audited FULL PASS. Its *selection* is rank-based with no exponentiated scores; its *weighting* is a normalization, and that is the relevant precedent here.
- **Host evaluation paths**: `crates/uor-r4-discrete`'s `eval-corpus` and `data/panels/` (frozen panels, `MANIFEST.sha256`).

## Proposed construction: potential-ratio lawful likelihood (PRLL)

1. **Scores unchanged.** At a scored step, `s[0..V)` are the model's Q24 signed integer scores on the admitted causal prefix. No exponential and no table is applied.
2. **Non-negative potentials, order-preservingly.** `phi[i] = (s[i] - min(s)) + 1` in exact i64; every `phi[i] >= 1`.
3. **Every token gets a legal minimum, then the ratio.** With `V` candidates and `R = 2^48 - V` (`config.rs:86` fixes `vocab_size = 4096`):
   - `base[i] = 1`;
   - `quota[i] = floor(phi[i] * R / sum(phi))` in **u128** (`phi*R` can approach `2^88`);
   - the remaining `R - sum(quota)` units go one each to the largest fractional remainders (`phi[i]*R mod sum(phi)`, ties by **lowest token id**);
   - `mass[i] = 1 + quota[i] + (0 or 1)`, so `mass[i] >= 1` and `sum(mass) = 2^48` exactly.
   The base unit is what makes zero mass impossible: with V=4096, `MAX_ABS_SCORE < 2^39` and 4,095 maximal scores, `sum(phi) < 2^52`, so the minimum-potential token's proportional quota can floor to 0 — its base unit keeps its bits finite.
4. **Bits, in integer fixed point.** For a scored token with `mass m in [1, 2^48]`: `bits = 48 - log2(m)`.
   - `log2(m)`: integer part `k = floor(log2 m)` from the bit length; fractional part by 12 repeated-squaring rounds (normalize `m/2^k` to `x in [1,2)`, square-and-halve, read each comparison bit), truncating at each round.
   - Per-token deviation from the true `-log2(m)`: at most `2^-12` bit from the 12 fractional rounds plus the deterministic truncation of the final rounding; the note states the bound as `<= 2^-12 + 2^-48` bit (the second term from the Q48 mass grid). `bits_q12 = 48*4096 - (k*4096 + fractional_q12)` is an integer; `bits per byte = sum(bits_q12) / (4096 * bytes)`, reported as an exact integer numerator and denominator, never a rounded float.
   - **No float anywhere**, so §3.7's float boundary does not apply.

### Why it is lawful (Card 1 answered by the owner)

- No softmax, log-softmax, softplus, sigmoid, tanh, GELU, normalized exponential, or renamed/table form of one: the normalizer is a linear ratio of non-negative integer potentials, with no exponential map and no exp table.
- No squashing: identity-or-shift, not a bounded curve.
- No dense matmul or GEMM: per-candidate integer arithmetic plus one exact u128 ratio per candidate.
- **Division is not in the artifact.** Under this construction the D11 artifact would emit integer **potentials only** (the current artifact instead emits Q48 masses through the banned softmax above); the ratio and remainder distribution would run in the offline GPU scoring harness, so no divide instruction enters the artifact. If an artifact-side normalizer is ever wanted, `stack_div_u128` (`stack/kernels.rs:304`, restoring divide) is the audited way, with the stated u128 width.
- **Geometry claim, softened.** The board's Normalization slot reads "**GAP: none.** Snapping to an E8 root may serve, but no record says so" (`docs/mechanisms/README.md:142`) — a *model-mechanism* gap. The Flock rank-weight normalizer above is an existing *read-weight* normalization; PRLL is a scoring normalization over the model's own H4 potentials, not a claim that the model has a normalization mechanism. The board's gap stands.

### Why it is lawful: testable properties (Card 1 answered)

1. Every mass is a non-negative integer **at least 1**.
2. The masses sum to exactly `2^48` — checked, never assumed.
3. Order and argmax are preserved (ties by lowest token id).
4. Relabeling candidates permutes masses exactly; ties keyed by token id, with a tied control.
5. Equal scores give exactly equal masses when `V` divides `2^48` (every power-of-two vocabulary, including 4096); otherwise deterministic ±1 remainder masses, stated by the control.
6. **No named banned function appears in the construction.** The one reading the note could not settle itself — whether a linear full-alphabet ratio is within §3.2 — was put to the owner as Card 1 and answered "Yes, a linear ratio": the ban targets exponential and squashing transforms, and a ratio of non-negative potentials by their sum is lawful, which the item's own exact-alphabet-sum requirement also assumes. The construction is lawful under that recorded reading.

### Finite host controls (before any GPU run)

1. **Tiny alphabet, enumerated**: V=4 hand-computed Q24 scores; exact expected masses, exact sum `2^48`.
2. **Worst-case zero-mass check**: V=4096, one score at `-MAX_ABS_SCORE`, the rest at `+MAX_ABS_SCORE`, the true token the minimum — mass exactly 1, bits exactly 48.
3. **Uniform case**: power-of-two V (4096) exactly equal masses; non-power-of-two V deterministic ±1 remainder masses.
4. **Ties**: equal-score candidates across the remainder split, asserted in both slot orders.
5. **Monotonicity/argmax** and **permutation equivariance** over random Q24 vectors.
6. **BPB arithmetic**: a synthetic sequence with known masses yields exactly the analytic `bits_q12`; a uniform baseline over V tokens scores `log2(V)` bits per token.
7. **Count-table baseline**: primary MLE baseline converts counts exactly — `mass_i = floor(count_i * 2^48 / total)` with remainder units by token id — and **zero-count tokens get mass 0, whose contribution is infinite and is reported as such** (the count model cannot score unseen targets). Secondary: add-one-smoothed counts through the same normalization, labelled "add-one".

## Crate choice

- **Pure scorer**: `crates/uor-r4-integer`, beside the Q24 potentials and the Q48 contract it consumes; the crate owns `stack_div_u128` and must not gain GPU dependencies.
- **Evaluation integration and reporting**: `crates/uor-r4-discrete`'s `eval-corpus` path, which owns the panel scoring and the report schema. Both are integer-only; the pure function's home is the integer crate, and the discrete crate only integrates it into `eval-corpus` and its report.

## Owner and PM decisions

1. **Card 1 — DECIDED by the owner: "Yes, a linear ratio (Recommended)".** Casey answered in Claude's chat at about 04:15Z on 22 October 2026; Claude recorded it verbatim on #2028 at 04:17Z: CHARTER §3.2 bans exponential and squashing transforms, dividing non-negative potentials by their sum is lawful, and the item itself asks for an exact sum over the alphabet. The construction is lawful under that reading; the conditionals this note carried while the card was open are resolved.
2. **Card 2 — `-log2(mass)` is a measurement, as this note reads it.** Cross-entropy is banned as an *implicit softmax*; nobody disputed that reading, so it stands.
3. **The held-out stream** (file, byte count, sha256) — the pre-registration names and freezes it.
4. **Calibration** — PRLL is lawful under the owner's Card 1 answer but not temperature-calibrated; whether the 0.90 bar may be read through an uncalibrated lawful likelihood is a PM/owner call.
5. **Family choice** — PRLL vs a polynomial distance kernel vs an occurrence-count ratio; each is its own pre-registration arm.

## What this unit does and does not do

- Does: the revised design note and its record, the issue post, ORDER and REPORT, the cancel probe, and the owner's Card 1 decision recorded verbatim.
- Does not: change code or tests, run any model or GPU work, name the held-out stream, register an instrument, or claim BPB numbers. The item stays `ready`; implementation and registered scoring are later units.

## Implementation (deepseek-17)

The host-only scoring core is `crates/uor-r4-integer/src/lawful_likelihood.rs`, wired into the selected crate root (`src/native_geometry.rs`, D29):

- `masses_from_scores(&[i64]) -> Result<Vec<u64>>` — the base-1 Q48 ratio: `phi = s - min + 1`, `R = 2^48 - V`, `quota = phi * R / sum(phi)` in u128, the remainder units by the largest fractional remainder (ties by lowest index), so every mass is at least 1 and the sum is exactly `2^48`.
- `mle_masses_from_counts(&[u64]) -> Result<Vec<u64>>` — the exact maximum-likelihood baseline: `floor(count * 2^48 / total)` plus the same remainder rule, with zero counts left at zero.
- `bits_q12(mass) -> Result<u64>` — integer fixed-point `-log2(mass / 2^48)` (bit length plus 12 squaring rounds), and `total_bits_q12`/`bits_per_byte` for exact integer BPB fractions.

Nine lib tests cover the seven finite host controls: tiny alphabet; V=4096 worst case (minimum mass exactly 1, exactly 48 bits); uniform power-of-two vocabulary; non-power-of-two ±1 remainders; token-id ties; monotonicity/argmax over 64 deterministic random vectors; permutation equivariance; analytic BPB and MLE; and the zero-count-is-unscorable rule. The module runs no model and reads no artifact; the registration unit (held-out stream, pre-registration, FIT, GPU scoring) is still open.

## Sources

- Item: [m4-lawful-likelihood-bpb](../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861); CHARTER §3.2, §3.6, §3.7, §4.8.
- Banned current path: `crates/uor-r4-integer/src/model.rs:3811,4541` (`softmax_into` over `tables.exp`); `geometric_read.rs` (Q31 exp table).
- Scores: `geometric_potential.rs`; Q48 contract: `sampling.rs`; restoring divide: `stack/kernels.rs:304`; vocabulary: `config.rs:86`.
- Read-weight normalization precedent: `archive/mechanisms/flock-rank-reads-2026-10-20.md`; `crates/uor-r4-integer/src/stack/flock.rs:382-470`; board gap: `docs/mechanisms/README.md:142`.
- Owner decision on Card 1: #2028 comment 2026-10-22T04:17:03Z (Claude, recording Casey's chat answer at ~04:15Z).
- Open question: `docs/research/open-questions.md:102`.

## Pre-registration sheet (deepseek-49, 2026-10-23)

The one-page pre-registration for the registered scoring run is [`docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md`](../../harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md), `TEST FITNESS: PENDING` pending a non-author review (Claude, Kimi or antigravity; deepseek authored it).

- It names and freezes the chat held-out stream (`heldout.u16`, 12,389,242 bytes, sha256 `e5f400b0…`, 6,194,589 tokens / 17,576,697 stream bytes), fixes the PRLL scoring formula and both byte bases, the 512-window primary arm (196,608 targets) and the 64-window diagnostic, the arms A0/B0/C0/U, the owner's 0.90 bar (not set or changed here) and the two registration conditions already in the design note (uncalibrated-likelihood reading and stream confirmation).
- It charges nothing: no GPU is used, no model is exported yet (`m4-export-uorlut01-cli`), no bar is registered, and the item stays `ready` until the FIT and the owner registration land.
