# `r-vocab-4096-vs-16384` — Matched 4,096 vs 16,384 Tokenizer Piece Comparison (`#2887`)

- **Issue:** `#2887` (`r-vocab-4096-vs-16384` — "Compare 4,096 and 16,384 tokenizer pieces")
- **Milestone:** `research` (`area: training`, `depends_on: [m1-faster-writer]`)
- **Pre-registration sheet:** [`docs/harness/prep/r-vocab-4096-vs-16384-2026-10-26.md`](../../harness/prep/r-vocab-4096-vs-16384-2026-10-26.md) (`TEST FITNESS: FIT`, `D68` decision card [`docs/loop/tasks/r-vocab-4096-vs-16384/antigravity-91/DECISION.md`](../../loop/tasks/r-vocab-4096-vs-16384/antigravity-91/DECISION.md))
- **Lab:** `antigravity` (`antigravity-91`, PR `#3276`)
- **Hardware (`B1` GPU runs):** `1x NVIDIA GeForce RTX 5090` (`vast-57796936`, driver `595.84`, CUDA `13.0` / `nvcc 12.8`, `sm_120`), sealed across two independent deterministic builds:
  - `Build A`: [`docs/labs/m1-faster-writer/qualification-build-a.json`](../m1-faster-writer/qualification-build-a.json) (`sha256:37419bdc303b79ac4beac0fab0ea292d74dc34ac9623754457e9107d89a40fd0`)
  - `Build B`: [`docs/labs/m1-faster-writer/qualification-build-b.json`](../m1-faster-writer/qualification-build-b.json) (`sha256:df583e45e1ca857cb698faf23c5a266fdc6abd3073be5f897662e432c82262bb`)

---

## 1. Headline Result (`tokens/s` First, `CHARTER.md` §1)

