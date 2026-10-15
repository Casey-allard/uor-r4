# #2243 — Invariant-addressed router gate (read_router) in the read forward (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T06:30:53Z
- **Updated:** 2026-10-12T06:37:20Z
- **Closed:** 2026-10-12T06:34:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2243
- **Merged:** 2026-10-12T06:34:13Z
- **Merge commit:** 49174351a971aa2b64e9157cdd596926c3f97860
- **Base / Head:** main / UOR-Foundation:deepseek/invariant-router
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line - count 0/3 - headline before to after: reply 24/232 + memory 24/40 to (this cycle) reported per clause; wrong-value failures of the routed read vs full softmax.

Prep slice (prep 1 of at most 2) for the invariant-addressed read. This PR puts the router gate behind the geometric read; it prepares the nine pre-registered fits (read_router=on / off / on+tangent-null x 3 data-order seeds, 8,000 steps) that the next slice runs on one 2-GPU pod.

What it does (all in crates/uor-r4-training):
- StackConfig.read_router: Option<RouteConfig> (serde default, skip_serializing_if = Option::is_none, the select field's pattern), written to and read back from config.json.
- StackModel::set_read_router(Option<RouteConfig>) / read_router(), refusing a non-geometric arch, a read-less pattern, and a flock select (two selection rules would be ambiguous).
- In geometric_read_with_source, when the router is active the read softmaxes over the precomputed kept positions (the query's own position + the nearest cells on the spanning tree) instead of the whole window; the score, NoRead slot, age table and softmax are unchanged, so a kept set covering the row is the full read bit for bit.
- ids through ReadSource (populated at the forward entry points); a routed forward without tokens is a hard error naming the router, never a silent fallback.
- stack_address.rs: Address::of_ident (cell = token % cells, representative = position_address), RouteConfig::balanced, routed_keep, serde on RouteConfig/RouteNull; the tree walk is O((depth + branching) * window) per row, reported honestly, not O(1).
- examples/geometric-stack.rs parses read_router=on|off,k,buckets,cells,inner_bits,null,step,offset and applies it next to read_override; settings record read_router{,_k,_buckets,_cells,_inner_bits,_null,_step}; read_override moved out of the supervision-only branch.

Field name: read_router=, not read_route= (which is Claude's starling read). read_router=off is the default everywhere.

Tests (all passing locally):
- off state reproduces the unrouted loss bit for bit (same model, two configs);
- k=window routed loss equals the full read's loss bit for bit;
- tangent holds cells / normal moves them; routed_keep at k=window keeps the whole prefix; config.json round-trips the router and null.

Checks: cargo check --lib and --examples pass; cargo fmt clean; 21 stack_address + 3 read_router tests pass. Clippy on uor-r4-training has 3 pre-existing errors in b3_e8_codecs.rs (0.7071 vs FRAC_1_SQRT_2), unrelated to this change. CI does not compile --features cuda, so the CUDA compile check is done on the pod in the next slice.

References #2029 (never Closes).

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6119046607 — 2026-10-12T06:33:12Z — Casey-allard

**Exact-head checks (self-review, D19) at head 5146568d2:**

- cargo check -p uor-r4-training --lib — pass
- cargo check -p uor-r4-training --examples — pass
- cargo fmt -p uor-r4-training — clean
- cargo test -p uor-r4-training --lib read_router — 3/3 pass: off-state loss bit-for-bit, k=window routed loss == full read bit-for-bit, config.json round-trip
- cargo test -p uor-r4-training --lib stack_address — 21/21 pass (tangent holds cells / normal moves them; routed_keep k=window keeps the whole prefix; balanced tree spans every cell)
- Clippy: this change is clean; 3 pre-existing errors in b3_e8_codecs.rs (0.7071 vs FRAC_1_SQRT_2), unrelated to this change.
- CUDA: not compiled by CI (no --features cuda); the pod bootstrap in the next slice is the CUDA compile check — never "CI passed" as the CUDA evidence.


---
_Comment/review ids in this file: 6119046607_

