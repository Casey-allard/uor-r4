# REPORT: cpu-training-pilot, deepseek-200

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[full-sequence boundary sheet](../../../../harness/prep/cpu-training-pilot-full-sequence-boundary-2026-11-05.md).
Base: `5ed64d3ca`.

## Status

Complete as **CONTROL_FAULT**. The full-history boundary was active and
materially different from its shuffled control, but its observable stopping
behavior was identical to the inherited reply-only boundary on both fresh
artifacts. The registered full-versus-reply non-vacuity gate therefore stopped
the sequence before repeats.

This is a no-effect result for this exact full-history table on the fresh
exact-memory panel, not a negative for geometric termination generally. The
deepseek-198 content selector remains positive in this run: treatment and the
reply-boundary control both produced 15 exact replies and retained their body
positions, while no-boundary and shuffled-full produced zero exact replies.

## What did not work

- Attempt 1 stopped at TRAIN preflight. Exact TRAIN replay made full-history
  and reply-only boundaries stop at the identical 241 positions, while the
  full boundary differed from shuffled at 226 positions. Requiring a
  full-versus-reply change on exact replay was an overstrict instrument gate;
  no seed-2775 panel was created.
- Attempt 2's first invocation exited before any build or panel because
  `/usr/bin/time` was placed outside the shell timeout helper. The empty
  scaffold is preserved separately.
- Attempt 2's fresh per-artifact gate then found full-versus-reply observable
  stop changes of **0** on both artifacts. This is the registered control
  fault: changing the boundary key from generated reply history to full
  prompt-plus-reply history did not change a single stop position.
- Full preamble-to-BOS stopping remains unsolved. Treatment made **66/94**
  premature stops against the four-stop ceiling.

## What changed

- Added `full_sequence_boundary.rs`, which counts nonterminal observations at
  internal reply positions and terminal observations only at final positions,
  over full-history suffixes of depth four through 32.
- Added a matched control that moves terminal counts among equal-depth keys
  while preserving every key, nonterminal count, and per-depth count total.
- Added `cpu-copy-full-boundary`, with treatment, reply-only, shuffled-full,
  no-boundary, local, writer and reference-oracle arms.
- Kept the deepseek-198 content table and depth arbitration fixed, including
  strict local priority at equal depth.
- Added explicit source accounting, response-row stop comparison and tests for
  terminal/internal ambiguity, shuffle invariants, natural-versus-learned BOS,
  stop timing and unreachable rows.

No serving, CUDA, float-training, exporter or Claude-owned source changed.

## Measurements

Fresh panel:
`local/deepseek/full-sequence-boundary/2026-11-05-attempt-2/panels/dd-2775`,
seed 2775, `dev,dev`, protocol 2, 128 conversations, 461 reference-reachable
replies per artifact.

- tokens SHA-256:
  `63b6163e84d5f10a1d87e2c2c38c9038c42548b61e5a55ea36d687c583272501`;
- response-mask SHA-256:
  `bdd2ae86fc1e7279c29145e4d5d8b83925f83265cd55308dc8b3ae1993082f08`;
- manifest SHA-256:
  `39762c8fe2fa09581a239a0e0d497c02674e3b168107db281ea7f7d361ba0d9c`.

The full boundary contained **76,169** suffixes, **11,083** terminal and
**93,080** nonterminal observations, **8,231** terminal-only suffixes and one
ambiguous suffix. The shuffle changed **16,470** terminal associations while
preserving the per-depth census.

| Seed | Arm | Exact | Premature | Opening | Body | Stops |
|---:|---|---:|---:|---:|---:|---:|
| 1 | content + full boundary | 15 | 66 | 69 | 662 | 185 |
| 1 | content + reply boundary | 15 | 66 | 69 | 662 | 185 |
| 1 | content + shuffled full | 0 | 4 | 69 | 662 | 9 |
| 1 | content, no boundary | 0 | 0 | 69 | 662 | 0 |
| 1 | content oracle | 25 | 0 | 69 | 662 | 461 |
| 2 | content + full boundary | 15 | 94 | 69 | 656 | 225 |
| 2 | content + reply boundary | 15 | 94 | 69 | 656 | 225 |
| 2 | content + shuffled full | 0 | 0 | 69 | 656 | 3 |
| 2 | content, no boundary | 0 | 0 | 69 | 656 | 0 |
| 2 | content oracle | 25 | 0 | 69 | 656 | 461 |

Observable stop changes from full to reply-only were **0/0**; full to shuffled
were **179/225**. The first comparison is the registered failure, so the run
stopped before repeats. The output root remains sealed and within the 32 MiB
cap at **5,268,949 bytes**.

Resources:

- release build: **22.50 s**, 1,224,620 KiB peak RSS;
- generator build: **10.71 s**, 1,253,812 KiB peak RSS;
- TRAIN preflight: **3.45 s**, 784,188 KiB peak RSS;
- panel generation: **0.63 s**, 6,976 KiB peak RSS;
- seed 1 / seed 2: **3.67 / 3.66 s**, **827,028 / 816,108 KiB** peak RSS;
- sealed `SHA256SUMS`:
  `310dca229a6889c4a4e228a7fd0dad60ce919589a25c89ff9b995685ba6fafe5`.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

Full prompt history did not repair stopping relative to the reply-only table.
Both learned boundaries fired at the same generated positions, even though
their source labels and table support differed. The shuffled full boundary
therefore establishes that terminal placements matter, but full-history keying
does not by itself create a different endpoint decision.

The content mechanism remains the useful result from deepseek-198. This unit
closes this full-history terminal/nonterminal table as a termination repair.
The next endpoint work should change the continuation objective or bind the
terminal action to the selected content source rather than enlarge the same
suffix key.

## Checks run at the head

- implementation source review: APPROVE at `ce180de32`;
- non-author TEST FITNESS: FIT at `71d9d2d9d` and again at `5fb6700fa`;
- separate D67 pre-run review: GO at `945f677f5`;
- focused full-boundary tests: **5 passed**;
- full CPU release suite: **380 passed**, 4 expected ignores;
- strict scoped Clippy and formatting: passed;
- runner shell syntax and all source/input/generator pins: passed;
- attempt-1 and invocation-fault evidence: preserved, compacted and sealed;
- fresh panel and both artifact evaluations: passed input and reachability
  gates;
- fresh per-artifact control gate: correctly returned `CONTROL_FAULT`;
- output compaction on the exit path and 32 MiB evidence limit: passed;
- repeats: not run after the registered stop condition fired.

## Delivery

Pending exact-head review, protected merge, fresh-main verification and claim
release.

## Skills used

No system-provided Codex skill applied. Repository pre-registration, FIT/GO,
D84 CPU research and protected-delivery procedures were used.

## Still open

- termination that changes position, not just terminal-table provenance;
- termination aligned to the selected content source or generated-state
  continuation;
- fresh-phrasing generalization beyond exact-memory recall;
- Claude's independent scale, export and serving track.

## Plain summary

The new full-history boundary is real and differs from its shuffled control,
but it stops at exactly the same positions as the old reply-only boundary on
both fresh artifacts. It therefore cannot repair premature termination. The
run stops as a control fault before repeats; the content selector remains a
positive mechanism worth preserving, and the next stop attempt must change the
endpoint objective rather than only the suffix key.
