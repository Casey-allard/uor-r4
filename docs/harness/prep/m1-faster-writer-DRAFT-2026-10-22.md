# Bucketed CUDA writer: finite qualification against the full scan (DRAFT)

- **Status:** SUPERSEDED by [`m1-faster-writer-2026-10-26.md`](m1-faster-writer-2026-10-26.md), `TEST FITNESS: NOT_FIT` (historical 23 Oct non-author audit recorded below; all three prerequisites are resolved in `m1-faster-writer-2026-10-26.md`). Nothing is trained. A FIT releases nothing; the GPU run needs FIT plus the registered bar.
- **Line:** M1 (#2029). **Item:** `m1-faster-writer` (#2840), step 4. **Lab:** claude.
- **Parent:** none. **Design note:** #2840 comment 6346787358. **Host controls:** `e10d109` (#2983), `crates/uor-r4-discrete/tests/writer_bucket_controls.rs`, 6 tests, and `docs/loop/tasks/m1-faster-writer/claude-34/REPORT.md`.
- **Rules:** CHARTER §3.1–3.6 (geometry only; no normalized-exponential read and no squashing; no dense matrix product; GPU-only research compute; integer artifact) and §3.9 (D5-b per-token sparsity); §4.5 pre-registration with a non-author FIT.
- **Base:** `origin/main` `e10d10926`. Every `discrete.cu` line cited below is at this base.

## 1. Why

`writer` (`crates/uor-r4-discrete/src/discrete.cu:307-345`) chooses among the members of K admitted address cells, but it finds them by scanning the whole vocabulary:
- `:309` calls `token_cell` (`:286-290`) on every token `0..V` to mark occupancy; any ABSENT cell returns MAX;
- `:311` calls it once more for the target's cell;
- `:313-325`: for each of up to K = `s[3]` admission passes, it evaluates `address_distance` on every occupied, unused cell under `cell_precedes` (`:301-303`), then scans all V tokens again with `token_cell(t) == cell` (`:319`).

Per row that is V + 1 + A·V `token_cell` calls, where A ≤ K is the number of admitted cells: 69,633 at V = 4,096 and A = K = 16. Admission evaluates up to K·256 cell distances. The choice is sparse; finding the members is O(V). The owner's lead metric is tokens/s, and this scan is the serving cost the architect audit names.

## 2. What changes

One new writer, built in its own module. It replaces the full-vocabulary scans with a CSR bucket index over the 256 root-pair address cells (`cell = root(block 0)·16 + root(block B−1)`).

- **Index.** `offsets[0..=256]`; `members[0..V)`, token ids grouped by cell, ascending id within a cell; each token's base cell; and an ABSENT count (the host controls showed that a single flag is not enough when the moved token is itself the ABSENT one). It is built on the device by a count/scan/scatter kernel in the new module that also counts ABSENT tokens. `discrete_export_token_cells` (`:367-372`) cannot be used as is: on ABSENT it returns without writing `cells[token]` and raises one status flag (`:370`).
- **Rebuild.** After each accepted move (`discrete_apply`, `:431`), at most one token changes bucket, so the index is updated for that token only. A full rebuild is the control (§5). The update cost per accepted move (index entries written) is counted and reported; it falls outside the per-row counts, and shifting `members` between two buckets can touch up to V entries.
- **Writer.** Occupancy is `offsets[c+1] > offsets[c]`. Admission is one pass over 256 cells ordered by (address distance, cell id). Membership walks each admitted bucket in id order with the unchanged rule `(d < best) || (d == best && t < best_t)`. Every output is produced from the same candidate set as today.
- **Kind 1** (virtual token/block move, `:26`, `:32`): only the moved token can change cell. It costs one `token_cell` call; its base bucket is walked skipping it, it is scored in its moved cell if that cell is admitted, occupancy is adjusted by ±1, and a moved-ABSENT result returns MAX. Kinds 0, 2 and 3 use the index unchanged.
- **The evaluate entry's top-8 scan** (`discrete_evaluate`, `:479-488`) also calls `token_cell` on every token, after `writer`. It walks the admitted buckets instead. Its result does not depend on visit order, because the insertion uses the total order (distance, token id).
- **Serving prune: out of scope for this sheet.** Skipping an admitted cell whose address distance exceeds the best token distance also skips that cell's rivals. That changes `token_rivals`, the rival distance, `token_hinge`, the loss and `telemetry[3]` (`:320-322`, `:327`, `:342`), and all three existing entries need those. No entry today is prediction-only. The prune, with its own prediction-only entry, is a named follow-up; the host controls already pin it (#2983).
- **Build.** `crates/uor-r4-discrete/build.rs` today compiles `src/discrete.cu` with nvcc `-ptx` and then generates five separate modules (decomposition, baseline-reuse, rows, rowsk, bvgk), each with its own `*_source.rs`, its own nvcc invocation, its own `admit_ptx_inventory` entry list and its own cubin in the final assembly loop. The bucketed writer follows that pattern: a new `src/bucket_source.rs` generator that emits `bucket.cu`, a new nvcc block placed after the bvgk block and before the cubin loop, a new `("bucket", bucket.ptx)` entry in that loop, and an admitted entry list. The ordinary `discrete.ptx` and the five existing modules stay byte-for-byte. NVRTC is banned (CHARTER §3.4, `docs/model/gpu-and-cuda.md`); nothing is appended to a shared CUDA source string.
- **Arithmetic.** Integer only. PTX admission already rejects any float or tensor-core instruction. No normalized-exponential read, no squashing, no dense matrix product.
- **No CPU fallback.** The host writers in `writer_bucket_controls.rs` are the finite control only.

## 3. Bar: PROPOSED, NOT REGISTERED

Casey registers the bar. This is the form proposed in the design note:

| Measure | Proposed bar |
|---|---|
| Output mismatches against the full scan | 0, on every row and kind (B1) |
| `token_cell` calls per row, in `writer` and the evaluate entry's top-8 scan | 0 for kinds 0, 2 and 3; at most 1 for kind 1 (B2) |
| Cell-distance evaluations per row, admission pass | at most 256, one per cell (B2) |
| Token-distance evaluations per row, membership walk only | equal to the members of the admitted cells; this sheet has no prune (B2) |

**What B2 counts.** In `writer`, the token-distance count is the membership walk only (`:319-322` in the full scan). The target's own distance (`td`, `:326`) is one more call per row in both writers and is reported separately. The evaluate entry's top-8 scan (`:483`) is counted separately again, with the same bar. Under kind 1, "members" means virtual membership: the moved token counts in its moved cell when that cell is admitted, and not in its base cell. These definitions match the host counter (`writer_bucket_controls.rs`).

**B1 compares every output `writer` writes** (signature at `:307`): the return value (the loss, or MAX); `prediction`; `token_rivals`; `address_rivals`; `admitted[0..16)`; `token_hinge`; `address_hinge`; `telemetry[0..6)`, which holds the rank hinge, the target's cell rank, occupancy, the eligible count, `loss_tr` and the reported loss; and `arithmetic_status`, including status 12. For the evaluate entry it also compares the top-8 list, its distances and the status-11 check (`:479-490`). On a MAX row, `writer` returns before writing some outputs (`:309`, `:325`), so both sides prefill every output buffer with the same sentinel and B1 compares the return value, the status and every buffer.

**Reporting.** For V = 4,096 and V = 16,384, report the maximum and the mean of each count, bucketed and full scan, from the same counters on the same rows. Today's full scan is V + 1 + A·V `token_cell` calls in `writer` (plus V more in the evaluate entry's top-8 scan) and up to K·256 admission evaluations. Two costs are outside B2 and are reported beside it: the rank-telemetry pass (`:330-333`, one evaluation per occupied cell, the same in both writers) and the index update cost per accepted move (§2).

