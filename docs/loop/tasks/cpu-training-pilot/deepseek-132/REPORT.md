# REPORT: Hamming stop signatures, deepseek-132

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[stop signatures](../../../harness/prep/cpu-training-pilot-stop-signature-2026-11-04.md).
Base: `e3959b140`.

## What did not work

- **The candidate does not terminate useful free replies.** Seed 1 emitted BOS
  on all 103 replies, but every stop was premature and no reply matched its
  reference. Seed 2 emitted BOS on 43 replies, also all premature; the other 60
  hit the 16-token cap. Both seeds produced zero exact-reference replies.
- **Teacher-forced terminal classification did not transfer to generated
  trajectories.** The gate recognized the reference final row at 97/103 on
  both seeds, but generated text diverges before that row. The candidate then
  fires on a different, premature endpoint.
- **The shuffled gate is not a monotone stop baseline.** It never fired on
  seed 1, while on seed 2 it fired immediately on all 103 replies. This
  supports the primary matched comparison but also shows that free-generation
  stopping is sensitive to the refitted random-label prototypes.
- **The matched teacher-forced false-stop count is nonzero.** It was 8/1,085
  on both seeds, under the registered 10-event ceiling but above zero.
- **This is not integer-serving evidence.** The candidate bypasses writer
  admission inside the CPU evaluator. The artifact and integer session remain
  unchanged; integration requires a separate coordinated parity unit.

## What changed

Added a CPU-only two-class stop candidate:

1. every complete 256-coordinate Q12 writer query is packed into four `u64`
   sign words;
2. each class deduplicates observed signatures and chooses a Hamming medoid,
   then up to seven farthest-first prototypes;
3. a query scores
   `nearest_nonterminal_hamming - nearest_terminal_hamming`; and
4. a margin fitted on training rows decides whether to emit BOS before the
   exact writer fallback.

The margin maximizes training terminal stops subject to
`nonterminal_stops <= floor(nonterminal_rows / 100)`. Ties choose the larger
margin and then the smaller threshold.

The evaluator compares true-label prototypes, class-preserving shuffled-label
prototypes, and the no-gate writer. Every decision records writer token, gate
firing and chosen token. Gated decisions must be BOS; ungated decisions must be
the writer token. Free generation reports all three arms.

## Measurements

The fixed split contains 98 training samples / 1,218 relevant decisions and
103 evaluation samples / 1,188 relevant decisions. There are 98 training
terminal rows and 103 held-out terminal rows.

| Seed | Arm | Training terminal | Training false | Held-out terminal | Held-out false |
|---:|---|---:|---:|---:|---:|
| 1 | true | 97/98 | 10/1,120 | **97/103** | **8/1,085** |
| 1 | shuffled | 39/98 | 0/1,120 | 0/103 | 34/1,085 |
| 2 | true | 96/98 | 5/1,120 | **97/103** | **8/1,085** |
| 2 | shuffled | 26/98 | 2/1,120 | 0/103 | 20/1,085 |

The no-gate teacher-forced arm selected BOS on 0/103 terminal and 0/1,085
nonterminal rows on both seeds. Both true-vs-no-gate and true-vs-shuffled
paired terminal gains were **97 rows**, against threshold **20**. The
registered teacher-forced bar therefore passed on both seeds.

Free generation at a 16-token cap did not:

| Seed | Arm | BOS complete | Premature | Cap | Exact | Opening |
|---:|---|---:|---:|---:|---:|---:|
| 1 | no gate | 0 | 0 | 103 | 0 | 0 |
| 1 | true gate | **103** | **103** | 0 | 0 | 0 |
| 1 | shuffled gate | 0 | 0 | 103 | 0 | 0 |
| 2 | no gate | 0 | 0 | 103 | 0 | 4 |
| 2 | true gate | **43** | **43** | 60 | 0 | 4 |
| 2 | shuffled gate | **103** | **103** | 0 | 0 | 4 |

The true gate fired on 103/1,552 generated decisions at seed 1 and 43/1,218 at
seed 2. It produced zero repeated 4-grams because every reply stopped early;
that is not a quality gain.

The seed-1 repeat matched every non-timing and non-resource field and all
retained artifacts. Its normalized report SHA-256 is
`ea26fb8cfbec827bb11dafe605a452f716fcb4c53520433c48945db43772abd4`.
Every arm had zero ungated writer-token mismatches.

Run walls were **2.293 / 2.273 s**, peak RSS **220,148 / 218,152 KiB** and 32
Rayon workers. The repeat took 2.299 s at 216,112 KiB. The release build took
17 s and peaked at 1,210,480 KiB. The pipeline occupies **8,280,196 bytes**.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the binary, build receipt, reports and
all gate/baseline artifact copies.

## Checks run at the head

- FIT review 1 returned NOT_FIT for a missing teacher-forced no-gate control,
  tautological ungated-token accounting, incomplete tests and unbound gate
  hashes.
- After repair, fresh FIT review returned FIT at `c2fc34f19`; final pre-run
  review returned GO at `c885f0e23`.
- Full `uor-r4-cpu-train` release suite: **212 library and 52 binary tests
  passed, 0 failed, 4 expected ignores**. Focused signature tests: **8/8**;
  driver tests: **4/4**.
- Strict no-deps all-target CPU Clippy with `-D warnings`, package formatting,
  and `git diff --check`: passed.
- D23, D25 and D32 ratchets: passed.
- All five public input hashes matched; source matched implementation pin
  `6bd0edff0`; the binary SHA-256 was stable across all runs.
- All per-root checksums passed; the repeat reports and retained artifacts
  matched after recursive removal of timing/resource fields.

## Skills used

No system-provided Codex skill applied. A read-only subagent was used for the
mechanism design audit and independent adversarial FIT and pre-run reviews.

## Still open

- A stop candidate that distinguishes a generated terminal row from premature
  continuation. The current classifier finds reference endings but not the
  model's own stopping point.
- Integer-session integration and exact client/server stop parity, only after a
  CPU candidate survives free generation.
- Content correctness, fresh phrasing, GPU qualification and useful chat.

## Plain summary

A compact full-query Hamming classifier learned the teacher-forced difference
between reply endings and continuation rows on both public seeds and passed the
registered held-out bar with low false stops. That success did not survive free
generation: it stopped prematurely on every seed-1 reply and all 43 seed-2
stops, producing no correct complete replies. The teacher-forced classifier is
useful evidence; the fixed gate is not yet a usable termination mechanism or
integer-serving change.
