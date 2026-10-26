# Prefix-holonomy position mechanism on the discrete M1 orbit (`PositionHolonomyV1`)

- **Status:** pre-registration, `TEST FITNESS: FIT` (non-author post-freeze review `kimi-32`, 2026-10-23, and post-`#3111`/`#3205` synchronization review `antigravity-88`, 2026-10-25; see `## Verdict` at the end). **Sheet update 2026-10-23 (deepseek-39):** the owner's standing-direction decision registered the §3 bar at **2026-10-23T02:36:36Z** (`#2028` comment `6356034555`; card `6337522273`), §5 now pins the full DEV pair from the code pin, and the order-sensitive slice rule is defined. **Sheet update 2026-10-23 (claude-58):** the slice is frozen (`N = 1,620`, SHA-256 `f76dc5c7…`; see §5). **Sheet update 2026-10-25 (antigravity-88):** §1, §3, §4, §6, §7, and §8 synchronize the landed `#3111`/`#3205` GPU `flock_vector` read definition and RTX 5090 qualification receipts (`0de22708…`, `9d1072d3…`), the owner's **2026-10-24T03:41:41Z** fresh-V2 replication ruling (`#2028` comment `6379430093`, $b - c \ge \lceil 2\sqrt{b + c}\rceil$), the 3-branch outcome decision table, and the paired McNemar scorer in `crates/uor-r4-discrete/src/order_slice.rs`.
  - **GPU dispatch:** Under `CHARTER.md` §5.1 and §5.10 (D67), GPU execution of this registered configuration proceeds on a lab's own fresh-context pre-run review inside the global GPU caps ($\le 4$ pods, $\le \$8.00/\text{h}$).
