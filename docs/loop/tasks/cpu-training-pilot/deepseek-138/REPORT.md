# REPORT: generated-state terminal content rows, deepseek-138

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `11f8303cc`.

## What did not work

- **The endpoint rows did not terminate any reply.** Seed 1 and seed 2 both
  produced **0/103** BOS completions, 0 exact replies and 103 cap stops.
- **The stop table perturbed content on seed 1.** Top-1 fell
  **73→60** and matching depth **64→46**, failing the non-inferiority gate.
- **Seed 2 was behaviorally unchanged.** Code and code-stop both read
  **58** top-1 and **39** depth, with no BOS stop.
- **The endpoint is learnable but does not transfer to free generation.** The
  augmented table raised leave-one-out hits from **300 to 325** on seed 1 and
  **284 to 315** on seed 2, but no held-out generated state selected BOS.
- **The shuffled control is not inert.** It stopped 2 replies on seed 1 and
  27 on seed 2, all non-exact and premature/shorter.
- **This is not serving evidence.** The artifact and integer session remain
  unchanged.

## What changed

The modulo-256 content readout can now add a target-BOS row when an actual
generated training trajectory reaches exactly the normalized reference
length. No rows are added after that offset. The driver runs writer, sign,
code, code-shuffled, code-stop and code-stop-shuffled arms over complete
replies.

`ContentPromptReading` now records generated and reference lengths. The bar
reports BOS-within-reference, shorter and longer separately.

## Measurements

| Seed | Table | Rows | LOO top-1 | Ambiguous | Covered | Threshold |
|---:|---|---:|---:|---:|---:|---:|
| 1 | code | 894 | 300 | 277 | 807 | 537 |
| 1 | code + endpoint | 978 | 325 | 340 | 881 | 529 |
| 2 | code | 894 | 284 | 262 | 806 | 533 |
| 2 | code + endpoint | 978 | 315 | 300 | 883 | 522 |

Generated readings:

| Seed | Arm | Top-1 | Depth | Opening | Distinct | Repeat 4-gram | BOS | Exact |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | code | 73 | 64 | 33 | 25 | 69 | 0 | 0 |
| 1 | code + endpoint | 60 | 46 | 30 | 27 | 71 | 0 | 0 |
| 1 | endpoint shuffle | 19 | 0 | 0 | 18 | 96 | 2 | 0 |
| 2 | code | 58 | 39 | 29 | 26 | 24 | 0 | 0 |
| 2 | code + endpoint | 58 | 39 | 29 | 27 | 25 | 0 | 0 |
| 2 | endpoint shuffle | 16 | 4 | 4 | 15 | 8 | 27 | 0 |

All endpoint-table BOS-within-reference counts are zero; shorter and longer
BOS counts are also zero. Every true stop arm hit the cap on all 103 prompts.

Build and runs:

- release build: **18 s**, peak RSS **1,208,568 KiB**, target
  **265,063,547 bytes**;
- walls: **0.86/0.87/0.85 s** at 32 Rayon workers for seed 1, seed 2 and the
  seed-1 repeat;
- run RSS: **223,136/223,100/222,784 KiB**;
- binary SHA-256:
  `6a0d708688e51da7e3bcd40cef3cc7a968d6e70a6724371a9b620276148026dc`;
- committed evidence: **31,849,539 bytes**;
- evidence `SHA256SUMS`:
  `11d8aab5b9620965035f7177948a96afcd4f85d20bd35d310ed6669ffe5ef4ef`.

Compact evidence:
[comparison.json](../../../../labs/cpu-training-pilot-content-stop-2026-11-04/evidence/comparison.json).

## Checks run at the head

- Independent FIT reviews rejected missing prep gates, incomplete repeat
  comparison, an untested non-inferiority predicate and ambiguous stop
  classification. Each was repaired before compute.
- Fresh FIT approved implementation pin `70a0ae4d3`; fresh GO approved sheet
  head `35e0258e2`.
- Full `uor-r4-cpu-train` release suite: **240 passed, 4 expected ignores**,
  plus **58 binary tests passed**.
- Strict CPU Clippy with `-D warnings --no-deps`, targeted rustfmt and
  `git diff --check` passed.
- All five public inputs and the implementation pin matched.
- Seed-1 normalized report and all six generated readout files matched the
  repeat byte-for-byte.
- Recursive evidence checksums passed.
- No integer-session, serving, CUDA, float-training or Claude-owned source
  changed.

## Skills used

No system-provided Codex skill applies. Read-only subagents performed the
independent FIT, repair re-review, pre-run GO and delivery reviews.

## Still open

- A separate two-class generated-state stop gate rather than adding BOS to
  the content target table.
- Serving-artifact integration for the positive modulo-256 content readout.
- Complete replies, fresh phrasing and useful local chat.

## Plain summary

Knowing where a generated training trajectory reached the reference endpoint
does not teach the held-out code table to stop there. The augmented table
improved its training-neighbour fit, but selected BOS on no held-out replies
and damaged seed-1 content. Generated-state terminal rows in the shared
content table are therefore closed; the positive content code remains intact.
