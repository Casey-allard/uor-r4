# #729 — graph tooling: derive fixed-partition recorded-corpus scale samples with bounded memory

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-15T23:17:40Z
- **Updated:** 2026-08-16T08:08:09Z
- **Closed:** 2026-08-16T08:07:39Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/729
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Discovered while making the #704 dense execution era safe for the existing #514/#531 scale workflow.

## Problem

The historical Python prefix sampler writes `stories = max(retained_story) + 1`. The compiler derives `train_cut = floor(0.8 * stories)`, so each sample size can silently move stories between construction and held-out partitions. Merged observations are shard-major and the same story may appear in multiple noncontiguous runs; naive filtering can also join two separated fragments and change reconstructed `input`/span semantics. The current shell workflow reports requested N even when a complete-run cut undershoots it.

Dense GPT-2 corpora add an optional 768-f32 hidden row stream. Materializing both the full source and derived hidden generations can require more than 12 GiB near the 2M-row scale point.

## Scope

- Replace the live Python prefix writer with a Rust `subsample-recorded-corpus` transaction built on the recorded-generation protocol in #728.
- Preserve the exact source `stories`, RNG, completion bit, record layout, execution identity, and fixed source train/held-out threshold.
- Select deterministic complete story runs from both fixed partitions, preserve source order, and never make two previously separated runs of the same story adjacent.
- Support registered 12/32/48/88-byte record eras and opaque nonzero f32-aligned hidden row widths.
- Stream selected record/hidden ranges into guarded staged outputs; do not retain full source and output hidden streams simultaneously.
- Publish dense then attention sidecars where applicable and commit the destination binding last. Refuse in-place or ambiguous derivations before mutation.
- Update `scale_sweep.sh` to use sibling input/compiled/cover/score roots, report actual retained N and partition counts, support an exact full-source point, and reject N above source size.
- Deprecate the writing behavior of `mc1_subsample_corpus.py` without erasing its historical role.

## Evidence and tests

- Pin one repeated-story shard fixture across all 12/32/48/88-byte layouts, exact selected source indices, reconstructed inputs/spans/anchors, dynamic hidden-byte alignment, actual-N undershoot, binding CID, and idempotent rerun.
- Cover fresh/case-alias/symlink/BUSY roots, markerless legacy versus dense-bound sources, malformed hidden alignment, staged records/hidden/meta crash recovery, foreign residue, and wrong-role byte preservation.
- Add a bounded-memory or streaming seam regression suitable for multi-GiB sparse hidden input without allocating it.
- Append an honesty correction to `docs/scaling_law.md`: prior #531 knee/beta results produced with the moving `max+1` split are descriptive and not fixed-partition comparable pending a new measurement.
- Run no replacement long measurement in this implementation issue. Any corrected scale rerun gets a separate predeclared run contract after merge.

This implementation stays entirely in `uor-r4` and depends on #728, not on new upstream `uor-matmul` work. References #704, #514, and #531.

## Comments (6)

### comment 5304703926 — 2026-08-15T23:19:42Z — Casey-allard

## Starting implementation state (2026-08-15)

A first Rust command exists only in the local #704 integration WIP. It is not yet accepted and no corrected scale measurement has run.

Binding decisions for this issue:

- preserve the exact source `stories` field and therefore the original compiler train cut;
- select complete runs from both fixed partitions in source order;
- repeated story IDs across shard-major runs are valid, but separated runs may not become adjacent after filtering;
- support 12/32/48/88-byte rows and opaque f32-aligned hidden widths;
- output N may undershoot the request and the script must report the actual committed count;
- use four sibling roots per scale point (`input`, `compiled`, `cover`, `score`);
- N equal to the finalized source count is the full-source lane; N above it is an error;
- prior #531 knee/beta evidence from the Python `max(story)+1` split will be labeled descriptive/non-comparable pending a separately preregistered rerun.

The implementation will be reviewed against one exact all-width repeated-story fixture plus crash, alias, BUSY, hidden-alignment, provenance, and bounded-memory cases. No replacement long measurement is authorized here.

### comment 5304714220 — 2026-08-15T23:22:49Z — Casey-allard

## Static audit of preserved WIP `0461aaf1`

The first implementation is intentionally not accepted yet. Exact blockers recorded before further edits:

