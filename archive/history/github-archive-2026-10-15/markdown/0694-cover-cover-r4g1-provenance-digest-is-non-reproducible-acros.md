# #694 — cover: cover.r4g1 provenance digest is non-reproducible across runs (HashMap-seeded)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T04:00:22Z
- **Updated:** 2026-08-14T04:39:39Z
- **Closed:** 2026-08-14T04:39:39Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/694
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Found while building the #670 cover canary (PR #693).**

`cover_command` writes `cover.r4g1`, but a bit-for-bit double run over the *same* compiled teacher corpus produces artifacts that differ in a single ~32-byte field in the header (around byte offset 24). Everything after (~offset 56 onward — the region/edge/prior payload) is byte-identical, and the induced structure is fully deterministic: the two `cover_report.json` files have identical `regions` and `recall`.

## Diagnosis
The differing 32 bytes are a provenance digest seeded per-run by the standard library's `HashMap`/`HashSet` `RandomState` (random SipHash keys per instance). Somewhere in `cover::emit_r4g1` / induction a digest is computed over a `HashMap`'s iteration order (or a hashed set), so the digest varies run-to-run even though the induced cover does not.

## Impact
- The R4G1 route artifact from `cover` cannot be κ-pinned or byte-reproduced the way the compile bundle can (the compile `tless_artifacts.bin` *is* byte-deterministic — see the #670 compile canary). This weakens any Gate-E-style reproduction gate applied to the cover R4G1, and means two identical cover runs yield different `cover.r4g1` CIDs.
- Workaround in the #670 canary (PR #693): assert *structural* determinism (identical `regions` + `recall`, stable artifact length) rather than raw-byte equality, and document the caveat in the test.

## Suggested fix
Make the digest input order-independent: sort keys before hashing, use a `BTreeMap`/sorted `Vec` where the digest is computed, or a fixed-seed hasher for that digest. Then `cover.r4g1` becomes byte-reproducible and the canary can assert byte-determinism directly.

## Repro
With the pinned GPT-2 snapshot present, run `gpt2_cover_certifies_source_parity_rows` (graph-cli) — an earlier revision asserted `bytes_a == bytes_b` on `cover.r4g1` and failed on exactly this header digest while `regions`/`recall` matched.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