| Comparison Axis | `V = 4,096` (`4,096` pieces) | `V = 16,384` (`16,384` pieces, `4x` vocab) | `V = 16,384 / V = 4,096` Ratio | Registered Bar (`B1`–`B3`) | Verdict |
|---|---|---|---|---|---|
| **GPU Writer `tokens/s` (`K = 8`, `FullScan`)** | **`855` / `853 tok/s`** (`39,268` evals/row) | **`219` / `218 tok/s`** (`150,243` evals/row) | `0.256x` throughput (`3.90x` slower; `3.83x` more evals/row) | Baseline full-vocabulary scan (`O(V · B)`) | Confirms `#2887` full-scan bottleneck |
| **GPU Writer `tokens/s` (`K = 8`, `Bucketed IdOrder`)** | **`47,904` / `47,151 tok/s`** (`56.0x` vs full scan; **`382`** evals/row, **`102.74x`** reduction) | **`22,977` / `22,884 tok/s`** (**`104.9x`** vs full scan; **`765`** evals/row, **`196.15x`** reduction) | `0.480x` throughput (**`2.00x`** evals/row instead of `3.83x`) | `B1`: `0` mismatches, `>= 10x` reduction at both `V = 4,096` and `V = 16,384` | **PASS** (`102.74x` at `4,096`; `196.15x` at `16,384`) |
| **GPU Writer `tokens/s` (`K = 8`, `Bucketed ServingPruned`)** | **`51,282` / `51,336 tok/s`** (`60.0x` vs full scan; **`365`** evals/row, **`107.53x`** reduction) | **`26,936` / `26,845 tok/s`** (**`123.0x`** vs full scan; **`592`** evals/row, **`253.50x`** reduction) | `0.525x` throughput (**`1.62x`** evals/row instead of `3.83x`) | `B1`: `0` mismatches, `>= 10x` reduction at both `V = 4,096` and `V = 16,384` | **PASS** (`107.53x` at `4,096`; `253.50x` at `16,384`) |
| **GPU Writer `tokens/s` (`K = 1`, `Bucketed ServingPruned`)** | **`118,081` / `118,081 tok/s`** (`40.0x` vs `2,951`; **`271`** evals/row, **`32.08x`** reduction) | **`86,486` / `85,637 tok/s`** (`114.4x` vs `756`; **`318`** evals/row, **`104.54x`** reduction) | `0.732x` throughput (**`1.17x`** evals/row: `256` cell distances fixed) | `B1`: `0` mismatches, `>= 10x` reduction | **PASS** (`32.08x` at `4,096`; `104.54x` at `16,384`) |
| **GPU Writer `tokens/s` (`K = 16`, `Bucketed ServingPruned`)** | **`31,840` / `31,714 tok/s`** (`66.5x` vs `479`; **`443`** evals/row, **`167.06x`** reduction) | **`16,689` / `16,655 tok/s`** (**`136.8x`** vs `122`; **`666`** evals/row, **`425.53x`** reduction) | `0.524x` throughput (**`1.50x`** evals/row under `ServingPruned`) | `B1`: `0` mismatches, `>= 10x` reduction | **PASS** (`167.06x` at `4,096`; `425.53x` at `16,384`) |
| **Uniform Occupancy (`K = 8`, `16` vs `64` tokens/cell)** | `39,270` $\to$ **`384`** evals/row (`0 + 256 + 128`, **`102.26x`** reduction, `0` mismatches) | `150,246` $\to$ **`768`** evals/row (`0 + 256 + 512`, **`195.63x`** reduction, `0` mismatches) | Exact **`2.00x`** per-row evaluations (`768 / 384`) | `B1`: `0` mismatches, `256 + K · (V / 256)` law | **PASS** (`0` mismatches on both builds) |
| **BPE Digit-Piece Census (`ctrl1` `Digits(true)+ByteLevel`)** | `10` single-digit, **`0`** multi-digit, `0` space-prefixed (`blake3:ed3f625d…`) | `10` single-digit, **`0`** multi-digit, `0` space-prefixed (`blake3:311a1e7c…`) | `0` multi-digit pieces at both `4,096` and `16,384` | `B2`: Tests whether `4x` vocab fixes digit splitting | **Negative on `V` alone** (`Digits(true)` blocks multi-digit merges at any `V`) |
| **BPE Digit-Piece Census (`ctrl2` plain `ByteLevel`)** | `10` single-digit, `5` (`5/0/0`) multi-digit, **`110`** space-prefixed (`blake3:81522e79…`) | `10` single-digit, `5` (`5/0/0`) multi-digit, **`109`** space-prefixed (`blake3:ec4c2c66…`) | `5` pure multi-digit and `110` vs `109` space-prefixed | `B2`: Tests whether `4x` vocab fixes `Ġ`-digit absorption | **Negative on `V` alone** (`ByteLevel` absorbs leading space into digits at both `V`) |
| **BPE Digit-Piece Census (`trt` `Digits(false)+ByteLevel`)** | `10` single-digit, **`109`** multi-digit (`100/100` 2-digit), **`0`** space-prefixed (`blake3:c38dabf5…`) | `10` single-digit, **`109`** multi-digit (`100/100` 2-digit), **`0`** space-prefixed (`blake3:411c366d…`) | Identical `109` pure multi-digit pieces (`0` space-prefixed) at both `V` | `B2`: `pure_multi_digit >= 100`, `"00".."99"` present, `space_prefixed == 0` | **PASS at both `V = 4,096` and `V = 16,384`** |
| **Discrete `B = 4` `StructuredV1` Cell Capacity (`B3`)** | **`256` cells × `16` tied interior roots = `4,096`** (`16` tokens/cell, `0` forced aliases) | Exceeds `StructuredV1` (`quota = 64 > 16`) and `Spec::validate` (`8,192` cap) | `4.00x` cell quota (`64` vs `16` tokens/cell) | `B3`: Exact code-level capacity audit | **`classification: instrument`** (5 explicit `4,096`/`8,192` guards documented below) |

---

## 2. Bar `B1`: Matched GPU Writer & Serving Cost (`V = 4,096` vs `V = 16,384`)

