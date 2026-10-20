# #867 — research(#833): rebuild + ratify the clean #755-native broad baseline

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T08:40:17Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-08-20T08:59:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/867
- **Merged:** 2026-08-20T08:59:15Z
- **Merge commit:** 1b3e46f2da93255c0187cd3e89ddab4e6a463101
- **Base / Head:** main / UOR-Foundation:issue-833-clean-rebuild
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Completes S0 item E (#833): re-establish the broad-canonical baseline on a
**source-complete, #755-native, deterministic, admitted, attested** bundle and
re-ratify the M.V.G. thresholds. Following the #866 preconditions (determinism +
re-pin), this executes the rebuild — with **no re-observe**: the raw teacher
observation (`.uor-models/obs-wiki-360m`, 256 shards / 361,693 records) was reused.

## What changed (in this PR)

- `scripts/obs_bundle_to_corpus.py` — densify story ids to a contiguous
  `0..stories-1` range. The #516 corpus dropped 6 whole stories (769, 771–775),
  leaving gapped ids the post-#755 `subsample-recorded-corpus` guard rejects. The
  densify is a deterministic, token-content-preserving relabeling of the boundary
  metadata and is the identity (byte-for-byte unchanged output) for a dense bundle.
- `docs/attested_broad_baseline_833.md` — the append-only measurement record.

The compiled bundle (`.uor-models/compiled/smollm2-360m-broad-clean`) is gitignored;
it is regenerable from the pinned inputs via the fixed converter + the recorded-corpus
compile chain.

## Acceptance-criteria status (#833)

- Reconstructable from pinned inputs / CIDs present — **met** (fixed converter →
  clean corpus κ `blake3:4692307368…`; `release-bundle.json` graph `blake3:bc2366f1…`,
  signature `blake3:6324aabe…`).
- Two deterministic builds reproduce byte identities — **met** (two `compile-recorded`
  builds byte-identical incl. `hierarchical_codes.json`, via #865).
- Offline admission + prediction canary — **met** (deployed R4Engine loads + predicts
  offline ~1.5 s).
- Baseline reports with slices / path attribution / status — **met** (record below).
- One reconciled current baseline — **met** (RETAINED; no README/ROADMAP/RESEARCH
  number change).

## Gate C — M.V.G. RATIFIED (RETAIN), held-out 72,130

| row (causal serving) | #516 pin | clean #755-native | Δ |
|---|---|---|---|
| Rule 1+2 precedence | 24.30% | 24.39% | +0.09 pp |
| best live (`on_fwd_strict_live`) | 31.48% | 31.11% | −0.37 pp |
| TLA-3 baseline | 28.21% | 28.12% | −0.09 pp |
| EXCT-miss (strict) | 25.7% | 27.36% | split shift |

All headline causal rows reproduce within the predeclared <~0.5 pp reachability bound.
Witness replay 64/64; candidate recall Rule 1+2 62.6% / 87.3%.

## Tests and verification

- `python3 scripts/check_claim_wording.py` — pass.
- Clean corpus passes the strict `subsample-recorded-corpus` guard; two byte-identical
  `compile-recorded` builds; deployed admission canary PASS; `evaluate-report` binding
  chain (source, tokenizer, scored-graph cid, attention operator) validated end-to-end.
- Diff is one utility script + one docs file (no Rust), so PR-side gates are
  claim-wording; the queue re-verifies the full ladder.

## Negative / UNAVAILABLE (kept, not hidden)

The teacher floor (3.6015 bits) and the teacher-forced table-native argmax rows are
RETAINED by invariance (unchanged teacher weights + exact GEMM). A full-population
re-measurement is **UNAVAILABLE at scale** under the current uor-matmul exact-GEMM
teacher (~1 position/s → ~24 h for 72k positions; #516 used the since-removed
Apple-Accelerate path) — documented, not asserted as a fresh PASS. The primary causal
quality rows above were freshly measured teacher-free against `corpus.next`.

## Compatibility / conformance / claim boundary

No format/ABI change; no `model/ids.toml` / `CONFORMANCE.md` change (R1 unaffected).
The baseline is RETAINED, so no promoted claim moves.

Closes #833


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

