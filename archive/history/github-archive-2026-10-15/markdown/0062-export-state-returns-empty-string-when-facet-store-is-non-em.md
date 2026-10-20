# #62 — export_state returns empty string when facet store is non-empty

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-22T19:46:25Z
- **Updated:** 2026-07-22T20:13:42Z
- **Closed:** 2026-07-22T20:12:46Z
- **Labels:** bug
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/62
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Found during #12 work (pre-existing). UorR4Router::export_state serializes self with serde_json::to_string and unwrap_or_default(); MultiFacetStore uses HashMap<Vec<u32>, _> whose keys cannot serialize as JSON object keys, so serialization errors and the unwrap_or_default silently returns "". The default corpus always populates the facet store, so export_state() effectively always returns empty today. Fix options: serialize the facet store as an array of (key, values) pairs, or skip it with serde(skip) + explicit re-export path. crates/uor-r4-router/src/lib.rs.

## Comments (1)

### comment 5050981107 — 2026-07-22T20:13:42Z — auser

Fixed in 60d8d28. MultiFacetStore's six HashMap<Vec<u32>, Vec<u64>> fields now serialize via a serde(with) helper encoding each map as key-sorted (key, values) pairs — round-trip safe and deterministic across runs. The round-trip test exercises a populated facet store instead of clearing it. (Same commit also swept up the in-flight #60 cover.rs via git add -A; it is not registered in mod.rs, so inert, and will be superseded by the verified final version when #60 lands.)


---
_Comment/review ids in this file: 5050981107_

