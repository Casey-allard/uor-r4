# #2268 — The D25 hygiene docs/figures sweep: the project's own documents agree with the ban

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T02:44:37Z
- **Updated:** 2026-10-13T02:49:34Z
- **Closed:** 2026-10-13T02:46:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2268
- **Merged:** 2026-10-13T02:46:42Z
- **Merge commit:** 19397d37b33f414e4064769414603dab0d1d5b18
- **Base / Head:** main / UOR-Foundation:kimi/docs-sweep-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: no model line — repository hygiene · count 0/3 · headline 37/232 → 37/232 (unchanged)

## The D25 hygiene sweep: the project's own documents and figures now agree with the ban

Source-only documentation work. Record: `docs/labs/docs-sweep-2026-10-13/README.md`.
Decision: **KEEP** (documentation hygiene — no capability, artifact or measurement is claimed or moved).
Base `84fa24669`. **No Rust file was touched**; `research/`, `.gitignore` and `scripts/storage/research-data`
were left alone.

The premise as I found it (the audit's line references were approximate, and the assigned worktree did not
exist — disk hygiene had removed my merged one — so I recreated it at `.worktrees/kimi-lab` from the current
main, one commit past the `f8cff324a` the brief named): main's own overview still said "D0-b to D20", the
project map still offered matmul as a training permission in three places, the geometry page still taught a
softmax read/head, a sigmoid copy gate and a SwiGLU MLP as current, and six figures carried pre-D25 numbers.

### Per item

| # | Item | Disposition |
|---|---|---|
| 1 | `README.md` | **Changed** — mechanism-brief list gains the invariant-addressed read and ternary/XOR fact sheet; a scope sentence records that the mechanism numbers come from the pre-D25 float form and keep their exact scope as historical evidence; the `docs/geometry.md` link is retitled as the historical float form; labs roster gains Kimi + project manager; "D0-b to D20" → "D0-b to D25"; acknowledgements gain Kimi. The Training section's "Held under D25" note was **already fixed** by #2263 and was left as is. |
| 2 | `ROADMAP.md` | **Changed** — date 12 → 13 October, M2 `count6/3` → `count7/3`, M1 row notes the merged OP1 Hamilton gate and OP2 fused Hamilton-projection operator. |
| 3 | `docs/PROJECT_MAP.md` | **Changed** — three D25 contradictions: the D0-b paragraph no longer offers matmul as a training permission; "Offline Rust training may use matmul" → scalar float arithmetic and no dense multiplication; the `uor-addr` row cites D11/D0-b for serving and D25 for the training ban. |
| 4 | `docs/geometry.md` | **Changed, nothing deleted** — a **Historical float form (pre-D23)** banner plus seven scoped notes (softmax read, gate/MLP naming the merged OP1/OP2, softmax head, sigmoid copy gate, the "Softmax at runtime" table re-labelled as open non-compliance, the §2 sentence, the "Life of one token" walkthrough). |
| 5 | `docs/labs/README.md` | **Changed** — explicit current roster: Kimi (PM, `kimi/pm1`, owner-appointed 12 October 2026, D25/#2029 comment 6133330776) on the objective layer and geometric operators/CUDA; Codex on CI + ratchets, M2 and the geometric-path repair; Claude on the starling/murmuration read and pod migration; DeepSeek on the invariant-addressed router/memory and the harness. The file's existing Kimi-as-PM note (#2259) was kept. |
| 6 | `docs/mechanisms/README.md` | **Changed** — two missing rows: `invariant-addressed-read.md` (M1 #2029) and `ternary-xor-evidence-factsheet.md` (M4 #2032, M5 #2033). |
| 7 | `docs/integration/project-track.md` | **Changed** — rung-0 reconciled with [Amendment 3](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6136219130) (the pre-registration resumes **unchanged** once the stack it trains is compliant); the D19 lane table is marked historical and cannot reactivate a lane. |
| 8 | Figures | **Changed** — `architecture.svg`, `milestones.svg` (stale `0.933 BPB` → served `0.886838 BPB`; `8 of 512` → `437 of 512`) with a `historical (pre-D25)` note; `geometry/next-token.svg` note for the softmax read/head, sigmoid copy gate and SwiGLU MLP it draws; `training-pipeline.svg` badge fixed + `held under D25`; `serving-pipeline.svg` `uor-r4.lut-stack/1` → `/3`; `geometry/hamming-rank-read.svg` caption (pre-registered, not measured). **`geometry-stack.svg` needed nothing** (no stale number, no banned-path depiction) and is untouched. The other thirteen `docs/figures/geometry/*.svg` core-geometry figures are preserved unchanged. New `assets/images/README.md` captions the Studio/transformer-era concept art as historical and preserved, not the current architecture. |
| 9 | Branch archive | **Added** — patch for the superseded draft `kimi/d23-learning-law-2026-10-12` (tip `158077125`, base `fe8b076a1`, 4 files / 395 lines, sha256 `c5d3a1b25f4e5bee9a8a0635f4c2873da127efadf8c39fb93e5cbcff4fdc7247`) + its INDEX section. Disposition: superseded (main rewrote the same design doc in `bbbdd362a`; the census and `softmax_path_check.py` landed byte-identically in `3f6d97bc4`). **The remote branch was not deleted** — the owner does that after this merge. |

### Checks at this exact head

- [x] Checks run at this exact head, offline, on the owner's laptop
- [x] `python3 scripts/check_no_softmax.py` — **OK, 1649 remaining across 145 files (baseline 1649)**
- [x] `python3 scripts/check_no_matmul.py` — **OK, 95 sites across 26 files (baseline 95)**
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] `python3 -c "xml.dom.minidom.parse(...)"` over all 20 `docs/figures/**/*.svg` — all parse
- [x] Archive patch: byte-identical to `git diff fe8b076a1 158077125`; applies cleanly at its pinned base with a throwaway git index (no working-tree mutation) and reproduces the branch tip tree `3de881cadc19329368eccbe6c1be0dea8e55f500` byte-for-byte; it deliberately does **not** apply to current main (base moved) and the INDEX row records that
- [x] Every relative link I introduced or edited resolves on disk (checked mechanically)
- [x] No `crates/**` file in the diff (`cargo fmt`/tests therefore not re-run — nothing Rust changed)
- [x] `scripts/harness/cli.py lint check` — 0 errors; its single warn is the registry check on the
      "no model line" declaration this brief specifies (a hygiene PR is on no milestone line, which the
      milestone registry cannot express; the harness's own tooling PR used the same form)

Pre-registration: not applicable — no compute and no model numbers in this PR.

Cost: one documentation session on the owner's laptop, no compute, no pod.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

