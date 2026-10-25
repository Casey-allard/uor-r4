# Full-Epoch and Frequent-Token Revisit Schedules (`r-full-epoch-revisit-schedules`, `#2878`)

- **Lab:** `antigravity` ([`#2878`](https://github.com/UOR-Foundation/uor-r4/issues/2878), Draft PR [`#3211`](https://github.com/UOR-Foundation/uor-r4/pull/3211), ORDER [`antigravity-66`](../../loop/tasks/r-full-epoch-revisit-schedules/antigravity-66/ORDER.md))
- **Milestone / Line:** `research` / `m1-discrete-attention` — mechanism [`discrete-learning`](../../mechanisms/discrete-learning.md) (`ACTIVE`)
- **Pre-Registration Sheet:** [`../../harness/prep/r-full-epoch-revisit-schedules-2026-10-25.md`](../../harness/prep/r-full-epoch-revisit-schedules-2026-10-25.md) (**TEST FITNESS: FIT**, **D68 DECISION REVIEW: PASS** at [`#2878` comment `6396559418`](https://github.com/UOR-Foundation/uor-r4/issues/2878#issuecomment-6396559418))
- **Current Item Status:** **`held`** (held on `#2836` `m1-bvg-k-stage-t-training`, Step 4 of [`PLAN.md`](../../../PLAN.md), and scope widening to include `crates/uor-r4-discrete/**` when `#2836` lands)

---

## 1. Arithmetic & Empirical Decomposition of the Coverage Question

Roadmap item `r-full-epoch-revisit-schedules` (`#2878`) asks whether the low `DEV` `top-1` of the 1-pass `16,404`-batch discrete runs is partly a **corpus target coverage** problem or a **coordinate revisit** problem. Combining exact corpus/schedule arithmetic from `crates/uor-r4-discrete/src/{training,completion}.rs` with the sealed W2 (`#2845`) and A8 (`#2831`) measurements decomposes the question into two orthogonal axes:

### 1.1 Axis A — Corpus Target Coverage (`43.28%` at `16,404` Batches vs `100.0%` at `37,906` Batches)

| Quantity | 1-Pass Canonical Dose (`COMPLETION_BATCHES = 16,404`) | Full-Epoch Schedule (`37,906` Batches) | Ratio / Difference | Source |
| :--- | ---: | ---: | ---: | :--- |
| **Batches (`64` rows / batch)** | `16,404` | `37,906` (`37,905` full + `1` tail batch of `5` new + `59` wrapped targets) | `2.3108x` (`+21,502` batches) | `crates/uor-r4-discrete/src/completion.rs:14` |
| **Eligible `TRAIN` Target Positions Covered** | `1,049,856 / 2,425,925` (**`43.28%`**) | `2,425,925 / 2,425,925` (**`100.00%`**) | `+1,376,069` targets (`+56.72%`) | `docs/model/training.md:23` |
| **W2 Unigram `DEV top-1` (`n = 14,287`)** | **`1,226 / 14,287`** (`8.58%`) | **`1,226 / 14,287`** (`8.58%`) | `0` rows difference (identical token-1 argmax) | [`docs/labs/m1-w2-capacity-capture/README.md`](../m1-w2-capacity-capture/README.md) line 15 |
| **W2 Exact Read-Code Table Coverage on `DEV`** | **`14,175 / 14,287`** (**`99.22%`**; `41,806` codes with count $\ge 2$) | $\ge 14,175 / 14,287$ (`>= 99.22%`) | At most `112 / 14,287` (`0.78%`) unseen-code fallback rows remain at `16,404` batches | [`docs/labs/m1-w2-capacity-capture/README.md`](../m1-w2-capacity-capture/README.md) lines 32–33 |
| **W2 Exact Read-Code Table `DEV top-1` (`T_code`)** | **`3,414 / 14,287`** (`23.90%`; `T_restr = 1,138 / 1,920 = 59.27%` on admitted rows) | Measured in Step 2 (`T_code(full_epoch)`) | `T_code(16k) = 3,414` already beats the Q4 writer (`149`) by **`+3,265` rows** on the `43.28%` slice | [`docs/labs/m1-w2-capacity-capture/README.md`](../m1-w2-capacity-capture/README.md) lines 13–45 |

**Key implication from W2 (`#2845`):** Because the `1,049,856` `TRAIN` targets in the first `16,404` batches already cover `14,175 / 14,287` (`99.22%`) of `DEV` read codes and support `3,414 / 14,287` `DEV top-1` (`1,138 / 1,920` on admitted rows), the missing `56.72%` of `TRAIN` targets is **not** what kept the `K = 1` Q4 writer at `149 / 14,287`. Instead, the dominant bottleneck inside `16,404` batches is **per-coordinate revisit starvation** (Axis B below) together with admission erosion (addressed by D40 in `#2831`, which lifted `FrozenZero` at `Q1` from `149` to `871 / 14,287`).

---

### 1.2 Axis B — Coordinate Revisit Starvation (`1` Visit per `16,404` Batches Under `K = 1` vs `BVG-K` and `FrequentTokenRevisitV1`)

In `crates/uor-r4-discrete`, the vocabulary has `V = 4,096` tokens, each represented by `B = 4` 8D E8-orbit blocks (`16,384` vector-block coordinates total). Under flat round-robin (`CanonicalVectorThenReadV1` / `InterleavedReadEvery8V1`):

| Schedule & Law | Batches | Visits per Vector Block | Total `TRAIN` Rows Seen per Token (`4` Blocks) | Top-`256` Frequent-Token Share of Coordinate Updates |
| :--- | ---: | ---: | ---: | ---: |
| **Legacy `K = 1` Flat (`16,404` batches)** | `16,404` | `~1.0` visit | `4 x 64 = 256` rows (`0.0106%` of `TRAIN`) | `256 / 4,096 = 6.25%` (uniform with rare tail tokens) |
| **Stage T `BVG-K` (`K = 16`) Flat (`C_one_pass`, `16,404` batches)** | `16,404` | `~16.0` visits | `64 x 64 = 4,096` rows (`0.169%` of `TRAIN`) | `6.25%` (uniform across all `4,096` tokens) |
| **Stage T `BVG-K` (`K = 16`) Frequent-Token Revisit (`H_freq_16k`, `16,404` batches)** | `16,404` | **Top-`256`: `~128` visits**; Ranks `257..1,024`: `~25.6` visits; Ranks `1,025..4,096`: `~4.3` visits | **Top-`256`: `32,768` rows** (`8.0x` flat); Ranks `257..1,024`: `6,554` rows; Tail: `1,092` rows | **`50.0%`** on top-`256` tokens, **`30.0%`** on `257..1,024`, **`20.0%`** on `1,025..4,096` |
| **Stage T `BVG-K` (`K = 16`) Full-Epoch Flat (`H_full_epoch_flat`, `37,906` batches)** | `37,906` | `~37.0` visits | `148 x 64 = 9,472` rows (`0.390%` of `TRAIN`) | `6.25%` (uniform across all `4,096` tokens) |
| **Stage T `BVG-K` (`K = 16`) Full-Epoch + Frequent-Token (`H_full_epoch_freq`, `37,906` batches)** | `37,906` | **Top-`256`: `~296` visits**; Ranks `257..1,024`: `~59.2` visits; Ranks `1,025..4,096`: `~9.9` visits | **Top-`256`: `75,776` rows**; Ranks `257..1,024`: `15,155` rows; Tail: `2,526` rows | **`50.0%`** on top-`256` tokens, **`30.0%`** on `257..1,024`, **`20.0%`** on `1,025..4,096` |

---

## 2. Why Execution Is Held on `#2836` (`m1-bvg-k-stage-t-training`)

1. **`K = 1` Cadence-Granularity Stop (`#2831`):** In [`docs/labs/m1-a8-full-dose-rerun/README.md`](../m1-a8-full-dose-rerun/README.md), running under `K = 1` single-coordinate proposals with active D40 64-batch whole-interval rollback (`floor = 1,187 / 3,847`) caused `Relation` to stop at batch `1,600` and `FrozenZero` to stop at batch `5,248` (`STOP_FOR_DIAGNOSIS: consecutive_rejections`) before reaching `16,404` batches, let alone `37,906` full-epoch batches.
2. **Stage T `BVG-K` (`K = 16`, `#2836`) Is Step 4 on `PLAN.md`:** Step 4 of [`PLAN.md`](../../../PLAN.md) (`m1-bvg-k-stage-t-training`, `#2836`) runs the `BVG-K` (`K = 16`) batched-coordinate Stage T learning law on the M1 critical path. Once `#2836` establishes the `C_one_pass` (`16,404`-batch `BVG-K` `K = 16`) baseline on `seed = 1, 2`, `r-full-epoch-revisit-schedules` (`#2878`) can reuse `C_one_pass` directly as its `16,404`-batch checkpoint and run only the continuation to `37,906` batches (`H_full_epoch_flat`) plus the `H_freq_16k -> H_full_epoch_freq` trajectory.
3. **Docs-Only Current Scope:** `docs/roadmap/items/r-full-epoch-revisit-schedules.md` line 10 restricts `scope:` to `[docs/harness/prep/r-full-epoch-revisit-schedules*, docs/labs/r-full-epoch-revisit-schedules/**, docs/loop/tasks/r-full-epoch-revisit-schedules/**]`.