**Writer time per row** on an RTX 5090, bucketed against full scan, is reported, not a bar.

## 4. Base artifact

None. The tables are seeded or pinned (§5), with no checkpoint and no training.

## 5. Data split and panel

**Tables (B = 4 blocks, K = 16 unless stated):**
- **pinned V = 4,096:** `address_structured_v1` (A8), file SHA-256 `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1` (`attention_learning.rs:33`). The file is not committed. It is rebuilt from TRAIN by `address-cells-codebook`, and the run checks the SHA before use. The PrimeZeta table `83c1380a…` (`:31`) is a second pinned row set if the reviewer wants it.
- **seeded V = 4,096 and V = 16,384,** from the same random-vocabulary generator the host tests use (`random_vocab`, a palette of roots plus perturbed non-root blocks), at seeds 1, 2 and 3.
- **No 16,384-piece table or tokenizer exists at this base.** The only 16,384 in the crate is a parameter count (4,096 tokens × 4 blocks, `measurement.rs:213`). This sheet proposes the seeded V = 16,384 table. **Open choice for the reviewer and owner:** the seeded table, or a 16,384-piece table built first under its own item.

**Reads `q`:**
- seeded queries (`random_query`): 4,096 rows per table and seed;
- real DEV reads: the forward pass's `q` on the 14,287-row DEV panel, with the pinned A8 table. This needs a driver path that runs `read_vector` and keeps `q`, as `rows_qualification.rs` and `bvgk_qualification.rs` do for their own modules. **Open choice:** include the real-read panel, or qualify on seeded rows only and defer it.

