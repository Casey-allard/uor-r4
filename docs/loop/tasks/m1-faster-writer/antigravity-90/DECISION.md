# DECISION (D68): Register Step-4 Bar (`B1`, `B2`, `B3`) and Pin Configuration for `m1-faster-writer` (`#2840`)

Lab: `antigravity`. Unit: `antigravity-90`. Item: [`m1-faster-writer`](../../../../roadmap/items/m1-faster-writer.md) (`#2840`, Milestone M1 `#2029`). Base: `origin/main` `bc6d4137949b03824912d84de8d04bd4a0e92347`.

## 1. The Question

The roadmap item [`docs/roadmap/items/m1-faster-writer.md`](../../../../roadmap/items/m1-faster-writer.md) (`#2840`) specifies:
- **Step 4:** *"Run a finite qualification on CUDA against the full-scan writer. It passes when (B1) outputs match on every control and (B2) measured per-row comparisons are below the bar Casey registers."*
- **Done when:** *"A writer whose measured per-row comparison count is below the full-vocabulary scan (about 5×10⁶ today), with finite controls that match the full-scan writer, and a CUDA qualification. How far below the scan counts as enough is not yet registered; register it in step 4."*

In `antigravity-35` (23 Oct), the draft pre-registration sheet [`docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md`](../../../harness/prep/m1-faster-writer-DRAFT-2026-10-22.md) received `TEST FITNESS: NOT_FIT` for three prospective reasons:
1. The §3 `B1`/`B2` bar was marked `PROPOSED, NOT REGISTERED`.
2. §5 left two open configuration choices: (a) whether to qualify `V = 16,384` on seeded `random_vocab` tables (`seeds 1, 2, 3`) or defer until a `16,384`-piece tokenizer/corpus is built (`#2872`), and (b) whether to include the `14,287`-row real-read `DEV` panel on pinned `A8` (`6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`) and pinned `PrimeZeta` (`83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b`).
3. §6 did not yet list the concrete admitted PTX `.entry` inventory in `src/bucket_source.rs` or the `writer-bucket-qualify` CLI invocation, and §3 did not state how target `ad = address_distance(q, g, s, target_cell)` at `discrete.cu:326` is accounted in `B2`.

Under `CHARTER.md §5.1` (`D68`), how should the step-4 `B1`/`B2`/`B3` bar be registered and how should the §3, §5, and §6 configuration choices be pinned in [`docs/harness/prep/m1-faster-writer-2026-10-26.md`](../../../harness/prep/m1-faster-writer-2026-10-26.md)?

## 2. The Decision

1. **Register the `B1`, `B2`, and `B3` finite CUDA qualification bar (`docs/harness/prep/m1-faster-writer-2026-10-26.md §3`, registered `2026-10-26T09:50:00Z`):**
   - **`B1` (Exact Parity):** `0` mismatches between `bucket_writer` and full-scan `writer` across every control group (`12` seeded `Device` models across `forward`, `evaluate`, and `propose`; `72` direct seeded sweep cases across `V ∈ {64, 256, 4096, 16384}`, `B ∈ {2, 4}`, `K ∈ {1, 8, 16}`, `seeds 1..=3` plus `36` virtual-move cases; all constructed edge cases; `64`-step sequential committed move updates; and all `14,287` real-read `DEV` rows on both pinned `A8` and pinned `PrimeZeta`).
   - **`B2` (Measured Per-Row Comparison Count & Incremental CSR Update):**
     - `B2a`: `token_cell_calls == 0` per row for `kind ∈ {0, 2, 3}` and `<= 1` for `kind == 1` (vs `V + 1 + A * V` in the full scan: `36,865` at `V = 4,096, A = 8`; `69,633` at `V = 4,096, A = 16`; `147,457` at `V = 16,384, A = 8`);
     - `B2b`: `cell_distance_evals == 256` per row in the admission pass (target `ad` at `discrete.cu:326` and the rank-telemetry pass at `discrete.cu:330-333` reuse `distance[256]` with `0` extra `cell_distance` calls);
     - `B2c`: `token_distance_evals <= K * max_c |B[c]|` (`+ 1` for `kind == 1`), with `ServingPruned <= IdOrder` on every row;
     - `B2d`: total per-row comparison count `(token_cell_calls + cell_distance_evals + token_distance_evals)` achieves **`>= 10x` reduction** (`reduction_x100 >= 1000`) relative to the full scan at `K = 8, B = 4` on `V = 4,096` and `V = 16,384` seeded/uniform tables and on the `14,287`-row `DEV` panel on both pinned `A8` and pinned `PrimeZeta`;
     - `B2e`: across `64` sequential steps of `discrete_apply` + `discrete_bucket_update`, `0` CSR mismatches against a fresh `discrete_bucket_build` and `entries_written <= 2 * V + 258`.
   - **`B3` (Negative Controls):** All `4` planted faults (`FAULT_STALE_BUCKET`, `FAULT_REVERSED_TIE`, `FAULT_OVER_PRUNED`, `FAULT_CORRUPTED_CSR`) must produce a non-zero mismatch (`mismatches_detected > 0`) caught by the qualification check.