Under the registered sheet [`docs/harness/prep/m1-faster-writer-2026-10-26.md`](../../harness/prep/m1-faster-writer-2026-10-26.md) (`#2840`), `writer-bucket-qualify` ran matched GPU evaluations at `V = 4,096` and `V = 16,384` (`B = 4`, `K ∈ {1, 8, 16}`, `3` seeds × `32` rows = `96` seeded rows per `(V, K)` / `288` seeded rows per `V`, plus `32` uniform-occupancy rows per `V` at `K = 8`) on `1x NVIDIA GeForce RTX 5090` (`vast-57796936`) across two independent `sm_120` builds (`Build A` `37419bdc…` and `Build B` `df583e45…`, `0` metric diffs).

### 2.1 Seeded Sweep Across `K ∈ {1, 8, 16}` (`V = 4,096` vs `V = 16,384`, `B = 4`, `96` rows per `(V, K)`)

| `V` | `K` | Mode | Build A `tok/s` (`rows/s`) | Build B `tok/s` (`rows/s`) | Speedup vs `FullScan` | `token_cell` / row | `cell_distance` / row | `token_distance` / row | Total Evals / row | Reduction vs `FullScan` | Mismatches |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `4,096` | `1` | `FullScan` | `2,951` | `2,943` | `1.00x` | `8,193` | `513` | `15` | `8,721` | `1.00x` | `0` |
| `4,096` | `1` | `Bucketed(IdOrder)` | **`115,523`** | **`111,627`** | **`39.1x` / `37.9x`** | **`0`** | `256` | `15` | **`271`** | **`32.08x`** | **`0`** |
| `4,096` | `1` | `Bucketed(ServingPruned)` | **`118,081`** | **`118,081`** | **`40.0x` / `40.1x`** | **`0`** | `256` | `15` | **`271`** | **`32.08x`** | **`0`** |
| `16,384` | `1` | `FullScan` | `756` | `754` | `1.00x` | `32,769` | `513` | `62` | `33,344` | `1.00x` | `0` |
| `16,384` | `1` | `Bucketed(IdOrder)` | **`84,063`** | **`83,478`** | **`111.2x` / `110.7x`** | **`0`** | `256` | `62` | **`318`** | **`104.54x`** | **`0`** |
| `16,384` | `1` | `Bucketed(ServingPruned)` | **`86,486`** | **`85,637`** | **`114.4x` / `113.6x`** | **`0`** | `256` | `62` | **`318`** | **`104.54x`** | **`0`** |
| `4,096` | `8` | `FullScan` | `855` | `853` | `1.00x` | `36,865` | `2,277` | `126` | `39,268` | `1.00x` | `0` |
| `4,096` | `8` | `Bucketed(IdOrder)` | **`47,904`** | **`47,151`** | **`56.0x` / `55.3x`** | **`0`** | `256` | `126` | **`382`** | **`102.74x`** | **`0`** |
| `4,096` | `8` | `Bucketed(ServingPruned)` | **`51,282`** | **`51,336`** | **`60.0x` / `60.2x`** | **`0`** | `256` | `109` | **`365`** | **`107.53x`** | **`0`** |
| `16,384` | `8` | `FullScan` | `219` | `218` | `1.00x` | `147,457` | `2,277` | `509` | `150,243` | `1.00x` | `0` |
| `16,384` | `8` | `Bucketed(IdOrder)` | **`22,977`** | **`22,884`** | **`104.9x` / `105.0x`** | **`0`** | `256` | `509` | **`765`** | **`196.15x`** | **`0`** |
| `16,384` | `8` | `Bucketed(ServingPruned)` | **`26,936`** | **`26,845`** | **`123.0x` / `123.1x`** | **`0`** | `256` | `336` | **`592`** | **`253.50x`** | **`0`** |
| `4,096` | `16` | `FullScan` | `479` | `477` | `1.00x` | `69,633` | `4,233` | `262` | `74,128` | `1.00x` | `0` |
| `4,096` | `16` | `Bucketed(IdOrder)` | **`28,941`** | **`28,751`** | **`60.4x` / `60.3x`** | **`0`** | `256` | `262` | **`518`** | **`143.10x`** | **`0`** |
| `4,096` | `16` | `Bucketed(ServingPruned)` | **`31,840`** | **`31,714`** | **`66.5x` / `66.5x`** | **`0`** | `256` | `187` | **`443`** | **`167.06x`** | **`0`** |
| `16,384` | `16` | `FullScan` | `122` | `122` | `1.00x` | `278,529` | `4,233` | `1,026` | `283,788` | `1.00x` | `0` |
| `16,384` | `16` | `Bucketed(IdOrder)` | **`13,038`** | **`12,979`** | **`106.9x` / `106.4x`** | **`0`** | `256` | `1,026` | **`1,282`** | **`221.28x`** | **`0`** |
| `16,384` | `16` | `Bucketed(ServingPruned)` | **`16,689`** | **`16,655`** | **`136.8x` / `136.5x`** | **`0`** | `256` | `410` | **`666`** | **`425.53x`** | **`0`** |

