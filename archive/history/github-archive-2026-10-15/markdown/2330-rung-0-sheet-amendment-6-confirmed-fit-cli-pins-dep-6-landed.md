# #2330 — Rung-0 sheet Amendment 6: CONFIRMED FIT — CLI pins, dep 6 landed (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T13:48:38Z
- **Updated:** 2026-10-13T13:55:02Z
- **Closed:** 2026-10-13T13:54:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2330
- **Merged:** 2026-10-13T13:54:33Z
- **Merge commit:** be5d8cd6e9e3a3f53edb15e764457434dfe7c0a5
- **Base / Head:** main / UOR-Foundation:kimi/rung0-sheet-amend6-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (unchanged)

## Rung-0 sheet Amendment 6: CONFIRMED FIT — the CLI pins and dependency 6 landed

Docs-only amendment of the rung-0 pre-registration sheet per Claude's CONFIRM ([#2029 comment 6146716704](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6146716704)) and the adjudication rule ([6145409089](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6145409089)). Record = the sheet itself; the newest-first `docs/integration/current-state.md` entry is the change log.
Decision: **NOT YET PROMOTED** — no model compute, no count reset, no hold change; headline unchanged (reply 37/232, v5 memory 20/40).

### What changed and why

- **Pinned CLI updated** with `train_mask=/workspace/rung0/data/mw-balp/train/response_mask.u8` and `expected_train_targets=2425925`, plus a prose pin recording: the mask member from `data/ft-balp.tar` (7,512,797 bytes, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`, manifest `mask_sha256`); the native CLI refuses without `train_mask` (fails closed); Claude's reproduced inventory (2,425,925 train masked targets); the train-reference top-8 `[1,16,14,435,340,351,3,302]` scoring 8.58% top-1 / 28.42% recall@8 on the 14,287 dev targets — so the +5pp bars now require ≈13.6% / 33.4%, unreachable by a constant emitter.
- **Dependency 6 marked LANDED** — PR #2328, merge commit `9e904a83448c451f738d5c1d45224dfdf73eb84e`: `MaskedTargets` train-only counting, DEV-as-TRAIN refusal by content hash, smallest-ID ties, the unmasked-exclusion fixed control, the diagnostic-only dev oracle, and artifact binding via `native_count_reference.json`; Claude's checklist confirms all six items.
- **Verdict restored to `TEST FITNESS: FIT`** (confirmed by Claude lab, independent non-author review, #2029 comment 6146716704, at `9e904a834` + Amendments 5–6; restored by the sheet owner per the adjudication rule); the standing sentences are kept — training and the M2 training/rental hold remain held, only the owner releases, and FIT lifts no hold.
- Front-matter log gains Amendment 6; claims.json releases my two stale claims (`kimi/pm-rule-r10-workspaces`, `kimi/rung0-sheet-amend5`) and carries this branch's claim, with claude's and both codex entries untouched.

### Verification

- [x] Checks run at this exact head (`52dbeaf57`) — commands and outcomes below
- [x] `python3 scripts/harness/cli.py check` — this branch **introduces no new errors**: it shows one **pre-existing** main error, `receipts.parse [docs/harness/ledger/codex-2026-10.jsonl]: line 27 has unknown kind 'receipt'` — Codex's onboarding receipt (12:36Z today) against the Python parser's kind whitelist (a harness-plane contract mismatch owned by the harness labs, not this docs PR). Proven pre-existing by `git stash` of all four changed files: the identical error appears on the pure `origin/main` tree; the erroring shard is byte-identical to `origin/main`'s
- [x] `python3 scripts/harness/cli.py lint check --body-file <this body>` — 0 errors (deliberate `record_path` warn, as in every sheet amendment)
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] Exactly one `TEST FITNESS:` verdict line in the sheet (FIT); docs only — no `crates/` changes, nothing under `local/` committed
- [x] Note for the harness owners: if CI's `harness rules and tests` job fails here, the failing rule is the pre-existing `receipts.parse` error above — one whitelist decision ('receipt') in `scripts/harness` or the Rust writer's kind, not this PR's content

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

