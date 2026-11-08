# REPORT: one-model-end-to-end, deepseek-40

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Base: `6a5d51172`.

**The global opening point table does not transfer to `m2-fresh256`.** It
changed 216 of 256 first tokens under `per-turn/v1` and 247 under
`one-user-turn/v1`, but found the correct first token in neither rendering
(0/256). One row per rendering reached top 32. The shuffled-target controls
also found 0/256 top-1, so the treatment has no measured opening gain.
Complete replies and complete-distinct pairs remained 0/256 and 0/128 in every
full-reply arm.

## What did not work, first

- **The earlier document-held-out positive did not generalize.** The sealed
  SFT mix's 64-example transfer screen found the first token on 26/64 held-out
  training-document openings. On the actual fresh M2 panel, the point table
  built from all 74,340 non-EOS TRAIN openings found 0/256.
- **The finite target vocabulary is part of the gap.** The 256 panel rows use
  20 distinct first-target tokens. Eight of those tokens never occur as a
  point-table target. The other twelve do occur, but none reached top 8 under
  either rendering; one row per rendering reached top 32.
- **A nearest prompt point is not a query-conditioned value reader.** Point
  changes were active, but they selected generic frequent reply openings
  instead of the role-specific literal requested by the question. The result
  is not explained only by absent token ids: 12/20 target ids were present.
- **Whole replies stayed at zero.** Memory-only stopped 52/256 per-turn and
  42/256 one-user. The point artifact stopped 20/256 and 45/256 respectively;
  both point and shuffle controls produced zero complete rows and zero
  complete-distinct pairs.

## What changed

`cpu-point-transfer-driver` now has an opt-in panel mode that:

- validates exact artifact, TRAIN corpus and all five panel file hashes, the
  complete panel manifest and unique case/history/pair identities;
- renders `m2-fresh256` exactly as `served-reply-panel` under either registered
  rendering;
- builds the point table only from TRAIN reply openings, skipping EOS;
- treats panel labels only as evaluation targets;
- refuses exact panel text, accepted replies and shared six-word grams in the
  TRAIN corpus;
- reports raw, point, shuffled-point and training-mode constant baselines,
  per stratum and overall, including per-stratum paired comparisons;
- checks every query and served-token decision for parity;
- emits `memory-scaffold.uorlut`, `point.uorlut`,
  `shuffled-point.uorlut`, and `report.json`, so the unchanged reply scorer
  can evaluate the complete path.

Point table construction and the document overlap audit are now parallel
across independent windows. The full 74,340-opening build and panel read used
32 threads rather than one. Raw and treatment openings use the artifact's
served nearest head; the writer query remains the point-table key.

## Measurements

The selected integer artifact is `9f349697...` (step 1,280). The repaired SFT
corpus is 6,444,377 tokens with 4,059,876 scored targets. The point table has
36,632 rows, width 256 and 444 target tokens.

### Opening transfer on all 256 fresh rows

| Rendering | Raw top-1 | Point changed raw | Point top-1 | Point top-8 | Point top-32 | Shuffled top-1 | Constant top-1 |
|---|---:|---:|---:|---:|---:|---:|---:|
| `per-turn/v1` | 0 | 216 | **0** | 0 | 1 | 0 | 0 |
| `one-user-turn/v1` | 0 | 247 | **0** | 0 | 1 | 0 | 0 |

Every `point_vs_shuffled`, `point_vs_raw` and `point_vs_constant` paired
gain was 0 against threshold 2. Query mismatches, point served-token
mismatches and shuffled served-token mismatches were all 0/256.

### Per-stratum opening controls

| Rendering | Stratum | Rows | Point changed raw | Point top-1 | Point top-8 | Point top-32 | Paired gain / threshold |
|---|---|---:|---:|---:|---:|---:|---:|
| `per-turn/v1` | `length2` | 64 | 52 | 0 | 0 | 0 | 0 / 2 |
| `per-turn/v1` | `length4` | 64 | 56 | 0 | 0 | 1 | 0 / 2 |
| `per-turn/v1` | `length8` | 64 | 56 | 0 | 0 | 0 | 0 / 2 |
| `per-turn/v1` | `reassert` | 32 | 24 | 0 | 0 | 0 | 0 / 2 |
| `per-turn/v1` | `update` | 32 | 28 | 0 | 0 | 0 | 0 / 2 |
| `one-user-turn/v1` | `length2` | 64 | 60 | 0 | 0 | 0 | 0 / 2 |
| `one-user-turn/v1` | `length4` | 64 | 62 | 0 | 0 | 0 | 0 / 2 |
| `one-user-turn/v1` | `length8` | 64 | 64 | 0 | 0 | 1 | 0 / 2 |
| `one-user-turn/v1` | `reassert` | 32 | 30 | 0 | 0 | 0 | 0 / 2 |
| `one-user-turn/v1` | `update` | 32 | 31 | 0 | 0 | 0 | 0 / 2 |

Every per-stratum point-vs-shuffled, point-vs-raw and point-vs-constant paired
gain was also zero against its row-count threshold. The overlap control passed
over 25,071 documents and 79,679 assistant replies with zero exact-panel,
accepted-reply or shared-six-word-gram hits.

### Complete replies

