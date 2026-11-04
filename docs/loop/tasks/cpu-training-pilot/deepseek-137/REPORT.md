# REPORT: modulo-256 content signature, deepseek-137

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `8cf088bd9`.

## What did not work

- **No reply completed.** All 103 prompts in every arm hit the 16-token cap.
  The code arm produced zero exact-reference and zero BOS-stopped replies.
- **The override remains sparse.** The code readout fired on 103/1,648 and
  113/1,648 generated decisions; the writer handled the other 1,545 and
  1,535.
- **Repetition remains substantial.** Seed 1 repeated a 4-gram in 69/103 code
  replies. Seed 2 did so in 24/103. The sign arm had 103/103 and 0/103,
  respectively, so duplication is not monotone across seeds.
- **The fitted boundary is broad.** The code threshold is 537 and 533 bits of
  2,048, around 26%. It is a training-fitted Hamming radius, not exact-code
  equality.
- **The panel is still repeated-key development data.** It is not fresh
  phrasing, useful conversation or a serving result.

## What changed

The CPU content readout now supports two deterministic signatures over the
same complete 256-Q12 generated query:

1. **sign**: the prior 256-bit sign code; and
2. **code**: each coordinate's low byte modulo 256, forming 2,048 bits.

Both modes use the same actual generated trajectories, training-derived
memory, 90% leave-one-out threshold rule, commonest-target vote, smaller-token
tie break, exact writer fallback and effective-map shuffled control. The
driver runs writer, true/shuffled sign and true/shuffled code over complete
generated replies.

## Measurements

Training-only readings:

| Seed | Mode | Unique signatures | LOO top-1 | Ambiguous | Covered | Threshold | Changed map |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | sign | 45 | 199/894 | 892 | 891 | 0 | 45/45 |
| 1 | code | **619** | **300/894** | **277** | 807 | 537 | 618/619 |
| 2 | sign | 46 | 201/894 | 879 | 887 | 0 | 46/46 |
| 2 | code | **635** | **284/894** | **262** | 806 | 533 | 634/635 |

The code expands distinct states, raises leave-one-out hits by 50.8% and
41.3%, and cuts ambiguous rows by 68.9% and 70.2%.

Aligned generated readings:

| Seed | Arm | Top-1 | Prefix depth | Opening | Distinct | Repeat 4-gram |
|---:|---|---:|---:|---:|---:|---:|
| 1 | writer | 22 | 0 | 0 | 2 | 24 |
| 1 | sign | 67 | 60 | 24 | 4 | 103 |
| 1 | sign shuffle | 4 | 0 | 0 | 4 | 98 |
| 1 | code | **73** | **64** | **33** | 25 | 69 |
| 1 | code shuffle | 2 | 0 | 0 | 31 | 96 |
| 2 | writer | 7 | 4 | 4 | 2 | 43 |
| 2 | sign | 42 | 33 | 25 | 7 | 0 |
| 2 | sign shuffle | 18 | 0 | 0 | 8 | 0 |
| 2 | code | **58** | **39** | **29** | 26 | 24 |
| 2 | code shuffle | 20 | 4 | 4 | 23 | 35 |

Prompt-paired code margins:

| Seed | Comparison | Gain | Only/control-only | Threshold |
|---:|---|---:|---:|---:|
| 1 | code vs writer top-1 | +51 | 39/14 | 22 |
| 1 | code vs shuffled top-1 | +71 | 38/1 | 19 |
| 1 | code vs sign top-1 | +6 | 29/19 | 21 |
| 1 | code vs writer depth | +64 | 33/0 | 18 |
| 1 | code vs shuffled depth | +64 | 33/0 | 18 |
| 1 | code vs sign depth | +4 | 24/15 | 19 |
| 2 | code vs writer top-1 | +51 | 37/3 | 19 |
| 2 | code vs shuffled top-1 | +38 | 34/11 | 21 |
| 2 | code vs sign top-1 | +16 | 24/16 | 19 |
| 2 | code vs writer depth | +35 | 25/0 | 15 |
| 2 | code vs shuffled depth | +35 | 25/0 | 15 |
| 2 | code vs sign depth | +6 | 18/14 | 17 |

Both seeds pass every structural, exposure, writer, shuffled and sign
comparison in the registered bar.

Compact evidence:
[comparison.json](../../../../labs/cpu-training-pilot-content-code-2026-11-04/evidence/comparison.json).

Build and runs:

- release build: **19 s**, peak RSS **1,203,536 KiB**, target
  **264,991,309 bytes**;
- walls: **0.60/0.59/0.60 s** at 32 Rayon workers for seed 1, seed 2 and the
  seed-1 repeat;
- run RSS: **220,484/220,760/220,692 KiB**;
- binary SHA-256:
  `c251688c95918bee5aac2bc721359b3dbd4471fb2870eee987d93a02efc02a17`;
- committed evidence: **14,418,550 bytes**;
- evidence `SHA256SUMS`:
  `5fb1a10fd88b03421062fd6b622e1a1b1658c29903d9c66137c784ad44b78f41`.

## Checks run at the head

- Independent FIT review approved implementation pin `0ff468aae`.
- Fresh independent GO review approved FIT-sheet head `ce9960d8e`.
- Full `uor-r4-cpu-train` release suite: **239 passed, 4 expected ignores**,
  plus **57 binary tests passed**.
- Strict CPU Clippy with `-D warnings --no-deps`, targeted rustfmt and
  `git diff --check` passed.
- All five public inputs, implementation pin, split and score dimensions
  matched.
- Seed-1 normalized report and all four generated readout files matched the
  repeat byte-for-byte.
- Recursive evidence checksums passed.
- No integer-session, serving, CUDA, float-training or Claude-owned source
  changed.

## Skills used

No system-provided Codex skill applies. Read-only subagents performed the
independent FIT, pre-run GO and exact-head delivery reviews.

## Still open

- A full serving artifact and integer parity for either content signature.
- Stop selection and continuation that can produce complete replies.
- Fresh-phrasing generalization and useful conversation.
- GPU qualification under a later owner decision.

## Plain summary

Keeping each generated query coordinate's low byte makes the geometric
content lookup materially sharper. The code has far fewer ambiguous training
states, beats both the writer and the behaviorally distinct shuffled control
on both seeds, and improves on the existing sign readout. It still only
overrides a small fraction of decisions and never completes a reply.
