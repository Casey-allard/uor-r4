# REPORT: within-cell orbit termination, deepseek-122

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `cb457faa4`.
Prep:
[orbit termination](../../../harness/prep/cpu-training-pilot-orbit-termination-2026-11-04.md).

## What did not work, first

- **The fixed cell-relative orbit-phase rule made no correct stop.** On seed 1,
  the trained gate stopped all 114 replies and every stop was premature; the
  matched label-permutation control also stopped all 114 and was likewise
  entirely premature. On seed 2, the trained gate stopped all 114, while its
  control stopped 48; all 162 stops were premature. Both paired comparisons
  failed on both seeds.
- **Reaching a gate was not the missing ingredient.** Every run entered gated
  cells: the trained gate made 244 and 114 gated decisions on seeds 1 and 2,
  respectively. The controls made 258 and 114. Their strict nearest-class
  choices selected ordinary continuation rows instead of the final row.
- **No arm completed an exact reference.** Exact-reference counts were zero
  for no gate, trained gate and shuffled control on both seeds, even though
  104/114 references fit the 16-token cap.
- **This is not the corpus-trained Hamming/memory/point artifact.** It changes
  only termination on the existing memory-trained public artifacts. It cannot
  qualify serving or language capability; GPU remains the qualification
  authority.
- The automated `loop claim` initially failed while creating its first commit
  because the generated message lacked the required `Lab:` and `Order:`
  trailers. It reported a complete rollback; the ORDER and claim were recreated
  manually before implementation.
- A point-key context repair was refused because Claude's open
  `one-model-end-to-end` order shares the `stack/format.rs` scope. The repair
  was not bypassed here.

## What changed

Added a cell-relative orbit-signature termination gate alongside the two
closed termination rules:

- every prompt-final and earlier reply-content training query is nonterminal;
  the final content row before corpus EOS is terminal;
- each Q12 query yields a 32-integer signature from the exact L1 cost to all 16
  orbit roots in the first and last eight-coordinate blocks, normalized within
  each block by its winning root;
- the query's writer top cell scopes the terminal and nonterminal training
  signatures;
- a cell is admitted only when it contains both classes; decode names decoder
  BOS only when the exact L1 distance to a terminal signature is strictly less
  than to every nonterminal signature, with ties and missing classes preserved
  as no-stop;
- a deterministic duplicate-safe label permutation preserves each cell, the
  signature multiset and both class counts; and
- the driver records changed signatures, gated/ungated decision counts,
  per-row premature/BOS/exact outcomes, exact CPU/serving parity and ordered
  reply digests.

The trained and shuffled arms are mutually exclusive with the prior point and
cell gates. The report schema advances to
`uor-r4.cpu-memory-panel-run/4`.

The reviewed implementation is four source files, `+791/-64` lines against
`cb457faa4`. The full initial PR was eight files, `+1146/-64`. This final
evidence/documentation update adds only task records and the notebook/roadmap
links.

## Measurements

Both seeds accepted all 32 memory-weight updates and reproduced the pinned
deepseek-109 trained artifacts. The no-gate digests matched deepseek-110. The
reference census remains 1,212 content tokens with 104/114 reachable.

| Seed | Arm | BOS complete | Premature | Non-premature BOS | Exact | Cap | Gated decisions |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | no gate | 0 | 0 | 0 | 0 | 114 | 0 |
| 1 | orbit gate | 114 | 114 | 0 | 0 | 0 | 244 |
| 1 | shuffled orbit gate | 114 | 114 | 0 | 0 | 0 | 258 |
| 2 | no gate | 0 | 0 | 0 | 0 | 114 | 0 |
| 2 | orbit gate | 114 | 114 | 0 | 0 | 0 | 114 |
| 2 | shuffled orbit gate | 48 | 48 | 0 | 0 | 66 | 114 |

| Seed | Terminal/nonterminal points | Both-class cells | Changed control cells/points | Trained gate SHA-256 | Shuffled gate SHA-256 |
|---:|---|---:|---|---|---|
| 1 | 102 / 131 | 7 | 7 / 34 | `2f31ce08…24b8` | `d1344d50…5440` |
| 2 | 101 / 41 | 6 | 6 / 32 | `2b03164c…5182` | `02dfc2f4…eef` |

