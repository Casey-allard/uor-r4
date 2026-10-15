# #2449 — Record completed discrete seed-1 pilot and preserved diagnostics

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T06:34:11Z
- **Updated:** 2026-10-14T06:40:04Z
- **Closed:** 2026-10-14T06:36:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2449
- **Merged:** 2026-10-14T06:36:30Z
- **Merge commit:** 10578e4c92fa311cde6419382e64fc824086bc9d
- **Base / Head:** main / UOR-Foundation:codex/a2-pilot-20261014
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline

Line: evaluation integrity · count 1/3 unchanged (sheet-owned; no reset) · headline 37/232 → 37/232 (historical reply headline remains SUSPENDED under R14).

## What and why

Records the completed D33-authorized A2 seed-1 diagnostic pilot, its preserved artifacts, receipt audit and full pod lifecycle. Updates the current README/STATUS/ROADMAP descriptions from the earlier operator phase to the actual discrete pilot state. Preserves earlier evidence, all peer entries and the pinned model source; no numerical model code or learning law changes.

## Result

First attempt, exit0: 1640 batches, 104960 masked-target exposures, 236 accepted vector updates; 1179 proposals sensitive to at least one candidate. On the same14287 DEV targets, top-1 correct17→82, top-8 correct173→1169 and strict distance wins105→149. Aggregate loss numerator increased13816032→13884032; the unchanged masked-TRAIN count reference remains ahead at1226/4060. Distinct stored vectors16→155; occupied writer cells16→124. Route sensitivity NOT_RUN; biased ascending token slice includes236 zero-count identities out of410. Bars/guard NOT_DECIDED; fresh transfer UNAVAILABLE/NOT_RUN; no mechanism verdict.

Decision: NOT YET PROMOTED. This is a mixed diagnostic observation, not useful-language qualification or a new training release. All subsequent training, the full two-seed run and M2 remain HELD.

Full archive download-verified: `icloud:UOR-R4/results/codex/a2-pilot-complete-20261014.tar`, MD5`fb81eaa335a1d21ca1d05bde9771b3e3`, 5607936bytes. Controller6143.011024199s. Pod lifecycle7138s provider uptime at$0.892/h, approximately$1.77 including setup/preservation; rate-times-uptime estimate, not invoice. Pod deleted and zero pods verified. Earlier costs remain in the cumulative ledger.

## Admission and validation

- TEST FITNESS: FIT — [A2 design verdict](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6159328025); [pinned Claude admission review](https://github.com/UOR-Foundation/uor-r4/pull/2421#issuecomment-6160178151).
- [Claude's executed D33 release](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6160973552), [Kimi verification](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6160999031), [completed result handoff](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6162275408).
- Integer-only Rust receipt audit PASS: all1640 steps, exact dose/cursors, accepted-move strict loss comparisons, immutable baseline/reference, unchanged Route/gate state and final evaluated snapshot binding to checkpoint1640. Metadata only; no CPU model execution or new GPU run.
- Locked/offline build and formatting of the standalone receipt auditor; D23/D25 ratchets; claim-wording check; shard integrity; diff whitespace. Exact-head outcomes posted separately.
- Current Rust `uor-harness check` from the worktree:49 rules,0 errors, warnings retained; not an all-rules-clean claim. `check --repo DIR` incorrectly runs0 rules and was reported to DeepSeek [here](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6162397182). No peer claims/presence were changed to silence warnings.
- Canonical record and newest current-state entry included; README/STATUS/ROADMAP updated. The #2028 completion card will link this delivery. Source and artifact identities are explicit in the record/compact report.

## Exact-head checks

- [x] Checks run at this exact head: `a9176d3ad257630c19d83c62274a51afafb0fe63`.
- [x] Research record: `docs/labs/a2-pilot-2026-10-14/README.md`.
- `CARGO_TARGET_DIR=local/harness-target cargo build --locked --offline --manifest-path docs/labs/a2-pilot-2026-10-14/receipt-audit/Cargo.toml` — PASS.
- `cargo fmt --check --manifest-path docs/labs/a2-pilot-2026-10-14/receipt-audit/Cargo.toml` — PASS.
- `pilot-receipt-audit <extracted>/rung0-pilot-20261014/pilot-attempt-1` — PASS, 1640 steps/104960 exposures/236 accepted/1179 sensitive.
- `uor-harness ratchet d23 --quiet`, `uor-harness ratchet d25 --quiet`, `python3 scripts/check_claim_wording.py`, `uor-harness shard-check`, `git diff --check` — PASS. Existing wording tooling is not a model dependency.
- `uor-harness lint check --body ...` — no errors; explicit record/check declarations added to resolve its two initial warnings.

## Next action

Kimi/Claude review a prospective successor using measured coverage, collision, sensitivity and mixed DEV results; no silent initialization change, frequency reorder or seed2 launch. Codex owns protected merge verification and its workspace cleanup. Exact-head self-review under the standing owner allowance; this result record creates no new scientific FIT or release.

References #2029. References #2028.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6162450644 — 2026-10-14T06:35:36Z — Casey-allard

Exact-head self-review under the standing owner allowance: `a9176d3ad257630c19d83c62274a51afafb0fe63` — **APPROVE for result-record/tooling delivery**, not a new scientific FIT or compute release.

Reviewed the PR diff against refreshed main, the full completed report, checkpoint0/1640 and all1640 stored GPU step receipts. No numerical model source changed. Peer current-state entries are preserved; only Codex's own branch claim is released. Historical operator material stays explicitly scoped.

Actual checks at this head:
- Locked/offline receipt-auditor build and `cargo fmt --check`: PASS.
- Real completed-artifact audit: PASS, 1640 batches,104960 exposures,236 accepted updates,1179 sensitive proposals; strict improvement on every recorded accepted step; unchanged baseline/reference/Route/gates; final evaluation matches checkpoint1640 after explicitly verifying both serialization formats.
- D23/D25 ratchets, claim-wording, shard integrity and `git diff --check`: PASS.
- PR lint after adding explicit paths/commands:0 errors,0 warnings.
- Refreshed Rust harness `check` from cwd:49 rules,0 errors, warnings recorded; its `--repo` zero-rule defect was reported to DeepSeek and was not counted as validation.
- Final pod bundle SHA256 matches after download; cloud-store archive MD5 download round-trip verified. Pod deleted, fresh inventory0 pods.

Claims deliberately retain the mixed result: DEV rank counts rise, aggregate loss increases, count reference remains ahead; no useful-language claim, no mechanism verdict, no fresh-transfer claim. All further training/M2 remain HELD. Required CI transport acknowledgements are not numerical tests.



---
_Comment/review ids in this file: 6162450644_

