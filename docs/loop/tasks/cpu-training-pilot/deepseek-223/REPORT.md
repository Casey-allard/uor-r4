# REPORT: cpu-training-pilot, deepseek-223

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[content-length sheet](../../../../harness/prep/cpu-training-pilot-content-length-2026-11-06.md).
Base: `bbb8c5332`.

## Status

Complete as **REJECT**. Both public artifacts and both fresh-process repeats
ran. TEST FITNESS: FIT and pre-run GO were recorded at implementation
`74793da76` before seed 2783 opened.

## What did not work

- Stored-length completion removed premature BOS, but it did not preserve the
  content trajectory from deepseek-198. The candidate made **6** exact replies
  and **560** correct body positions on each artifact. The original boundary
  arm made **14** exact and **760** body positions.
- The candidate retained the better body count of its local-length control
  (**425**) but failed the registered comparison with the boundary control,
  which supplied the **760**-position bar.
- The result was not competitive with full stored retrieval, which made **42**
  exact replies and **879** body positions on each artifact.
- Changing the stop from boundary to retrieved length changed the complete
  reply on **236/457** rows relative to both local and shuffled controls. The
  mechanism was active; it changed content as well as termination.
- The reference-length oracle reached only **16** exact and **599** body
  positions. A correct length alone did not recover the boundary arm's content
  quality, so the stored-length length is not the only limitation.

## What changed

Registered full-sequence suffix content with reference-independent stored-length
completion and matched controls on fresh seed 2783.

The evaluator now supports an exact stored-length endpoint without changing the
default boundary mode. Length completion is recorded separately from BOS, and
exact-reference identity is independent of which legal stop ended the row.

## Measurements

Seed 2783 contained 128 conversations, 16,319 tokens, 5,163 response tokens
and **457** reachable replies. The same panel and readings appeared on both
artifacts:

| Arm | Exact | Body positions | Premature BOS | Generated tokens |
|---|---:|---:|---:|---:|
| full content + stored length | 6 | 560 | 0 | 4,967 |
| full content + boundary | 14 | 760 | 73 / 90 | 18,706 / 17,302 |
| shallow global + stored length | 6 | 521 | 0 | 4,967 |
| local longest + stored length | 0 | 425 | 0 | 4,967 |
| shuffled suffix + stored length | 0 | 425 | 0 | 4,967 |
| writer + stored length | 0 | 38 / 52 | 0 | 4,967 |
| full retrieval | 42 | 879 | 0 | 4,967 |
| reference-length oracle | 16 | 599 | 0 | 4,706 |

The full content candidate made **1,517** deeper-global decisions on seed 1
and **1,562** on seed 2. Every one of the 457 reachable replies completed by
stored length, with zero premature BOS. The candidate beat local and shuffled
controls on exact and body positions on both artifacts, and changed
**2,271/2,270** steps from local and **2,274/2,273** from shuffled. Those
bounded positives do not recover the failed retention bar.

The equal-cost length comparisons do not explain the content loss: the
reference-length oracle generated the same **4,706** reference-content tokens
but scored only 599 body positions, versus 760 for the longer boundary arm.
The boundary arm's extra comparisons can find correct content after the stored
length endpoint; stopping at the retrieved length exposes how much the suffix
trajectory still depends on continued arbitration.

Artifact seed 1 pinned:

- panel tokens:
  `15790736fef71aeddfb2d5a2bd5686379e478690d2a83c5bda9a57f19bd166f1`;
- response mask:
  `913592f695e841a07512345090e62ff71393ad437e5fb5a2e956f52b5f624946`;
- manifest:
  `90e8f9e9a1dabc393b124a66e2c1fed0c9d3f670bf0128b60b00674032d87b78`;
- `seed1.json`:
  `00a25eb913fed6be5bd1163dac439be08378c90334dff4448cca8293964490ed`;
- `seed2.json`:
  `38a5a40cb7e9e7f8137bea5854dc1e11a93eed95195b2e1dffbba94f5fed14a6`.

Each artifact's repeat report matched its first report byte-for-byte after
timing removal. Raw `SHA256SUMS` was
`440b75dd443a18600c967095f2807c29e043272681e1c3a44a997ed9dd6fb293`.
Retained evidence was **151,172 bytes**.

Resources:

- four scored processes completed in **5.84-5.92 s** at about **10.0 CPU
  cores** each;
- peak scored-process RSS was **265,988 KiB**;
- the release driver build took **29.90 s** and peaked at **1,360,392 KiB**;
- the panel generator took **0.64 s** and peaked at **7,444 KiB**.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

The stored length works as a completion budget: it emits every row and removes
premature BOS. It is not a quality-preserving substitute for the boundary
trajectory in this construction. The full-sequence content selector remains
supported by the earlier deepseek-198 result; this test closes the fixed
combination of that selector, nearest retrieved length and 64-token cap.

The next attempt should change the continuation or stopping state available
to the same full-history content trajectory instead of treating a retrieved
reply length as sufficient termination evidence. Native model-derived
termination remains open.

## Checks run at the head

- fresh panel and output roots absent at registration.
- focused stored-length tests: 2 passed;
- driver usage test: 1 passed;
- full `uor-r4-cpu-train` library: 404 passed, 4 expected ignores;
- scoped Clippy passed for the driver and library with the pre-existing
  `type_complexity` lint exempted; dependency crates still emit pre-existing
  warnings;
- the new evaluator and driver source files pass rustfmt; whole-repository
  `cargo fmt --all -- --check` fails on pre-existing formatting drift outside
  this delivery;
- `git diff --check` passed;
- source, driver and release binary hashes frozen before GO.
- both artifacts emitted all 457 reachable rows with zero premature BOS in the
  length-completion arm;
- both artifacts beat local and shuffled exact/body controls;
- both artifacts failed the registered boundary retention comparison;
- seed-1 and seed-2 fresh-process repeats matched after timing removal;
- seeded BOS transitions were zero in both content and shuffled suffix tables;
- the suffix shuffle changed 75,034 targets while preserving support,
  observations and successor multisets;
- D23, D25, D32 and claim-wording ratchets passed;
- `prep-check` reports the content-length sheet FIT and complete;
- `loop report --check` and `loop pr-check --local` passed;
- `loop gates --full`: 11/11 green after one transient `loop-drive` rerun
  cleared a `Text file busy` test-fixture race;
- `xtask docs-catalog --check`: 1,175 records, 20 lines, zero missing paths;
- `xtask roadmap check` remains blocked by the pre-existing
  `infra-desktop-client-tauri` status/issue mismatch, which this diff does not
  touch;
- implementation review was a local exact-head self-review because independent
  delegation was unavailable during the registered run.

## Skills used

No system-provided Codex skill applied.

## Still open

- a model-derived stop that preserves the full-sequence content trajectory;
- a matched termination objective that changes behavior rather than only table
  assignments;
- Claude's independent end-to-end serving track.

## Plain summary

Full-sequence suffix content plus the nearest retrieved reply length emitted
all 457 replies without premature BOS and beat local and shuffled length
controls. It made only 6 exact replies and 560 body positions, below the
760-body boundary control, so the registered body-retention bar failed. The
stored length is a working budget but not yet a quality-preserving stop.
