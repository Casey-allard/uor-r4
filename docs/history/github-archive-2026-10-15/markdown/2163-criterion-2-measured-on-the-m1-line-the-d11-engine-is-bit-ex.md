# #2163 — Criterion 2 measured on the M1 line: the D11 engine is bit-exact and the model serves at 1.20-1.24 BPB against the <= 0.90 target (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T15:09:19Z
- **Updated:** 2026-10-10T15:13:30Z
- **Closed:** 2026-10-10T15:10:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2163
- **Merged:** 2026-10-10T15:10:49Z
- **Merge commit:** fd577eb73d9a243bf0c61c41ede3f5326f33e8c7
- **Base / Head:** main / UOR-Foundation:deepseek/d11-serving-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Result: criterion 2 measured for the first time on this line — the D11 engine is **bit-exact** (`d11 − d10 nll = 0.0`, `max |Δlogit| = 0` on every window of all three artifacts) and the served readings are **1.20138 BPB** (29M base) / **1.24402** (D10) / **1.24233** (B1) against the **≤ 0.90** target. The criterion is **NOT MET**, the engine is not the reason, and the comparison the acceptance implies — the recorded **0.933 BPB** cell — belongs to a **19.9M artifact fine-tuned on chat-v0-p2 alone**, i.e. in-distribution with this held-out split.** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: D11 serving of the M1 artifacts · count 0/3 · headline criterion 2 unmeasured → measured **NOT MET**

## What and why

Criterion 2 of #2029's acceptance — *"the same model exports and serves under D11 at ≤ 0.90 BPB on the chat held-out stream"* — **had never been read on this line's artifacts**, every one of which now carries a copy pointer; the export path's documented behaviour for a pointer model (both integer engines serve it as its mixture) had not been exercised here either. Pre-registered on #2029 before any compute.

## Result

| artifact | export sha256 (`model.lut`) | `d11 − d10 nll` | `max |Δlogit|` | D11 tok/s | D10 tok/s | served nll | **served BPB** | criterion 2 (≤ 0.90) |
|---|---|---:|---:|---:|---:|---:|---:|---|
| A0 `chat-29m-B-lr5e-4` (base) | `b55bdf6b…` (16,871,620 B) | **0.0** | **0** | 68.0 | 127.5 | 2.362828 | **1.20138** | **NOT MET** |
| A1 D10 (cycle 1) | `bbbd1ff9…` | **0.0** | **0** | 67.9 | 130.5 | 2.446684 | **1.24402** | **NOT MET** |
| A2 B1 (cycle 2) | `3f39a91a…` | **0.0** | **0** | 80.0 | 136.9 | 2.443367 | **1.24233** | **NOT MET** |
| reference: the 19.9M chat-v0-p2 stack (recorded, not this run) | — | — | — | — | — | — | **0.933** | NOT MET by 0.033 |

Exports used the ladder's own recipe (GPTQ, `damp=0.01`, 24,576 calibration positions, `mlp` padded 732 → 736); both engines ran on the same 64 windows of `chat-cuda-20261008/data/heldout.u16` (**6,194,589 tokens, 17,576,697 bytes, 2.837427 bytes/token, 0 out-of-vocab** — the recorded basis, reproduced with `make_lens.py`'s byte accounting). `first_difference` is `null` on every window, so **no part of the gap is a serving artefact**.

**Two facts, one of them actionable.** The fine-tunes are **worse** on this stream than the base they came from (1.2440/1.2423 against 1.2014) — the recall mixture that buys the memory half costs in-distribution chat likelihood. And the recorded **0.933** belongs to a **smaller** artifact fine-tuned on **chat-v0-p2 alone**, while every artifact here was fine-tuned on a **mixed** stream: **distribution, not scale, is what separates them**, which names the next piece — an in-distribution chat-only fine-tune of the 29M base, scored on both this stream and the criterion-1 panels.

**Criterion 1 is unaffected** and remains blocked on the owner, exactly as the [blocked card](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6098512039) states; this cycle changes nothing there and no threshold, panel or stream is reinterpreted.

## Checklist

- [x] Checks at this exact head: `python3 scripts/check_claim_wording.py` (docs-only change; no Rust path touched — the round's checks are the engines' own: bit-exact nll and logit equality on every window, the export reports' quantiser settings and per-tensor errors, and the byte basis recomputed from the tokenizer).
- [x] Research record: `docs/labs/d11-serving-2026-10-10/README.md`
- [x] Newest-first `docs/integration/current-state.md` entry with **Next:**
- [x] `STATUS.md` row and M1 cell; #2028 M1 table posted after the merge
- [x] Evidence bundle: cloud-store `d11-serving-2026-10-10`, **173,056 bytes, md5 `285c63eb1ba14f2910781dcd37a02224`** (the 16.9 MB LUTs stay on the canonical volume at `/workspace/uor-r4/deepseek/d11-20261010/`)
- [ ] After merge: verify on fresh `origin/main`, delete branch and worktree, `scripts/storage/uor-hygiene --apply`, final card on #2029

References #2029


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6098918116 — 2026-10-10T15:09:26Z — Casey-allard

**Validation at the exact head `13c5c1525f7568e40c308e29ccc7b83423d74e02`** (docs-only change: one new record, one `current-state.md` entry, one `STATUS.md` row and cell; no Rust source, schema or CLI path is touched, so no compile or test applies).

| check | command | result |
|---|---|---|
| claim wording | `python3 scripts/check_claim_wording.py` | **pass** — "no unlinked machine-verified/exact-equivalence wording" |
| engine exactness | `geometric-stack d11-evaluate artifact=… model=… valid=… windows=64` | **`d11 − d10 nll = 0.0`, `max |Δlogit| = 0`, `first_difference: null`** on all three artifacts and every one of the 64 windows |
| export recipe | `geometric-stack export … calibration_windows=64 damp=0.01` | GPTQ, 24,576 calibration positions, `mlp_padded_from 732 → to 736`; LUT 16,871,620 B each, sha256 recorded per artifact |
| byte basis | `make_lens.py` on the held-out stream | **6,194,589 tokens, 17,576,697 bytes, 2.837427 bytes/token, 0 out-of-vocab** — reproduces the recorded basis, so `bits_per_byte` is computed from the repository's own accounting |
| served readings | per-window nll from the same reports | 1.20138 BPB (A0), 1.24402 (A1), 1.24233 (A2) against the ≤ 0.90 target |

Evidence bundle: cloud-store object `d11-serving-2026-10-10`, **173,056 bytes, md5 `285c63eb1ba14f2910781dcd37a02224`**.



---
_Comment/review ids in this file: 6098918116_

