# #1777 — Probe native unary/joint geometric cue complementarity without refitting

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T02:05:00Z
- **Updated:** 2026-10-09T13:17:21Z
- **Closed:** 2026-10-06T22:45:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1777
- **Merged:** not merged
- **Merge commit:** 6f9c7d6a3eecef03ff7949564836bb8db57fe44e
- **Base / Head:** codex/geometric-cue-joint-transfer-20261006 / UOR-Foundation:codex/geometric-cue-coadapt-probe-20261006
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The frozen joint-cue donor improves development completion but does not transfer. This adds a four-arm development-only local sensitivity probe, with separately anchored unary/joint partial gradients and independent integer export/reload/generation. No optimizer or held-out predictions.

At numerical head `6a234ffbf9b614e07bd31e2a4a6ee800b3686951`, baseline/unary/joint/both complete replies are **74/74/70/70 of512**; first-source **296/295/296/295**. Complete-output interaction is zero in every row. The CE interaction is descriptive and does not justify a larger coadaptation fit. All donors and negative arms are retained.

Independent source and saved-result reviews confirm bindings, packed deltas, baseline replay and counts. Saved first divergences show three unchanged-source Copy/terminal crossings, one token-alias loss despite the correct raw Copy winner and one initial source flip. This supports a distinct shared-source cue factorization test; raw margin cancellation remains arithmetic, not an executed generation result.

Validation at the numerical head on Linux x86_64: formatting,48 fitter tests and release fitter/observer builds PASS (212.328s, CPU8, peak RSS4.762GB); actual probe PASS (72.872s, CPU16, peak RSS3.260GB, CUDA hidden, exit0). Executable SHA `29218de82f3f5d1546328572ca9b60ec86aa8df9e27170fb56f73aeab9d4cc4a`. Final head `da7e0f4352d92baa41ec5b607f179a1fd20975f4` changes only documentation; numerical source identical. Claim-wording/diff checks PASS. No chat or geometric-advantage qualification.

Durable evidence: `/workspace/uor-r4/codex/cue-coadapt-probe/probe-1`; local receipts archived with roundtrip MD5 in `icloud:UOR-R4/results/codex/codex-finished-research-cleanup-20261006.tar`. Stacked on PR1775; not yet delivered to main.

References #820.


## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### comment 6007868113 — 2026-10-06T02:07:35Z — Casey-allard

Independent source review at 6a234ffbf9b614e07bd31e2a4a6ee800b3686951: investigative and adversarial reviewers both PASS the explicit mode/option guard, unchanged old batch, separate identical anchored unary/joint losses, disjoint gradients, frozen baseline and exact quarter reload, full512 native generation and all-prior comparisons. Build/tests/model remain pending; no source review is counted as executed validation.

### comment 6007922190 — 2026-10-06T02:12:14Z — Casey-allard

PR1777 exact numerical head6a234ffbf9b614e07bd31e2a4a6ee800b3686951: formatting,48 fitter tests and release fitter/observer build PASS on Linux x86_64 host54c9c82b187d (212.328s CPU8, peakRSS4,761,841,664B). No local build.
Fourarm native sensitivity is running under the posted card, no fresh predictions or selection. Frozen executable SHA29218de82f3f5d1546328572ca9b60ec86aa8df9e27170fb56f73aeab9d4cc4a, configSHA5cc2230a47c919ddb7f72cd45070dd1c1e3d8eecfa2c0bf21b1c716ecdbe8547. Selectedtrial12 joint33c6021151e84b27cb222a333303ccff093219e46c661b0ca689bd9b92929254. Baseline,U,J,U+J all startfromthatdonor; legalstepschosenindependentlyperfamily from same all512 development gradient. Baseline mustreproduce74 completions. Currentrun no result yet; noadoption. DeepSeek softmax/tensorcore unchanged.


### comment 6007947108 — 2026-10-06T02:14:33Z — Casey-allard

Native unary/joint local sensitivity completed at PR1777 numerical head6a234ffbf9b614e07bd31e2a4a6ee800b3686951 (no optimizer, selection or heldout predictions).

| Development512 | Baseline | Unary935 −quarter | Joint2 −quarter | Both |
|---|---:|---:|---:|---:|
| Complete replies |74|74|70|70|
| First factual source |296|295|296|295|
| Both paired sources /256 |83|83|86|86|
| Both paired complete /256 |7|7|7|7|
| Native CE |0.9544226152|0.9558346258|0.9513687195|0.9518923123|

Baseline exactly reproduces selected trial12 counts/CE. Unary gives1 complete gain/1 loss; joint gives1 gain/5 losses; combination2 gains/6 losses. Complete-output second difference0, with no interacting completion rows; CE second difference−0.0008884 is descriptive, not synergy. This does not justify the proposed broader coadaptation fit, nor retire either geometric family. Preserve all four arms. Reviews now inspect saved first-divergence paths and whether token-alias credit or stopped terminal-source branching explains the CE/output mismatch. No unchanged dose/fresh redraw.

Actual model72.872s CPU16, peakRSS3,260,166,144B, Linux x86_64 host54c9c82b187d, CUDA hidden, exit0/no stop. Executable29218de82f3f5d1546328572ca9b60ec86aa8df9e27170fb56f73aeab9d4cc4a; config5cc2230a47c919ddb7f72cd45070dd1c1e3d8eecfa2c0bf21b1c716ecdbe8547. Full sealed evidence /workspace/uor-r4/codex/cue-coadapt-probe/probe-1. Formatting,48 tests and fitter/observer release builds pass; separate source reviews pass. Result interpretation reviews pending. DeepSeek's softmax/tensorcore work is unchanged. PR1777 remains stacked and unmerged.


### comment 6026820453 — 2026-10-06T22:45:09Z — Casey-allard

RECORD: frozen unary/joint complementarity controls produced completions74/74/70/70, without useful interaction gain. Retained result/controls: https://github.com/UOR-Foundation/uor-r4/pull/1777. Branch preserved.


---
_Comment/review ids in this file: 6007868113, 6007922190, 6007947108, 6026820453_

