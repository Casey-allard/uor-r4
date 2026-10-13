# The D25 hygiene sweep: overview, roadmap, project map, the geometry page, rosters, figures and one archived branch (2026-10-13)

**Question.** After D25 froze the rules (no softmax/softplus/sigmoid/tanh, no dense matrix multiplication in
training or serving, GPU-only model compute), which of the project's *own* overview, roadmap, map, teaching and
figure documents still tell a reader the opposite — and can they be corrected with dated banners and scoped
notes rather than by deleting the historical material?

**Identity.** Base `84fa24669` (origin/main: #2266 restored `research/`), branch
`kimi/docs-sweep-2026-10-13` (this PR). Source-only: **no Rust file was touched** (`git status` has nothing under `crates/`), no model
compute, no pod. `research/`, `.gitignore` and `scripts/storage/research-data` were left alone as instructed.

## What was run, item by item

| # | Item | Disposition |
|---|---|---|
| 1 | `README.md` | **Changed** (6 edits): the mechanism-brief list gains the invariant-addressed read and the ternary/XOR fact sheet; a scope sentence records that the mechanism numbers below come from the pre-D25 float form (float training, dense matmul, softmax read/head) and keep their exact scope as historical evidence; the `docs/geometry.md` link is retitled as the historical float form; the labs roster gains Kimi and the project-manager role; "D0-b to D20" → "D0-b to D25"; the acknowledgements list gains Kimi. The **Training** section already carried its "Held under D25" note from #2263 — left as is. |
| 2 | `ROADMAP.md` | **Changed**: date 12 → 13 October; the M2 row's `count6/3` → `count7/3`; the M1 row now records that the **OP1 Hamilton gate and the OP2 fused Hamilton-projection operator are merged** while operator preparation continues and no new result exists. |
| 3 | `docs/PROJECT_MAP.md` | **Changed** (3 places that contradicted D25): the D0-b paragraph no longer offers matmul as a training permission (its training clause is historical; the serving contract is unchanged); "Offline Rust training may use matmul" → scalar floating-point geometric arithmetic, no dense multiplication (D25); the `uor-addr` row now cites D11/D0-b for serving and D25 for the training ban. |
| 4 | `docs/geometry.md` (highest risk) | **Changed, nothing deleted**: a **Historical float form (pre-D23)** banner at the top states that the page's path descriptions (softmax read and head, sigmoid copy gate, SwiGLU MLP over dense projections) are banned in executed paths, while the mechanisms it illustrates remain the architectural reference; seven scoped notes mark the softmax read, the gate/MLP (naming the merged OP1/OP2), the softmax head, the sigmoid copy gate, the "Softmax at runtime" table (now labelled open non-compliance rather than an end state), the §2 sentence and the "Life of one token" walkthrough. |
| 5 | `docs/labs/README.md` | **Changed**: an explicit current roster — Kimi (project manager, session `kimi/pm1`, owner-appointed 12 October 2026, D25/#2029 comment 6133330776) on the objective layer and geometric operators/CUDA kernels; Codex on CI and the ratchets, M2 and the geometric-path repair; Claude on the starling/murmuration read and the pod migration; DeepSeek on the invariant-addressed router/memory and the harness. The file's existing "Current start (12 October)" note (from #2259) already named Kimi as PM and was kept. |
| 6 | `docs/mechanisms/README.md` | **Changed**: two missing rows added in the existing format — `invariant-addressed-read.md` (M1 #2029) and `ternary-xor-evidence-factsheet.md` (M4 #2032, M5 #2033). |
| 7 | `docs/integration/project-track.md` | **Changed**: the rung-0 sentence is reconciled with [Amendment 3](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6136219130) — the pre-registration [6124773996](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6124773996) **resumes unchanged once the stack it trains is compliant**; the D19-era lane table is marked historical and superseded by the operator plan and D25's assignments, and cannot reactivate a lane. |
| 8 | Figures | **Changed**: `architecture.svg` and `milestones.svg` (stale `0.933 BPB` → served `0.886838 BPB`; `8 of 512` → `437 of 512`) plus an in-figure `historical (pre-D25)` note; `geometry/next-token.svg` gains the note for the softmax read/head, sigmoid copy gate and SwiGLU MLP it draws; `training-pipeline.svg` (badge number fixed, `held under D25` note); `serving-pipeline.svg` (`uor-r4.lut-stack/1` → `/3`); `geometry/hamming-rank-read.svg` (caption: pre-registered mechanism, not a measured current path). **`geometry-stack.svg` needed nothing** (no stale number and no banned-path depiction found) and is untouched. The other thirteen `docs/figures/geometry/*.svg` core-geometry figures are **preserved unchanged**. New `assets/images/README.md`: a dated caption recording that the Studio/transformer-era concept art there is historical and preserved, not the current architecture. |
| 9 | Branch archive | **Added**: [patch](../../history/branch-archive/kimi_d23-learning-law-superseded-20261012.patch) (sha256 `c5d3a1b25f4e5bee9a8a0635f4c2873da127efadf8c39fb93e5cbcff4fdc7247`) for the superseded draft branch `kimi/d23-learning-law-2026-10-12` (tip `158077125`, merge base `fe8b076a1`, one commit, 4 files / 395 added lines) plus its INDEX section. Disposition: superseded — main carries a newer rewrite of the same design doc (`bbbdd362a`) and the census + `softmax_path_check.py` files landed byte-identically (`3f6d97bc4`). The remote branch was **not** deleted (the owner does that after this merge). |

## Numbers, with scope

- `python3 scripts/check_no_softmax.py` — **OK, 1649 remaining across 145 files (baseline 1649)**, unchanged by
  this PR (it scans `crates/**` source only, and no source file was touched).
- `python3 scripts/check_no_matmul.py` — **OK, 95 sites across 26 files (baseline 95)**; informational: 102
  tensor-core intrinsic occurrences in 1 file.
- `python3 scripts/check_claim_wording.py` — PASS.
- All **20** SVGs under `docs/figures/` parse as XML after the edits (whole tree, not just the edited files).
- Archive patch: applies cleanly at its pinned base `fe8b076a1` (verified with a throwaway git index, no
  working-tree mutation) and the resulting tree **equals the branch tip tree
  `3de881cadc19329368eccbe6c1be0dea8e55f500` byte-for-byte**.

**Decision: KEEP** — documentation hygiene. This is **no model line**: count 0/3, headline unchanged (M1 v5
memory 20/40, reply 37/232); no capability, artifact or measurement is claimed or moved. Nothing here is
mechanism evidence.

## Limitations

- The sweep corrects the documents the audit named. It is not a full-repository prose audit: `STATUS.md`'s deep
  history, `CONTRIBUTING.md`, the older `docs/labs/*` records and the mechanism briefs' *bodies* were not
  re-read for D25 contradictions (dated records keep their text by policy; what matters is that a reader cannot
  mistake them for the current contract, which is what the banners fix).
- Figures keep their original drawings: the notes and numbers are corrected, the banned-path geometry is not
  redrawn. A reader who only sees the image still sees the old shape, with a note.
- The geometry page remains a *historical* document by design; it was not rewritten into the current contract,
  because that contract's executed form (gate/projection/head replacement) is still landing.
- `research/` and the storage scripts are untouched (another lab's just-merged #2266 owns their disposition).
- The archived patch does not apply to current main (its base moved) — that is the recorded disposition, and
  the INDEX row says so.

## Next

Wire nothing here; follow the operator plan: OP3a (the product-key head) after OP1/OP2, then the compliant
rung 0 once the whole executed path is lawful, device-qualified, FIT and explicitly released. The one
documentation follow-up this sweep leaves is inside the phase card's own terms: the docs that describe the
*remaining* dense path should be revisited as each rung lands.

**Cost:** one documentation session on the owner's laptop; no compute, no pod.
