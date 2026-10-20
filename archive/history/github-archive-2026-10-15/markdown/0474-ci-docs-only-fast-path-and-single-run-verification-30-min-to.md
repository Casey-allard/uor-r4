# #474 — ci: docs-only fast path and single-run verification (30 min to under a minute for docs)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-07T05:20:48Z
- **Updated:** 2026-10-09T13:08:59Z
- **Closed:** 2026-08-07T05:57:34Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/474
- **Merged:** 2026-08-07T05:57:34Z
- **Merge commit:** ddfe86d6f60238106339affacbd7e3ab5579bc5e
- **Base / Head:** main / UOR-Foundation:ci-diet-2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Every change currently costs ~30 minutes: the `gates` job runs ~15 min on `pull_request`, then the merge queue re-runs it on `merge_group`, plus the heavy jobs. A README edit pays full price. Two approved changes.

**(A) Docs-only fast path.** A PR whose diff is documentation-only skips the compile ladder and runs the claim-wording gate. **Fails closed:** any non-doc path in the diff runs everything, an empty or failed diff runs everything, and `merge_group` / `push` / `workflow_dispatch` always run everything — the queue's own verification can never be skipped by a path guard.

The path set is `*.md` anywhere plus `docs/**.pdf`, **minus two exclusions**, and the exclusions are the interesting part: `docs/**` as a blanket was rejected because that directory also holds `docs/transformerless/gate_c_pinned.json`, the load-bearing pinned Gate C record; and `hologram_r4_formal_monograph.md` and `INFERENCE_OPERATION_CONTRACT.md` are `include_str!`d into Rust and asserted on by the BDD suite, so editing them can genuinely break the build. Guard logic was simulated against nine diffs (docs-only, mixed, pinned-record, each excluded doc, empty, lockfile, workflow) — fast path fires only on the first two.

**(B) Single-run verification.** `gates` becomes `merge_group`-only; PRs get a new `gates-pr` reporting the same required check name with fmt + clippy for fast feedback. Slow work (nextest, BDD, doctests, no_std ladder, deterministic rebuild, κ-reproduction, Gate C trend) runs once, in the queue, on the actual merge result.

**The queue-bricking risk, addressed explicitly.** Every required check name must be reported on both triggers or PRs hang forever. Computed mechanically from the parsed YAML by evaluating each job's `if:` per event — exactly one job reports each name per event, none unreported, none doubled:

| required check | pull_request | merge_group |
|---|---|---|
| fmt / clippy / tests / no_std / κ | `gates-pr` | `gates` |
| cargo audit | `audit` | `audit-mq-stub` |
| fuzz smoke (nightly libfuzzer) | `fuzz-smoke-pr-stub` | `fuzz-smoke` |
| wasm-pack build | `wasm-pr-stub` | `wasm` |
| Gate C trend alarm | `gate-c-trend-pr-stub` | `gate-c-trend` |

**Accepted trade:** test/no_std/κ breakage now surfaces at queue time rather than PR time — a bad change costs a dequeue instead of a red PR. The queue's unconditional full run is what makes that safe. Also noted: deleting a doc still takes the fast path and would fail later in the queue (`pdf_traceability.rs` asserts evidence paths exist).

YAML parses; claim-wording gate passes. This is statically validated only — **the first docs-only PR after this lands should be watched** to confirm the fast path fires and all five checks report green. PR #473 (README) is exactly that test case.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

