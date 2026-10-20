# Stage M2 Trained-In `ReadSpec::Relation` vs. `ReadSpec::Route` Discriminator Pre-Registration (`2026-10-15`)

- **Lab / Seat:** `antigravity` (`ag1`)
- **Milestones:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) and M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029), tracked under [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028)
- **Authority:** Owner Decision [D34](../../integration/DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate) (`4cef409fc`), Claude's independent adversarial review [#2482](https://github.com/UOR-Foundation/uor-r4/pull/2482) ([`docs/labs/spiralcore-attention-independent-review-2026-10-14/README.md`](../spiralcore-attention-independent-review-2026-10-14/README.md) §6, §8, §9 item 3, §10, §11), Stage M1 finite CUDA construction qualification pre-registration [#2494](https://github.com/UOR-Foundation/uor-r4/pull/2494) (`docs/harness/prep/m1-finite-relational-head-2026-10-14.md`, `TEST FITNESS: FIT` by Claude at `4f0181b2c`), and Codex's Route learning-slot receipt analyzer [#2492](https://github.com/UOR-Foundation/uor-r4/pull/2492) (`docs/labs/route-learning-slot-2026-10-15/README.md`)
- **Deliverable:** [`docs/harness/prep/m4-finite-relational-head-2026-10-15.md`](../../harness/prep/m4-finite-relational-head-2026-10-15.md) (`TEST FITNESS: PENDING`, docs-only, zero compute, zero qualification, zero hold changes)

---

## 1. Summary of What Was Pre-Registered

Following Stage M1 finite CUDA construction qualification pre-registration ([#2494](https://github.com/UOR-Foundation/uor-r4/pull/2494), `TEST FITNESS: FIT` by Claude at `4f0181b2c`), this delivery prepares the **Stage M2 trained-in corpus discriminator pre-registration** in the [D34](../../integration/DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate) reranker-first attention sequence:

1. **Registered Learner Change (`SchedulePolicy::InterleavedReadEvery8V1`, `training.rs:30-35, 225-286`):**
   - Under `CanonicalVectorThenReadV1`, `PreparedDriver::coordinate(index)` proposes `VectorBlock` for batches `0..16,383` and Read parameters (`Route` or `Gate`) only in the final `20` batches (`16,384..16,403`).
   - `SchedulePolicy::InterleavedReadEvery8V1` (`"interleaved_read_every_8_v1"`) keeps the exact `schedule_cycle = V * B + B * S = 16,404`-batch pass (`1,049,856` masked targets in identical order, with `next_coordinate = (completed_batches % schedule_cycle) as u32` while `parameter_count` is `16,404` for `Route` and `16,429` for `ReadSpec::Relation`) and quarter checkpoints `0 / 4,101 / 8,202 / 12,303 / 16,404`, mapping every 8th batch ($s \bmod 8 == 7$) to the next read coordinate $r = \lfloor s / 8 \rfloor \bmod N_{\text{read}}$ (`2,050` read proposals per pass: `512 / 1,025 / 1,537 / 2,050` cumulative at Q1–Q4) and the remaining `14,354` batches ($s \bmod 8 \ne 7$) to $v = (s - \lfloor s / 8 \rfloor) \bmod (V \times B)$ (`3,589 / 7,177 / 10,766 / 14,354` cumulative at Q1–Q4).
   - In `ReadSpec::Route` ($N_{\text{read}} = 20$), `Route(0..10)` are proposed `103` times each and `Route(10..20)` `102` times each (`2,050` total).
   - In `ReadSpec::Relation { kept: 8, sweeps: 1 }` ($N_{\text{read}} = 45$), `Gate(0..25)` are proposed `46` times each and `Gate(25..45)` `45` times each, yielding **`920` level-combine gate proposals** (`Gate(0..20)`) and **`1,130` relation-trit proposals** (`Gate(20..45)`, `2,050` total).
   - Also documents the `18,432`-batch full-vector coverage option (`s % 9 == 8`, `16,384` `VectorBlock` + `2,048` Read = `18,432` batches, quarters `4,608 / 9,216 / 13,824 / 18,432`) so Claude and the owner can choose between exact `16,404`-batch dose parity and full `16,384`-vector coverage during `TEST FITNESS` review.
2. **Pre-Run Schedule & Sub-Panel Instrument Controls (`S1`–`S4`, `E1`–`E3`, `0` updates):**
   - `S1` (`interleaved_schedule_24_step_fixture`): 24-step fixture (`V = 10, B = 2, C = 3, S = 2`, `schedule_cycle = 24`, `parameter_count = 24` for `Route`/`Flock` and `34` for `Relation`) verifying exact slot assignment and `next_coordinate = (completed_batches % schedule_cycle) as u32` round-trip to `Cursor { completed_batches: 24, accepted_updates: 0, corpus_pass: 12, next_target: 0, next_coordinate: 0 }`;
   - `S2` (`interleaved_schedule_c192_counts_and_wrap`): `C192 / B4 / V4096` quarter and full-pass coordinate frequencies and `16,404 -> 0` cursor wrap;
   - `S3` (`lineage_and_cursor_refusal_across_schedules`): cross-schedule and cross-`read_spec` checkpoint refusals;
   - `S4` (`split_gate_family_counters`): disjoint per-family counters (`VectorBlock`, `Route`, `GateCombine`, `GateRelation`) in `steps.jsonl` / `batches.jsonl` and `report.json` / `audit.json`;
   - `E1` (`copy_successor_subpanel_partition_exact`): reproduces exact `ft/dev` counts (`0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` / `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`: `14,287` eligible targets, `8,648` query-recurrence rows, **`3,847` copy-successor rows $\mathcal{P}_{\text{succ}}$** with distance buckets `[50, 2872, 900, 25]` and `0` rows at $d = 1$ so 100% of $\mathcal{P}_{\text{succ}}$ rows have $j^* + 1 \le p - 2$ inside the causally eligible candidate pool, `5,182` any-occurrence successor rows, `10,440` complement rows);
   - `E2` (`subpanel_aggregates_add_to_full_panel`): bit-exact additive partition of `top1_correct`, `top8_correct`, `B3 = admitted_cell_count_sum - address_rival_count_sum`, hinge sums, and `admitted_strict_wins` across $\mathcal{P}_{\text{succ}} \sqcup \mathcal{P}_{\text{comp}} = \mathcal{P}_{\text{full}}$ (verified on sealed seed-1 receipts: cursor `0` Full `17 / 173 / 5680` = $\mathcal{P}_{\text{succ}}$ `1 / 68 / 1658` + $\mathcal{P}_{\text{comp}}$ `16 / 105 / 4022`; Q1 `4,101` Full `96 / 1090 / 2201` = $\mathcal{P}_{\text{succ}}$ `21 / 239 / 500` + $\mathcal{P}_{\text{comp}}$ `75 / 851 / 1701`; Q2 `8,202` Full `94 / 967 / 2131` = $\mathcal{P}_{\text{succ}}$ `18 / 222 / 487` + $\mathcal{P}_{\text{comp}}$ `76 / 745 / 1644`; Q3 `12,303` Full `96 / 953 / 1954` = $\mathcal{P}_{\text{succ}}$ `22 / 213 / 424` + $\mathcal{P}_{\text{comp}}$ `74 / 740 / 1530`);
   - `E3` (`zero_trit_attribution_snapshot_invariance`): verifies the in-memory `gates[20..45] = 0` attribution copy.
3. **Matched Stage M2 Corpus Discriminator Bar (`B1`–`B3` at Cursor `16,404`):**
   - `B1` (**Copy-successor sub-panel top-8 gain above seed noise**): $\text{top8}_{\mathcal{P}_{\text{succ}}}(\text{Relation-interleaved}) - \text{top8}_{\mathcal{P}_{\text{succ}}}(\text{Route-interleaved}) > \Delta_{\text{seed}}(\mathcal{P}_{\text{succ}}) = \max(|\text{seed } 1 - \text{seed } 2|, 39\text{ rows})$;
   - `B2` (**Full-panel non-regression guard on `14,287` rows**): no regression on `top1_correct`, `top8_correct`, or `B3` beyond $\Delta_{\text{seed}}(\mathcal{P}_{\text{full}})$;
   - `B3` (**Zero-trit attribution check at `16,404`, no retraining**): $\text{top8}_{\mathcal{P}_{\text{succ}}}(\text{Relation-interleaved}_{16,404}) > \text{top8}_{\mathcal{P}_{\text{succ}}}(\text{Relation-interleaved}_{16,404}\text{ with } g[e,l]=0)$ and `accepted_updates(GateRelation) >= 1`;
   - **Fresh transfer on `KEEP` (D22 §4):** zero-update `discrete-stack eval-corpus` evaluation on a disjoint held-out slice of $\ge 2,000$ targets ($\ge 500$ copy-successor targets).
4. **Hard Pre-Execution Gates (`G1`–`G5`):**
   - Execution stays strictly **`HELD`** behind: **(G1)** Stage M1 construction `PASS` (`#2494`), **(G2)** seed-1 cursor `16,404` `Route` learning-slot observation (`#2492`), **(G3)** `S1`–`S4` & `E1`–`E3` `PASS`, **(G4)** Claude `TEST FITNESS: FIT` on `#2032`, and **(G5)** explicit owner personal hold release.

---

## 2. Verification & Governance Receipts

- `uor-harness prep-check --json`: `m4-finite-relational-head-2026-10-15.md` reports `"fitness": "PENDING", "complete": true, "may_compute": false, "missing": []` across all 8 required blocks (`bar`, `base`, `split`, `steps`, `stop`, `cost`, `power`, `fit`).
- `uor-harness ratchet d23` & `uor-harness ratchet d25`: `0` violations.
- `uor-harness check-formats` & `uor-harness shard-check`: `ok: true`.
- `python3 scripts/check_claim_wording.py`: `0` violations.
- Zero model/CUDA compute, zero pod leases, zero head adoption, and zero hold changes occurred.
