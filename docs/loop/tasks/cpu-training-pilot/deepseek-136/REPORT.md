# REPORT: generated-state Hamming content readout, deepseek-136

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `74919a81a`.

## What did not work

- **The readout still does not produce conversational replies.** Both seeds had
  zero exact-reference and zero BOS-complete replies. Every prompt hit the
  16-token cap in every arm.
- **Training-only leave-one-out prediction was weak.** Only **199/894** and
  **201/894** rows chose the correct target, and **892/894** and **879/894**
  had multiple targets at their nearest distance. The held-out prompt-paired
  gains are real under the registered rule, but this is a sparse override
  signal rather than a learned content model.
- **The readout fired mainly at reply openings.** Seed 1 used the readout on
  103/1,648 observed decisions; seed 2 used it on 117/1,648. The distance-zero
  threshold fell back to the writer at 1,545 and 1,531 decisions.
- Seed 1's true arm repeated a 4-gram in all 103 replies and produced only four
  distinct non-empty replies. The gain is alignment evidence, not diversity or
  useful-chat evidence.

## What changed

Added a CPU-only generated-state Hamming content readout:

1. each actual generated training trajectory contributes `(256-bit sign
   signature, reference target)` rows;
2. fitting selects the nearest leave-one-out distance threshold covering at
   least 90% of rows;
3. decode chooses the commonest target at minimum XOR/popcount distance,
   breaks equal-count target ties by smaller token id, and falls back to the
   exact writer beyond the threshold;
4. independent prompt evaluation uses ordered Rayon lanes for writer,
   true-target and shuffled-target arms; and
5. the shuffled control must change the effective commonest-target map keyed
   by unique signature, not merely reorder rows.

The exact-head review caught a potential inert-shuffle control before compute.
The implementation now rejects an unchanged per-signature mapping, reports the
number of changed assignments, and has a repeated-signature regression test.

## Measurements

| Seed | Rows | LOO top-1 | Ambiguous LOO | Coverage | Threshold |
|---:|---:|---:|---:|---:|---:|
| 1 | 894 | 199 | 892 | 891 | 0 |
| 2 | 894 | 201 | 879 | 887 | 0 |

Aligned generated readings:

| Seed | Arm | Top-1 | Prefix depth | Opening | Exact | BOS | Readout | Fallback |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | writer | 22 | 0 | 0 | 0 | 0 | 0 | 0 |
| 1 | true | **67** | **60** | **24** | 0 | 0 | 103 | 1,545 |
| 1 | shuffled | 4 | 0 | 0 | 0 | 0 | 103 | 1,545 |
| 2 | writer | 7 | 4 | 4 | 0 | 0 | 0 | 0 |
| 2 | true | **42** | **33** | **25** | 0 | 0 | 117 | 1,531 |
| 2 | shuffled | 18 | 0 | 0 | 0 | 0 | 110 | 1,538 |

Prompt-paired margins:

| Seed | Comparison | Gain | Only/control-only | Threshold |
|---:|---|---:|---:|---:|
| 1 | true vs writer top-1 | +45 | 29/18 | 21 |
| 1 | true vs shuffled top-1 | +63 | 29/4 | 18 |
| 1 | true vs writer depth | +60 | 24/0 | 15 |
| 1 | true vs shuffled depth | +60 | 24/0 | 15 |
| 2 | true vs writer top-1 | +35 | 31/5 | 18 |
| 2 | true vs shuffled top-1 | +24 | 32/18 | 22 |
| 2 | true vs writer depth | +29 | 25/4 | 17 |
| 2 | true vs shuffled depth | +33 | 25/0 | 15 |

Both seeds pass every registered training-coverage, exposure, top-1 and depth
bar. Seed 1 changed **45/45** effective signature assignments under the
shuffled control; seed 2 changed **46/46**.

True mapping SHA-256:

- seed 1: `dad81de4c82a05898f2f16afd5aa587472d60bf93102d042dc51075377c80edd`;
- seed 2: `24d4329ae8755163a51a3feb994929289bfa74b35960f6ba831ce0e61c57d585`.

Build and run:

- release build took **18 s**, peaked at **1,214,752 KiB** and occupied
  **264,899,680 bytes**;
- runs took **0.31/0.29/0.29 s** at **32 Rayon workers**;
- run RSS peaked at **211,888/204,008/210,044 KiB**;
- binary SHA-256 is
  `ad6435db480cac5d6046158b37e875101ed776233343cc3e82435c2dd82a2daf`;
- retained pipeline is **3,096,605 bytes**; and
- evidence `SHA256SUMS` is
  `482e4631ca8e27e03c7a953ef8c395786b323e23e5b53a37ae8d67a0368c0825`.

## Checks run at the head

- Exact-head code review rejected an initially hash-only shuffle check. The
  effective mapping guard, changed-assignment report and repeated-signature
  regression repaired it; a fresh independent review approved the repaired
  pin `acaeb3e49`.
- Full `uor-r4-cpu-train` release suite: **239 passed, 4 expected ignores**,
  plus **57 binary tests passed**.
- Strict CPU Clippy with `-D warnings`, targeted rustfmt, `git diff --check`,
  prep shape, source pin and all five public input hashes passed.
- Both seeds and the seed-1 repeat exited 0.
- Seed-1 normalized reports and both readout evidence files match the repeat
  byte-for-byte; recursive evidence checksums pass.
- No serving, integer-session, CUDA, float-training or Claude-owned source
  changed.

## Skills used

No system-provided Codex skill applies.

## Still open

- Exact-head delivery review, protected merge and fresh-main verification.
- A serving-contract design for carrying this readout through the integer
  artifact without changing Claude's track.
- Multiple-row or learned-rotation variants if this single-row, zero-threshold
  configuration is extended.
- Complete replies, BOS termination, fresh phrasing, throughput and useful
  local chat remain unproven.

## Plain summary

Exact generated-state Hamming lookup substantially raises aligned next-token
choices and matching-prefix depth over both writer and shuffled controls on
both public seeds. It stays a small fallback override, leaves all replies
incomplete and often repetitive, and does not yet justify a serving or chat
claim.
