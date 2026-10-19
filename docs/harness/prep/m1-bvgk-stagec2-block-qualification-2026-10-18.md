# BVG-K Stage C2: block propose, verify and apply — device-level GPU qualification — pre-registration

- **Line:** M1 W3 BVG-K (#2029). **Lab:** claude. **Parent design:** `m1-batched-verified-coordinates-DRAFT-2026-10-18.md` §2, §4 and §8 step 3.
- **Source:** C2 pieces H1 (#2716), H2a (#2715), H2b (#2717) and H3 (#2719) are merged on main. The qualifier is `bvgk-qualify` (`crates/uor-r4-discrete/src/bvgk_qualification.rs`).
- **What runs:** finite GPU controls on **synthetic fixture devices only**.
  - No corpus, no driver transition, no training.
  - The owner approved this scope in the Claude chat (2026-10-18, about 23:39Z): "Yes, run on FIT", for a qualifier that "commits updates only on small synthetic fixture models (no corpus, no training)".
- **Commit path used:** only the new BVG-K `apply_block`, on fixture devices.
  - **The guarded single-move commit path (`Device::apply`) is never called.** Its doc comment ("finite proposal qualification must not call this method") is unchanged (owner direction, Claude chat, 2026-10-18).
  - Every fixture commit is counted in `applied_updates`.
- **Rules:**
  - D22 and D24: GPU only; the host compares device results.
  - D23, D25 and D30: integer CUDA only, D29 PASS.
  - D40: no owner phrase is involved.
  - The training hold and the M2 hold stay **HELD**.

## Bar (every control must PASS; any FAIL stops for diagnosis, D31)

- **V0, identity.** `bvgk.cu` begins with `rowsk.cu`, and `rowsk.cu` begins with `rows.cu`. The receipt records the rows, rowsk and bvgk image hashes and `driver_source_sha256`.
- **RC, re-carry parity.** On an independent fixture device restored from the same snapshot, `carry_windows` of the same windows reproduces the pre-state rows-K losses exactly on every lane. This holds for batches built by `carry` and by `carry_windows`.
- **UT, original untouched.** Table, routes, gates and epoch are byte-identical before and after every `propose_block` with |S_t| ≥ 2. The joint verify runs on the scratch clone only.
- **JR, joint reference.** For every |S_t| ≥ 2 proposal, `l_joint` equals an independent reference, in exact integer equality. The reference is built as follows:
  1. Each S_t member is applied alone as a K = 1 `propose_block` and `apply_block`, on its own fixture device restored from the pre-state.
  2. The member deltas are overlaid on the pre-state. Disjoint storage is required; two members writing the same word is a FAIL.
  3. The composed state is re-carried and scored, and lane 0 of `[best]` is the reference.
- **DC, decision coverage (V4, V5).** The sweep covers both kinds, K ∈ {2, 4, 16}, rows {7, 64}, seeds 7 and 11, and objectives A and R. It uses **every** chunk of each kind's inventory at each K, under three eligibility patterns that cycle by chunk: all eligible, every third coordinate withheld, and a single eligible coordinate.
  - **Expected members, two ways.** The expected members come from the device's own rule, as K = 1 blocks on an independent reference. They are also checked **independently of that shared rule**, against the raw C1 lane losses and the mask: every member must be eligible, have C1 status 0, and have some lane strictly below lane 0. An ineligible coordinate is never a member.
  - **Coverage counts.** Coverage counts proposals. Joint-wins and joint-loses-or-ties need |S_t| ≥ 2.

  For every proposal:
  - the record matches the decision rule (empty → no commit; |S_t| = 1 → {best}; |S_t| ≥ 2 → S_t iff `l_joint < l_best`, else {best});
  - a joint commit has commit mask = member mask;
  - after `apply_block`, a fresh lane-0 score equals `committed_loss`, and it is `< l_before` whenever a commit happened (V5).

  **Coverage is required.** Each of empty, single, joint-wins and joint-loses-or-ties must be observed at least once. A category never observed is `UNDERPOWERED`, an instrument blocker and not a negative. It is repaired by a posted one-line amendment that widens the sweep.
- **V3, block apply equals forced sequential apply.** For every joint commit, the table, routes and gates after `apply_block` equal the JR overlay composition. The epoch is compared separately: +1 for the block against +|S| summed over the single references.
- **RF, refusals.** Each must be refused with its own message and leave state unchanged:
  - K = 0 and K = 65;
  - a mixed-kind block;
  - an eligibility-length mismatch;
  - a repeated parameter;
  - foreign BVG-K kernels from another fixture device (a fresh binding);
  - a proposal applied on another device;
  - a stale proposal (epoch moved);
  - a repeated apply of the same proposal.
- **C1R, C1 regression.** The exact C1 V2 parity sweep is rerun on the split `score_rows_k` (`v2_scoring_parity_counts`). It must stay exact.
- **TP, throughput (diagnostic only, never a gate).**
  - Shape: V4096 / B4 / C192 / K16, Route, objective A, seed 1, checkpoint 0, 64 rows.
  - Calls: `propose_block` at K ∈ {1, 4, 16} and the single `propose_rows` (scoring only).
  - Sampling: 1 warmup + 20 timed, median, nearest-rank p95, all raw samples. The receipt records whether |S_t| ≥ 2 (scratch-clone cost) occurred.
  - No Stage T bar follows from it.

## What this redesign leaves uncovered (stated, not claimed)

1. **The BVG-K apply kernel against the ordinary single-move kernel on GPU.**
   - V3 and JR compare `apply_block` of S_t with `apply_block` of single members: the same commit kernel on both sides, at K = |S| and K = 1.
   - What links the BVG-K writes to the qualified ordinary `discrete_apply` writes is only the H2a host drift-pins (the move, proposal and gate writes are text-identical). No GPU control compares them.
2. **K = 1 identity with today's law (V1).** That the block path at K = 1 reproduces `transition` bitwise needs the ordinary commit path and a driver transition. Both are out of scope here.
3. **Driver-level controls.**
   - The checks are: `transition_block` / `run_block_transitions` accounting (`epoch == accepted_updates`, `committed_moves`), FrozenZeroV1 through the driver's policy mask, the all-ineligible no-op batch, and V9 checkpoint and resume.
   - Every driver transition requires a `CorpusUpdateAdmission`, which only an owner-released pilot dose (1640 or 16404) can issue. No fixture-only admission path is added.
   - These become **pre-registered first-batch controls of any future owner-released Stage T run**, and must pass there before any Stage T result is read.
4. **Shared measurement paths.**
   - JR's reference and the device's `l_joint` both re-carry the windows on a restored or cloned state, through `carry_windows`. A re-carry fault common to both is covered only by RC, which checks re-carry on the pre-state rather than on a post-commit state.
   - `score_rows_k` scores `l_before`, the lane-0 references and DC's independent lane check. A fault common to all of them is covered only by C1R, the C1 parity rerun against `propose_rows`.
5. **Corpus-scale behaviour.** Fixtures are V16 / B2–B4 / C12. The K16 shape appears only in the diagnostic TP. Admission, occupancy and learning effects at Dose 1 are Stage T questions.

## Base artifact, data split and cost

- **Base artifact:** none. Fresh `Device::new_with_objective` fixtures with deterministic seeds, plus independent devices restored from their snapshots.
- **Data:** deterministic integer windows and targets from a fixed LCG. No corpus rows.
- **Cost:** under 45 min on one RTX 5090, about $0.75.
  - The build takes about 10 min.
  - The controls are small-shape launches.
  - TP is 3 × 21 block calls plus 21 single calls at about 125 ms each.

## Steps

1. Merge this sheet and the qualifier after exact-head review.
2. Get TEST FITNESS from a fresh-context adversarial reviewer subagent. Owner direction makes these replace cross-lab FIT for Claude lanes. Every finding is posted on #2029.
3. Run on one RTX 5090 via `uor-pod`, at the merged head:
   1. the D29 checker;
   2. `cargo build --release --locked --features cuda -p uor-r4-discrete --bin bvgk-qualify`;
   3. `bvgk-qualify <fresh exclusive dir>`, which writes `bvgk-qualification.json` and exits non-zero on any FAIL.
4. Post the receipt and its SHA, archive the run, and delete the pod.

## Stop rule

- Any FAIL stops for diagnosis. It is never a negative.
- `UNDERPOWERED` and CUDA or build faults are instrument blockers.
- Owner caps and the disk floor are hard stops.

## Minimum test power

- These are finite exact equalities. One disagreement fails its control.
- Power for decision coverage is enforced by the coverage requirement in DC. Power for JR comes from requiring at least one |S_t| ≥ 2 proposal (joint-wins and joint-loses-or-ties both qualify), and power for V3 from at least one joint commit (joint-wins).

TEST FITNESS: FIT (owner direction 2026-10-18 in the Claude chat: Claude's fresh-context adversarial reviewer subagents replace cross-lab FIT for Claude lanes). The review returned FIT-WITH-FIXES, and every fix is applied:
- F1, the mixed-kind refusal text, was verified against `rowsk_device.rs`, where the refusal is raised.
- F2 adds DC's independent membership checks against the raw C1 lanes and the eligibility mask.
- F3 widens the sweep to every chunk and three eligibility patterns.
- F4 and F5 correct the sheet text and the uncovered list.

The owner approved the compute for this fixture-only qualifier in the same chat.
