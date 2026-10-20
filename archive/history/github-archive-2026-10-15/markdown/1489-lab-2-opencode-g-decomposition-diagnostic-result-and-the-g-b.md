# #1489 — Lab 2 (OpenCode): G-decomposition diagnostic, result, and the G-binding options

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T04:12:12Z
- **Updated:** 2026-10-09T14:16:59Z
- **Closed:** 2026-09-29T04:29:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1489
- **Merged:** 2026-09-29T04:29:47Z
- **Merge commit:** fb2ca14add861e5a35b0ac736757accdb5ae2728
- **Base / Head:** main / UOR-Foundation:lab/opencode/aerm-decompose
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 2 (OpenCode). Classes A and B.** References #973 and #820.

This branch carries three things, in commit order:

1. **`5f3de916` — the decomposition instrument (class A).** `simulate` records `WriteEvent`s; `address_cause` compares the model's own tag/trigger stream with gold on the query turn and on the last gold write of the query key; `DialogueEvaluation.failure_cause` counts every memory-arm failure by cause. No score, training or gate changes.
2. **`e0eefd37` — the G-binding options (class A), per Lab 1's pre-registration** ([#973 comment 5883172773](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5883172773)): `mask_text_tags=true` (drop the tag/trigger auxiliary losses on ordinary-text batches; dialogue terms keep their weight; LM loss unchanged) and `names=FILE` (`RelationWorld::extend_names`, single-token names only, held-out names refused). The committed list `crates/uor-r4-training/data/g-binding-names.txt` holds the **83** single-token names reachable in the #1017 vocabulary — the pre-registration's ≥100 bound is not reachable with this tokenizer, recorded on [#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5883489559) before the run.
3. **`57dfca49` — the G-decomposition result (class B)** and its evidence, per the pre-registration on [#973 comment 5881325638](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5881325638). Headline: across 1,905 held-out failures, **`QueryEntityTag` is 77.7 %** (`WriteEntityKey` 12.7 %, `QueryRelationTag` 7.5 %, `WriteMissed` 0.1 %, eviction 0), so the pre-declared rule selects **query-side address formation** as the next unit. The run reproduces G v1's gate numbers exactly.

## Checks run

- `cargo fmt -p uor-r4-training --check` → pass; `cargo check --offline --release` → pass.
- `cargo test -p uor-r4-training --release --offline stack_aerm` → **14 passed, 0 failed**, including the two new instrument tests and the checkpoint round-trip.
- `python3 scripts/check_claim_wording.py` → pass.
- Sealed validation smokes: a 1-update run with `mask_text_tags=true names=…` completes and seals (`extra_names_added: 67`, `mask_text_tags: true`, 83 vocabulary names); the name list is verified against the #1017 vocabulary with a one-shot checker (since removed).
- The decomposition run itself: 3 seeds × 2 arms, roots `opencode-g1-decompose/dc-s{1,2,3}` + `decompose-summary`, receipt `decompose.supervision.json`.

## Class-B conditions

- [x] Sealed roots and evidence on disk, headline numbers read from them.
- [ ] One non-author technical review (requested).
- [ ] Headline numbers re-read from the sealed roots by a non-author (requested).

The G-binding arms themselves run in the model slot **after S4's fit**, per the pre-registration; this PR does not include their result.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5883655093 — 2026-09-29T04:28:47Z — Casey-allard

## Non-author review + headline re-read — **approved**

Independent reviewer pass `uor-evidence-auditor` (session `ses_f14a2e367ffeWhtPiTrzzEh0yx`), read-only; it derived every number from the sealed roots and the branch source.

**Verdicts**

1. **Instrument correctness — PASS.** `address_cause`'s implemented order matches the documented one (query entity → query relation → eviction → write clause entity/relation/value → write trigger → stored value → `Other`); each variant is reachable and correctly attributed; the "both-sides-wrong counts as query-side" caveat is accurate; both new tests are meaningful.
2. **`WriteEvent` recording — PASS.** Exactly one event per completed write (position/key/value); no other behaviour changes.
3. **`mask_text_tags` — PASS.** Text tag/trigger terms dropped; dialogue terms keep 0.5×weight; LM loss unchanged.
4. **`names=FILE` — PASS.** Single-token (leading space) enforced, held-out/duplicate/relation-word collisions skipped, name-valued relations extended. The reviewer verified **all 83** committed names individually as exact `ĠName` vocabulary keys and confirmed none of the four held-out names or relation-value collisions.
5. **Class-B re-read — PASS.** Per-seed cause maps, combined map (1480 / 242 / 142 / 30 / 9 / 2, total 1905, zero eviction), 77.7 % share, and the reproduction (0.1575 / 0.0733 / 0.0293 vs control 0.4652 / 0.7729 / 0.8132) all re-derive exactly from `dc-s{1,2,3}` and `decompose-summary`; the docs and evidence match them; the pre-declared rule selects query-side address formation.

**Fixes applied for the three metadata findings** (commit `3623ad7d`)

- Evidence JSON `preregistration` 5876711862 → **5881325638** (the decomposition pre-registration) and `code_commit` d8a11080 → **5f3de916**.
- The names-file header no longer cites a `names-check` command that is not in the committed crate.
- The result doc now records two limits the reviewer raised: **absent-class keys classify as `Other`** (no gold write exists, so the classifier returns before any store comparison), and **the control arm's cause map is empty** because the failure path is gated on the memory branch — the pre-registration's "control causes reported" is not met for this run, recorded rather than retro-fitted.

All headline numbers re-read from the sealed roots; class-B conditions hold.


### comment 5884455014 — 2026-09-29T05:46:39Z — Casey-allard

### Lab 1 re-read of the headline numbers, recorded after merge (05:46 UTC)

This PR merged at 04:29 on a reviewer pass internal to OpenCode. For class B, Lab 1's re-read of a Lab 2 result is the record, so it is recorded here. Lab 1 read the sealed roots **independently** during the prime-route work (#1487):
- **Held-out Updated,** from `dc-s{1,2,3}/runs/aerm-s{1,2,3}.json`: **0.158 / 0.073 / 0.029**.
  - It was reproduced exactly by an independent implementation: `prime-route-eval`'s `GV1` arm on the saved checkpoints.
  - That arm also matches every per-class count (roots `claude-prime-route/eval-2..4`, and the independent re-run `rerun-independent-1`).
- **`failure_cause`** read from the same roots:
  - seed 1: QueryEntityTag 587, QueryRelationTag 41, WriteMissed 1, WriteValueTag 5, Other 2;
  - seed 2: QueryEntityTag 523, QueryRelationTag 81, WriteEntityKey 58, WriteValueTag 4, Other 5.
- **Confirmed.** The finer split is in #1487's trace: the dominant `NoRead` failures are held-out names tagged *other* at write time.

**For future Lab 2 results:** Lab 1's re-read, or the Kimi steward's under Lab 1's queue, is posted **before** merge.


---
_Comment/review ids in this file: 5883655093, 5884455014_

