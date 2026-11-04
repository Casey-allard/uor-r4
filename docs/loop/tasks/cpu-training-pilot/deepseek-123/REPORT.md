# REPORT: target point medoids, deepseek-123

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[point medoids](../../../harness/prep/cpu-training-pilot-point-medoid-2026-11-04.md).
Base: `5b3b8f361`.

## What did not work, first

- **The fixed target-medoid construction did not pass the registered bar.** On
  seed 1 it scored 0/60 unseen-key openings, the same as point-off and its
  shuffled medoid. On seed 2 it scored 4/60: the required four-row gain over
  point-off passed, but the shuffled medoid scored 3, leaving a gain of one
  against the required threshold of two.
- **Collapsing 48 raw rows to 18 target medoids lost the raw neighborhood
  signal.** The already-served eight-row raw vote remained 3/60 on both seeds,
  and raw nearest reached 2/60 and 1/60. The one-row-per-target representation
  erased that local evidence on seed 1.
- **The complete teacher-forced non-loss check was not sufficient for a
  positive.** Seed 2's medoid reached 167/2,050 versus 163 point-off and 166
  shuffled medoid, but the pre-registered opening comparison is the primary
  bar and failed. Seed 1's medoid tied point-off and shuffled medoid at 169;
  it remained below raw point (171) and raw vote (172).
- **This is not a general medoid or geometry negative.** It closes one
  deterministic construction: one observed L1 medoid per target over this
  fixed 60-example training set. It does not reject target-specific local
  neighborhoods, a different medoid population, or full-corpus training.
- The first automated claim again failed on its generated commit trailers and
  rolled itself back. The branch and worktree were recreated manually. The
  first manual ORDER push was refused because its item link was split across
  lines and the required REPORT stub was absent; both were corrected before
  implementation was published.
- The first FIT review returned **NOT_FIT**. The mechanism, controls and bar
  passed review, but the run recipe did not pin source drift, persist a binary
  receipt and output checksums, record resource/storage bounds, or name the
  exact repeat fields. The sheet now adds all four before a second FIT.
- The second FIT review found that the recipe built and checked the shared
  `main` checkout rather than this branch worktree. The recipe now separates
  `SOURCE` from `SHARED`; the third review returned FIT at docs head
  `3aadd33af6f46af7159eb4a86c51adbf85de8350`.
- The public manifest says `split=train`. "Unseen-key" therefore means those
  exact opening keys were absent from the learned memory and point tables, not
  that base-model exposure is proven zero.

## What changed

Added target-level geometric medoids to the existing unseen-key point-transfer
driver without changing its raw-row artifacts. Training groups the production
point rows by supervised target and chooses an observed medoid by exact L1 sum.
The unit also builds a matched shuffled-target medoid control, evaluates both
through the complete Hamming/memory/point path and applies the pre-registered
paired opening margin plus complete teacher-forced non-loss check.

The existing raw-row artifacts remain byte-identical to deepseek-106. The
driver now emits target-medoid and shuffled-target-medoid artifacts in addition
to memory-only, raw nearest, raw vote and shuffled raw-row control artifacts.
Each point-bearing arm reports how many of the 60 openings its point table
changed relative to point-off.

## Measurements

Both seeds selected the same fixed split: 8 training keys and 8 disjoint
evaluation keys, 60 training and 60 evaluation examples, 54/50 documents and
zero evaluation exact-memory hits. Training produced 48 raw point rows and 18
target medoids at width 256 on both seeds. Every point-bearing arm changed all
60 served openings relative to point-off.

| Seed | Point-off | Raw nearest | Raw vote k=8 | Target medoid | Shuffled medoid |
|---|---:|---:|---:|---:|---:|
| 1 | 0/60 | 2/60 | 3/60 | 0/60 | 0/60 |
| 2 | 0/60 | 1/60 | 3/60 | 4/60 | 3/60 |

| Seed | Medoid vs point-off gain/threshold | Medoid vs shuffled gain/threshold | Full medoid / point-off / shuffled | Registered pass |
|---:|---:|---:|---|---|
| 1 | 0/2 | 0/2 | 169 / 169 / 169 | no |
| 2 | 4/4 | 1/2 | 167 / 163 / 166 | no |

All six arms on both seeds and the repeat retained zero mismatches over
**2,110 positions, 540,160 Q12 coordinates, 2,110 token decisions and 16,880
kept cells per seed**. Opening target distance was identical across arms.
Medoid walls were **1.627/1.635 s** and peak RSS was **302,464/302,552 KiB**
with 32 Rayon workers. The seed-1 repeat matched every non-timing field, with
normalized report SHA-256
`5d49a9dd45cbc89ca7039ea67c2b7d822e700199cf89432981b1f5ca225134a4`.
All roots used binary SHA-256
`682269716e87552fc4b3adb64dcb396272cb091b95a436cabb356a3c45f5e23f`.
The three roots occupy 39,739,320 bytes.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the binary, reports, raw rows and
medoid artifacts. Raw artifacts remain under
`local/deepseek/point-medoid/2026-11-04-attempt-1/`.

## Checks run at the head

- Focused point-transfer release tests: 16 passed, including medoid selection,
  distance and target ties, deterministic construction, shuffled targets,
  served-query agreement, malformed tables, paired margin arithmetic and
  non-zero point-fallback exposure.
- Fresh FIT passed at docs head
  `3aadd33af6f46af7159eb4a86c51adbf85de8350`, bound to source
  `b17638480e1b2f264b511c0351639e669fb6bc46`.
- Registered two-seed and deterministic-repeat runs completed; every per-root
  `SHA256SUMS` verifies.
- Full `uor-r4-cpu-train` release suite: 195 library tests passed, 4 release
  benchmarks ignored, plus all binary suites (229 tests passed in total).
- Strict no-deps all-target CPU Clippy with `-D warnings`: passed.
- Formatting, `git diff --check`, report check, roadmap check (139 items),
  documentation catalog (1,175 records, zero missing paths), prep check, D23,
  D25 and D32 ratchets, and all 11 full loop gates passed.

## Skills used

No system-provided skill applied.

## Still open

- A target-specific local representation that preserves the raw neighborhood
  evidence while reducing noisy labels without post-observation tuning.
- Complete-reply and fresh-phrasing capability, corpus-scale training and GPU
  qualification.
- Claude's materialized long-window export, when available, for the separate
  `cpu-served-forward-audit`.

## Plain summary

This unit tests a different point-training representation: one observed
geometric medoid per target instead of the raw per-example rows and their local
vote. The medoid was reached, changed all 60 openings and kept exact serving
parity. It nevertheless failed the matched comparison: seed 1 tied both
controls at zero, and seed 2's one-row advantage over its shuffled medoid was
below the registered threshold. The raw-row neighborhood remains the stronger
tested representation, but this bounded CPU result is not a useful-chat or GPU
qualification.