- sampler materializes full source records and hidden bytes, then allocates complete derived records and hidden again; the bounded stream API is unused;
- `scale_sweep.sh` places `input/` inside the compile output root, so deterministic inventory rejects a fresh run and later graph directories break reruns;
- the documented/default 2M point always calls the sampler, while the command rejects N greater than or equal to source N; there is no exact-full lane;
- no direct sampler tests exist in this WIP;
- `docs/scaling_law.md` still describes Python/copy and the false moving-split claim;
- CLI help still implies `copy-recorded-attention` can derive dense provenance even though code correctly refuses that path.

These findings are the #729 worklist, not hidden acceptance debt. The command/script/docs will not be represented as complete until the all-width fixture, bounded streaming, four-root/full-N workflow, and honesty correction are green.

### comment 5305399013 — 2026-08-16T02:48:48Z — Casey-allard

Clean handoff: #729 remains pending behind #728 and is tracked in draft PR #730. No further sampler implementation ran after the existing audit. Required work remains bounded streaming, fixed-partition 12/32/48/88 tests, four-root scale script layout, full-N lane, actual-N reporting, crash/BUSY/residue coverage, and Python/docs honesty correction. Consolidated resume state: https://github.com/UOR-Foundation/uor-r4/pull/730#issuecomment-5305398275

### comment 5306130344 — 2026-08-16T06:31:37Z — Casey-allard

Clean handoff point after pushed commit `9aeecc2c` on draft PR #730. No #729 edits are currently in flight and the worktree is clean.

Next bounded implementation sequence for #729:
1. rewire `subsample-recorded-corpus` to stream binding-verified records/opaque hidden rows instead of retaining source+output generations in memory;
2. pin fixed source-partition run selection for 12/32/48/88-byte records, repeated shard runs, no boundary stitching, hidden-row alignment, and actual-N undershoot;
3. repair `scale_sweep.sh` to use sibling input/compiled/cover/score roots and an exact full-source lane;
4. deprecate/correct the historical Python writer and max(story)+1 partition claim;
5. add direct crash/rerun, BUSY/alias, residue, bounded-memory, and script-contract tests; then rerun the repository gates.

This work remains entirely in uor-r4. There is no Casey-authored uor-matmul PR or external repository dependency for the remaining scope.

### comment 5306381439 — 2026-08-16T07:45:34Z — Casey-allard

## Implementation complete on draft PR #730

Pushed commit `6f1f53ff` (`fix(#729): stream fixed-partition corpus samples`).

Delivered:

- bounded 64 KiB streaming of selected record and opaque hidden-row ranges; no source-plus-output corpus materialization;
- one sorted source/destination authority, no-follow alias/symlink/BUSY refusal, dense-first execution sidecars, source final verification, and canonical binding last;
- deterministic complete-run selection against the exact source train cut, with no filtered boundary stitching, all 12/32/48/88-byte record eras, actual-N undershoot, exact-full N, and N-above-source refusal;
- direct markerless/dense policy, crash/rerun, wrong-role, idempotence, raw-row/span/input/anchor, hidden-alignment, literal binding-CID, and multi-GiB-offset bounded-reader tests;
- four-sibling `input/compiled/cover/score` scale workflow, source-derived defaults, exact-full point, requested/actual/train/held cross-checks, optional dense support, and rerun-stable fake-R4 contract tests;
- non-writing historical Python tombstone and an append-only #531 scaling-law honesty correction. No replacement long measurement was run.

Evidence:

- `cargo test --workspace --offline` — PASS
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` — PASS
- `cargo fmt --all -- --check` — PASS
- graph-format no-default and alloc ladders — PASS
- claim-wording and R5 audit — PASS
- graph CLI 125/125; graph compiler 199/199; scale workflow 6/6 — PASS
- two independent exact reviews — P0/P1/P2 = 0
- final reviewed snapshot identifier before commit: `ec58131fb466bf3725e41264f48d78f1c19cabf059507e5356635b9f64610d6e`

Issue remains open until PR #730 passes its refreshed checks and merges through the queue.

### comment 5306462722 — 2026-08-16T08:08:09Z — Casey-allard

Merged via PR #730 in merge commit 830aafde4c571e6356ee920ef4cc1108eb493997 after the full merge-queue ladder passed. The Rust fixed-partition sampler, bounded streaming publication, all-width/hidden/full-N/recovery tests, four-root scale workflow, and scaling-law honesty correction are now on main.


---
_Comment/review ids in this file: 5304703926, 5304714220, 5305399013, 5306130344, 5306381439, 5306462722_

