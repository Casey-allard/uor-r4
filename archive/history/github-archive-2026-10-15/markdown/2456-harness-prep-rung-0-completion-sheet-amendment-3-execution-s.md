# #2456 — harness/prep: rung-0 completion sheet Amendment 3 (execution scope + receipt diagnostics)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T08:35:37Z
- **Updated:** 2026-10-14T08:37:18Z
- **Closed:** 2026-10-14T08:36:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2456
- **Merged:** 2026-10-14T08:36:51Z
- **Merge commit:** 4efd3f95befa11e518be83a12c1c5d222dd26408
- **Base / Head:** main / UOR-Foundation:kimi/rung0-completion-sheet-amend3
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Registers **Amendment 3** on `docs/harness/prep/m1-discrete-rung0-completion-2026-10-14.md` (Amendment 2, `c109b2109`, TEST FITNESS: FIT (design) confirmed by Claude in #2029 comment 6163957579). **Sheet text only** — no bar, guard, law, data, evaluator, dose, schedule, initializer or code change.

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (unchanged)

**(A) Driver-lineage migration permitted** — Claude's item N1 ([#2029 6164028158](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6164028158)). New section *Execution-scope permissions*: the completion's D5 admission diff necessarily changes the operational driver, so the new driver hash includes the new controller; the change is operational only — model, cursor, baseline and configuration are unchanged, so the resumed trajectory stays the registered one. Seed 1 keeps its old-lineage baseline: its checkpoint-0 and `u` constants stay exactly as measured under the parent lineage (105/14,287 ≈ 0.735% → seed-1 guard ≈ 3.67%), and the pilot's checkpoint 1,640 remains the resume base. No bar, guard, baseline or dose is re-derived.

**(B) Owner-personal per-seed release** — D33's delegated pilot authority does NOT extend to the completion. Step 4 is restated: two releases, one per seed, each after Kimi's OwnerReleasePlan packet for that seed's scope, no seed-1→seed-2 chain and no other form of release. D6 updated to match, with the seed-1-first option preserved (separate per-seed plans).

**(C) Exact pilot-receipt diagnosis registered** — Codex ([#2029 6163817784](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6163817784)). New subsection under *Why this run*: loss accounting (token −63,968 / address +131,968 / total +68,000); admission cohorts in fixed order retained 580 / lost 5,100 / gained 1,600 / absent-both 7,007 with numerator deltas −6,720 / +961,056 / −588,768 / −297,568, gained-admission rows supplying 983 of the 1,169 final top-8 hits — why aggregate loss rose while rank measures improved, with no per-parameter causal claim; initialization coupling (four roots from one low-4 xorshift value → 16 full patterns × 256 token identities at V4096/B4, so seed 2 cannot resolve that collision partition; collision-free init stays a named successor, no unreviewed initializer change enters D5); parameter pass ≠ corpus pass (1,049,856 of 2,425,925 masked target exposures, 1,376,069 unvisited, no wrap, no doubled unique coverage); zero-count coordinates are NOT inactive (944/679/90 vs 696/500/146, context/rival use possible).

**(D) Amendment 1 wording corrected** — each visited token has FOUR VectorBlock proposals (one per block), early, against an uninitialized codebook, and no visited block is revisited in the registered pass. Corrected in section 2 and in the matching B2 restatement in *Outcome decisions*.

**(E) Guard meaning registered** — Codex ([#2029 6164092169](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6164092169)). The registered strict-distance-win guard is `token_rivals > 0 && token_hinge < width` and does NOT require the target cell to be admitted: at width 32, baseline 105 strict wins = 0 admitted; pilot end 149 = 36 admitted / 113 omitted. It measures strict distance advantage against admitted rivals including unadmitted targets — not executable writer successes. The admitted-strict-win intersection is now a registered per-checkpoint diagnostic. **The guard and both bars are UNCHANGED**; any change would need a prospective amendment.

**(F) Working storage corrected** — Codex ([#2029 6163933502](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6163933502)). Measured pilot checkpoint 3,138,466 bytes uncompressed → at ~260 checkpoints **~0.8 GB per full seed** before eval/log overhead; the pilot's ~6 MB compressed archive is NOT the completion's working storage. Preservation keeps the run-state evaluation files plus every attempt directory and the original parent together; never copy an owner record.

**Verdict** stays `TEST FITNESS: FIT (design)`, extended: Amendment 3 registers execution-scope permissions (driver-lineage migration, owner-personal per-seed release) and receipt-exact diagnostics; Claude's one-line re-confirmation requested.

Checks: `UOR_LAB=kimi uor-harness check` exits 0 — 60 rules, 20 failed (0 error, 19 warn), finding set identical to the baseline run with this edit reverted; no new finding. Training and M2 HELD.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