### 2.2 Uniform-Occupancy Scaling Law (`V = 4,096` `16` tokens/cell vs `V = 16,384` `64` tokens/cell, `K = 8`)

| Edge Case | `V` | Tokens / Cell | `FullScan` Evals / row | `Bucketed(IdOrder)` Evals / row | Decomposition (`token_cell + cell_distance + token_distance`) | Reduction vs `FullScan` | `V = 16,384 / V = 4,096` Work Ratio | Mismatches |
|---|---:|---:|---:|---:|---|---:|---:|---:|
| `uniform_v4096_16_per_cell` | `4,096` | `16` | `39,270` | **`384`** | `0 + 256 + (8 × 16 = 128)` | **`102.26x`** | `1.00x` | **`0`** |
| `uniform_v16384_64_per_cell` | `16,384` | `64` | `150,246` | **`768`** | `0 + 256 + (8 × 64 = 512)` | **`195.63x`** | **`2.00x`** (vs `3.83x` full scan) | **`0`** |

**What `B1` proves:**
1. Under the legacy full-scan writer, quadrupling the vocabulary from `4,096` to `16,384` multiplied per-row primitive evaluations by **`3.83x`** (`39,270 -> 150,246` at `K = 8`) and cut GPU writer throughput by **`3.90x`** (`855 -> 219 tok/s`).
2. Under the CSR cell-bucketed writer (`#2840`), `token_cell` drops to **`0`** and the `256` cell distances are independent of $V$. Consequently, quadrupling $V$ from `4,096` to `16,384` increases total primitive evaluations by only **`1.17x`** at `K = 1` (`271 -> 318`), **`2.00x`** at `K = 8` `IdOrder` (`384 -> 768` uniform, `382 -> 765` seeded), **`1.62x`** at `K = 8` `ServingPruned` (`365 -> 592`), and **`1.50x`** at `K = 16` `ServingPruned` (`443 -> 666`), while delivering **`26,936 tok/s`** at `V = 16,384, K = 8` (**`123.0x` faster** than `FullScan`'s `219 tok/s` at `V = 16,384` and **`31.5x` faster** than `FullScan`'s `855 tok/s` at `V = 4,096`).

---

## 3. Bar `B2`: Matched BPE Tokenizer & Numeric-Boundary Comparison (`V = 4,096` vs `V = 16,384`)

Re-verified via `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-tokenizer --no-fail-fast` (`23 / 23` tests pass, including `reaches_16384_on_bounded_synthetic_text_and_preserves_runtime_roundtrip` and `number_aware_three_arm_comparison_and_digit_piece_census`) on the deterministic `170,176`-byte benchmark corpus (`sha256:05f21705704413d77e95cba3eb457682b96be12af460e28d33e54705605c91bd`):

