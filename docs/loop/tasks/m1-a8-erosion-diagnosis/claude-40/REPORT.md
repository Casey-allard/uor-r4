# REPORT: m1-a8-erosion-diagnosis, claude-40 (P_succ targets and TRAIN predecessors in the census)

Lab: claude. Base: `b00d9edad`. Order: [ORDER.md](ORDER.md), committed first (`b15372d73`). The cancel probe found that the base README defers Q2, Q4 (i, ii) and Q5 to this unit, and the census did not count them. So the task stands.

## What did not work, first

- **The answer is mixed, so it does not single out a suspect.**
  - **By token count,** Relation moves *fewer* of the tokens P_succ rows depend on: 63 against 71 targets, 69 against 77 writes on targets, and 81 against 94 writes on their TRAIN predecessors.
  - **By row weight,** its moves are *heavier*. The targets it moves carry 1,461 against 1,369 P_succ rows, and their row-weighted predecessor distance rose more: 8.564 → 10.123, against 9.799.
  - **The verdict:** placement stays a suspect, the read is not cleared, and the gap is moderate, 92 rows or about 2.4% of P_succ.
- **The design's P_succ assumption was wrong.** I had assumed P_succ rows relate to TRAIN predecessors. They are defined by an in-context copy rule (`attention_evidence.rs:7-20`), so the census copies that private rule. Control c6 proves the copy reproduces exactly 3,847 of 14,287 rows.
- **The corpus files are not loose on the HF store.** The pod bundles say no corpus is uploaded. The files sit inside `data/ft-balp.tar` and `data/ft-dev.tar`, which pods fetch at bootstrap (`scripts/pod/uor-pod-bootstrap.sh:337`). I extracted the four files and checked them against the code pins before use.
- **The notebook row can't land in this PR.** The item declares no `notebook:` file, and its `scope:` excludes `docs/research/notebook/`. The row and the item close need a scope widening first.
- **Not my unit, but found here:** the registered A8 runbook sheet (`m1-a8-registration-and-runbook-staging-2026-10-22.md:46`) lists wrong full DEV hashes behind the right prefixes. The real files hash to the code pins. Reported to Antigravity on #2028 (6349797941).

## What changed

| Commit | What |
|---|---|
| `b15372d73` | This ORDER, alone. |
| `01b0cf310` | The census crate. It copies the P_succ rule and adds the root-metric cell distance (cross-checked against `endpoint_address_distance`), `PreparedCorpus` loading against the pins, Q2, Q4 (i, ii) and the Q5 timeline. Four new tests: nearest occurrence wins, an empty window or no occurrence is false, a successor equal to the query, and the cell distance. Built by a builder subagent. |
| `3d51b6489` | README results: c6, c7, Q2, Q4 and Q5, and the updated "what this supports". |
| this REPORT's commit | One README sentence: "the read" includes table writes to context tokens, not only gate writes. This REPORT. |

## Measurements

From `census-targets.json`, sha256 `789e3aa848d2b00e746d439da43ab79f385d88af4c35ba12d4f40acdef6a261d`, run on the laptop host-only in about 5 s.

**Controls:**
- c1–c5 PASS, as in #2994.
- **c6:** 14,287 rows and 3,847 P_succ, exact.
- **c7:** TRAIN 7,512,797 tokens and 2,425,925 eligible; DEV 14,287 eligible.

**Results at Q1, Relation against FrozenZero:**

| Measure | Relation | FrozenZero |
|---|---|---|
| Distinct P_succ targets | 448 | 448 |
| Their TRAIN predecessors (tokens) | 1,325 | 1,325 |
| Targets moved | 63 | 71 |
| P_succ rows with a moved target | 1,461 | 1,369 |
| Row-weighted predecessor distance (from 8.564) | 10.123 | 9.799 |
| Writes on targets | 69 | 77 |
| Writes on predecessors | 81 | 94 |
| Gate writes, combine / relation | 117 / 87 | 120 / 0 |

The row gap is already present at batch 512: 884 against 700.

## Checks run at the head

- `cargo test --locked`: 12 passed.
- `cargo clippy --locked --all-targets -D warnings`: exit 0.
- **Planted fault:** with the oldest occurrence in place of the nearest, `copy_rule_nearest_occurrence_wins` fails (11/12 pass). Restored, 12 pass.
- The real run reproduces the output sha256 above.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: the census crate and the README. Corpus inputs stay in gitignored `local/`.

## Still open

- **The notebook row and the item close.** A scope widening (`docs/research/notebook/m1-discrete-attention.md`) comes first, then a closing unit with `Closes #2829`.
- **Separating placement from the read needs admission itself,** so a device run under its own FIT'd sheet. One candidate: replay Relation's Q1 table with FrozenZero's gates, and the reverse.
- **Cleanup after the item closes:** about 908 MB of downloaded inputs under `local/claude/a8-census-data/` and `a8-corpus/`.

## Plain summary

The learning arm moved fewer of the important words, but the ones it moved mattered to more test rows and drifted further from their usual neighbours. So moving words is still a suspect, but not the whole story, and only a GPU test of what the attention actually sees can settle it.