| Rendering | Arm | Complete | Pairs | EOS stops | Cap stops | Distinct non-empty |
|---|---|---:|---:|---:|---:|---:|
| `per-turn/v1` | memory only | 0/256 | 0/128 | 52 | 204 | 31 |
| `per-turn/v1` | point | 0/256 | 0/128 | 20 | 236 | 53 |
| `per-turn/v1` | shuffled | 0/256 | 0/128 | 28 | 228 | 43 |
| `one-user-turn/v1` | memory only | 0/256 | 0/128 | 42 | 214 | 47 |
| `one-user-turn/v1` | point | 0/256 | 0/128 | 45 | 211 | 82 |
| `one-user-turn/v1` | shuffled | 0/256 | 0/128 | 47 | 209 | 69 |

The per-turn point arm reduced stopping relative to memory-only without
creating content. The one-user arm moved three EOS stops and no complete row.
Neither is a useful-chat result.

### Cost

Both release runs used 32 logical threads on an AMD Ryzen 9 9950X
(16 cores/32 threads). A low-priority, redundant debug validator was still
resident, so these are conservative costs: `per-turn/v1` took 123.5 s at
327,988 KiB peak RSS; `one-user-turn/v1` took 121.0 s at 326,912 KiB. The cost
is dominated by the full point-table construction; the 256-case panel read and
parallel overlap audit follow it.

The complete retained-run receipt is
[`evidence/resources.json`](./evidence/resources.json). It covers 18 retained
opening reports and 20 reply reports, including retries and the 2,713.369 s
debug validation run: a measured wall-time lower bound of 4,993.408 s, with
334,120 KiB peak opening RSS. Reply-run peak RSS was not instrumented. Retained
evidence and direct inputs total 908,645,503 bytes (866.552 MiB); regenerable
main/worktree build trees total 79,517,551,883 bytes, and temporary review
outputs total 3,609,217 bytes. At accounting time the filesystem had
154,773,643,264 bytes (144.144 GiB) free, clearing the 30 GiB floor and 128 MiB
margin. The canonical shared model-time ledger
`.uor-models/native-joint-learning-2026-09-04/model-time.json` was `ABSENT`.
Builds, tests, source/corpus preparation, delivery, and failed runs without a
retained report remain explicitly unmeasured in the receipt.

## Evidence

- Selected artifact:
  `9f349697c3b7cd054f70fe66cb86ffb407ebbf0dcdf63ec16b386174760558e1`.
- Memory scaffold:
  `25326d6a2ccfe0f617f3493e81ffc1435691b7e8c519d2f6d613c09e74302326`.
- Point artifact:
  `7fc90a3e0ef761b4a9f24f24d8bcf590a8a91d5486cee725dfb581283f915171`.
- Shuffled point artifact:
  `2d24deaa237eaf2d523e47d9ab7fd6e71c83fb6f84ce3bf96fd8de38c0e99ec9`.
- Opening reports:
  `4f03afcb…` (`per-turn`) and `a65b9be3…` (`one-user`).
- Reply reports:
  per-turn memory/point/shuffled `5c3793f4…` / `b11c617f…` / `606a166b…`;
  one-user memory/point/shuffled `70bca093…` / `d1b6c8bc…` / `5018e014…`.
- Full evidence:
  [summary.json](./evidence/summary.json).
- Cumulative D84 receipt:
  [resources.json](./evidence/resources.json).
- Local probe root:
  opening reports and artifacts are in
  `local/deepseek/opening-panel-20261108/attempt-10/`; the byte-identical
  treatment/control artifacts' complete-reply reports remain in `attempt-5/`.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-point-transfer-driver`: 29 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 455 passed,
  0 failed, 4 ignored.
- `cargo test --release --offline --locked --features cpu-parity --bin
  served-reply-panel`: 15 passed.
- `cargo check --offline --locked -p uor-r4-cpu-train --bin
  cpu-point-transfer-driver`: passed.
- Both release opening runs completed over 256/256 rows, passed the
  corpus/panel overlap audit and wrote all three artifacts plus `report.json`.
- `served-reply-panel` ran unchanged on memory-only, point and shuffled
  artifacts under both renderings.
- Targeted `rustfmt --check`: passed.
- `cargo clippy --no-deps` reports no warning in the changed driver; the crate
  still has ten pre-existing library warnings.
- `uor-harness loop gates --full`: green. The first invocation hit a transient
  harness fake-`gh` `Text file busy` failure; the unmodified rerun passed.

## Skills used

No system-provided Codex skill applied. The repository's D22, D67, D84,
geometry and delivery rules were used directly.

## Still open

- This closes the global point-table configuration on this SFT trunk and this
  panel. It does not close spiral geometry, exact memory or a trained
  query-conditioned value reader.
- The next opening mechanism must select or construct a target token that can
  be absent from TRAIN openings, or form a role-conditioned source-value
  pointer, then hand that opening to the existing continuation path.
- `m2-reply512` remains unavailable. The exact panel generator is present in
  history, but exact bytes were not recovered in this unit.
- Claude's capped-mw-balp run remains the main stopping/corpus track and was
  not touched.

## Plain summary

On the real fresh test, the laptop model's nearest-point opening table changes
216 or 247 of 256 answers but still never starts with the right token. The old
success on held-out training documents was not transfer. Eight of the twenty
expected starting tokens had never appeared as point-table targets, and twelve
that had appeared were still not in the top eight. Every stratum and paired
control agrees, and the corpus has no detected panel overlap. The same models
also completed no full replies. This tells us to stop tuning that global point
lookup and build a role-conditioned value reader/copy path for the first token.
