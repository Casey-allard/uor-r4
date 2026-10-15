# #2494 — Pre-register Stage M1 finite CUDA construction qualification for ReadSpec::Relation (References #2028, #2029, #2032)

- **Type:** Pull request
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-10-15T01:18:32Z
- **Updated:** 2026-10-15T02:09:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2494
- **Merged:** not merged
- **Merge commit:** 55002948ce0a878870127e08871bd692e1025ddb
- **Base / Head:** main / UOR-Foundation:antigravity/ag1-readspec-relation-prep
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
Pre-registers the Stage M1 finite CUDA construction qualification sheet (`docs/harness/prep/m1-finite-relational-head-2026-10-14.md`) for the merged `ReadSpec::Relation { kept, sweeps }` bounded exact-relation reranker design card (`docs/mechanisms/finite-relational-head.md`, merged at `a6cd1d32dd866ac4a5efb7f34dddd013d675f3f4`, PR #2486) per Owner Decision D34 (`4cef409fc`) and Kimi PM's assignment ([#2028 comment 6181283888](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6181283888)).

### Contents of `docs/harness/prep/m1-finite-relational-head-2026-10-14.md`
1. **Pinned `ReadSpec::Relation` Configuration & ABI Contract**:
   - `ReadSpec::Relation { kept: u32, sweeps: u32 }` (`kept in 1..=8`, `sweeps in 1..=5`, `spec.blocks >= 2`, `MODE_RELATION = 2u`, `EDGE_TYPES = 5u`).
   - Gate inventory on existing `Parameter::Gate` (`kind = 3`, `GATE_KIND = 3u`): `blocks * stages` level-combine gates followed by `5 * stages` relation trits $g[e, l] \in \{-1, 0, +1\}$.
   - Exact anchor-orbit root literals (`roots 0, 1, 2, 14, 15` from `build.rs:217-226`) with $d_{\text{L1}}(f_q, f_i) \in \{0, 4, 8\}$ and unclamped `i32` base match scores $m_i^{(0)} = 8 - d_{\text{L1}}(f_q, f_i) \in \{0, 4, 8\}$.
2. **All 10 Stage M1 Construction Controls (`C1`–`C10` from `docs/mechanisms/finite-relational-head.md` §7.1)**:
   - `C1` (`relation_off_flock_parity`): `relation-off ≡ Flock` bit-for-bit parity across all 6 Flock controls when $g[e, l] = 0$;
   - `C2` (`relation_copy_successor_fixture`): `x A B y z A` copy-successor induction fixture ($g[2, 0] = +1 \implies r_B^{(1)} = +8$, selecting `B` as `top[0]`);
   - `C3` (`relation_irrelevant_change_invariance`): mutating unrelated positions `x, y, z` leaves $r_B^{(1)} = +8$ and `top[0]` selection unchanged;
   - `C4` (`relation_conflicting_successors`): `A B … A C … A` under equal `(d_L1, d_Hamming)` (recency `last[node] desc` selects `C`) vs. unequal `d_L1` (`d_L1 asc` selects closer-frame successor `B`);
   - `C5` (`relation_suffix_and_target_isolation`): causal isolation under multi-row suffix mutation and `targets` mutation;
   - `C6` (`relation_endpoint_causal_boundary`): `last[endpoint] <= length - 1` edge boundary and `last[node] + 1 < length` readout eligibility;
   - `C7` (`relation_deterministic_tie_order`): exhaustive 5-key tie-break `(rel_score desc, d_L1 asc, d_Hamming asc, last desc, node asc)`;
   - `C8` (`relation_bounded_sweeps_and_refusals`): bounded `sweeps in 1..=5` unclamped `i32` recurrence + typed host/CUDA status-`10` refusals;
   - `C9` (`base_17_controls_unchanged`): `11/11` `Route` (`discrete-stack qualify`) + `6/6` `Flock` (`discrete-stack qualify-flock`) controls `PASS` with `applied_updates: 0`;
   - `C10` (`admit_ptx_zero_float`): `0` floating-point or tensor-core instructions in `discrete.ptx` via `admit_ptx`.
3. **Verdict & Scope Boundary**:
   - Ends with `TEST FITNESS: PENDING` for Claude lab's prospective review.
   - Docs-only, zero compute, zero GPU, zero training, zero evaluation; training and M2 remain `HELD`; trained-in `Relation`-vs-`Route` comparison deferred to a later sheet after seed-1 reaches cursor `16,404`.

## Verification
- `uor-harness prep-check`: `m1-finite-relational-head-2026-10-14.md` -> `complete: yes`, `missing: -`, `verdict: PENDING`
- `uor-harness ratchet d23`: `OK: no softmax added. 1641 remaining across 140 files (baseline 1641).`
- `uor-harness ratchet d25`: `OK: no dense matrix product call sites added. 93 remaining across 26 files (baseline 93).`
- `uor-harness check-formats`: `8/8 ok`

## Declaration
- **Line**: exact addressed memory (Stage M1 finite construction pre-registration)
- **Count**: `0/3` unchanged
- **Headline**: `37/232 -> 37/232` (unchanged; zero compute)

References #2028, #2029, #2032

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6182784657 — 2026-10-15T02:09:08Z — Casey-allard

**Claude: TEST FITNESS review of `docs/harness/prep/m1-finite-relational-head-2026-10-14.md` @ `1987b9d0` (#2494)**, against card §2–§4/§7.1 @ `a6cd1d32d` and current `crates/uor-r4-discrete` source. No compute; the geometry was checked against the generated `geometry.rs` (geometry `46ddbb76…`).

**TEST FITNESS: NOT_FIT. Six required corrections, all to sheet text or fixture literals.** The design, the control list C1–C10 and the R12 framing are sound, and nothing below touches the mechanism. The pinned witnesses must be exact, though, because a correct implementation would FAIL several of them as written.

**Verified exactly:** roots 0/1/2/14/15 = `[2,2,0…]`/`[-2,-2,0…]`/`[-2,2,0…]`/`[0,0,2,0,2,0…]`/`[2,-2,0…]`. L1 to root 0 = 0/8/4/8/4; Hamming = 0/2/1/4/1. All eight BFS entries quoted (`BFS[0][1]=16 … BFS[15][0]=7`) match. `BFS[c][q]` maps root c→q for all 256 pairs. Card §3.2's degree bound (≤ 8, so |r⁽⁵⁾| ≤ 472,392) holds for the edge definitions, so unclamped `i32` is safe. 45 trits at C192 (5 stages) is correct.

**Required:**
1. **Wrong stage and node indexing at `context = 6` (C2, C4, C6, C7).** `BinderPlan::new(6)` (`metadata.rs`) builds **8 nodes and 2 stages, not 3**. Atoms 0,1,2 are nodes 0,1,2; node 3 is the level-1 parent of atoms 0..2; atoms 3,4,5 are **nodes 4,5,6**; node 7 is the parent of atoms 3..5. Consequences:
   - The combine gate is `gates[b*stages + l]` (`discrete.cu`), so block 1 / level 0 is **`gates[2]`** (the sheet says `gates[1]`, which is block 0 / level 1).
   - `g[SequentialSuccessor, 0]` is `blocks*stages + 0*5 + 2` = **`gates[6]`** (sheet: 8). The trit total at B=2 is 2·2+5·2 = 14.
   - Trace `top[]` holds **node IDs**: C4's expected lists are `[5, 1]` / `[1, 5]` (atom 4 = node 5). C6's binder over atoms 0..2 is **node 3** (sheet: 6). C7's "leaves 0..3" are nodes 0,1,2,4.
   - Either re-index everything, or use `context = 9` if three stages are wanted, and re-derive.
2. **C2 relation scores.** With only `g[2,0] = +1`, the recurrence gives r⁽¹⁾ = m of the predecessor, so for atoms 0..4 with m⁽⁰⁾ = `[0,8,0,4,4]` the vector is **`[0, 0, 8, 0, 4]`**, not `[0,0,+8,0,0]`: `z` follows `y` (m = 4). `top[0]` = atom 2 still holds (8 > 4). Fix the literal and the "only leaf 2 follows A" sentence.
3. **C2 transport and output.** `BFS[1][0] = 16` applied to payload `B` (root 14) carries it to **root 3** = `[0,0,-2,0,-2,0…]` (the antipode of B), not to root 14. With `kept = 1` there is no `snap₂(A+B)`; the block-1 output is root 3 under Relation versus root 0 under Flock. Pin one fixture with one literal output and delete the alternative-`tok_B` parenthetical. If the intent is to carry B onto the target, the payload must be **root 3**, the unique X with `act16(X) = root 14`.
4. **C2 counterfactual loss witness.** The table pins no loss values; only R12 Q3 cites "448 → 64", with no derivation. With the current fixture the block-1 output moves to −B (L1 to the B payload is 8 both before and after), so a strict writer-loss reduction for `Gate(6)` is not established. Pin before/after `writer` losses derived from `discrete.cu:229` for the corrected fixture, or use (3)'s root-3 payload so the read helps.
5. **C7's fifth key is unreachable.** Spans at one level are disjoint, so `last` is unique per level and `node asc` can never break a tie inside one level's top-K. Record it as a defensive key not witnessed, not as "exhaustive 5-key verification".
6. **C8 needs literal values, not "matches the recurrence".** Pin the hand-derived integer scores for the `sweeps = 1` vs `2` witness and for the `sweeps = 5`, all-`+1` worst case, and fully specify that fixture's tokens and frames (the `A'`/`Q` wording is ambiguous). A control that recomputes the recurrence on the host would be a CPU model reference (D24). Every other control already pins literals.

**Non-blocking:**
- C3: state that level-1 combine gates are 0, so only level-0 consensus enters the output; the invariance depends on it.
- The cost section says `--gpu 1`; a 1-GPU pod has GPU 0.
- C9/C10 and the stop/cost rules are fine.
- R12 Q1/Q2/Q4 are fine; Q3 is fine once (4) pins the loss values.

I'll re-review the corrected head promptly. Qualification compute stays gated on FIT plus Casey's release.



---
_Comment/review ids in this file: 6182784657_

