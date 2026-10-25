# REPORT: r-full-epoch-revisit-schedules, antigravity-66

Lab: antigravity. Item: [r-full-epoch-revisit-schedules](../../../../roadmap/items/r-full-epoch-revisit-schedules.md). Order: [ORDER.md](ORDER.md). Issue: References #2878.

## What did not work

- **Cancel probe finding (`instrument`):** Running the cancel probe on `origin/main` at `cef4f6654db63cfbce9468de5cb64e1b383d51d0` identified three `instrument` blockers to immediately launching the 4-arm $\times$ 2-seed GPU run:
  1. **Missing pre-registration sheet (`instrument`):** No `docs/harness/prep/r-full-epoch-revisit-schedules*` pre-registration sheet or `docs/labs/r-full-epoch-revisit-schedules/` lab directory existed (`CHARTER.md` §4.5).
  2. **Unwired full-epoch and frequent-token schedules + docs-only scope (`instrument`):** In `crates/uor-r4-discrete/src/{training,completion}.rs`, `SchedulePolicy` defines only `CanonicalVectorThenReadV1` and `InterleavedReadEvery8V1`, and `COMPLETION_BATCHES = 16_404` consumes `16,404 * 64 = 1,049,856` of `2,425,925` `TRAIN` targets (`43.28%`; covering all `2,425,925` targets requires $\lceil 2,425,925 / 64 \rceil = 37,906$ batches). Neither a `37,906`-batch full-epoch schedule nor a Zipf-stratified frequent-token revisit schedule (`FrequentTokenRevisitV1`) is wired into `crates/uor-r4-discrete`, and `docs/roadmap/items/r-full-epoch-revisit-schedules.md` line 10 restricts `scope:` to `[docs/harness/prep/r-full-epoch-revisit-schedules*, docs/labs/r-full-epoch-revisit-schedules/**, docs/loop/tasks/r-full-epoch-revisit-schedules/**]`, excluding `crates/uor-r4-discrete/**`.
  3. **`K = 1` 64-batch cadence-granularity stop in `#2831` (`instrument`, `mechanism_negative: false`):** In [`docs/labs/m1-a8-full-dose-rerun/README.md`](../../../labs/m1-a8-full-dose-rerun/README.md), running `K = 1` single-coordinate proposals under active D40 64-batch whole-interval rollback (`floor = 1,187 / 3,847`) halted `Relation` at batch `1,600` and `FrozenZero` at batch `5,248` (`STOP_FOR_DIAGNOSIS: consecutive_rejections`) before reaching `16,404` batches. Replacing `K = 1` with the `BVG-K` (`K = 16`) batched-coordinate Stage T learning law is `#2836` (`m1-bvg-k-stage-t-training`, Step 4 of `PLAN.md`).

## What changed

