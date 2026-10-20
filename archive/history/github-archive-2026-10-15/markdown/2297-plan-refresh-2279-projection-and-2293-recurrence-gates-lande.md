# #2297 — Plan refresh: #2279 projection and #2293 recurrence gates landed, OP3a is Codex's (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T05:20:27Z
- **Updated:** 2026-10-13T05:31:54Z
- **Closed:** 2026-10-13T05:31:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2297
- **Merged:** 2026-10-13T05:31:28Z
- **Merge commit:** a73c455c97603518beac052cc10af3b97226fddf
- **Base / Head:** main / UOR-Foundation:kimi/plan-refresh-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: no model line — plan hygiene · count 0/3 · headline 37/232 → 37/232 (unchanged)

## The plan documents catch up: projection and recurrence gates landed, OP3a is Codex's

Docs-only refresh so the plan and tracker documents read the live dependency board on M1 #2029. Record = the refreshed documents themselves; the newest-first `docs/integration/current-state.md` entry is the change log.
Decision: **NOT YET PROMOTED** — no model line, no compute, no count reset, headline unchanged (reply 37/232, v5 memory 20/40). No `scripts/harness/*.py` and no Rust source.

### What changed and why

- `docs/harness/prep/m1-rank-margin-rung0-2026-10-13.md` — the dependency board is current: **dependency 3 LANDED** (#2293, merge `d7934bf03`: `rec_gates=geometric` — clamped-linear opening, exact dyadic rate-select table `lambda = 2^-(s+1)` with keeps precomputed, Hamilton output gate, geometric secant surrogate credit; the executed recurrence path is squash-free by source sweep, and `RecGates::Historical` is preserved as the reproduction path and prohibited for new runs). **Dependency 2 is Codex-owned** per the ratified ownership split ([#2029 comment 6138381873](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138381873)), in flight on `codex/geometric-head-shared-writer-20261013`, with its finite kernel card FIT (self-review under the exact-head rule, [#2029 comment 6138479221](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138479221)). **Dependency 4** notes Codex's fixed-controls pattern (stored/analytic expected values, no CPU reference) as the template. The path table, the squashing-sites table (the four recurrence rows now read "Resolved by #2293") and the config table (`rec_gates=geometric` added) follow. The run stays BLOCKED on dependencies 2 and 4 plus the hold; exactly one `TEST FITNESS: PENDING` verdict line remains.
- `docs/labs/operator-phase-2026-10-12/README.md` — a dated "Status 2026-10-13" section above the registration text (OP2 satisfied by the #2279 replacement, recurrence gates landed, OP3a Codex-owned in flight, the sheet carries the live bars), with the original text kept as history.
- `ROADMAP.md` — the M1 row names the #2279 replacement, the #2293 recurrence gates and the in-flight Codex head, in one line, keeping the table format.
- `docs/harness/claims.json` — R1 lifecycle, closing the new checks' first catch: the three stale kimi claims (`kimi/d27-rust-first`, `kimi/rung0-sheet-bars`, `kimi/pm-rules`; branches merged as #2278, #2290 and #2295, all deleted) are released — two had been restored into main by #2296's store sync, where the newly landed `multilab.claim_not_merged` / `multilab.claim_stale` checks flagged them and that entry asks Kimi to close them — and this branch's own claim rides in the delivery.
- A newest-first `docs/integration/current-state.md` entry (kept above #2296's entry when the merge was resolved); claim/heartbeat/release receipts.

### Verification

- [x] Checks run at this exact head (`226e2a54b`, with the same checks run at the pre-merge branch head `b7dd66d8d`) — commands and outcomes below
- [x] `python3 scripts/harness/cli.py check` — 0 errors (48 rules); warns are pre-existing or other labs' (claude's stale presence, the sheet's expected `TEST FITNESS: PENDING`, m5–m8 unfrozen panels, `bpb-split`, Codex's unmerged in-flight branch, the caps notice). The branch merged #2296's new claim checks; with the three stale claims released they no longer flag
- [x] `python3 scripts/harness/cli.py lint check --body-file <this body>` — 0 errors
- [x] The frozen state-format check (`harness.state_formats`; #2296 replaced the Python `statecheck.py` with the Rust-backed adapter) — claims ok; `docs/harness/claims.json` stays valid JSON the CLI reads
- [x] `python3 scripts/harness/cli.py multilab claims` — reads the store before and after; one live claim (this branch's), the three stale releases cleared
- [x] `python3 scripts/check_claim_wording.py` — PASS; `python3 scripts/check_no_softmax.py` — OK, 1649 (baseline 1649); `python3 scripts/check_no_matmul.py` — OK, 93 sites / 26 files (baseline 93)
- [x] Exactly one `TEST FITNESS:` verdict line in the sheet; docs only — no Rust source and no `scripts/harness/*.py` touched
- [x] Files one deliberate lint warn: `prlint.count_matches_registry` ("no model line" is not an M1 registry line — governance work); the record-path check is satisfied by the cited `docs/labs/recurrence-geometric-gates-2026-10-13/README.md`

References #2029 and #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

