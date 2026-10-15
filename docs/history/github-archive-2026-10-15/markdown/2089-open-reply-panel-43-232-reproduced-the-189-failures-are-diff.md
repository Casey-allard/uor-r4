# #2089 — Open reply panel: 43/232 reproduced, the 189 failures are diffuse, Phase 2 not opened (#2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T21:04:10Z
- **Updated:** 2026-10-09T21:08:18Z
- **Closed:** 2026-10-09T21:06:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2089
- **Merged:** 2026-10-09T21:06:37Z
- **Merge commit:** e232272cc956ef057f45a15e258df21cd62d68ea
- **Base / Head:** main / UOR-Foundation:deepseek/open-reply-panel-2026-10-09
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek. CPU only: no pod, no training, no float model generation, no grading.

## What this is
Phase 1 of the open reply panel diagnostic, and it stops there. Criterion 1's largest gap is 43/232 `acceptable` against a 116/232 target (2.7x). The 189 failures are classified per category with the grader's own verdict fields, and **no single mechanism dominates**, so no candidate was trained and no pod was opened. A 2.7x target against diffuse failures is not one piece.

## Identities
- Panel (NOT tracked in main): `ladder/panel/everyday-32.json` sha256 `945c0c97…b480` (32 rows) + `heldout-200-a.json` `019cc6f6…d2c1` (100) + `heldout-200-b.json` `5434cfd9…436c` (100) = 232 rows, 224 single-turn. Archived as `icloud:UOR-R4/results/deepseek/ladder.tar` (3,306,919,424 B, md5 `ed9456d6…baa8`).
- Report: report root `ladder/grades/chat-29m-B-lr5e-4-powered-7b`, `report.json` sha256 `72ab32a2…59d4`; model `d8a3c971…8068` (28,958,761 params), greedy, `max_new_tokens=64`; instrument `chat-grade` executable sha256 `a5071cfe…4239` (still on the laptop at `~/.local/share/uor-r4/bin/chat-grade-a621514c`, re-hashed equal); grader ollama `qwen2.5:7b` digest `845dbda0…b697e`.
- All 232 report rows bind to the panel bytes on id, category and user turns: 0 mismatches.
- This panel has **no frozen row checks** (the embedded checks carry only `conv-*` ids), so `acceptable` is judge-only and there is no deterministic `check_pass` to be primary — unlike v4/v5.

## Result
- 43/232 reproduced from the sealed report's own fields (`fluent` 70, `relevant` 53, unparsed 0).
- Verdict fields over the 189 failures: `neither` 152 = **80.4%**, `fluent_only` 27 = 14.3%, `relevant_only` 10 = 5.3%. Per category and per tier in the record (K-clean fails 86.5%, K-ill-posed 76.1% — the well-posed rows fail more often).
- Deterministic markers: reply cut mid-clause 125/189 = 66.1%, verbatim 5-gram repeat 46, no marker at all 41 (21.7%). The 86 rows with neither marker are still accepted only 29.1%, so acting on the markers leaves the panel near 68/232 — not 116.
- The 96M `chat-100m-C` report (sha256 `3d5a2914…ccf6`) scores 46/232 with the same shape; the diffusion is stable across the recorded 43–46/232 plateau.

## Criterion defect recorded
Criterion 1 on this panel rests on a judge-only reading. A judge-stability re-grade was pre-registered and then declined as decision-irrelevant: v4 measured 4/64 verdict changes at the same digest (~6%, roughly ±14 cells on 232) while the gap is 73 cells. Recommendation in the record: give the reply panel deterministic row checks in the v4/v5 style, or state a tolerance, before anyone trains against 116/232.

## Evidence
`icloud:UOR-R4/results/deepseek/reply-panel-open-2026-10-09.tar` (760,832 B, md5 `1740b21f181779221458bb8c32008f4f`) holds the panel bytes, both sealed reports, the rebuilt replies, the classifier and its outputs; the restore command was run and round-trips, and the restored copy reproduces 232 rows / 43 acceptable / 0 mismatches. `scripts/open_reply_panel_classify.py` (`--selftest`, `--panel` binding check) regenerates every number; per-row `verdicts-29m.tsv` / `verdicts-100m.tsv` are in the record directory.

## Disclosures
- One laptop `grade-replies` run against the local ollama judge started 16:45 on 2026-10-09, before the prohibition on laptop grading reached this session, and was killed at 17:00. It wrote `attempt.json` and no `report.json`; **no number here comes from it**.
- An earlier extraction had been written inside another lab's worktree; it was verified as a duplicate of `icloud:…/panel-open-chat-29m-B-lr5e-4-report-root.tar` and removed from `~/uor-r4-local/reply-panel-recon-found-20261009/`. Nothing of this session's remains outside its own worktree.

## Docs
New canonical record plus a newest-first `current-state.md` entry ending in "Next:"; stale ladder paths fixed in `data/panels/README.md`, `docs/research/step0/0a-panel-taxonomy.md` and `docs/compute/ladder-runbook.md`. `python3 scripts/check_claim_wording.py` passes. STATUS/ROADMAP/#2028 unchanged — criterion 1 is not met and 43/232 has not moved.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6089231485 — 2026-10-09T21:04:24Z — Casey-allard

Exact-head validation at `74a5370c5a2c0510a4559dc8be718f28c56ae4b6` (local, no Rust source changed so no build is applicable):

```text
python3 scripts/check_claim_wording.py        -> Claim-wording gate passed
python3 scripts/open_reply_panel_classify.py --selftest   -> selftest ok
classifier --panel on the sealed 29M report   -> panel binding: 232 rows checked, 0 mismatches
                                                 rows=232 acceptable=43 failing=189
classifier --panel on the sealed 96M report   -> panel binding: 232 rows checked, 0 mismatches
                                                 rows=232 acceptable=46 failing=186
all relative markdown links in the changed docs -> none unresolved
```

The reproduced run used the evidence bundle restored from iCloud (`cloud-store fetch reply-panel-open-2026-10-09`, round-trip md5 verified `1740b21f181779221458bb8c32008f4f`), not the working copy, so the numbers are checkable from the store alone. No model was loaded, generated or graded; nothing here changes a served artifact or a milestone.

### comment 6089253616 — 2026-10-09T21:05:52Z — Casey-allard

Rebased onto fresh main `b48a104bc` after the merge of #2086/#2087/#2088 — the only conflict was the newest-first block in `docs/integration/current-state.md`, resolved by keeping the two just-merged entries above this record's entry. New head `d95decd9c`; diff is unchanged (8 files, +1154/−1). Re-ran at the new head: `check_claim_wording.py` passes, classifier `--selftest` ok, panel binding 232/232 with 0 mismatches. PR is MERGEABLE; `required acknowledgements` passes and the skipped jobs are the by-design skips.


---
_Comment/review ids in this file: 6089231485, 6089253616_