| Arm | Pre-Tokenizer Policy | `V` | Runtime Identity (`blake3`) | File `sha256` | `pure_single_digit` | `pure_multi_digit` (2d / 3d / 4d+) | `"00".."99"` Coverage | `space_prefixed_digit` (`Ġ`+digits) | `" 2026"` Encoding | `"2026"` Encoding |
|---|---|---:|---|---|---:|---:|---:|---:|---|---|
| `pinned` | `Digits(true)+ByteLevel` (external ref) | `4,096` | `d36d3e87…` (sha256) | `d36d3e8700a123e6…` | `10` | **`0`** (`0 / 0 / 0`) | `0 / 100` | `0` | `[Ġ, 2, 0, 2, 6]` | `[2, 0, 2, 6]` |
| `ctrl1` | `Digits(true)+ByteLevel` | `4,096` | `blake3:ed3f625d4fbbfbc2…` | `9fc81d936a39b5a9…` | `10` | **`0`** (`0 / 0 / 0`) | `0 / 100` | `0` | `[Ġ, 2, 0, 2, 6]` | `[2, 0, 2, 6]` |
| `ctrl1` | `Digits(true)+ByteLevel` | `16,384` | `blake3:311a1e7c582be16e…` | `c6b1fef8767e5a79…` | `10` | **`0`** (`0 / 0 / 0`) | `0 / 100` | `0` | `[Ġ, 2, 0, 2, 6]` | `[2, 0, 2, 6]` |
| `ctrl2` | Plain `ByteLevel` | `4,096` | `blake3:81522e79624f40f8…` | `a922cfe42c47092c…` | `10` | `5` (`5 / 0 / 0`) | `5 / 100` | **`110`** | `[Ġ20, 26]` | `[20, 26]` |
| `ctrl2` | Plain `ByteLevel` | `16,384` | `blake3:ec4c2c66f6bd2a58…` | `e704f2dcbb14e499…` | `10` | `5` (`5 / 0 / 0`) | `5 / 100` | **`109`** | `[Ġ20, 26]` | `[20, 26]` |
| **`trt`** | **`Digits(false)+ByteLevel`** | **`4,096`** | `blake3:c38dabf516cc142e…` | `84dee8b03e5951e9…` | **`10`** | **`109` (`100 / 6 / 3`)** | **`100 / 100`** | **`0`** | **`[Ġ, 2026]`** | **`[2026]`** |
| **`trt`** | **`Digits(false)+ByteLevel`** | **`16,384`** | `blake3:411c366d7e307a87…` | `46dbe12715660f1b…` | **`10`** | **`109` (`100 / 6 / 3`)** | **`100 / 100`** | **`0`** | **`[Ġ, 2026]`** | **`[2026]`** |

**What `B2` proves:**
1. **Vocabulary size alone (`4,096` $\to$ `16,384`) has zero effect on digit splitting under `Digits(true)` (`ctrl1`):** at both `V = 4,096` and `V = 16,384`, `pure_multi_digit = 0` because `digits_individual = true` isolates every ASCII digit before BPE merges run.
2. **Vocabulary size alone (`4,096` $\to$ `16,384`) does not fix space-prefixed digit absorption under plain `ByteLevel` (`ctrl2`):** at both `V = 4,096` and `V = 16,384`, `pure_multi_digit = 5` (`5 / 0 / 0`) and `space_prefixed_digit = 110` / `109`.
3. **Switching pre-tokenization to `Digits(false)+ByteLevel` (`trt`) achieves identical numeric piece coverage at `V = 4,096` and `V = 16,384`:** both `V = 4,096` (`blake3:c38dabf5…`) and `V = 16,384` (`blake3:411c366d…`) form all `100` two-digit strings `"00".."99"`, `6` three-digit pieces, and `3` four+-digit pieces (`109` pure multi-digit pieces total) with `0` space-prefixed digit tokens, allocating the remaining `12,288` merges at `V = 16,384` to general lexical words and phrases rather than numerals.

---

## 4. Bar `B3`: Discrete `B = 4` Substrate & Codebook Capacity Boundary (`V = 4,096` vs `V = 16,384`)

