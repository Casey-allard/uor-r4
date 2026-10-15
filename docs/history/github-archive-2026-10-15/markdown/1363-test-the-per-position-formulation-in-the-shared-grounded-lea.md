# #1363 — Test the per-position formulation in the shared grounded learner: the prose deficit was the curriculum (References #820, #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-23T08:08:00Z
- **Updated:** 2026-10-09T13:17:38Z
- **Closed:** 2026-09-23T17:12:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1363
- **Merged:** 2026-09-23T17:12:21Z
- **Merge commit:** 8deb92f979e5562c2ecff1e958bba9a7a61bb655
- **Base / Head:** main / UOR-Foundation:codex/lexical-formulation-20260923
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Executes the milestone the [September 23 audit](docs/integration/ordinary-lexical-audit-2026-09-23.md) recorded: train the retained `TlModel` on the per-position, full-context objective with the larger target batch **while keeping the grounded Copy/Stop supervision**, and check the prose recovery plus the authored/held-out panels on open development.

`References #820` and `References #973`. Nothing is closed here.

## What changed

**Runner only.** `crates/uor-r4-core/src/bin/ordinary-lexical.rs` gains declared options `--objective window|position`, `--curriculum delivered|interleaved`, `--prose-slots`, `--ground-slots`, `--ground-weight`, plus a read-only `--condition <new root> --artifact <path>` mode that scores **one sealed artifact** under both prose conditionings without touching it. The model source `learner/transferable_lexical.rs` is **unchanged** (`sha256 03f83eb8…b250`, the principal-reviewed hash).

**Baseline guard.** With no new option set, the default path reproduces the delivered artifact **bit for bit**: `sha256 fe3e8a638cd50a19741065cb00ac787b63bb59f8ad5776fd8bfcb162c37a912a`, development `7.372198274238398`, every gain and interval identical. Every arm below therefore runs through one unchanged learner and one unchanged default path.

## Measured result (served window conditioning — the recurrence the served multi-token path executes)

Same corpus, split, seed 13, lr 0.02, 2,000 steps; same 5,376 development targets; paired document-cluster intervals.

| arm | objective | grounded draws/step (weight) | served bits/target | preflight A | held-out | preflight B | training |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `olx-form-1` (guard = delivered) | window | delivered schedule | **7.3722** | 32/32 | 3/3 | 2/2 | 36/36 |
| `olx-form-2` | window | 0 | **6.6841** | 0/32 | 0/3 | 0/2 | 0/36 |
| `olx-form-3` | window | 2 (4.0) | **6.9686** | 31/32 | 1/3 | 2/2 | 35/36 |
| `olx-form-4` | position | 24 (4.0) | **7.0327** | 32/32 | 3/3 | 2/2 | 36/36 |
| `olx-form-5` | position | 0 | **6.6903** | 0/32 | 0/3 | 0/2 | 0/36 |

- `olx-form-2` − delivered **−0.6881 [−0.7686, −0.6091]** — crosses donor E (6.8701) for the first time. Removing the grounded rehearsal mix and the grounded-only final phase and keeping prose at full batch accounts for the whole per-position diagnostic's recovery.
- `olx-form-4` − delivered **−0.3395 [−0.4388, −0.2447]** with **full** grounded-panel retention.
- `olx-form-4` − `olx-form-3` **+0.0641 [−0.0142, +0.1490]**, and `olx-form-5` − `olx-form-2` **+0.0062 [−0.0883, +0.1012]**: the per-position objective is a wash for the served metric and is **retired as a necessary training change**, remaining a legitimate diagnostic conditioning.
- `olx-form-5` reproduces the retained `olx-channel-1` `recurrence_only` arm **exactly** through a different runner path (6.6903113844186946 / 6.203876300792837).

## Correction

The retained audit's "about 1.17 bits" compared each artifact under **its own** conditioning. Under the served conditioning the per-position prose-only artifact recovers **0.682 [0.581, 0.790]**; roughly 0.49 bits was the conditioning change, and serving enjoys that conditioning only for the first generated token. The audit's data set, artifact and sealed numbers are unchanged; the correction is scoped and recorded in the new result document and every public restatement.

## Controls and verification

- Default-path artifact identity to the delivered baseline (the guard above).
- Independent fresh-process deserialisation of the milestone artifact (`olx-form-4-audit`): training 36/36, held-out 3/3, class 4/4, preflight A 32/32, preflight B 2/2 with identity-erased aliasing and source-disabled copy loss, five mixed-session phases identical.
- `--condition` re-derives the delivered artifact's stored window score exactly and the retained channel arm's numbers exactly.
- 12 retained `transferable_lexical` unit tests pass; `cargo fmt --check` passes; offline release build passes; `scripts/check_claim_wording.py` passes.
- Step-probe trajectory of the guard run, which the prior audit could not obtain because no intermediate checkpoint was persisted.

## Limits

One pinned local corpus, one split, one seed per arm, one schedule family; the grounded world is authored and tiny, so per-arm panel retention is a fixture result, not general language. Greedy generation still collapses on every arm; sampling NOT_RUN. The tuned two-token count reference (5.1217) still beats every arm by 1.56–1.91 bits/target, so the 64-coordinate ternary state under-uses the two tokens available at the decision point. Energy UNAVAILABLE; the served kernel executes no multiplier and no floating-point arithmetic. No capability promotion and no geometric-advantage claim.

## Resources

Five fits, two scoring runs and one audit: **4,776,000 ms of measured machine time**. The complete milestone is charged once at **7,300,000 ms**; the owner ledger advanced from 387,915,640 to **395,215,640 / 396,700,000 ms**. The prospective projection (6,000,000 ms, recorded before any fit, limit extended 390,700,000 → 396,700,000) was exceeded by 1,300,000 ms because the five-arm schedule was larger than projected; the balance stayed positive. Physical free after the work is 55,628,864 KiB, above the 24 GiB reserve plus the 128 MiB stop margin. No paid or external compute; no unique artifact or preserved root modified.

## Preserved / new

Preserved and unmodified: `ordinary-lexical-3` (byte-identical to the new guard artifact), `olx-channel-1`, `olx-audit-1..5`, and the owner checkout. New sealed roots: `olx-form-1..5`, `olx-form-4-audit`, `olx-condition-1/2`.

## One evidence-supported next step

Keep the served window conditioning and raise the grounded presence until the authored panels are fully retained (the configuration `olx-form-3` was one case short of), pre-declaring 32/32 preflight A, 3/3 held-out and 4/4 class as acceptance and measuring served prose between `olx-form-3`'s 6.9686 and `olx-form-2`'s 6.6841; then attack the remaining 1.56-bit local deficit to the tuned two-token count reference, which is a local-recency problem rather than a long-range transport one. Signed H4/shared relative transport stays conditional on a witnessed order/role/distant-interference failure against an information- and compute-matched ordinary control.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

