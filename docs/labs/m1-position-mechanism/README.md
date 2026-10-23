# M1 position mechanism: prefix-holonomy transport on the discrete 64-frame / 16-root orbit

- **Date:** 2026-10-22
- **Lab:** `antigravity`
- **Issue:** `#2843` (`m1-position-mechanism`)
- **Decision card:** `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337522273`)
- **Pre-registration:** [`docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md`](../../harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md) (`TEST FITNESS: PENDING`)
- **Code:** [`crates/uor-r4-discrete/src/position_mechanism.rs`](../../../crates/uor-r4-discrete/src/position_mechanism.rs)

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

Non-author `TEST FITNESS` review of the draft sheet: **NOT_FIT**, recorded in the sheet's `## Verdict`. No compute is cleared. The three pinned prerequisites:
1. owner-registered bar (the linked `#2028` comment is antigravity's decision card `6337522273`, not an owner answer);
2. the full DEV token/mask pin — the sheet must use the authoritative code pin in `attention_learning.rs` (`0919e90f953e…` / `a838bb1a67e9…`; the staging sheet's `0919e90f813b…` / `a838bb1ae20c…` pair appears nowhere else and needs an errata);
3. a deterministic definition and SHA-256 for the new multi-occurrence order-sensitive slice (`N ≥ 200`).

Verified at the review head: `cargo test -p uor-r4-discrete --lib position_mechanism` = 5 passed; no float, dense matmul or squashing form on the path; the step table exports as `16 × B` `u8` words; the parent A8 sheet is `TEST FITNESS: FIT`; all 8 blocks present.