**Call kinds.** `writer` is called from `discrete_forward` (`:380`, kind 2, the loss kernel), from `discrete_counterfactual` (`:394` baseline kind 2; `:400` the proposal kind 0, 1 or 3, sixteen lanes, gate lanes 0..2) and from `discrete_evaluate` (`:474`, kind 2 with telemetry). All three entries and all four kinds are compared: kind 0 in Route mode and kind 3 in Flock or Relation mode (the other pairing is refused with status 9, `:388`), kinds 1 and 2 in both, each in both `s[11]` modes.

**Edge cases (finite, each pinned by a row):**
- a cell-id tie under `cell_precedes`, including a tie at the K-th admitted slot;
- a token-id tie within one cell and across two admitted cells;
- an ABSENT token (MAX, status 8), including an ABSENT moved token;
- `kind==1` moves into, out of and across admitted cells;
- K = 1 and K = 16, and a vocabulary with fewer than K occupied cells;
- a kind-1 move whose moved token is not the target and whose base cell is admitted, the case where a double-counted moved token shows in `token_rivals`.

**Moves (the index update against a full rebuild).** Each seeded table runs two sequences:
- **Scripted edits.** There are 64 one-token, one-block table edits, written the way `discrete_apply` (`:431`) writes an accepted move and followed by the index update. They are drawn by the run's seed and always include a block-0 and a block-(B−1) edit that change a token's bucket, and an interior-block edit that does not.
- **The real loop.** There are 64 steps of the existing `discrete_counterfactual` → `discrete_apply` loop. Only accepted moves change the table.

After every change, the incrementally updated index must equal a full rebuild entry for entry, and B1 is re-run on the panel rows. The stale-index fault (§8) fires on these sequences. No accepted move can make a token ABSENT: such a proposal's loss is MAX, and `discrete_apply` refuses a proposal that does not lower the loss (status 5). ABSENT is therefore exercised by the kind-1 rows and by tables built with an ABSENT token.

## 6. Source

- `src/bucket_source.rs` and the `build.rs` block (§2), with its admitted entry list: the index build, the per-move update, the bucketed `writer` and the bucketed top-8 scan;
- a device wrapper beside `rows_device.rs`;
- `src/bin/writer-bucket-qualify.rs`, after `rows-qualify.rs`;
- a counters build, compiled separately (a macro, so the plain module has no counter code), adding the three per-row counts. **Control:** the plain and instrumented builds must give identical outputs on every row.
- The rows, rowsk and bvgk modules carry their own generated copies of `writer`. Whether they switch to the bucketed writer is a later item, outside this sheet.

## 7. Steps

1. FIT by a non-author lab.
2. Casey registers the bar in §3.
3. Build the kernel module and its Rust qualification binary. Host checks: the 6 host tests, a stub-nvcc `cargo check`, the PTX admission self-test, and `cargo fmt`.
4. Run on the pod: one RTX 5090 through `scripts/pod/uor-pod`, after `uor-pod status`. Save results off the pod before `down`.
5. Record one row in `docs/research/notebook/m1-discrete-attention.md`.
6. Open a PR with `Closes #2840`.

## 8. Controls and negatives

