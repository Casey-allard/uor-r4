# ORDER: r-full-epoch-revisit-schedules, antigravity-66

Lab: antigravity. Item: [r-full-epoch-revisit-schedules](../../../../roadmap/items/r-full-epoch-revisit-schedules.md). Base: `cef4f6654`.

## Goal

Author and FIT-review the one-page pre-registration sheet (`docs/harness/prep/r-full-epoch-revisit-schedules-2026-10-25.md`, `CHARTER.md` §4.5) and lab synthesis (`docs/labs/r-full-epoch-revisit-schedules/README.md`) for `r-full-epoch-revisit-schedules` (`#2878`) — comparing the 1-pass `16,404`-batch baseline (`1,049,856 / 2,425,925 = 43.28%` `TRAIN` targets) against a full-epoch schedule (`37,906` batches covering `2,425,925 / 2,425,925 = 100%` `TRAIN` targets) and a Zipf-stratified frequent-token coordinate revisit schedule (`FrequentTokenRevisitV1`) under Stage T `BVG-K` (`K = 16`) + D40 admission gating — and transition `r-full-epoch-revisit-schedules` to `status: held` under a fresh-context D68 decision review until `#2836` (`m1-bvg-k-stage-t-training`, Step 4 of `PLAN.md`) completes.

## Cancel probe (run first)

Ran on `origin/main` at `cef4f6654db63cfbce9468de5cb64e1b383d51d0`:

1. `ls docs/harness/prep/r-full-epoch-revisit-schedules* docs/labs/r-full-epoch-revisit-schedules` -> no pre-registration sheet or lab directory exists (`instrument` missing).
2. In `crates/uor-r4-discrete/src/{training,completion}.rs`, `SchedulePolicy` defines only `CanonicalVectorThenReadV1` and `InterleavedReadEvery8V1`, and `COMPLETION_BATCHES = 16_404` consumes `16,404 * 64 = 1,049,856` of `2,425,925` `TRAIN` targets (`43.28%`; covering all `2,425,925` targets requires `\lceil 2,425,925 / 64 \rceil = 37,906` batches). Neither a full-epoch driver schedule (`37,906` batches) nor a frequent-token revisit schedule is implemented in `crates/uor-r4-discrete` (`instrument` missing), and `r-full-epoch-revisit-schedules.md` line 10 restricts `scope:` to `[docs/harness/prep/r-full-epoch-revisit-schedules*, docs/labs/r-full-epoch-revisit-schedules/**, docs/loop/tasks/r-full-epoch-revisit-schedules/**]`, excluding `crates/uor-r4-discrete/**`.
3. In `#2831` (`docs/labs/m1-a8-full-dose-rerun/README.md`), under `K = 1` single-coordinate proposals + D40 64-batch rollback (`floor = 1,187 / 3,847`), `Relation` halted at batch `1,600` and `FrozenZero` halted at batch `5,248` on `STOP_FOR_DIAGNOSIS: consecutive_rejections` (`instrument` cadence-granularity coupling) before reaching `16,404` batches. Replacing `K = 1` with `BVG-K` (`K = 16`) Stage T updates is `#2836` (`m1-bvg-k-stage-t-training`, Step 4 of `PLAN.md`).
4. In W2 (`#2845`, `docs/labs/m1-w2-capacity-capture/README.md`), the exact read-code table on the `1,049,856` `TRAIN` targets of the `16,404`-batch slice covers `14,175 / 14,287` (`99.22%`) of `DEV` rows and scores `T_code = 3,414 / 14,287` (`T_restr = 1,138 / 1,920`) vs `149 / 14,287` for the Q4 writer, localizing the `16,404`-batch deficit primarily to **per-coordinate revisit starvation** (`1` visit / `64` rows per coordinate under `K = 1`, vs `16` visits per pass under `BVG-K` `K = 16`).

- **Classification:** **`instrument`** (pre-registration sheet and lab directory not yet authored; `#2836` Stage T `BVG-K` law still open; item `scope:` excludes `crates/uor-r4-discrete/**`).

## Files in scope

- `docs/harness/prep/r-full-epoch-revisit-schedules*`
- `docs/labs/r-full-epoch-revisit-schedules/**`
- `docs/loop/tasks/r-full-epoch-revisit-schedules/**`
- Always-allowed paths (`docs/loop/README.md` lines 103, 190–194): `docs/roadmap/items/r-full-epoch-revisit-schedules.md` (`status:` line only) and `docs/roadmap/README.md`.

## Done when

- `docs/harness/prep/r-full-epoch-revisit-schedules-2026-10-25.md` and `docs/labs/r-full-epoch-revisit-schedules/README.md` are committed, a fresh-context non-author `TEST FITNESS: FIT` + D68 decision review (`status: ready -> status: held` until `#2836` completes) is posted on `#2878`, and `uor-harness loop gates --full` + `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Never stop to wait; decide via D68 review and log on `#2878`.