Auditing `crates/uor-r4-discrete` and `crates/uor-r4-integer` identifies five explicit code-level guards that bound `V` in the full discrete driver (`PreparedDriver` / `Device` / `D11` export):

| # | File & Line | Guard / Constant | Current Bound | Geometric / Architectural Reason | What `V = 16,384` Requires |
|---:|---|---|---|---|---|
| 1 | [`crates/uor-r4-discrete/src/address_cells.rs:322,462`](../../../crates/uor-r4-discrete/src/address_cells.rs#L322) | `quota = v / CELLS > 16` (`CELLS = 256`, `ROOTS = 16`) | **`V <= 4,096`** (`16` tokens/cell across `256` cells) | `address_cells::build` (line `322`) rejects `v / CELLS > 16`, and `StructuredV1` (line `462`) sets block `0` and block `B-1` from the cell index `(c / 16, c % 16)` while tying **all** interior blocks `1..B-1` to the same single orbit root `ROOTS[k]` (`k ∈ 0..16`). Thus each of the `256` cells has only `16` distinct interior states (`256 × 16 = 4,096` distinct vectors total). | At `B = 4` (`2` interior blocks: block `1` and block `2`), use `FullVectorV1`-style independent interior indexing `(k / 16, k % 16)` (as in `address_cells.rs:383-460`) and raise line `322`'s `v / CELLS <= 16` bound to `64`, giving `16 × 16 = 256` distinct interior pairs per cell (`256 × 64 = 16,384` injective tokens with `0` forced collisions). |
| 2 | [`crates/uor-r4-discrete/src/lib.rs:83`](../../../crates/uor-r4-discrete/src/lib.rs#L83) | `!(2..=8192).contains(&self.vocabulary)` | `V <= 8,192` | `Spec::validate()` hard-caps `vocabulary` at `8,192`. | Raise `Spec::validate()` upper bound from `8192` to `16384`. |
| 3 | [`crates/uor-r4-discrete/src/prime_zeta.rs:108`](../../../crates/uor-r4-discrete/src/prime_zeta.rs#L108) | `count > 8192` (`8192nd` prime = `84,017`) | `V <= 8,192` | `first_n_primes` caps the prime-address sieve at `8,192` primes. | Extend `first_n_primes` sieve limit to `16,384` primes (`16,384th` prime = `180,503`). |
| 4 | [`crates/uor-r4-discrete/src/discrete.cu:633`](../../../crates/uor-r4-discrete/src/discrete.cu#L633) & [`device.rs:1890`](../../../crates/uor-r4-discrete/src/device.rs#L1890) | `vocabulary > 8192` in `discrete_count_score` | `V <= 8,192` | GPU/host count-table reference buffers size `vocab <= 8192`. | Raise `discrete_count_score` / `count_reference` max vocabulary to `16,384`. |
| 5 | [`crates/uor-r4-discrete/src/d11_export.rs:149,386`](../../../crates/uor-r4-discrete/src/d11_export.rs#L149) & [`discrete_d11.rs:667`](../../../crates/uor-r4-integer/src/discrete_d11.rs#L667) | `!(2..=8192).contains(&spec.vocabulary)` | `V <= 8,192` | `D11` exporter (`validate_and_extract_cells` / `import_d11`) and `IntegerDiscreteModel::from_bytes` enforce `2..=8192`. | Raise `D11` format validator cap from `8192` to `16384`. |

---

## 5. Notebook Records (`CHARTER.md` §5.6)

1. **`docs/research/notebook/m1-discrete-attention.md` (row 79, merged in `5a104c6f8`, `#3272`):** Already records the matched `V = 4,096` vs `V = 16,384` GPU CSR cell-bucketed writer runs (`Build A` `37419bdc…` and `Build B` `df583e45…` on `1x RTX 5090` `vast-57796936`) against `B1` and `B2`.
2. **`docs/research/notebook/data-tokenizer.md` row (for the next render/notebook sync, as `r-vocab-4096-vs-16384.md` at `origin/main` omitted `notebook:` in its frontmatter so `loop guard` keeps `data-tokenizer.md` read-only in this worktree, matching `#3230` `claude-110` and `#3270` `kimi-38`):**

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-10-26 | Matched `V = 4,096` vs `V = 16,384` tokenizer piece comparison (`r-vocab-4096-vs-16384`, `#2887`, `antigravity-91`; sheet `r-vocab-4096-vs-16384-2026-10-26.md`): (1) matched GPU writer evaluations across `K in {1, 8, 16}` (`3` seeds x `32` rows = `96` rows per `(V, K)`) and uniform occupancy (`16` vs `64` tokens/cell at `K = 8`) on `1x RTX 5090` (`vast-57796936`, `Build A` `37419bdc…`, `Build B` `df583e45…`), (2) matched 3-arm BPE tokenizer & digit-piece census (`ctrl1` `Digits(true)+ByteLevel`, `ctrl2` plain `ByteLevel`, `trt` `Digits(false)+ByteLevel`, `2` runs/arm on `170,176` B corpus `05f21705…`), and (3) `B = 4` discrete codebook capacity audit (`B3`). | GPU writer (`K = 8` seeded): `V = 4,096` `FullScan` `855 tok/s` (`39,268` evals/row) $\to$ `IdOrder` **`47,904 tok/s`** (`382` evals/row, `102.74x` reduction) $\to$ `ServingPruned` **`51,282 tok/s`** (`365` evals/row, `107.53x` reduction) vs `V = 16,384` `FullScan` `219 tok/s` (`150,243` evals/row, `3.90x` slower) $\to$ `IdOrder` **`22,977 tok/s`** (`765` evals/row, `196.15x` reduction, `2.00x` `V=4096` work) $\to$ `ServingPruned` **`26,936 tok/s`** (`592` evals/row, `253.50x` reduction, `1.62x` `V=4096` work); uniform `K = 8` `384` vs `768` evals/row (`2.00x`, `0` mismatches). BPE digit census: `ctrl1` `0` multi-digit at both `4,096` (`blake3:ed3f625d…`) and `16,384` (`blake3:311a1e7c…`); `ctrl2` `5` (`5/0/0`) multi-digit and `110` vs `109` space-prefixed (`blake3:81522e79…`, `blake3:ec4c2c66…`); `trt` `Digits(false)+ByteLevel` **`109` multi-digit (`100/100` `"00".."99"`) and `0` space-prefixed at both `4,096` (`blake3:c38dabf5…`) and `16,384` (`blake3:411c366d…`)**. `B = 4` `StructuredV1` ties interior roots `1..B-1`, capping injective cell capacity at `256 x 16 = 4,096` (`address_cells.rs:322,462`) alongside `8,192` validator caps (`lib.rs:83`, `prime_zeta.rs:108`, `discrete.cu:633`, `d11_export.rs:149,386`, `discrete_d11.rs:667`) | `B1` (`0` GPU parity mismatches and `>= 10x` primitive evaluation reduction at both `V = 4,096` and `V = 16,384`), `B2` (`12/12` deterministic BPE runs, lossless round-trip, `>= 100` multi-digit pieces and `0` space-prefixed digits on `trt`), `B3` (exact `B = 4` capacity boundary audit) | positive (`B1`–`B3` met; CSR cell bucketing cuts the `4x` vocabulary cost scaling from `3.83x` to `2.00x` `IdOrder` / `1.62x` `ServingPruned` at `K = 8` (`1.17x` at `K = 1`); pre-tokenization `Digits(false)`, not vocabulary size, fixes number splitting; `V = 4,096` retained for M1–M3 until independent interior-block indexing `(k / 16, k % 16)` widens `StructuredV1` beyond `256 x 16 = 4,096`; closes `#2887`) | [r-vocab-4096-vs-16384](../../labs/r-vocab-4096-vs-16384/README.md) |