Each planted fault runs in the instrumented build and must produce a mismatch or a count above the bar:
- the tie rule flipped (`t > best_t`);
- the index stale by one move (the update skipped after one bucket-changing edit in §5's move sequences);
- an ABSENT token ignored by the index build;
- under kind 1, the moved token not skipped in its base bucket, so it is counted twice. This shows only when the base cell is admitted and the moved token is not the target; §5 pins such a row.

On the host (#2983), the flipped tie rule fails 4 of the 6 tests and an ignored ABSENT token fails the ABSENT test. The stale index and the double-counted moved token are new in this qualification. A planted fault that passes is an instrument fault. A negative closes this configuration, never the mechanism (CHARTER §4.1).

## 9. Stop conditions and outcomes

- **Any mismatch is a hard stop,** diagnosed as an instrument or kernel fault (D31-style), not a mechanism negative.
- Owner caps: at most 4 pods and $8/h across labs.
- **Outcomes:**
  - B1 and B2 pass: the writer is replaced;
  - B1 passes, B2 misses: keep it and iterate on the layout;
  - B1 fails: an instrument or kernel fault, so diagnose.

## 10. Cost and minimum test power

**Cost.** Minutes on one RTX 5090. Planning figure: under one hour at $0.40–$0.60/h, so under $1 including build and bootstrap. This is a planning figure, not a cap.

**Test power.** Equality is exhaustive on its rows, so power is coverage. Every edge case in §5 has a row, all four kinds and both `s[11]` modes run, and the four planted faults must each be caught.

## 11. Verdict

### Why NOT_FIT (prerequisites in priority order)

1. **The §3 bar is not registered (`PROPOSED, NOT REGISTERED`).** Under CHARTER §5.1 and §5.2, a reviewer cannot register a bar; the owner/PM must register the §3 `B1`/`B2` bar with an ISO-UTC timestamp and §3 must quote that registration before `FIT` can be granted.
2. **§5 leaves open design choices that change the tested configuration.** Under D22, a `FIT` verdict clears one single, unambiguous configuration. §5 currently leaves two explicit open choices (`Open choice for the reviewer and owner`, plus the optional `PrimeZeta` table):
   - **Reviewer recommendation on Choice 1 (`V = 16,384` vocabulary source):** Pin the **seeded `V = 16,384` table** (`random_vocab`, seeds 1, 2, 3) in this sheet. `B1` and `B2` are exact algorithmic invariants over arbitrary CSR bucket distributions, whereas building a real 16,384-piece tokenizer and corpus is a separate held item (`#2872`).
   - **Reviewer recommendation on Choice 2 (Real-read `DEV` panel & `PrimeZeta`):** Pin **both** the 14,287-row DEV real-read panel (`q` from `read_vector`) on pinned `A8` (`6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`) **and** the pinned `PrimeZeta` table (`83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b`), and include the authoritative full DEV token/mask SHA-256 digests (`0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` / `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`) from `crates/uor-r4-discrete/src/attention_learning.rs`. That tests the GPU bucketed writer on real read vectors across both predecessor-crowded (`A8`) and quota-uniform (`PrimeZeta`) cell distributions at negligible extra GPU time.
3. **Concrete PTX entry inventory and CLI wiring in §6, plus `:326` target `ad` clarification in §3:**
   - In §6, list the exact `.entry` kernel names that `src/bucket_source.rs` will admit via `admit_ptx_inventory` and the exact `writer-bucket-qualify` CLI invocation.
   - In §3 ("What B2 counts"), state explicitly whether `ad = address_distance(q, g, s, target_cell)` at `discrete.cu:326` reuses the admission pass's `distance[target_cell]` (0 extra calls) or is counted outside the 256-call admission-pass bar alongside `:330-333`.

Everything else checked and holds: all 8 required blocks are present (`prep.sheet_shape`); every `discrete.cu` line number (`:20-34`, `:270-496`), call-count formula (`V + 1 + A*V = 69,633` at `V = 4,096, A = K = 16`), and table SHA-256 (`6600bdb1…` and `83c1380a…` in `attention_learning.rs:31-33`; `16384` parameter inventory in `measurement.rs:213`) matches `origin/main` verbatim; `crates/uor-r4-discrete/tests/writer_bucket_controls.rs` (`e10d10926`, `#2983`) passes all 6 finite host controls; the path uses pure integer math compiled ahead-of-time via `nvcc` (`build.rs`) with zero float, squashing, or matmul instructions; and the 4 planted faults in §8 are each verified to fail `B1` or `B2`.

Re-request FIT once the bar is registered and §§3, 5, and 6 pin one configuration; this verdict clears no compute (D22).

### Recorded review

```text
TEST FITNESS: NOT_FIT - M1 #2029 m1-faster-writer
sheet: docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md
reviewer: antigravity (non-author; fresh-context FIT review for #2840 at 47d781a49)
note: Prospective finite qualification design, math, citations, and 4 planted faults are sound and verified line-by-line against discrete.cu (lines 20-34, 270-496), writer_bucket_controls.rs (6/6 host controls in e10d10926), attention_learning.rs, measurement.rs, and build.rs. NOT_FIT because the configuration and bar are not yet pinned: (1) section 3's B1/B2 bar is marked PROPOSED, NOT REGISTERED and awaits owner/PM registration with an ISO-UTC timestamp (CHARTER 5.1/5.2); (2) section 5 leaves two open design choices (seeded V=16,384 vs a built 16,384-piece table; including the 14,287-row DEV real-read panel on A8 vs seeded queries only, plus optional PrimeZeta 83c1380a...) - reviewer recommends pinning seeded V=16,384 plus the 14,287-row DEV real-read panel (with full DEV token/mask SHA-256 pins) on both A8 and PrimeZeta; (3) section 6 names src/bucket_source.rs and src/bin/writer-bucket-qualify.rs (to be built in Unit 3) without listing the exact admitted PTX .entry inventory or CLI argv, and section 3 should state whether target ad = address_distance(q,g,s,target_cell) at discrete.cu:326 reuses the admission distance array or is counted outside the 256-call admission bar. Re-request FIT once the bar is registered and sections 3, 5, and 6 pin one configuration; this verdict clears no compute (D22).
date: 2026-10-23
A FIT verdict clears compute for the configuration in that sheet only (D22).
```
