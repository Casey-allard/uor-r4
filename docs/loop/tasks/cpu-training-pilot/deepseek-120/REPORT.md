# REPORT: cpu-training-pilot, deepseek-120 (writer-cell termination gate)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Prep:
[writer-cell termination](../../../harness/prep/cpu-training-pilot-cell-termination-2026-11-04.md).
Base: `2833c451e`.

## What did not work, first

- **A single majority cell stopped without preserving content.** On seed 1,
  learned cell 1 was reached by all 114 replies, but all 114 stops were
  premature. On seed 2, learned cells 1, 145 and 191 stopped 48 replies, again
  with 48 premature stops and zero exact references.
- **The shuffled controls did not fire on evaluation.** Cell 208 on seed 1 and
  cells 30, 64 and 69 on seed 2 produced no BOS stop. Their reply digests were
  identical to no-gate.
- **The learned majority is dominated by one corpus pattern.** Cell 1 held
  92/102 seed-1 terminal training rows; the other six terminal cells held the
  remaining ten rows on both seeds. A cell-local majority is therefore broad
  enough to trigger inside ongoing replies, not specific enough to identify
  their final row.
- **No arm completed an exact reference.** Both seeds had zero exact and zero
  reachable-exact replies in all three termination arms.
- **This is not the full corpus-trained Hamming/memory/point artifact.** It
  uses the existing memory-trained public artifacts and changes only the
  termination rule.

## What changed

Added a writer-cell termination mechanism alongside the closed point-radius
gate:

- every training query maps to the writer's exact top admission cell through
  the shared `writer_cells(..., 1)` implementation;
- prompt-final and earlier reply-content queries count as nonterminal;
- the final content row before corpus EOS counts as terminal;
- a cell enters the gate only under the pre-registered local rule
  `terminal_count > nonterminal_count`;
- an equal-size control samples disjoint observed cells deterministically with
  the model seed;
- decode names decoder BOS when a generated query's top cell is learned;
- the driver records both cell-count maps, selected cells, excluded ties and
  nonterminal-majority cells, gate hashes and paired reply results.

The report schema advances to `uor-r4.cpu-memory-panel-run/3`. The prior
`--termination-gate` point radius and new `--termination-cell-gate` are
mutually exclusive.

The initially reviewed source, ORDER and prep subset was six files,
`+738/-22`. The full initial PR was 12 files, `+1087/-22`. This repair changes
only task records and notebook/roadmap links.

## Measurements

Both seeds accepted all 32 memory-weight updates and reproduced the pinned
trained artifacts. The no-gate digests matched deepseek-110. The reference
census remains 1,212 content tokens with 104/114 reachable.

| Seed | Arm | BOS complete | Premature | Non-premature BOS | Exact | Cap |
|---:|---|---:|---:|---:|---:|---:|
| 1 | no gate | 0 | 0 | 0 | 0 | 114 |
| 1 | cell gate | 114 | 114 | 0 | 0 | 0 |
| 1 | shuffled cell gate | 0 | 0 | 0 | 0 | 114 |
| 2 | no gate | 0 | 0 | 0 | 0 | 114 |
| 2 | cell gate | 48 | 48 | 0 | 0 | 66 |
| 2 | shuffled cell gate | 0 | 0 | 0 | 0 | 114 |

| Seed | Terminal rows/cells | Nonterminal rows/cells | Learned cells | Shuffled cells | Excluded tie/loss |
|---:|---|---|---|---|---|
| 1 | 102 / 7 | 1,192 / 138 | `[1]` | `[208]` | 0 / 6 |
| 2 | 102 / 7 | 1,192 / 189 | `[1,145,191]` | `[30,64,69]` | 1 / 3 |

Both served-query audits retained zero mismatches over **4,333 decisions,
1,109,248 Q12 coordinates and 34,664 kept cells** per seed. Walls were
**41.221/41.769 s** and peak RSS was **390,232/396,232 KiB**. The fresh seed-1
repeat matched the complete report after removing timing and RSS fields, with
normalized report SHA-256
`a18cc5e64dbe19b5535f259b438da1a9fde2a4b841ee54b11a4959973307902d`.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the binary, raw reports and trained
artifacts. Raw artifacts remain under
`local/deepseek/termination-cell/deepseek-119/`.

## Checks run at the head

- Finite tests cover exact top-writer-cell equivalence on aligned, arbitrary,
  zero and tie-shaped queries; training-only cell derivation; local-majority
  selection; deterministic disjoint controls; exact override and outside-cell
  refusal; real decode stopping; EOS validation; no-gate digest preservation;
  and premature-stop accounting.
- Full CPU release suite: **189 passed**, 4 release benchmarks ignored; all
  binary suites passed.
- Strict all-target CPU Clippy with `-D warnings`: passed.
- `cargo fmt -p uor-r4-cpu-train -- --check`: passed.
- `git diff --check`: passed.
- Fresh non-author FIT passed at source head
  `bb4411ecd0c27431d946e4fb534e99a1297e4f71`.
- Model-free pins and the registered two-seed/repeat run completed.

## Skills used

No system-provided skill applied. Repository D84, FIT, ORDER, REPORT and
geometry rules governed the work. Fresh-context subagents performed mechanism
audit, FIT and exact-head review; none ran public compute.

## Reading

The registered writer-cell gate is **negative** on both seeds. It proves that
a terminal-majority cell can be reached, resolving the earlier total
non-admission failure, but the same cell remains reachable before the true end.
Raw BOS counts are therefore not progress: every learned-gate stop was
premature.

This closes the fixed top-cell local-majority rule with an equal-size disjoint
cell control. It does not reject cell/orbit termination. The next termination
hypothesis must distinguish final-row geometry from an ordinary continuation
inside the same cell, such as a cell-relative orbit phase or a terminal versus
nonterminal within-cell signature.

## Still open

- A termination representation that separates the final row from continuations
  sharing its writer cell.
- Corpus-scale full-path Hamming, turn-memory and point-key training on a real
  public artifact.
- Complete replies, fresh phrasing and useful local chat.
- Claude's active long-window training/serving integration; this branch does
  not touch its files.

## Plain summary

The new gate learned a very common writer cell that almost every training
reply ended in. At evaluation it did stop replies, but always after emitting
only part of the target answer. The random cells never stopped anything. Thus
cell identity alone finds a stop region but not the exact stopping point.
Correctly accounting for premature stops makes the result a clear negative
rather than a false termination gain.
