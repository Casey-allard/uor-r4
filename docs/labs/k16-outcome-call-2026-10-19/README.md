# K16-A vs K16-R redirect: registered two-seed outcome call — 2026-10-19

- **Lab:** Claude, the registered outcome-review seat. **Posted:** #2029 `6257335075`, 2026-10-19 00:58Z, ahead of the 01:30Z due time.
- **Sheet:** `docs/harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md`, Parts 1–4, with outcome labels at Sheet:206–212.
- **Scope:** grading of stored outputs only. No model, scorer, GPU or fresh-slice execution.

## Inputs

- **Sealed custody** (#2029 `6256611456`): training object `codex-cf1-k16-four-role-data1-training-root-part-000-2324z`, inner `payload.tar.gz` SHA-256 `1ca211b4975c61e8aaaf21a2b01911d7842f213ae4f1cd0ba1997e41f6fdfe77`, matched after the iCloud fetch.
- **Member map:** `6256619207`.
- **Final evaluation files.** The four `state-ROLE/evaluation-16404.json` raw SHA-256s match the custody pins exactly:
  - A s1 `6936cf9983f18c2e08d13e4a97e1b942f6fc3c60f14b2fa704fff3e5f86ae5f3`
  - R s1 `bb6885e8ee0d30e8efce0f6a08a9b22873894023a229056a710ef5bb0f27575c`
  - A s2 `d7976a0bce774f79c2ee06df5a8f81879e53846cc73a5e50309e3d84e7068180`
  - R s2 `0dfd6fa5707012832fc88e4ab1f39e9ef86a6be1ccaa47f2cd7daace5f080df9`
- **Per-role state:** every role is at cursor 16,404, with `updates_applied_by_evaluation = 0` and `evaluation_mutates_model = false`.
- **Checkpoint-0 `u` = 0** for every K16 arm and seed (packet G-Pre table: 0 token strict wins).
- **Extracted fields:** `evidence/k16-outcome-fields.jsonl`, SHA-256 `ea6f904895795a9d6d8899e617dd0182c38bce1f60888370549352942b0d4502`, extracted with `jq` from the sealed files.

## Fields

| Role | accepted | target_admitted | top1 | top8 | occupied | n̄ = admitted_token_count_sum / 14,287 | strict win rate |
|---|---:|---:|---:|---:|---:|---:|---:|
| A s1 | 407 | 2,237 | 65 | 547 | 209 | 351.8 | 52 / 14,287 = 0.36% |
| R s1 | 130 | 12,089 | 101 | 515 | 20 | 3,403.9 | 39 = 0.27% |
| A s2 | 410 | 3,167 | 463 | 1,295 | 192 | 376.1 | 212 = 1.48% |
| R s2 | 129 | 12,466 | 230 | 824 | 19 | 3,480.7 | 17 = 0.12% |

`target_admitted_rows` equals `admitted_cell_count_sum − address_rival_count_sum` on all four roles.

## Gates (each must hold on both seeds)

- **G-Adm (≥ 4,894).** A: **FAIL / FAIL**. R: PASS / PASS.
- **G-Occ (R against the matched A, at t = 16,404).** **Triggered on both seeds, on both clauses.**
  - Seed 1: occ_R 20 < max(32, ⌈0.5 · 209⌉ = 105), and n̄_R 3,403.9 > 2 · 351.8.
  - Seed 2: occ_R 19 < 96, and 3,480.7 > 2 · 376.1.
- **Bar (i) (top1 ≥ 1,941) and Bar (ii) (top8 ≥ 4,775).** FAIL on all four roles.
- **G-Guard (u = 0, so the threshold is max(1.00%, min(5u, u + 5 pp)) = 1.00%).** A s2 passes and the other three fail. It is not decisive, because no arm passes the bars.
- **A against R (Δ_noise rule).**
  - On `target_admitted_rows`, R's two-seed mean of 12,277.5 exceeds A's 2,702 by 9,575.5, against Δ_noise,adm = max(930, 377, 87) = 930. The advantage is **void** because R triggers G-Occ.
  - R's top-1 and top-8 means trail A's by 98.5 and 251.5 rows, within Δ_noise,top1 = 398 and Δ_noise,top8 = 748.

## Call

**`OCCUPANCY_COLLAPSE_TRIGGERED`**: R triggers G-Occ on both seeds, with K16-A failing G-Adm on both seeds. There is **no KEEP**. These are configuration negatives at this exact scope (D22), and no mechanism family is closed.

- **R** (`OccupiedCellRankHingeV1`, K16 / B4 / C192 / `rows_v1` / 16,404 batches / seeded-device init) collapses to 19–20 occupied cells.
  - Each row then admits about 3,400 of 4,096 tokens, so the G-Adm pass is vacuous pass-through.
  - Registered route: occupancy-regularized objectives.
- **A** (`NearestAddressMarginV1`, same scope) keeps occupancy healthy (16 → 192–209 cells) and raises top-1 from 19 to 65 and 463.
  - But target admission falls from 14,287 at checkpoint 0 to 2,237 and 3,167, below G-Adm.

**Fresh transfer: `NOT_TRIGGERED`.** G-Trans runs only when an arm reaches KEEP (Sheet:130). The fresh slice (binding `4bd108ad…`) stays sealed and unopened. Frozen `ft/dev` is not reported as fresh.

**Admission-arm selection: A.**
- R is excluded on G-Occ, because its admission is collapse, not discrimination.
- K16 gives A no admission advantage, since it ends below G-Adm on both seeds. So the injective sheet's registered K8 / objective-A configuration stands under its §Sequencing re-check.
- At **initialization**, D39 item 2's alias ceiling binds: the seeded codebook caps top-1 at **1,986 rows**, barely above Bar (i) at 1,941 (checkpoint-0 census `6254480425`). The injective PrimeZeta codebook (ceiling 14,287) removes that from step 0.
- **Corrected 01:1xZ (#2029 `6257392642`):** the Q4 census below shows training broke much of the aliasing (A reaches a Q4 ceiling of about 9,000 rows), yet A's top-1 stays at 65 and 463. So aliasing does **not** explain A's final ranking failure. Ranking capacity, read and schedule remain open causes (the parent's RANK_BOTTLENECK route). The injective test stays next as D39 item 6's registered step, not as a claimed sole fix.
- **Next:** the injective attention test.

## Amendment 7 coverage census at Q4 (#2029 `6257392642`)

This is host grouping and counting of the stored tables only. `evidence/k16_q4_census.rs` was built as an example at the frozen-lineage commit `b70d52089`, and its driver hash `9a1b121b…` equals the lineage's (checked). It checks each checkpoint's raw SHA against the member map, its inline model hash, cursor 16,404, and DEV through the lineage's own pins. Output: `evidence/k16-q4-census.json`, SHA-256 `59ba80a65d56c8a64785fde17903ab9439f4cfc9db747cbaa42027cebc60e0ab`. Cell metrics remain UNAVAILABLE for Route.

| Role | distinct vectors (cp0 → Q4) | alias ceiling raw (cp0 → Q4) | DEV rows on a non-representative | tokens in alias groups |
|---|---|---|---:|---:|
| A s1 | 16 → 279 | 1,986 → 9,238 | 5,049 | 3,848 |
| A s2 | 16 → 274 | 1,986 → 9,061 | 5,226 | 3,851 |
| R s1 | 16 → 86 | 1,986 → 7,161 | 7,126 | 4,033 |
| R s2 | 16 → 82 | 1,986 → 5,645 | 8,642 | 4,045 |

G-Cap at Q4 is ALIAS_CEILING_ABOVE_BAR on all four roles (bar 1,941).
- **D21:** this line does not move the M1 headline.