2. **Pin both §5 open choices and the §6 concrete PTX/CLI inventory in `docs/harness/prep/m1-faster-writer-2026-10-26.md`:**
   - Pin Choice 1 to **seeded `V = 4,096` and `V = 16,384` tables** (`seeds 1, 2, 3`) plus uniform `V = 4,096` (`16` tokens/cell) and `V = 16,384` (`64` tokens/cell) tables in `writer_bucket_qualification.rs`.
   - Pin Choice 2 to **both pinned `A8` (`6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`) and pinned `PrimeZeta` (`83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b`) across all `14,287` real-read `DEV` rows** (`ft/dev/tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `ft/dev/response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`).
   - Pin the 7 admitted `.entry` kernels in `crates/uor-r4-discrete/src/bucket_source.rs` (`discrete_bucket_build`, `discrete_bucket_update`, `discrete_bucket_writer_eval`, `discrete_fullscan_writer_eval`, `discrete_bucket_forward`, `discrete_bucket_counterfactual`, `discrete_bucket_evaluate`) and the `writer-bucket-qualify` CLI in `crates/uor-r4-discrete/src/bin/writer-bucket-qualify.rs`.

## 3. Why (Whole-Project Context)

1. **Whole-project chain (`PLAN.md`, `M1 #2029`, `#2840 -> #2887`):**
   - `CHARTER.md §3.10` makes `tokens/s` the primary metric, and `PLAN.md` ranks `m1-faster-writer` (`#2840`) as the open M1 infrastructure item that removes the `O(V)` per-row full-vocabulary scan (`V + 1 + A * V` `token_cell` calls per row) and unblocks `#2887` (`r-vocab-4096-vs-16384`).
   - On `V = 4,096` (`K = 8`, `256` occupied cells, `16` tokens/cell on `A8`), the full scan evaluates `36,865` `token_cell` calls + `2,277` `cell_distance` calls + `128` `token_distance` calls = `39,270` evaluations per row (`561,050,490` across the `14,287`-row `DEV` panel), whereas the CSR bucketed writer evaluates `0` `token_cell` calls + `256` `cell_distance` calls + `128` `token_distance` calls = `384` evaluations per row (`> 100x` reduction at `V = 4,096` and `~196x` at `V = 16,384`).
2. **Why the registered `B1`/`B2`/`B3` bar is exact and non-arbitrary:**
   - `B1` (`0` mismatches) enforces exact functional equivalence with `discrete.cu`'s full-scan `writer`.
   - `B2a` (`0` `token_cell` calls for `kind != 1`, `<= 1` for `kind == 1`), `B2b` (`256` `cell_distance` calls), and `B2c` (`<= K * max_c |B[c]|` `token_distance` calls) are the exact structural invariants of CSR bucket indexing over 256 address cells, and `B2d` (`>= 10x` total comparison reduction) is a conservative floor well below the `~102x–196x` reduction achieved on `V = 4,096` and `V = 16,384`.
3. **Why pinning both seeded `V = 16,384` and the `14,287`-row `DEV` panel on `A8` and `PrimeZeta` is strictly stronger:**
   - It adopts every recommendation of the 23 Oct `NOT_FIT` review verbatim, testing both `V = 16,384` scaling and real `read_vector` queries across predecessor-structured (`A8`) and complex-orbit (`PrimeZeta`) codebooks in one self-contained GPU qualification run (`< 5` minutes on `1x RTX 5090`).

## 4. Rejected Alternatives

- **Qualify on seeded synthetic queries only and skip the `14,287`-row `DEV` panel:**
  - Rejected because scoring all `14,287` real-read `DEV` rows on both `A8` and `PrimeZeta` takes less than a minute on an RTX 5090 and proves end-to-end `read_vector` + `bucket_writer` parity on the project's canonical evaluation split.
- **Wait on a `16,384`-piece tokenizer/corpus (`#2872`) before qualifying `m1-faster-writer` (`#2840`):**
  - Rejected because `#2840` is an exact algorithmic and CUDA kernel qualification whose dependency list is `none` (`docs/roadmap/items/m1-faster-writer.md:9`), and `#2887` (`r-vocab-4096-vs-16384`) depends on `#2840` finishing first.

## 5. Cost to Reverse

Reversible by git revert; `discrete.cu`, `ordinary.cubin`, and `driver_source_sha256` (`04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`) are untouched.

## 6. Review Record (D68, D67, and `TEST FITNESS: FIT`)

- **Reviewer:** `antigravity` fresh-context non-author reviewer (`a6fa66ad-16e7-4c7f-9f27-dc43fa7494e2`)
- **Verdicts:**
  - `TEST FITNESS: FIT` on `docs/harness/prep/m1-faster-writer-2026-10-26.md`
  - `DECISION REVIEW (D68): PASS`
  - `PRE-RUN REVIEW (D67): PASS`

```text
DECISION REVIEW (D68): PASS - M1 #2029 m1-faster-writer (#2840)
card: docs/loop/tasks/m1-faster-writer/antigravity-90/DECISION.md
reviewer: antigravity (fresh-context non-author decision reviewer a6fa66ad-16e7-4c7f-9f27-dc43fa7494e2)
whole_project_context: Verified against CHARTER.md (sections 3, 4.5, 5.1, D65-D69), PLAN.md, docs/roadmap/items/m1-faster-writer.md (#2840), and docs/roadmap/items/r-vocab-4096-vs-16384.md (#2887). #2840 has no upstream dependencies (depends_on: []) and directly unblocks #2887 (r-vocab-4096-vs-16384) while removing the O(V) per-row writer scan (V + 1 + A*V token_cell calls per row) on the M1/M4 serving path.
verdict: PASS. The registered B1/B2/B3 bar is exact and non-arbitrary (0 parity mismatches, 0 token_cell calls for kind != 1 and <= 1 for kind == 1, 256 cell_distance calls, <= K * max_c |B[c]| token_distance calls, >= 10x total comparison reduction at V=4,096 and V=16,384, 0 CSR update mismatches across 64 sequential commits, and 4/4 planted faults caught). Pinning both seeded/uniform V=16,384 tables and the 14,287-row real-read DEV panel on pinned A8 and PrimeZeta adopts all 23 Oct review recommendations, and the change is fully reversible without touching discrete.cu or driver_source_sha256.
date: 2026-10-26T09:59:00Z
```

```text
PRE-RUN REVIEW (D67): PASS - M1 #2029 m1-faster-writer (#2840)
sheet: docs/harness/prep/m1-faster-writer-2026-10-26.md
reviewer: antigravity (fresh-context non-author pre-run reviewer a6fa66ad-16e7-4c7f-9f27-dc43fa7494e2)
criteria:
  1. whole_project_chain: Verified. m1-faster-writer (#2840) is ready with depends_on: [] and is the sole blocker on r-vocab-4096-vs-16384 (#2887) and the D5-b per-token sparsity requirement (CHARTER 3.9).
  2. mechanism_and_inputs_ready: Verified. bucket_source.rs, writer_bucket_device.rs, writer_bucket_qualification.rs, src/bin/writer-bucket-qualify.rs, src/bin/prime-zeta-codebook.rs, docs/labs/r-prime-zeta-vs-nulls/evidence/spec-v4096-b4-c192-k8.json, and build.rs are present in the worktree, and ft-dev.tar (0919e90f... / a838bb1a...) and a8-rebuilt.json (6600bdb1...) are present in caseyallard/uor-r4-store (with the 5 pre-flight audit fixes applied).
  3. unlock_value: Verified. A pass closes #2840 (m1-faster-writer -> done) and unblocks #2887 (r-vocab-4096-vs-16384); a B2 miss keeps the exact CSR index and iterates on bucket layout; a B1/B3 failure diagnoses the kernel/instrument defect.
  4. fit_verdict: Verified. TEST FITNESS: FIT recorded on docs/harness/prep/m1-faster-writer-2026-10-26.md.
  5. lease_and_caps: Verified. 0 pods currently active (<= 4 pods cap) and $0.00/h spend (<= $8.00/h cap); planned 1x RTX 5090 lease (~$0.50-$0.80/h for < 20 min, < $0.40 total) is well within caps.
date: 2026-10-26T09:59:00Z
```