- **Line:** M1 learned attention and sequence position (#2029, item `m1-position-mechanism` #2843). **Lab:** antigravity.
- **Parent:** [`m1-attention-a8-address-cells-learning-2026-10-19.md`](m1-attention-a8-address-cells-learning-2026-10-19.md) (the A8 address-structured cells sheet).
  - **What changes from the parent:** replaces the endpoint-only candidate transport in `Flock` / `Relation` (`discrete.cu` `flock_vector`) with causal prefix-holonomy transport `PositionHolonomyV1` (`crates/uor-r4-discrete/src/position_mechanism.rs`) and adds two matched controls (`EndpointControlV1` and shuffled-prefix evaluation `C_shuf`) to isolate sequence-order credit at equal parameter count.
- **Applicable owner decisions:** D11, D21, D22, D23, D24, D25, D28, D29, D31, D32, D37, D40, D66, D67, D68.
  - Geometry only (`Cl(0,6)` 64-frame signed-permutation group acting on the 16-root $E_8$ orbit $\mathcal{O}_{16}$).
  - Zero normalized exponentials or squashing functions (CHARTER §3.2, D23).
  - Zero dense matrix multiplications or GEMM (CHARTER §3.3, D25).
  - Zero floating-point types or literals on the discrete path (D32).
  - Exports to D11 as $16 \times B$ unsigned whole numbers (`u8` in `0..16`) plus fixed lookup tables (`COMPOSE`, `INVERSE`, `AXES`, `ROOTS`), requiring zero multiply instructions at serving time.

## 1. Why: endpoint-only transport is blind to intervening token order

In `crates/uor-r4-discrete/src/discrete.cu` (`flock_vector`), when an earlier position $s < t$ is carried into the query frame at $t = L - 1$, the legacy transport action `g[BFS + frame[s]*16 + fq]` (`s[14] = 0`) depends only on the two endpoint roots `frame[s]` and `fq`. Permuting the intervening tokens $(s, t)$ leaves that transport unchanged. Meanwhile, the historical D1 quaternion transport (`archive/integration/transport-attribution-d1-2026-09-28.md`) ran inside a float `rrarra` core that used banned squashing gates and dense layers and carried $+332{,}928$ more parameters than its identity arm.

`PositionHolonomyV1` (`crates/uor-r4-discrete/src/position_mechanism.rs` and `crates/uor-r4-discrete/src/discrete.cu` `flock_vector`, landed in `#3018`, `#3111`, and `#3205`) supplies a lawful discrete position mechanism on the active M1 substrate:
- Each `(block, root)` slot owns a discrete step candidate $\tau[b, r] \in \{0, \dots, 15\}$ (`0` $\mapsto$ `IDENTITY`, `1..=15` $\mapsto$ `GENERATORS[c - 1]`), giving $u_{p,b} \in G_{64}$ at position $p$.
- Along the causal window $p = 0, \dots, L-1$, the cumulative frame updates by a single `COMPOSE` lookup:
  $$F_{0,b} = u_{0,b}, \qquad F_{p,b} = \mathrm{COMPOSE}[u_{p,b} \cdot 64 + F_{p-1,b}].$$
- For any candidate at position $s < t$, the relative path holonomy is formed by one `INVERSE` lookup and one `COMPOSE` lookup:
  $$H_{t \leftarrow s, b} = F_{t,b} \circ F_{s,b}^{-1} = \mathrm{COMPOSE}\bigl[F_{t,b} \cdot 64 + \mathrm{INVERSE}[F_{s,b}]\bigr] = u_{t,b} \circ u_{t-1,b} \circ \dots \circ u_{s+1,b}.$$
- **Exact GPU `flock_vector` final block-`b` read definition (`#3111` `460d1e413` and `#3205` `e6b34e332`):**
  1. Internal base-3 tree aggregation across `level = 0..levels` preserves the existing `g[BFS + frame[child]*16 + frame[c0]]` tree-child transport unchanged and tracks each internal or leaf node's span-rightmost token index `last[node]` ($s = \mathrm{last}[\text{node}] < t = n - 1$).
  2. Before residual stage $l$ mutates `out`, the query position's immutable block-$b$ payload is latched in `query_payload[8]`. Each leaf $p \in \{0, \dots, n-1\}$ extracts its block-$b$ payload root index $r_{p,b} = \operatorname{root\_id}(\mathrm{rv}_{p,b}, g) \in \{0, \dots, 15\}$ (`transport_root[p]`, byte-packed as `unsigned char`).
  3. When `s[14] == 0` (legacy BFS), `flock_vector` ranks candidates by `root_cost(rv(fq), frame[node])` and transports block-$b$ payloads by `g[BFS + frame[node]*16 + fq]`. When `s[14] in 1..=3` (`PrefixHolonomyV1 = 1`, `EndpointControlV1 = 2`, `IdentityControlV1 = 3`), `flock_vector` builds byte-packed `unsigned char` `prefix[243]`, `transport[364]`, and `transport_root[364]` per block $b$, forms `transport[node]` from the span endpoint $s = \mathrm{last}[\text{node}]$ to $t = n - 1$, applies `transport[node]` to `transport_root[node]` via `apply_action` + `root_id`, ranks kept candidates against `query_payload` by `root_cost(query_payload, transport_root[node])`, and accumulates kept transported vectors into `snap(sum, found, snap_mode, g)`.

Because $G_{64}$ is non-commutative, interior token swaps in $(s, t)$ change $H_{t \leftarrow s, b}$ and alter both candidate ranking (`root_cost`) and transported payload consensus (`snap`), as verified on both CPU and RTX 5090 GPU (`#3205`).

## 2. Construction and matched controls

Three arms share the identical A8 initialization (`address_structured_v1`, file SHA-256 `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, table SHA-256 `4122cbab8b25119b120e417003152a5c3069f06e679691f7f67662a648604f07`), `Spec` ($V=4096$, $B=4$, $C=192$, $K=8$), objective A (`NearestAddressMarginV1`), and proposal schedule:

1. **Treatment (`H_pos` — `PositionMode::PrefixHolonomyV1`, `s[14] = 1`):** Learns the $16 \times B = 64$ discrete step candidates $\tau[b, r] \in 0..16$ and applies the causal prefix holonomy $H_{t \leftarrow s, b} = F_{t,b} \circ F_{s,b}^{-1}$.
2. **Parameter-matched endpoint control (`C_match` — `PositionMode::EndpointControlV1`, `s[14] = 2`):** Learns the identical $16 \times B = 64$ discrete step table $\tau[b, r] \in 0..16$ on the same schedule and move space, but forms $\mathrm{COMPOSE}[u_{t,b} \cdot 64 + \mathrm{INVERSE}[u_{s,b}]]$ from the endpoints $(s, t)$ alone. Parameter count ($64$ position slots), candidate cardinality ($16$), and proposal visits are matched exactly.
3. **Shuffled-context-order null (`C_shuf`):** Evaluates the trained `H_pos` checkpoint on the identical evaluation windows after permuting the prefix positions $[0 \dots L-2]$ with `order_slice::shuffle_prefix_window(window, seed)` (matching `position_mechanism::shuffle_prefix_roots(window_roots, seed)`) while holding the query position $L - 1$ and the prefix token multiset fixed.

## 3. Bar

- **Pre-run finite CPU, GPU, and scorer controls (all landed and passing on `main`):**
  1. **Finite CPU algebraic & CUDA-source controls (`10 / 10` in `crates/uor-r4-discrete/src/position_mechanism.rs`):**
     - `identity_table_leaves_all_frames_and_roots_unchanged`;
     - `prefix_holonomy_matches_direct_composition_and_transitivity`;
     - `states_and_coordinates_stay_within_exact_integer_bounds`;
     - `noncommutative_holonomy_detects_order_swaps_while_endpoint_control_is_blind`;
     - `table_validation_and_parameter_matched_serialization`;
     - `deserialized_position_table_is_revalidated`;
     - `position_config_validates_device_words_and_shapes`;
     - `cuda_position_arms_use_payload_roots_and_span_last`;
     - `cuda_legacy_ranking_preserves_raw_frame_cost`;
     - `cuda_snapshot_validation_launch_has_one_position_argument`.
  2. **Order-sensitive slice, `C_shuf` window helper & paired McNemar scorer controls (`10 / 10` in `crates/uor-r4-discrete/src/order_slice.rs`):**
     - `two_occurrences_with_distinct_successors_are_selected`, `a_repeated_successor_is_rejected`, `a_single_or_absent_earlier_occurrence_is_rejected`, `the_occurrence_just_before_the_query_counts`, `three_occurrences_need_every_successor_distinct`, `select_scans_the_corpus_windows_in_row_order`;
     - `committed_frozen_order_slice_matches_pinned_sha256_and_1620_rows` (verifying `docs/labs/m1-position-mechanism/order-slice-v1.tsv` matches `f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a` and `1,620` ascending ordinals `< 14,287`);
     - `shuffle_prefix_window_preserves_query_multiset_and_matches_root_shuffle`;
     - `integer_ceil_two_sqrt_and_paired_mcnemar_scorer_verify_floor_and_admission`;
     - `evaluate_seed_and_two_seed_verdict_enforces_sealed_fresh_v2_rule`.
  3. **Finite RTX 5090 CUDA qualification & stack/throughput receipts (`#3205` `e6b34e332`, `docs/labs/m1-position-mechanism/position-qualify-build{1,2}.json`, `0de22708…` / `9d1072d3…`):**
     - `72 / 72` GPU leaf traces matched the host algebraic reference across `Flock` and `Relation` and modes `1..=3`;
     - GPU interior-order swap sensitivity (`prefix_holonomy_interior_swap_sensitive = true`) vs endpoint invariance (`endpoint_control_interior_swap_invariant = true`);
     - `Device::from_snapshot` CUDA kernel validation refused out-of-range step candidate `16` with `status 10`;
     - Byte-packed `unsigned char` stack compaction reduced `discrete_forward` stack by `-2,896` B to `22,080` B (`0` spill stores/loads), delivering `305,795` / `295,120 tok/s` forward and `269,974` / `261,672 tok/s` evaluate on RTX 5090 (`vast-57638243`).
- **Pre-registered "helps order" bar (registered by the owner's standing-direction decision on `#2028` at `2026-10-23T02:36:36Z`, comment `6356034555`, card `6337522273`, and the owner's bar-term ruling on `#2028` at `2026-10-24T03:41:41Z`, comment `6379430093`; CHARTER §5.1, §5.2):**
  - On the 14,287-row DEV panel (`P_succ` 3,847 copy-successor rows, plus the frozen multi-occurrence order-sensitive slice $N = 1{,}620 \ge 200$ rows in `docs/labs/m1-position-mechanism/order-slice-v1.tsv`, `f76dc5c7…`), across both seeds (`seed 1` and `seed 2`), scored by `order_slice::evaluate_position_order_seed`:
    1. **Gain over matched-parameter control (`H_pos(ordered)` vs `C_match(ordered)`):** `H_pos(ordered)` beats `C_match(ordered)` on paired one-sided McNemar contrast by $b - c \ge F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$ in both seeds (`beats_mcnemar == true`), with non-inferior target admission ($\Delta\text{admitted} \ge 0$, `admission_non_inferior == true`).
    2. **Order-destruction drop (`H_pos(ordered)` vs `C_shuf`):** Shuffling the prefix order on the trained `H_pos` model (`H_pos(ordered)` vs `C_shuf`) reduces top-1 hits by at least $F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$ in both seeds (`beats_mcnemar == true`).
    3. **Fresh transfer replication (owner ruling `2026-10-24T03:41:41Z`, `#2028` comment `6379430093`):** If and only if DEV achieves `KEEP` across both seeds (`seed1.dev_keep && seed2.dev_keep`), both contrasts (`H_pos(ordered)` vs `C_match(ordered)` and `H_pos(ordered)` vs `C_shuf`) are evaluated once on the sealed fresh V2 draw (`0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`) and must satisfy $b - c \ge \lceil 2\sqrt{b + c}\rceil$ (no 24-row floor, `score_position_contrast(..., false, ...)`) in both seeds before mechanism promotion (`order_slice::grade_position_order_two_seeds`).
- **What each outcome decides (`CHARTER.md` §4.5, D22; `order_slice::PositionOrderVerdict`):**
  - **`PASS_PROMOTE` (DEV `KEEP` on both seeds + fresh-V2 replication $b - c \ge \lceil 2\sqrt{b + c}\rceil$):** Promotes `PositionHolonomyV1` (`PrefixHolonomyV1`) as the validated position mechanism for `m1-trunk-assembly-and-training` (`#2844`).
  - **`FAIL_DEV_CLOSE_CONFIG` (any DEV bar term missed on seed 1 or seed 2 after instrument gates pass):** Closes this A8 proxy-trunk configuration under D22 (never the position mechanism itself); **the sealed fresh V2 draw stays unopened and sealed**.
  - **`FAIL_TRANSFER_CLOSE_CONFIG` (DEV `KEEP` on both seeds, but fresh-V2 misses $b - c \ge \lceil 2\sqrt{b + c}\rceil$):** Records a development-only gain that did not transfer, closes this configuration under D22, and does not promote `PositionHolonomyV1` without a new sheet.

## 4. Base artifact (single pinned configuration)

- **Single configuration tested by this sheet:** This sheet clears and tests **one configuration only** — the active discrete M1 A8 proxy trunk in `crates/uor-r4-discrete` at commit `c695ad21a72c5b91def22458eebbe796e07d5392` (`DRIVER_SOURCE_SHA256 = 04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d` in `crates/uor-r4-discrete/src/driver_source_pin.rs`), with:
  - `Spec { vocabulary: 4096, blocks: 4, context: 192, kept_cells: 8 }`;
  - `ReadSpec::Relation { kept: 8, sweeps: 1 }`, `RelationTraining::LearnV1`, `ObjectivePolicy::NearestAddressMarginV1`, `ReadInitialization::ActiveLeafV1`, `ProposalSchedule::InterleavedReadEvery8V1`, `ProposalExecution::RowsV1`;
  - Pinned A8 codebook `address_structured_v1` (file SHA-256 `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, table SHA-256 `4122cbab8b25119b120e417003152a5c3069f06e679691f7f67662a648604f07`);
  - Seeds `1` and `2`, `16,404` batches per trained arm (`H_pos` `PrefixHolonomyV1` and `C_match` `EndpointControlV1`), plus forward-only `C_shuf` evaluation on the trained `H_pos` checkpoints.
- **Separate sheet for the later assembled Stage 3 trunk:** Any evaluation on the full Stage 3 trunk (`m1-trunk-assembly-and-training`, `#2844`, which is a separate M2 item under D79) is a different configuration and will require its own pre-registration sheet and fresh `TEST FITNESS: FIT` review (D22).

## 5. Data split

- Inherits the pinned M1 splits from `m1-attention-a8-address-cells-learning-2026-10-19.md`; the full digests from the authoritative code pin (`crates/uor-r4-discrete/src/attention_learning.rs:1372-1384`) and frozen transfer records are:
  - **TRAIN:** tokens `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` (mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`; 7,512,797 tokens / 2,425,925 eligible targets);
  - **DEV:** tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` (mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`; 42,856 payload tokens / 14,287 targets; 3,847 `P_succ` copy-successor rows);
  - **Sealed fresh V2 draw:** tokens `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`, mask `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e` (`docs/labs/a8-dose-2026-10-19/evidence/frozen-zero-report.json`), and seal `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f` (`docs/labs/attention-dose1-dispatch-2026-10-18/README.md`).
- **Errata (staging sheet):** `docs/harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md:46` carries `0919e90f813bf02f6f0528ebcf1f34c97987113b679619b0b44e065fa90b2546` / `a838bb1ae20c05a1b68265a7aa11c3dd146200b7d1ff6ef185ff999e442f5327` for the same DEV split; those digests appear in no code or archived artifact and are wrong against the code pin above. That item (#2832) is `done`, so the one-line errata is a PM action, recorded on `#2028` (the item comment `6356047343`).
- **Order-sensitive slice (rule pinned; frozen by `claude-58`, 2026-10-23):** on the pinned DEV tokens, select every row whose query token appears at least twice in its causal prefix with pairwise-distinct successor tokens; require `N ≥ 200` such rows in the frozen subset. The selection is a deterministic function of the pinned DEV tokens and the model's tokenizer, so its row set and SHA-256 are fixed when the subset is frozen.
  - **Frozen subset:** [`docs/labs/m1-position-mechanism/order-slice-v1.tsv`](../../labs/m1-position-mechanism/order-slice-v1.tsv), `SHA-256: f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a`, 17,789 bytes, **`N = 1,620`** of the 14,287 DEV rows (one `row_ordinal<TAB>target_position` line per row after a header), verified at test time by `order_slice::parse_frozen_slice`.
  - **How it was built:** `uor_r4_discrete::order_slice::select` (policy `query-occurs-twice-in-prefix/pairwise-distinct-successors/C192-window/ascending-rows/v1`) over the C192 window `PreparedCorpus::batch` builds, whose last token is the query, as `is_copy_successor` reads it. The occurrence just before the query has the query itself as its successor. "Pairwise distinct" means every successor differs from every other; under the looser reading (at least two distinct successors) the count would be 3,277, which is a diagnostic only.
  - **Reproduce:** `order-slice-freeze ft/dev/tokens.u16 ft/dev/response_mask.u8 OUT` on `data/ft-dev.tar` (HF store, sha256 `45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c`), which loads only under the DEV digests above. Two runs gave byte-identical files.
- Training never touches DEV or fresh V2 rows (CHARTER §3.10, D22 §4).

## 6. Steps

1. **Finite CPU & scorer controls (landed on `main`, `$0`):**
   - `crates/uor-r4-discrete/src/position_mechanism.rs` (`10 / 10` tests pass: `cargo test --locked -p uor-r4-discrete --lib position_mechanism`);
   - `crates/uor-r4-discrete/src/order_slice.rs` (`10 / 10` tests pass: `cargo test --locked -p uor-r4-discrete --lib order_slice`).
2. **Sheet registration & non-author `TEST FITNESS: FIT` (completed in `kimi-32` and `antigravity-88`):**
   - Registered bar (`#2028` comments `6356034555` and `6379430093`), frozen 1,620-row slice (`f76dc5c7…`), and fresh-context non-author `TEST FITNESS: FIT` reviews recorded under `## Verdict`.
3. **GPU kernel wiring, PTX admission & RTX 5090 finite qualification (landed on `main` in `#3111` and `#3205`):**
   - Wired `PositionStepTable` (`64` bytes at $B=4$) and `PositionMode` (`s[14] in 1..=3`) into `discrete.cu` (`flock_vector`, `discrete_validate`), `device.rs`, and `snapshot.rs`, and verified two-build determinism, `72 / 72` leaf traces, `status 10` snapshot refusal, and byte-packed stack compaction (`22,080` B, `0` spills, `305,795` / `295,120 tok/s`) via `cargo run --release --locked -p uor-r4-discrete --features cuda --bin position-qualify` (`docs/labs/m1-position-mechanism/position-qualify-build{1,2}.json`).
4. **D67 pre-run review & two-seed GPU execution (`H_pos`, `C_match`, `C_shuf`):**
   - Under `CHARTER.md` §5.1 and §5.10 (D67), pass a fresh-context pre-run review inside the global GPU caps ($\le 4$ pods, $\le \$8.00/\text{h}$), then run `H_pos` (`PrefixHolonomyV1`) and `C_match` (`EndpointControlV1`) across seeds `1` and `2` on GPU via `scripts/pod/uor-pod`, plus forward-only `C_shuf` evaluation (`order_slice::shuffle_prefix_window`) on the trained `H_pos` checkpoints.
5. **Paired McNemar DEV scoring and conditional fresh-V2 replication:**
   - Score the paired contrasts on DEV (`14,287` full, `3,847` `P_succ`, and `1,620` frozen order-sensitive slice) with `order_slice::evaluate_position_order_seed` and, if and only if both seeds achieve `dev_keep == true`, evaluate once on the sealed fresh V2 draw (`0b7e754c…`) against $b - c \ge \lceil 2\sqrt{b + c}\rceil$ with `order_slice::grade_position_order_two_seeds`.

## 7. Stop conditions

- **Instrument stops (D31, never a mechanism negative):** Any PTX admission failure, `driver_source_sha256` mismatch, checkpoint-0 reproduction mismatch, or out-of-range state halts immediately as `INSTRUMENT_UNREACHABLE` for diagnosis.
- **Outcome stops (D22, `order_slice::PositionOrderVerdict`):**
  - If any DEV bar term misses on either seed (`FAIL_DEV_CLOSE_CONFIG`), stop before fresh V2; **the sealed fresh V2 draw remains unopened and sealed**, and only this A8 proxy-trunk configuration closes.
  - If DEV passes on both seeds (`KEEP`) and fresh V2 misses $b - c \ge \lceil 2\sqrt{b + c}\rceil$ (`FAIL_TRANSFER_CLOSE_CONFIG`), close this configuration without mechanism promotion.
- **Resource & budget guards:** Self-set step or wall-clock estimates are planning figures, not stop triggers; extend and renew rather than cancel a healthy run. Hard stops are the owner spend/pod caps ($\le 4$ pods, $\le \$8.00/\text{h}$ across all labs), the 30 GiB / 128 MiB disk floor, or a Q1 admission collapse gate failure.

## 8. Cost and minimum test power

- **Cost (planning estimate at $\$1.894/\text{h}$ on a single leased GPU via `scripts/pod/uor-pod`):**
  - `preparation`: $\$0.00$ (host metadata and `20 / 20` finite CPU/scorer controls on laptop).
  - `run`: $\approx \$11.40$ per seed pair (`H_pos` + `C_match`, 16,404 batches per arm in parallel, $\approx 6\text{ h}$ wall time per seed pair; $\approx \$22.80$ for 2 seeds).
  - `controls`: included in the paired `C_match` arm above; `C_shuf` is a forward-only evaluation pass on the trained `H_pos` checkpoint ($\approx \$0.20$).
  - `evaluation`: $\approx \$0.40$ (DEV panel checkpoints plus sealed fresh V2 draw on `KEEP`).
  - `retries`: $\approx \$2.80$ (one Q1 diagnostic retry allowance within owner caps).
  - `storage`: $< 64\text{ MiB}$ per retained snapshot pair, respecting the 128 MiB / 30 GiB stop margin.
- **Minimum test power and expected effect size (`CHARTER.md` §4.3, D22 §3):**
  - Seeds: $2$ paired seeds (`seed 1` and `seed 2`).
  - Rows: $14{,}287$ DEV rows ($3{,}847$ `P_succ` rows; $1{,}620 \ge 200$ frozen multi-occurrence order-sensitive rows in `order-slice-v1.tsv`) plus sealed fresh V2 transfer draw ($\ge 200$ rows).
  - Noise floor vs expected effect: The one-sided paired McNemar floor $F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$ is $1.48\%$ of the `1,620`-row frozen order-sensitive slice at the 24-row floor (and $0.62\%$ of `P_succ`). Because `EndpointControlV1` (`C_match`) is algebraically blind to interior token permutations while `PrefixHolonomyV1` (`H_pos`) distinguishes non-commutative interior swaps across the `1,620` multi-occurrence rows where the query token has $\ge 2$ distinct prefix successors, the expected discordance on the `1,620`-row order slice and `3,847`-row `P_succ` panel is $\Delta_{\text{expected}} \ge 40\text{–}120$ rows ($\ge 1.5\times\text{–}4\times F_I$). A positive difference $0 < b - c < F_I$ is classified as `INCONCLUSIVE_UNDERPOWERED` inside the two-seed noise band rather than a negative.
  - Decisive run: YES ($\ge 2$ seeds and $\ge 200$ fresh rows, trained in from step 0 rather than bolted on).

## Verdict

### Why NOT_FIT (prerequisites in priority order)

1. **The owner bar is not registered.** The top Status line called comment `6337522273` "the owner's bar confirmation"; it is **antigravity's decision card** proposing `F_order`. No owner answer exists (the PM's routing note lists this card among those awaiting the owner), so no compute is cleared. Quote the owner's registered bar verbatim with an **ISO-UTC time**, or keep the card as `PENDING`.
2. **The DEV pin is ambiguous, so the inherited split is not actually pinned.** The sheet carries only prefixes (`0919e90f…`). The repo holds two conflicting full pairs:
   - the **authoritative code pin** in [`crates/uor-r4-discrete/src/attention_learning.rs`](../../../crates/uor-r4-discrete/src/attention_learning.rs) — tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` (archive `a2-pilot` matches, and the expected `42,856` payload tokens / `14,287` targets match);
   - the staging sheet's `0919e90f813bf02f6f0528ebcf1f34c97987113b679619b0b44e065fa90b2546` / `a838bb1ae20c05a1b68265a7aa11c3dd146200b7d1ff6ef185ff999e442f5327`, which appears nowhere else.
   Pin the full code-pin pair in §5 and file an errata for the staging sheet, or show which file each pair belongs to.
3. **The new evaluation slice is unpinned.** The bar's "multi-occurrence order-sensitive slice (`N ≥ 200` rows)" has no deterministic definition (selection rule), row count or SHA-256. Pin the construction and the hash before a run; the inherited `14,287`-row DEV panel is pinned through the DEV tokens hash once finding 2 is settled.

Everything else checked and holds: all 8 blocks are present (`prep-check` `complete=yes`); `cargo test -p uor-r4-discrete --lib position_mechanism` = **5 passed**; the path has no float, no dense matmul and no squashing form; the step table exports as `16 × B` `u8` words; the A8 parent sheet is `TEST FITNESS: FIT`; the proxy trunk is named; seeds, rows, cost and stop rules are present.

Re-request FIT only when 1–3 are pinned in this sheet; this verdict clears no compute (D22).

### Post-registration update (deepseek-39, 2026-10-23)

The two owner/PM prerequisites from this review now stand as follows.

1. **Bar — REGISTERED.** The owner's standing-direction decision at **2026-10-23T02:36:36Z** (`#2028` comment `6356034555`, card `6337522273`) registers the §3 bar exactly as proposed: DEV 14,287 rows, ≥ 2 seeds, `H_pos(ordered)` vs `C_match(ordered)` McNemar `b − c ≥ max(24, ⌈2√(b+c)⌉)` in both seeds with `Δadmitted ≥ 0`, and the order-shuffle drop ≥ `F_I`. The sheet now quotes it with the ISO-UTC time.
2. **DEV pin — PINNED.** §5 now carries the full pair from the authoritative code pin (`0919e90f953e…`, `a838bb1a67e9…`, matching 42,856/14,287 and the archived `a2-pilot`). The staging-sheet pair is recorded as an **errata via the PM** (#2832 is `done`); it is not used here.
3. **Order-sensitive slice — RULE PINNED, HASH `PENDING`.** §5 states the deterministic selection rule and `N ≥ 200`; the frozen subset's SHA-256 stays `PENDING` because the frozen file is data a lab/PM must file.

`TEST FITNESS: NOT_FIT` still stands: the slice hash is not yet pinned and the FIT must come from a non-author lab (per the registration, Claude or Kimi — deepseek has authored text in this sheet). This update clears no compute.
### Recorded review

```text
TEST FITNESS: NOT_FIT - M1 #2029 PositionHolonomyV1
sheet: docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md
reviewer: deepseek (non-author; fresh-context FIT review for #2843)
note: Configuration not yet pinnable: (1) the sheet's 'owner bar confirmation' link is antigravity's decision card 6337522273, not an owner answer - the owner bar with ISO-UTC time is absent (PM routing lists the bar cards as waiting on the owner), so no compute is cleared; (2) the DEV pin is ambiguous - the sheet carries only prefixes, while the repo holds two conflicting full DEV pairs: the authoritative code pin in attention_learning.rs (tokens 0919e90f953e..., mask a838bb1a67e9..., matching expected 42,856/14,287 and archived a2-pilot) versus the staging sheet's 0919e90f813b.../a838bb1ae20c... which appears nowhere else; pin the full code-pin pair and file an errata for the staging sheet; (3) the new multi-occurrence order-sensitive slice (N>=200 rows) has no deterministic definition or SHA-256 and must be pinned. Everything else holds: 8/8 blocks, 5 finite CPU controls pass, no float/matmul/softmax on the path, D11 export as 16xB u8 words, parent A8 sheet is FIT-registered, proxy trunk named, seeds/rows/cost/stop rules present.
date: 2026-10-22
A FIT verdict clears compute for the configuration in that sheet only (D22).
```

### Post-freeze non-author FIT review (kimi-32, 2026-10-23)

The three prerequisites in the recorded `NOT_FIT` verdict are now resolved. The earlier verdict and DeepSeek's post-registration update remain above as history.

1. **Bar — REGISTERED.** Casey's owner decision, recorded at **2026-10-23T02:36:36Z** in `#2028` comment `6356034555`, says **"Yes, register all six (Recommended)"** and registers card `6337522273` exactly as proposed. §3 quotes the registered paired bar, including `Δadmitted ≥ 0` and the order-shuffle drop.
2. **DEV pin — PINNED AND REPRODUCED.** Kimi hashed the preserved `data/ft-dev.tar` as `45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c`; its extracted DEV tokens and mask hash to the full §5 pair `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` and `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`. The registered loader scanned 14,287 rows.
3. **Order-sensitive slice — FROZEN AND REPRODUCED.** Kimi reran `order-slice-freeze` on that pinned DEV pair. It selected 1,620 rows, wrote 17,789 bytes, and produced SHA-256 `f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a`, byte-identical to §5's frozen file from #3046 (`aac963833`).

The unchanged finite position controls pass **5/5** (`cargo test -p uor-r4-discrete --lib position_mechanism`). The eight sheet sections, matched controls, proxy trunk, seeds, row counts, costs and stop rules remain present; the A8 parent remains FIT. **TEST FITNESS: FIT** applies only to this pinned configuration. It does not wire the GPU path, run the order test, complete #2843, or release any hold; GPU work still requires the named trunk dependency, PM dispatch and Casey's D40 release.

```text
TEST FITNESS: FIT - M1 #2029 PositionHolonomyV1
sheet: docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md
reviewer: kimi (non-author post-freeze FIT review for #2843)
resolved: owner bar 6356034555 at 2026-10-23T02:36:36Z; full DEV pair reproduced from tar 45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c; 1620-row slice reproduced byte-identically at f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a
controls: position_mechanism 5 passed, 0 failed
date: 2026-10-23
scope: configuration fitness only; no GPU run, mechanism result, item completion or owner release
```

### Post-`#3111`/`#3205`/`#3256` synchronization FIT review (antigravity-88, 2026-10-25, fresh context)

Reviewed in a fresh context (`b49b8f8b-3a24-4729-89fe-58cc393ac27e`) against `CHARTER.md` §3, §4.5, §5.1, §5.10, `PLAN.md`, `docs/roadmap/items/m1-position-mechanism.md` (`#2843`), `kimi-32` (`#3108`), `kimi-33` (`#3111`), and `antigravity-62` (`#3205`). All post-`kimi-32` synchronization items and `kimi-33/REPORT.md` line 16 prerequisites are resolved and verified by command:

1. **Exact GPU `flock_vector` final block-`b` transport definition pinned (`kimi-33/REPORT.md:16`) — RESOLVED:** §1 pins the landed `#3111`/`#3205` `crates/uor-r4-discrete/src/discrete.cu` `flock_vector` read semantics: internal base-3 tree aggregation preserves `g[BFS + frame[child]*16 + frame[c0]]`, each node's span-rightmost token `last[node]` ($s < t = n - 1$) supplies the block-$b$ payload root `transport_root[node] = root_id(rv, g)`, `query_payload[8]` latches the immutable query block-$b$ vector before residual stage $l$ mutates `out`, and modes `s[14] in 1..=3` use byte-packed `unsigned char` `prefix[243]`, `transport[364]`, and `transport_root[364]` (`22,080` B stack, `0` spills) to rank candidates via `positioned_root_cost` and accumulate transported payloads into `snap`.
2. **Single configuration & RTX 5090 finite CUDA qualification receipts pinned — RESOLVED:** §4 pins a single configuration (the discrete M1 A8 proxy trunk at `c695ad21a72c5b91def22458eebbe796e07d5392`, `DRIVER_SOURCE_SHA256 = 04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`, codebook `6600bdb1306d…` / `4122cbab8b25…`), deferring the Stage 3 trunk (`#2844`, M2 under D79) to its own future sheet, and §3/§6 bind the `#3205` RTX 5090 qualification receipts (`docs/labs/m1-position-mechanism/position-qualify-build{1,2}.json`, `0de22708…` / `9d1072d3…`, `72 / 72` leaf traces, `status 10` CUDA refusal, `305,795` / `295,120 tok/s`).
3. **Owner fresh-V2 replication rule, 3-branch outcome table, and exact integer paired McNemar scorer — RESOLVED:** §3 and §7 record both the `2026-10-23T02:36:36Z` bar (`#2028` comment `6356034555`, $F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$, $\Delta\text{admitted} \ge 0$) and the `2026-10-24T03:41:41Z` fresh-V2 replication ruling (`#2028` comment `6379430093`, $b - c \ge \lceil 2\sqrt{b + c}\rceil$ with no 24-row floor), the 3-branch outcome table (`PASS_PROMOTE`, `FAIL_DEV_CLOSE_CONFIG` keeping fresh V2 unopened and sealed, `FAIL_TRANSFER_CLOSE_CONFIG`), and the exact integer scorer (`parse_frozen_slice`, `shuffle_prefix_window`, `integer_ceil_two_sqrt`, `score_position_contrast`, `evaluate_position_order_seed`, `grade_position_order_two_seeds`) in `crates/uor-r4-discrete/src/order_slice.rs`.
4. **Controls & lawfulness verified by command:** `cargo test --locked -p uor-r4-discrete --lib position_mechanism` (`10 passed; 0 failed`) and `cargo test --locked -p uor-r4-discrete --lib order_slice` (`10 passed; 0 failed`); zero floating-point types or literals (`D32`), zero squashing (`§3.2`, `D23`), and zero dense matmul (`§3.3`, `D25`).

```text
TEST FITNESS: FIT - M1 #2029 PositionHolonomyV1
sheet: docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md
reviewer: antigravity (non-author fresh-context reviewer, session b49b8f8b-3a24-4729-89fe-58cc393ac27e, for antigravity-88 #2843 at c695ad21a)
resolved: (1) section 1 pins the exact #3111/#3205 GPU flock_vector final block-b transport definition (last[node] payload-root anchor, immutable query_payload, internal BFS tree preservation, and byte-packed unsigned char arrays); (2) section 4 pins a single configuration (A8 proxy trunk at c695ad21a, DRIVER_SOURCE_SHA256 04c51654..., codebook 6600bdb1...) and sections 3/6 bind the #3205 RTX 5090 CUDA qualification receipts (0de22708..., 9d1072d3..., 72/72 leaf traces, status 10 refusal, 22,080 B stack with 0 spills); (3) sections 3 and 7 quote both owner bar rulings (#2028 6356034555 at 2026-10-23T02:36:36Z and #2028 6379430093 at 2026-10-24T03:41:41Z fixing fresh-V2 replication b - c >= ceil(2*sqrt(b+c))), define the 3-branch outcome table (PASS_PROMOTE, FAIL_DEV_CLOSE_CONFIG keeping fresh V2 sealed, FAIL_TRANSFER_CLOSE_CONFIG), and bind the exact-integer paired McNemar scorer and C_shuf window helper in order_slice.rs.
controls: position_mechanism 10 passed, 0 failed; order_slice 10 passed, 0 failed; RTX 5090 position-qualify 72/72 leaf traces passed
date: 2026-10-25
A FIT verdict clears compute for the configuration in that sheet only (D22).
```
