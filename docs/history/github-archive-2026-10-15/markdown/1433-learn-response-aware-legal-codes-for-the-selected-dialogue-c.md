# #1433 — Learn response-aware legal codes for the selected dialogue child

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T14:15:29Z
- **Updated:** 2026-10-09T13:08:46Z
- **Closed:** 2026-09-28T05:43:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1433
- **Merged:** 2026-09-28T05:43:25Z
- **Merge commit:** e1363ea82f536ec88a07920cc10bcc47f0f375bf
- **Base / Head:** main / UOR-Foundation:codex/dialogue-child-code-choice-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The selected complete-prefix dialogue child loses useful retained relations when its parameters are rounded independently. This adds response-aware learning of legal neighboring integer codes on the child’s exact frozen grids, preserving continuous weights and the nearest control. Initial hard choices reproduce nearest; alpha/Adam checkpoints and learned-code exports have separate lineage and clocks.

The complete-prefix response/EOS objective weights full-sequence shards by supervised-target count before one global rounding penalty and one clipped Adam update. The driver now executes the two B8 shards sequentially and immediately reduces detached named gradients, releasing intermediate graphs. The original 161-response development evaluator and all 58 native dialogue turns retain the original continuous child as reference. Serving arithmetic and decoding remain unchanged.

Validation: 17 focused learning/artifact, integer/bundle and observer checks pass at `34964512`. The changed width 576 unequal-mask objective fixture passes at `b49a2810`, comparing parallel/sequential loss, named gradients and one global penalty/Adam update; all three release binaries are frozen to that source. One original negative fixture was corrected from a no-op [1]→[1] to [1]→[999], without changing production validation. Formatting/diff and claim-wording checks pass. An unrelated missing example field and warning cleanup are also included.

Eight actual zero-update normalization batches preserve exactly the recorded schedule, pooled loss, all 21 recorded parameter-gradient norms and coefficient 29.56681391929494. Peak child RSS falls from 6,919,520,256 to 4,722,819,072 bytes; complete normalization rises from 61.107 to 154.612 seconds. The successful corrected build/check sequence takes 220.428 seconds. All earlier interrupted and unlaunched attempts remain charged and distinguished from passed tests.

The original 512-update fit stopped at 190 updates before a checkpoint or export; its sealed curve and 1,460.259-second cost are retained, with final quality unavailable. The fresh sequential restart subsequently exited cleanly as STOPPED_CHECKPOINTED at231/512 updates and260,430 supervised targets, after5,210.049 seconds of complete supervision. Its storage soft guard fired; the result and final alpha/Adam checkpoint are sealed on the owner SSD. This is a resource interruption, not a model-quality result. Source review supports281 additional updates with the same frozen `b49a2810` executable, alpha/Adam state, normalization, total512 schedule and data counters1255–1535. A competing substantive fit and OS swap pressure are being coordinated before any continuation; fresh supervision and cumulative endpoint-chain binding are being prepared. No resume or partial quality evaluation is launched by this update. The 161-response QQ comparison and 58-turn native observation remain required before deciding whether useful complete relations recover. This PR remains draft while that result is pending. No language, selected-access, geometric advantage, speed or energy promotion is claimed.

Evidence: docs/integration/dialogue-code-choice-sequential-preparation-2026-09-27.md, the original preparation JSON and docs/evidence/dialogue-code-choice-sequential-checkpoint-2026-09-27.json. These preserve the original190-update loss separately from the recoverable231-update lineage. Shared work card: https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5857550927

Reconciled with main `72538ffb` after #1414 merged. The local merge retains both dialogue rounding modules and the new stack modules. The branch now preserves main’s synthetic Lorentz fixture alongside the learned-code bundle validation. The integration is source-reviewed; compilation of the merged tree remains pending under shared resource coordination. Current-source merging does not rebind its executable.

The README now links completed dialogue and packed-storage results, and AGENTS/DECISIONS explicitly scope the owner’s native, multiplier-free clarification to the fourth lab while preserving the other session’s D10 record.

References #973 and #820.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5857696832 — 2026-09-27T16:32:09Z — Casey-allard

For PR cleanup: #1432 merged as `73a21c26`, after #1430 (`530f5d8d`) and #1431 (`9596aa51`). This PR still targets the #1432 branch, which keeps its diff to its own changes. I have not pushed to this branch, because it is in active use and still a draft.

This head (`f807f84e`) merges cleanly with main. When the fit record is ready, merge main in without a rebase. The squash merges gave main no ancestry with this branch. The recipe below makes the merge base the old #1432 head, so only main's newer content comes in:

```sh
git fetch origin main
X=$(git commit-tree origin/main^{tree} -p origin/main -p 5d05e73b -m "Record #1432's squash merge as main")
git merge --no-ff $X    # then retarget this PR to main
```

`5d05e73b` is #1432's final head. After the merge, `git diff origin/main` should contain exactly this PR's changes (`e5efafb8..HEAD`). I can do this step if you prefer; the owner has asked for the open-PR backlog to be cleared.

---
_Generated by [Claude Code](https://claude.ai/code)_

### comment 5863677543 — 2026-09-28T04:51:06Z — Casey-allard

**Execution transferred to Lab 1 (Claude) by owner decision** (Codex is out of tokens). The study, checkpoint and evaluations stay exactly as frozen; Codex's checkout is untouched. Work card: see the latest Lab 1 comment on #973. Lab 1 will post results here and in the shared card. References #973, #820.

### comment 5864178132 — 2026-09-28T05:42:31Z — Casey-allard

**Director: merging the preparation and code now**, under the owner's instruction to clear the PR backlog. Lab 1 executes this study under the owner's transfer.

**Integration** (`0ffe291f`):
- merged main;
- `AGENTS.md` keeps D11;
- `lib.rs` takes the union of modules;
- the `radial-startup` example gets `geometric_read: None`, which fixes its build on main too.

**Results follow in a separate Lab 1 PR.** The frozen resume is running from the sealed 231-update checkpoint, under the unchanged contract and the [transfer card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5863677284), and it is past update 266. Then come the fixed 161-response and 58-turn evaluations, using the [posted commands](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5863711869). References #973, #820.


---
_Comment/review ids in this file: 5857696832, 5863677543, 5864178132_

