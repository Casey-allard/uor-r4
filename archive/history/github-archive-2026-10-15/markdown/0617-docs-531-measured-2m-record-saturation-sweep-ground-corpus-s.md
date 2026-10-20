# #617 — docs(#531): measured 2M-record saturation sweep — ground corpus-scale β

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-11T10:53:39Z
- **Updated:** 2026-10-09T14:17:03Z
- **Closed:** 2026-08-11T11:10:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/617
- **Merged:** 2026-08-11T11:10:53Z
- **Merge commit:** c1376283177edf5c815ed56ebd97375257b294d0
- **Base / Head:** main / UOR-Foundation:feat/531-beta-sweep
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Pins down the corpus-scale question in `docs/scaling_law.md` with the first saturation sweep taken **past** the coverage knee.

Two teachers — SmolLM2-360M and SmolLM2-135M — were each teacher-forced over the same 17,000-article Simple-Wiki corpus to **1,995,026 records** using the new batched-inference path (#616, ~2h/run at batch 64). One `scale_sweep.sh` per teacher sub-samples that one corpus and re-runs compile → cover → score at 50k → 1.9M records.

## Results

| records | 360M top-1 | 360M EXCT-miss | 135M top-1 | 135M EXCT-miss |
|---:|---:|---:|---:|---:|
| 50,000 | 5.05% | 45.95% | 5.77% | 46.17% |
| 200,000 | 5.89% | 37.73% | 6.22% | 35.92% |
| 500,000 | 5.48% | 26.89% | 6.11% | 27.76% |
| 1,000,000 | 5.37% | 21.52% | 5.95% | 20.94% |
| 1,500,000 | 5.31% | 17.52% | 5.78% | 18.85% |
| 1,900,000 | 5.38% | 16.67% | 5.84% | 16.42% |

- **Coverage knee measured (~1.5M records).** EXCT-miss marginal slope collapses from −1.07 to −0.21 pp/100k (360M) across the 1.5M→1.9M segment; 135M mirrors it. This empirically grounds the calculator's `N_REF = 2,000,000` floor rather than assuming it.
- **Coverage is teacher-independent**, as the mechanism predicts: the two EXCT-miss columns track within ~1–2pp at every scale.
- **top-1 (resolution) stayed flat ~5–6% for both teachers** with no scale trend, so this run exposes **no teacher-dependent resolution knee**. The two coverage knees coincide, so a β slope cannot be fit; fitting β to coincident coverage knees would return ≈0 — an artifact, not an exponent.

## β decision (honest)

β is **not** pinned here. It stays **provisional 0.5, bounded [0.45, 1.0]** — no longer for want of a past-knee observe (now done) but for want of a resolution metric that scales with corpus size *and* separates teachers. What #531 settles: the coverage-knee floor is measured and teacher-independent, and batched inference makes 2M-record observes routine.

## Verification

Corpus integrity: both corpora 1,995,026 records, dense `stories.jsonl` (17,000 lines), `merged.bin` (360M κ `d122ab38`, 135M κ `cde559db`). Gauntlet: `cargo fmt --check`, `cargo xtask validate` (check-model / audit-limits / audit-deferral), `cargo build --release`, `cargo test --workspace --no-run`, `cargo clippy --workspace --all-targets -D warnings` — all green. Docs-only change; `model/ledger.toml` untouched.

Closes #531.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