The paired margin was `max(2, ceil(2*sqrt(b+c)))`. Every comparison had
`b = c = 0` in non-premature BOS completion, so the required margin was two
and the observed margin was zero. Exact-reference non-loss held only because
both sides were zero; functionality still failed.

Every base arm on both seeds and the repeat retained zero mismatches over
**4,333 decisions, 1,109,248 Q12 coordinates and 34,664 kept cells** across
five served arms. Walls were **41.488/42.112 s** and peak RSS was
**390,312/405,672 KiB** with 32 Rayon workers. The fresh seed-1 repeat matched
the complete normalized report, with SHA-256
`1b398bcad5a0c2dc422306a89ff7cb33d1467b5eef73186237ccbb803078076d`.
All three roots used binary SHA-256
`188c224fb9a7c091ebd152d8c383fefdf43237f538bdb5adef1c7de8f443f474`.
The three roots occupy 28,083,525 bytes.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the binary, raw reports and trained
artifacts. Raw artifacts remain under
`local/deepseek/inner-cell-orbit/2026-11-04-attempt-1/`, and every root's
`SHA256SUMS` verifies.

## Checks run at the head

- Focused orbit tests cover aligned roots, exact integer costs, block
  normalization, cell equality, malformed width refusal, strict-terminal and
  tie/no-stop behavior, deterministic re-derivation, duplicate-safe changed
  controls, training-only terminal derivation, gated-decision exposure,
  premature-stop accounting and no-gate digest preservation.
- The full `uor-r4-cpu-train` release suite passes: 195 library tests passed,
  4 release benchmarks ignored, plus every binary suite.
- Strict all-target CPU Clippy with `-D warnings`: passed.
- `cargo fmt -p uor-r4-cpu-train -- --check`: passed.
- `git diff --check`: passed.
- Fresh non-author FIT passed at source head
  `d143e8bafc9376cd755072a784d4adfe5b8b72ab`.
- The registered two-seed/repeat run, output checksum verification and
  deterministic comparison completed.
- Report check passed; roadmap check covered 139 items; documentation catalog
  covered 1,175 records with no missing paths; prep-check found 165 sheets and
  marked this sheet complete and FIT.
- `uor-harness check`: 0 errors; all 129 findings were pre-existing
  coordination/state warnings.
- `uor-harness loop gates --full`: all 11 gates green.
- D23, D25 and D32 ratchets: no added sites. Shard integrity retained 2,678
  receipts, and archive verification found zero modified or deleted files.

## Skills used

No system-provided skill applied. Repository D84, FIT, ORDER, REPORT and
geometry rules governed the work. Fresh-context subagents performed mechanism
audit and FIT; none ran public compute.

## Reading

The registered cell-relative orbit-phase gate is **negative** on both seeds.
It establishes that the mechanism was reached and did stop replies, but its
terminal/nonterminal nearest-class rule selected ongoing content as a stop.
The matched label permutation was not a harmless control: on both seeds it
also reached gated cells and made only premature stops.

This closes the fixed 32-value, two-block, full-16-root nearest-class
configuration. It does not close geometric termination or the broader
quaternion/SpiralCore transport program. A next attempt must introduce a
different within-cell representation or a continuation-state invariant rather
than retune this signature's coordinates, radius, cell majority or stop
thresholds after seeing this panel.

## Still open

- A termination representation that distinguishes the final row after the
  content is complete, rather than one that fires during ordinary continuation.
- Corpus-scale full-path Hamming, turn-memory and point-key training on a real
  public artifact.
- Complete replies, fresh phrasing and useful local chat.
- Claude's active long-window training/serving integration; this branch does
  not touch its files.

## Plain summary

The model can enter the cells used by reply endings, and the new orbit-phase
signature can be compared exactly inside those cells. On both model seeds that
comparison chose continuing text as if it were the ending. Both controlled
comparisons therefore made zero correct stops, so this particular geometric
stopping rule failed even though the plumbing worked.
