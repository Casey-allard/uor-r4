# M1 position mechanism: prefix-holonomy transport on the discrete 64-frame / 16-root orbit

- **Date:** 2026-10-22
- **Lab:** `antigravity`
- **Issue:** `#2843` (`m1-position-mechanism`)
- **Decision card:** `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337522273`)
- **Pre-registration:** [`docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`](../../harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md) (`TEST FITNESS: FIT`)
- **Code:** [`crates/uor-r4-discrete/src/position_mechanism.rs`](../../../crates/uor-r4-discrete/src/position_mechanism.rs), [`crates/uor-r4-discrete/src/order_slice.rs`](../../../crates/uor-r4-discrete/src/order_slice.rs)

## 1. Summary

In `crates/uor-r4-discrete`, `Flock` and `Relation` transport a candidate at position $s < t$ into the query frame at $t = L - 1$ using `g[BFS + frame[s]*16 + fq]`, which depends only on the two endpoint roots (`frame[s]` and `fq`) and is invariant under any permutation of the intervening tokens $(s, t)$.

This record delivers:
1. **Geometric construction (`PositionHolonomyV1`):** Causal prefix scan $F_{p,b} = \mathrm{COMPOSE}[u_{p,b} \cdot 64 + F_{p-1,b}]$ in the 64-element signed-permutation group $G_{64} \subset \mathrm{Cl}(0,6)$, where each `(block, root)` slot selects one of 16 legal move candidates (`0` $\mapsto$ `IDENTITY`, `1..=15` $\mapsto$ `GENERATORS[c - 1]`). Relative path holonomy from $s$ to $t$ is computed in $O(1)$ table lookups via $H_{t \leftarrow s, b} = F_{t,b} \circ F_{s,b}^{-1} = \mathrm{COMPOSE}[F_{t,b} \cdot 64 + \mathrm{INVERSE}[F_{s,b}]]$.
2. **Matched controls:**
   - `PositionMode::EndpointControlV1`: uses the identical $16 \times B$ parameter table and move space, but forms $\mathrm{COMPOSE}[u_{t,b} \cdot 64 + \mathrm{INVERSE}[u_{s,b}]]$ from the endpoints $(s, t)$ alone, matching parameter count ($64$ slots at $B = 4$) and proposal budget while removing intervening-path order.
   - `PositionMode::IdentityControlV1`: identity transport (`IDENTITY` at every step).
   - `shuffle_prefix_roots`: deterministic splitmix64 Fisher-Yates shuffle of prefix positions $0..L-1$ preserving the query token at $L - 1$ and the prefix token multiset.
3. **Five finite CPU controls in `crates/uor-r4-discrete/src/position_mechanism.rs`:**
   - `identity_table_leaves_all_frames_and_roots_unchanged`
   - `prefix_holonomy_matches_direct_composition_and_transitivity`
   - `states_and_coordinates_stay_within_exact_integer_bounds`
   - `noncommutative_holonomy_detects_order_swaps_while_endpoint_control_is_blind`
   - `table_validation_and_parameter_matched_serialization`

## 2. Verification

- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib position_mechanism::tests`: `5 passed; 0 failed`.
- `~/.cargo/bin/cargo test -p uor-r4-discrete --lib`: `203 passed; 0 failed; 1 ignored` in an unsandboxed host worktree (from `198 passed; 0 failed; 1 ignored` at base); in a sandboxed review worktree where `ps -p` is restricted, the pre-existing test `controller::tests::explicit_unlock_refuses_live_or_foreign_holders_and_preserves_dead_lock` fails both at base and at head (`202 passed; 1 failed; 1 ignored` vs `197 passed; 1 failed; 1 ignored` at base).
- `uor-harness ratchet d23`, `d25`, and `d32`: all green (0 growth).

## 3. Non-author FIT review (deepseek-36, 2026-10-22)

Non-author `TEST FITNESS` review of the draft sheet: **NOT_FIT**, recorded in the sheet's `## Verdict`. No compute is cleared. The sheet was later updated (`deepseek-39`, 2026-10-23) after the owner's **2026-10-23T02:36:36Z** standing-direction registration (`#2028` comment `6356034555`, card `6337522273`): the bar is now quoted as registered, §5 carries the full DEV code-pin pair, and the order-sensitive slice rule is defined with its hash `PENDING` for the frozen subset. The three original prerequisites:
1. owner-registered bar (the linked `#2028` comment is antigravity's decision card `6337522273`, not an owner answer);
2. the full DEV token/mask pin — the sheet must use the authoritative code pin in `attention_learning.rs` (`0919e90f953e…` / `a838bb1a67e9…`; the staging sheet's `0919e90f813b…` / `a838bb1ae20c…` pair appears nowhere else and needs an errata);
3. a deterministic definition and SHA-256 for the new multi-occurrence order-sensitive slice (`N ≥ 200`).

Verified at the review head: `cargo test -p uor-r4-discrete --lib position_mechanism` = 5 passed; no float, dense matmul or squashing form on the path; the step table exports as `16 × B` `u8` words; the parent A8 sheet is `TEST FITNESS: FIT`; all 8 blocks present.

## 4. Frozen order-sensitive slice (claude-58, 2026-10-23)

The position sheet's §5 rule for the multi-occurrence order-sensitive DEV slice is now frozen.

- **File:** [`order-slice-v1.tsv`](order-slice-v1.tsv)
  - SHA-256 `f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a`, 17,789 bytes.
  - A `row_ordinal<TAB>target_position` header, then one line per selected row, in ascending row order.
- **Count:** **N = 1,620** of the 14,287 DEV rows. The sheet needs `N ≥ 200`.
- **Rule:** [`order_slice.rs`](../../../crates/uor-r4-discrete/src/order_slice.rs), policy `query-occurs-twice-in-prefix/pairwise-distinct-successors/C192-window/ascending-rows/v1`.
  - A row's window is the one `PreparedCorpus::batch` builds at `context = 192`: it stays inside the document, and its last token is the query.
  - The row is selected when the query token occurs at least twice earlier in the window and the tokens following those occurrences are pairwise distinct.
  - The occurrence just before the query has the query itself as its successor, as in `is_copy_successor`.
  - The row ordinal is the index in the eligible inventory, the same ordinal the DEV evaluation's batches use.
- **Looser reading, a diagnostic only:** requiring at least two distinct successors, with repeats allowed, would select 3,277 rows. It is not the frozen slice.
- **Input:** `data/ft-dev.tar` from the HF store (`caseyallard/uor-r4-store`, sha256 `45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c`, 143,360 bytes).
  - `ft/dev/tokens.u16` is UORT sha256 `0919e90f…`, and `ft/dev/response_mask.u8` is `a838bb1a…`.
  - The binary loads only under these registered digests and the pinned inventory: 42,856 payload tokens, 14,287 eligible targets, vocabulary 4096, BOS id 0.

To reproduce:

```sh
hf download caseyallard/uor-r4-store data/ft-dev.tar --repo-type dataset --local-dir <dir>
tar -xf <dir>/data/ft-dev.tar -C <dir>/x
cargo run --release --locked -p uor-r4-discrete --bin order-slice-freeze -- \
  <dir>/x/ft/dev/tokens.u16 <dir>/x/ft/dev/response_mask.u8 <out>.tsv
```

Two runs gave byte-identical files, both printing `selected 1620` and the SHA-256 above.

**Tests:** `cargo test -p uor-r4-discrete --lib order_slice` gives 6 passed. They cover:
- two occurrences with distinct successors are selected;
- a repeated successor, a single occurrence, an absent occurrence and an empty window are each rejected;
- the occurrence just before the query counts;
- three occurrences need every successor distinct;
- an end-to-end corpus fixture, through `PreparedCorpus::from_bytes` and `select`.

**Negative gate:** dropping the distinctness check fails 3 of the 6 tests.

This freeze clears no compute. The sheet still needs Kimi's non-author FIT, since Claude has now authored sheet text, plus the assembled trunk, PM dispatch and Casey's release.

## 5. GPU finite qualification, `from_snapshot` CUDA status-10 refusal, and byte-packed stack / throughput measurement (antigravity-62, 2026-10-25)

Resolves the post-`#3111` audit requirements in `#2843` comment `6391161531` and `PLAN.md` line 68 (`ORDER.md`: `docs/loop/tasks/m1-position-mechanism/antigravity-62/ORDER.md`, PR `#3205`, commit `77d7cb5681fe8c0f94d370316dc009505f5f1460`, D67 pre-run review `#2843` comment at `06b8f5843b428df54b834ddc56a2a53909f3cea0`).

### 5.1 Environment & Artifact Provenance

- **Pod / GPU:** `vast-57638243` (`1 x NVIDIA GeForce RTX 5090`, 32,607 MiB, driver `580.178.04`, compute capability `12.0`, `nvcc` 12.8 V12.8.93, `UOR_NVCC_ARCH=compute_120`).
- **HF Store Archive:** `caseyallard/uor-r4-store:results/antigravity/m1-position-mechanism-2026-10-25` (40 files; `SHA256SUMS` digest `dc61c5edf29444996cd6a659ce476e00459d834bcea87ae8bb1f6ab37d8876c5`; tarball `2f49a3474182035dcc03fc6bfd288bfc68825d27a49f7dc71fb6a075726f8899`).
- **Committed JSON Receipts:**
  - [`position-qualify-build1.json`](position-qualify-build1.json) (`SHA-256: 0de22708f4b3c1e961f2e5469aa9f088e219fb16fd535e943988a1cf909106f8`)
  - [`position-qualify-build2.json`](position-qualify-build2.json) (`SHA-256: 9d1072d3b7199c26b1ae3eb19f96c5ff69b107ce20121b365454625b61b7c4fe`)
  - [`position-qualify-dc5f409b0.json`](position-qualify-dc5f409b0.json) (`SHA-256: 679512ff64b6f7ec0b762aefbf79947aca015a02d5991b35bf5ebca404c72281`)
- **Two-Build Determinism (`77d7cb568` Build 1 vs Build 2):**
  - `driver_source_sha256`: `31789abea8b955cb0e137257fc9e8b3e5eb42865fec7728396fa377c9d4228ea` (identical)
  - `geometry_sha256`: `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` (identical)
  - `kernel_sha256` (`discrete.ptx`): `be2b90a8ec7704be94db115c4627fef71467659c9623ff00503ea4fa56637699` (identical)
  - `ordinary_sha256` (`ordinary.cubin`): `a150695dffeb4b5602fec112af539d9c17f60d6b09c2a6af6a5f7725c8bc8512` (identical)
  - `vectors_sha256` across all 8 `(read_mode, position_mode)` arms: byte-for-byte identical between Build 1, Build 2, and `dc5f409b0`.

### 5.2 Finite GPU Controls (`position_device_tests` & `position-qualify`)

All finite GPU controls passed on the RTX 5090 (`cargo test --release --locked -p uor-r4-discrete --features cuda --lib position_device_tests -- --include-ignored`: `3 passed; 0 failed` in `0.33s`; `unit_snap_device_tests`: `3 passed; 0 failed` in `0.22s`; `position_mechanism::tests`: `10 passed; 0 failed`; `position_device_controls`: `3 passed; 0 failed`):

1. **Device-vs-host exact match across modes `1..=3` in `Flock` and `Relation`:** Across 2 rows $\times$ 2 blocks $\times$ 3 leaves $\times$ 3 modes (`PrefixHolonomyV1` `s[14]=1`, `EndpointControlV1` `s[14]=2`, `IdentityControlV1` `s[14]=3`) $\times$ 2 read specs (`Flock { kept: 3 }`, `Relation { kept: 3, sweeps: 1 }`) = **72 leaf traces**, every GPU traced action `lv0[12 + k]` and carried root `lv0[20 + k]` matched the host finite algebraic reference (`relative_holonomy`, `apply_frame_to_root`) bit-for-bit, and `IdentityControlV1` with a non-trivial step table matched `PrefixHolonomyV1` with the all-zero identity step table in both output vectors and flock traces (`identity_control_matches_identity_table = true`).
2. **Interior-order sensitivity on the GPU:** On a 4-token window `[0, r1, r2, 3]` vs `[0, r2, r1, 3]` with all 3 prefix leaves kept (`Flock { kept: 3 }`, `UnnormalizedControlV1`), `EndpointControlV1` produced identical GPU output vectors (`endpoint_control_interior_swap_invariant = true`) while `PrefixHolonomyV1` produced distinct GPU output vectors (`prefix_holonomy_interior_swap_sensitive = true`).
3. **Snapshot roundtrip and device-side `status 10` refusal:** `Device::snapshot` $\to$ `Device::from_snapshot` roundtripped all 3 position modes with identical GPU vectors and traces (`snapshot_roundtrip_modes_verified = 3`), refused `ReadSpec::Route` with a position table (`route_position_refused = true`), and delegated `position_steps` candidate range validation (`< 16`) to `discrete_validate` on the GPU so a deserialized `Snapshot` with step candidate `16` was refused by CUDA kernel validation with `"discrete CUDA snapshot validation refused with status 10"`.

### 5.3 Per-Thread `ptxas` Stack Compaction (`5844d911c` vs `dc5f409b0` vs `77d7cb568`)

Compacting `prefix[243]`, `transport_root[364]`, and `transport[364]` in `flock_vector` (`crates/uor-r4-discrete/src/discrete.cu`) from 4-byte `U` (`3,884` B raw) to 1-byte `unsigned char` (`971` B raw, since frame indices are in `0..64` and root indices are in `0..16`) reduced compiled `sm_120` per-thread stack frames by **2,896 bytes (-11.4% to -11.6%)** across all three main kernels, with **0 register spill stores and 0 spill loads**:

| Kernel entry (`sm_120`) | Pre-`#3111` (`5844d911c`) | `#3111` 32-bit `U` (`dc5f409b0`) | Compacted `unsigned char` (`77d7cb568`) | Delta vs `#3111` (`dc5f409b0`) | Delta vs Pre-`#3111` (`5844d911c`) | Spill stores / loads |
|---|---:|---:|---:|---:|---:|---:|
| `discrete_forward` | 21,056 B (80 regs) | 24,976 B (80 regs) | **22,080 B** (87 regs) | **-2,896 B (-11.60%)** | +1,024 B (+4.86%) | 0 B / 0 B |
| `discrete_counterfactual` | 21,536 B (88 regs) | 25,456 B (96 regs) | **22,560 B** (96 regs) | **-2,896 B (-11.38%)** | +1,024 B (+4.75%) | 0 B / 0 B |
| `discrete_evaluate` | 21,184 B (80 regs) | 25,104 B (93 regs) | **22,208 B** (93 regs) | **-2,896 B (-11.54%)** | +1,024 B (+4.83%) | 0 B / 0 B |

### 5.4 RTX 5090 Throughput (`tokens/s`) at Full M1 Shape (`V=4096, B=4, C=192, K=8`, 64 rows $\times$ 10 iterations = 122,880 tokens/arm)

| Read mode | Position mode (`s[14]`) | `#3111` (`dc5f409b0`) forward `tok/s` | Compacted (`77d7cb568` B1 / B2) forward `tok/s` | `#3111` (`dc5f409b0`) evaluate `tok/s` | Compacted (`77d7cb568` B1 / B2) evaluate `tok/s` | Output `vectors_sha256` (all builds) |
|---|---|---:|---:|---:|---:|---|
| `Flock (K=8)` | `legacy_bfs (0)` | 325,322 | **327,988 / 327,775** | 285,756 | **286,998 / 287,023** | `9dd7cdfe4aa180a4…` |
| `Flock (K=8)` | `prefix_holonomy_v1 (1)` | 302,954 | **305,795 / 305,829** | 268,234 | **269,974 / 269,999** | `a267ccf13fc55e95…` |
| `Flock (K=8)` | `endpoint_control_v1 (2)` | 309,736 | **312,159 / 312,080** | 273,494 | **274,899 / 274,925** | `9dd7cdfe4aa180a4…` |
| `Flock (K=8)` | `identity_control_v1 (3)` | 312,850 | **315,313 / 315,203** | 275,868 | **277,328 / 277,317** | `9dd7cdfe4aa180a4…` |
| `Relation (K=8, S=1)` | `legacy_bfs (0)` | 318,331 | **321,588 / 321,438** | 281,054 | **282,417 / 282,411** | `9dd7cdfe4aa180a4…` |
| `Relation (K=8, S=1)` | `prefix_holonomy_v1 (1)` | 291,705 | **295,120 / 295,079** | 259,538 | **261,672 / 261,683** | `a267ccf13fc55e95…` |
| `Relation (K=8, S=1)` | `endpoint_control_v1 (2)` | 297,933 | **301,073 / 300,930** | 264,455 | **266,329 / 266,321** | `9dd7cdfe4aa180a4…` |
| `Relation (K=8, S=1)` | `identity_control_v1 (3)` | 300,908 | **304,054 / 303,947** | 266,800 | **268,637 / 268,641** | `9dd7cdfe4aa180a4…` |

## 6. Paired order-slice scorer, `C_shuf` window helper, and post-`#3205` `FIT` synchronization (`antigravity-88`, 2026-10-25)

Resolves the remaining pre-registration and scorer prerequisites noted in `kimi-33/REPORT.md` (`#3111`, lines 16 and 64) and closes `#2843` (`needs_gpu: false`, PR `#3256`, [`DECISION.md`](../../loop/tasks/m1-position-mechanism/antigravity-88/DECISION.md)):

1. **Frozen slice verification, `C_shuf` window helper & paired McNemar scorer (`crates/uor-r4-discrete/src/order_slice.rs`):**
   - `parse_frozen_slice`: parses `docs/labs/m1-position-mechanism/order-slice-v1.tsv` and verifies its SHA-256 (`f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a`), row count (`1,620`), and strictly ascending `row_ordinal < 14,287` and `target_position` invariants.
   - `shuffle_prefix_window`: applies the identical splitmix64 Fisher-Yates permutation of `position_mechanism::shuffle_prefix_roots` to prefix `Identity` entries `0..L-1` while keeping the query token `window[L - 1]` and prefix token multiset fixed (`C_shuf`).
   - `integer_ceil_two_sqrt`, `score_position_contrast`, `evaluate_position_order_seed`, and `grade_position_order_two_seeds`: exact integer paired McNemar scorer (zero floating-point operations, `D32`) verifying both $b - c \ge F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$ with $\Delta\text{admitted} \ge 0$ (`H_pos(ordered)` vs `C_match(ordered)`) and $b - c \ge F_I$ (`H_pos(ordered)` vs `C_shuf`) across full DEV (`14,287`), `P_succ` (`3,847`), and the frozen order slice (`1,620`), plus the owner's `2026-10-24T03:41:41Z` fresh-V2 replication rule ($b - c \ge \lceil 2\sqrt{b + c}\rceil$, no 24-row floor, `#2028` comment `6379430093`) and 3-branch outcome decision table (`PassPromote`, `FailDevCloseConfig` keeping fresh V2 unopened and sealed, `FailTransferCloseConfig`).
   - **Unit controls:** `cargo test --locked -p uor-r4-discrete --lib order_slice` -> `10 passed; 0 failed`; `cargo test --locked -p uor-r4-discrete --lib position_mechanism` -> `10 passed; 0 failed`.
2. **Pre-registration sheet synchronization (`docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`):**
   - Pinned the exact `#3111`/`#3205` GPU `flock_vector` final block-$b$ transport definition (`last[node]` payload-block root anchor `tr = root_id(rv, g)`, immutable `query_payload`, internal tree aggregation on `g[BFS + frame[child]*16 + frame[c0]]`, and byte-packed `unsigned char` stack arrays), cited the `#3205` RTX 5090 qualification receipts (`0de22708…`, `9d1072d3…`), pinned the single A8 proxy-trunk configuration (`c695ad21a`, `DRIVER_SOURCE_SHA256 = 04c51654…`), and recorded the fresh-context non-author `TEST FITNESS: FIT` and D68 decision review.
3. **Training status (`CHARTER.md` §4.6):** The two-seed GPU order training run (`H_pos` vs `C_match` and `C_shuf`) remains `NOT_RUN` (mechanism construction, finite CPU/GPU controls, and pre-registration only).


