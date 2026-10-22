# REPORT: m4-lawful-likelihood-bpb, deepseek-9 (repair of the design note)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `e9748f179e5427b992cd41c7777b830eb67b7e00`. References #2861; does not close it — the item stays `ready`.

## What did not work, first

Answering the counted CHANGES record at `e9748f1`, point by point:

- **REQUIRED 1, a token could get zero mass and infinite bits.** `phi = s - min + 1` is positive, but `floor(phi*2^48/sum)` can still floor to 0: with V=4096, `MAX_ABS_SCORE < 2^39` and 4,095 maximal scores, `sum(phi) < 2^52`, so the minimum-potential token's proportional share can be below 1, and the largest-remainder pass gives its units to the largest remainders. Fixed: **every token gets base mass 1 first**, then `R = 2^48 - V` is split by ratio with the remainder units; every `mass >= 1` and the sum is exactly `2^48`. Added the reviewer's control: V=4096, one score at `-MAX_ABS_SCORE` and the rest at `+MAX_ABS_SCORE`, the true token the minimum — its mass must be exactly 1, its bits exactly 48 (finite).
- **REQUIRED 2, how bits are computed was unnamed.** Fixed: integer fixed-point `log2` — integer part from the bit length, fractional part by 12 repeated-squaring rounds, accumulated as `bits_q12` with a stated per-token error bound of `2^-12` bit and an exact integer numerator/denominator for bits per byte. Float is explicitly ruled out (§3.7), so the discrete crate can host the scorer.
- **REQUIRED 3, division versus the D11 artifact.** Fixed: the artifact emits integer **potentials only**; the ratio and remainder distribution run in the offline GPU scoring harness, so no integer-divide instruction is emitted in the artifact. An artifact-side normalizer, if ever wanted, uses `stack_div_u128` (`stack/kernels.rs:304`, restoring divide) under the instruction audit, and the note states that `phi*R` approaches `2^88` and needs u128.
- **REQUIRED 4, no crate chosen.** Fixed: `crates/uor-r4-integer` for the pure scorer (it owns Q24 potentials, the Q48 contract and `stack_div_u128`), `crates/uor-r4-discrete` for `eval-corpus` integration and the GPU run's report; the integer crate gains no GPU dependency.
- **REQUIRED 5, the geometry claim and its missing citations.** Fixed: the note now cites the mechanism board's Normalization row ("GAP: none. Snapping to an E8 root may serve, but no record says so"), the Flock read's no-normalization hard-rank precedent and the softmax-free read experiments' rank table, and it **softens the claim**: PRLL is a scoring normalization over the model's H4 potentials, not a claim that the model has a normalization mechanism; the board's gap stands.
- **LOW 6, ties and uniform exactness.** Fixed: ties are keyed by **token id**, not slot index, with a tied control asserting the ordering across both slot orders; equal masses are promised only when `V` divides `2^48` (every power-of-two vocabulary, including 4096), and the non-dividing case is stated as deterministic ±1 remainder masses.
- **LOW 7, the count-table baseline's smoothing.** Fixed: the primary baseline is the count model's own maximum-likelihood probabilities (`count_i/total`) through the same bits arithmetic, **not** through the PRLL shift; a secondary add-one-smoothed arm through PRLL is labelled as such, because the shift changes a minimum count above zero.
- **The two PM questions are now decision cards, not decisions.** (1) Whether a linear full-alphabet ratio is inside CHARTER §3.2 — the ban's reason is a hidden dense global normalization, and `sum(phi)` is one; the item itself requires an exact alphabet sum. (2) Whether `-log2(mass)` is a measurement or the banned cross-entropy. Asked on #2028; the note calls nothing lawful until (1) is answered.
- **Still not in this unit**: no code, no tests, no model or GPU run, no held-out stream, no BPB number. That is the repair's boundary and the item stays `ready`.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/labs/m4-lawful-likelihood-bpb/README.md` | docs | +82/−40 | base-1 masses with the u128 ratio and worst-case control; fixed-point log2 with its error bound; potentials-only artifact and `stack_div_u128` fallback; crate choice; softened geometry claim with citations; token-id ties and exact-uniform statement; count-baseline smoothing; the two PM cards |
| `docs/loop/tasks/m4-lawful-likelihood-bpb/deepseek-9/{ORDER,REPORT}.md` | docs | small | this repair order and report |

**Size:** about 130 changed lines; the ORDER's estimate was about 120 (1.1). No source, test, generated file or item-status change.

## Measurements

- **No numeric result is claimed.** The numbers in the note are arithmetic bounds, not measurements: `MAX_ABS_SCORE < 2^39`, V=4096, `R = 2^48 - 4096`, `sum(phi) < 2^52`, the minimum token's ratio share `< 1` and its base mass `1` with `48` bits.
- `loop gates --full`: GREEN on the five gates; `report --check` PASS; `pr-check --local` PASS on every rule.

## Checks run at the head

- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates.
- `UOR_LAB=deepseek uor-harness loop report --check`: PASS.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS on every rule.
- `cargo fmt`/tests/clippy: no source changed; unchanged from the base.

## Skills used

- **solution-integrity** for the premise/falsifier pass; **holding-the-goal** for keeping the note inside what a design unit can claim while the §3.2 reading is pending.

## Still open

- **The PM's two readings** (linear full-alphabet ratio under §3.2; `-log2` as measurement) are asked on #2028 and are the gate on calling PRLL lawful.
- **The implementation unit**: the pure scorer plus the seven finite host controls once a family is approved (or PRLL as a declared arm).
- **The registration unit**: held-out stream naming/hash, one-page pre-registration, non-author FIT, then registered GPU scoring under an owner release.
- **The item stays `ready`** after this PR merges; the next unit continues from this note.

## Plain summary

- **The reviewer caught a real hole.** The first note's ratio could give the true token zero probability and therefore infinite bits; the fix gives every token at least one unit of mass before the ratio, so even the worst-case minimum has exactly 48 bits — finite and checked.
- **Everything the review asked for is now explicit**: how bits are computed (integer fixed-point log2 with a stated error bound), where division happens (never in the artifact; u128 in the offline scorer), which crate owns what, and what the mechanism board actually supports (the Normalization slot is a gap, and the note no longer dresses a scoring ratio up as a model mechanism).
- **Two calls are the PM's, not mine**: whether a linear full-alphabet normalization is allowed by the squashing ban, and whether `-log2` counts as measurement. They are decision cards on #2028.
- **No code was written and no measurement claimed**: the item is a design note, still `ready`; implementation and registered scoring are their own units.