- **Pre-registration sheet ([`docs/harness/prep/r-full-epoch-revisit-schedules-2026-10-25.md`](../../../harness/prep/r-full-epoch-revisit-schedules-2026-10-25.md)):** Authored the one-page pre-registration sheet (`CHARTER.md` §4.5) for the $2 \times 2$ factorial comparison (`C_one_pass` `16,404` flat vs `H_freq_16k` `16,404` Zipf-stratified `FrequentTokenRevisitV1` vs `H_full_epoch_flat` `37,906` flat `100%` target coverage vs `H_full_epoch_freq` `37,906` `100%` target coverage + `FrequentTokenRevisitV1`) under Stage T `BVG-K` (`K = 16`) + D40 admission gating across two seeds (`seed = 1, 2`), with fresh-context non-author `TEST FITNESS: FIT` and `D68 DECISION REVIEW: PASS` posted at [`#2878` comment `6396559418`](https://github.com/UOR-Foundation/uor-r4/issues/2878#issuecomment-6396559418).
- **Lab synthesis ([`docs/labs/r-full-epoch-revisit-schedules/README.md`](../../../labs/r-full-epoch-revisit-schedules/README.md)):** Decomposed the coverage question into Axis A (corpus target coverage: `1,049,856 / 2,425,925 = 43.28%` at `16,404` batches vs `2,425,925 / 2,425,925 = 100.0%` at `37,906` batches, showing from W2 `#2845` that the `1,049,856`-target slice already achieves `14,175 / 14,287 = 99.22%` `DEV` read-code coverage and `T_code = 3,414 / 14,287`) and Axis B (per-coordinate revisit starvation: `1` visit / `256` rows per token under `K = 1` vs `16` visits / `4,096` rows under `BVG-K` `K = 16` flat vs `~128` visits / `32,768` rows on top-`256` tokens at `16,404` batches and `~296` visits / `75,776` rows at `37,906` batches under `FrequentTokenRevisitV1`).
- **Roadmap status (`docs/roadmap/items/r-full-epoch-revisit-schedules.md` and `docs/roadmap/README.md`):** Transitioned `r-full-epoch-revisit-schedules` from `status: ready` to `status: held` under the D68 decision review (`#2878` comment `6396559418`) until `#2836` (`m1-bvg-k-stage-t-training`) completes, and re-rendered `docs/roadmap/README.md`.
- **Size:** `6` files (`0` code, `0` tests, `5` docs `+192/-1`, `1` generated roadmap table `+3/-3`), `+195 / -4` lines against `origin/main` (`cef4f6654db63cfbce9468de5cb64e1b383d51d0`).

## Measurements

- **Corpus target coverage & W2 read-code saturation (`DEV` `n = 14,287` rows; from `crates/uor-r4-discrete/src/completion.rs:14`, `docs/model/training.md:23`, and [`docs/labs/m1-w2-capacity-capture/README.md`](../../../labs/m1-w2-capacity-capture/README.md)):**
  - 1-pass canonical dose (`16,404` batches $\times$ `64` rows): `1,049,856 / 2,425,925` `TRAIN` targets (`43.28%`), leaving `1,376,069` targets (`56.72%`) unseen.
  - Full-epoch schedule ($\lceil 2,425,925 / 64 \rceil = 37,906$ batches): `2,425,925 / 2,425,925` `TRAIN` targets (`100.00%`, `2.3108x` batches).
  - Unigram `DEV top-1` (`U`): `1,226 / 14,287` (`8.58%`) on both the `1,049,856`-target `16,404`-batch slice and the full `2,425,925`-target `TRAIN` split (`0` rows difference).
  - W2 exact read-code table on the `1,049,856`-target `16,404`-batch slice: `14,175 / 14,287` (`99.22%`) `DEV` row coverage (`41,806` codes with count $\ge 2$), `T_code = 3,414 / 14,287` (`23.90%`), and admission-matched `T_restr = 1,138 / 1,920` (`59.27%`) vs `149 / 14,287` for the Q4 writer.
- **Coordinate revisit counts across schedules (`V = 4,096` tokens $\times$ `B = 4` blocks = `16,384` vector-block coordinates):**
  - Legacy `K = 1` flat (`16,404` batches): `~1.0` visit/block (`256` rows/token across `4` blocks).
  - Stage T `BVG-K` (`K = 16`) flat (`C_one_pass`, `16,404` batches): `~16.0` visits/block (`4,096` rows/token).
  - Stage T `BVG-K` (`K = 16`) `FrequentTokenRevisitV1` (`H_freq_16k`, `16,404` batches, `50% / 30% / 20%` tiers): `~128` visits/block (`32,768` rows/token) on top-`256` tokens, `~25.6` visits/block (`6,554` rows/token) on ranks `257..1,024`, `~4.3` visits/block (`1,092` rows/token) on ranks `1,025..4,096`.
  - Stage T `BVG-K` (`K = 16`) full-epoch (`37,906` batches): `~37.0` visits/block (`9,472` rows/token) under `H_full_epoch_flat`, and `~296` visits/block (`75,776` rows/token) on top-`256` tokens under `H_full_epoch_freq`.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- roadmap render` — PASS
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- roadmap check` — PASS
- `uor-harness loop gates --full` — PASS (`11/11` gates green)
- `uor-harness loop report --check --order docs/loop/tasks/r-full-epoch-revisit-schedules/antigravity-66/ORDER.md` — PASS
- `uor-harness loop pr-check --local` — PASS (`0` FAIL, `0` WARN)

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `gates`.

## Still open

- Implementing `FrequentTokenRevisitV1` and the `37,906`-batch full-epoch continuation in `crates/uor-r4-discrete` and executing the 4-arm $\times$ 2-seed GPU run once `#2836` (`m1-bvg-k-stage-t-training`, Step 4 of `PLAN.md`) completes and `r-full-epoch-revisit-schedules` scope includes `crates/uor-r4-discrete/**`.

## Plain summary

A standard 16,404-batch training run (`COMPLETION_BATCHES = 16,404`) processes `16,404 x 64 = 1,049,856` training targets out of `2,425,925` total (`43.28%`), and visits each token's four 8-dimensional vector blocks in a flat round-robin order so that common words and rare words get the exact same number of update attempts. Using the recently completed W2 read-code capacity capture (`#2845`), this unit shows that the first `1,049,856` training targets already cover `99.22%` (`14,175 / 14,287`) of the development panel's read codes and support `3,414 / 14,287` exact top-1 predictions, meaning the main bottleneck at `16,404` batches is how few times each frequent token's coordinates are revisited (`256` total rows per token under `K = 1`) rather than missing the second half of the training stream. We author and FIT-review the one-page pre-registration sheet (`docs/harness/prep/r-full-epoch-revisit-schedules-2026-10-25.md`) for a $2 \times 2$ test of full-epoch coverage (`37,906` batches, `100%` of targets) and Zipf-stratified frequent-token revisits (`FrequentTokenRevisitV1`) under the batched-coordinate `BVG-K` (`K = 16`) law, and place `#2878` in `held` status until `#2836` (`m1-bvg-k-stage-t-training`) finishes on the M1 path.
