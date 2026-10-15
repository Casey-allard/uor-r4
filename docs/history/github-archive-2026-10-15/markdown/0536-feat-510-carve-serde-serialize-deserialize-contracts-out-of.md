# #536 — feat(#510): carve serde Serialize/Deserialize contracts out of audit-limits

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T08:26:37Z
- **Updated:** 2026-10-09T13:08:51Z
- **Closed:** 2026-08-09T08:45:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/536
- **Merged:** 2026-08-09T08:45:06Z
- **Merge commit:** 53423ac94d4634ed1e3844979c264c6b32f65101
- **Base / Head:** main / UOR-Foundation:feat/510-r5-audit-serde
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The `audit-limits` gate (R5) flags any shipped-crate `Result<...>` whose error is not a sanctioned type. serde's `Serialize`/`Deserialize` contract is a **false positive**: a `Serializer::serialize`-shaped helper must return `Result<S::Ok, S::Error>` and a `Deserialize::deserialize`-shaped helper must return `Result<T, D::Error>`, where the error is the *data format's* (the caller's serde impl), not a limitation this model reports. Those signatures are fixed by the trait and cannot name a sanctioned type — no conversion could ever satisfy the gate.

Carve them out like a sanctioned return: a `Result<...>` tail containing `S::Ok`, `S::Error`, or `D::Error` is a serde associated-type contract, not the model's error surface. This clears the `#[serde(with)]` helpers (core `endomorphism.rs`, router's f64-vector codecs, …) without loosening the gate for any real error return.

Unblocks the path to a zero-violation `audit-limits` and closing #510.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

