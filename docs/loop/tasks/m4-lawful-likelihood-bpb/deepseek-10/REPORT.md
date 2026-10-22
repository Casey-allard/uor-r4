# REPORT: m4-lawful-likelihood-bpb, deepseek-10 (repair of the repaired design note)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `effe1e9c1b3961608092eda425017036c6901bff`. References #2861; item stays `ready`.

## What did not work, first

Answering the fresh-context CHANGES record at `effe1e9c1`. **Correction (deepseek-13):** that record was advisory, not counted — the gate classes the `receipts.jsonl` files as `other`, so this PR is not doc-only to the gate and a same-lab record is refused under D46; an other-vendor APPROVE at the head is required.

- **BLOCKING 1, the note contradicted itself on lawfulness.** It said "this note calls nothing lawful until [Card 1] is answered" and then said "PRLL is lawful but not temperature-calibrated"; the same unqualified claim sat in the "Why it is lawful" heading and property 6. Fixed: the status, the heading ("Why this would be lawful if §3.2 permits a linear ratio"), property 6 and Card 4 are all conditional; nothing here is called lawful while Card 1 is open.
- **BLOCKING 2, required edit 5 was still incomplete.** The note put the softmax-free learned rank table under "Prior art for no-normalization reads" and called the Flock read "no-normalization", but the integer implementation ships an actual rank-weight normalization: `RAW_RANK_WEIGHTS_Q16`, `RECIPROCAL_Q32` and `rank_table_q31` (`stack/flock.rs:382-470`), documented as `w_r ∝ 1/(r+1)` or a learned per-head table with Q31 rank tables in `uor-r4.lut-stack/3` (`archive/mechanisms/flock-rank-reads-2026-10-20.md`). Fixed: the note now cites that normalization as the existing read-weight precedent and keeps the board's Normalization row as a *model-mechanism* gap that still stands.
- **BLOCKING 3, "the integer model emits no probabilities" was false.** `model.rs:4541` `softmax_into` reads `tables.exp` and normalizes, and the step path calls it at `model.rs:3811` into `session.last_probabilities`. The model does emit Q48 masses; what it does not emit is a **lawful** probability. Fixed: the problem section says exactly that, and the artifact sentence is now "under this construction the D11 artifact would emit integer potentials only" (the current artifact emits Q48 masses through the banned softmax).
- **LOW, log2 precision.** Added the 12 fractional squaring rounds, the truncation at each round, and the stated per-token bound (`<= 2^-12 + 2^-48` bit, the second term from the Q48 mass grid).
- **LOW, count baseline conversion.** Added the exact MLE conversion (`floor(count_i*2^48/total)` with token-id remainder), the zero-count behaviour (mass 0, infinite contribution, reported as such), and the add-one secondary arm.
- **LOW, crate hosting sentence.** Clarified that the pure scorer lives in the integer crate while the discrete crate runs `eval-corpus` integration and reporting; both integer-only.
- **LOW, the REPORT's size figure.** No estimate this time: the measured repair diff is 39 insertions / 39 deletions in the note (81 lines after the repair), exactly `git diff --numstat effe1e9c1..HEAD -- docs/labs/m4-lawful-likelihood-bpb/README.md`.
- **LOW, the revised note was not on #2861.** Posted after the commit: the delta summary (conditional lawfulness, rank-weight precedent, emissions correction, lows).
- **Still not in this unit:** no code, tests, model or GPU run, no held-out stream, no BPB number; the item stays `ready`.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/labs/m4-lawful-likelihood-bpb/README.md` | docs | +39/−39 (81 lines) | conditional lawfulness throughout; the Flock rank-weight normalization cited as the read-weight precedent with the board's model-mechanism gap kept; the current model's Q48-masses-through-the-banned-softmax stated exactly with code paths; artifact wording made conditional; log2 precision, count-baseline conversion and crate split tightened |
| `docs/loop/tasks/m4-lawful-likelihood-bpb/deepseek-10/{ORDER,REPORT}.md` | docs | small | this repair order and report |

**Size:** the repair is 39 insertions / 39 deletions in the note (measured), plus the task-folder files. No source, test, generated file or item-status change.

## Measurements

- **No numeric result is claimed.** The only arithmetic in the note is bounds: `MAX_ABS_SCORE < 2^39`, V=4096, `sum(phi) < 2^52`, minimum mass 1 with 48 bits, and the fixed-point error bound `<= 2^-12 + 2^-48` bit per token.
- Checks at the head: `loop gates --full` GREEN on five; `loop report --check` PASS; `loop pr-check --local` PASS; CI green at the prior head and re-running.

## Checks run at the head

- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates.
- `UOR_LAB=deepseek uor-harness loop report --check`: PASS.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS on every rule (`commit-order` confirms deepseek-8 first alone and each repair ORDER alone).
- `cargo fmt`/tests/clippy: no source changed; unchanged from the base.

## Skills used

- **solution-integrity** for the claim/citation pass; **holding-the-goal** for keeping lawfulness conditional while the §3.2 card is open.

## Still open

- **Card 1** (is a linear full-alphabet ratio inside §3.2?) and **Card 2** (`-log2` as measurement) are on #2028; the note calls nothing lawful until Card 1 is answered.
- **Implementation unit**: the pure scorer and the seven finite host controls once a family is approved.
- **Registration unit**: held-out stream naming/hash, pre-registration, non-author FIT, registered GPU scoring under an owner release.
- **The item stays `ready`** after this PR merges.

## Plain summary

- **The review found three real overclaims.** The note said it was lawful while the §3.2 question is open; it called a rank-weight table "no normalization" although the repo ships one; and it said the model emits no probabilities when it emits Q48 masses through the banned softmax. All three are corrected with code and archive references.
- **The repair is wording and citation precision, not new design.** The construction itself (base-1 masses, exact Q48 sum, no exp table, integer bits) is unchanged.
- **Everything remains conditional:** the note is a design record; the PM's two cards gate calling it lawful, and no code, model run or BPB number is claimed. The item stays `ready`.