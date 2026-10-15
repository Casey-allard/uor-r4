# #732 — format/#637-1: freeze PROV/1 provenance-roots section

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T14:32:34Z
- **Updated:** 2026-10-09T14:17:15Z
- **Closed:** 2026-08-16T14:50:33Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/732
- **Merged:** 2026-08-16T14:50:33Z
- **Merge commit:** 633f9d499c1c013fcd2e14e889e45aaebb279137
- **Base / Head:** main / UOR-Foundation:issue-637-prov1-format
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## References #637

**Not** Closes — this is phase 1 of the 3-phase decomposition Casey approved on #637 (format-only freeze; producer propagation and consumer/era adoption are separate follow-up PRs).

### Background

#637's own format audit found R4G1 PROV (section 0x08) already a known mandatory section — readers accept it and preserve its bytes — but its payload schema, semantic validation, and writer emission were never specified beyond a one-line RFC summary ("provenance roots: source model κ, corpus CIDs, license refs, evidence roots"). Casey approved proceeding on the recommended direction (use the existing PROV section, no new TLA8 era) without waiting on @afflom/@auser sign-off; [design comment posted on #637](https://github.com/UOR-Foundation/uor-r4/issues/637#issuecomment-5307879665) before implementation, inviting their review even though it's no longer blocking.

### PROV/1 wire layout

Grounded in existing precedent rather than invented from scratch: mirrors FWDA's (#399, `fwda.rs`) MAGIC+VERSION+checked-bounds+canonically-sorted-list convention for the variable evidence-root tail, and HEAD's fixed 32-byte digest-slot convention for the identity fields.

```
offset  size   field
0       4B     magic = "PRV1"
4       2B     version u16 (=1)
6       1B     presence bitmap: bit0=source_manifest_kappa(#597) bit1=geometry(#600)
               bit2=tokenizer_adapter(#601) bit3=attention_operator(#602)
               bit4=dense_operator(#704) bit5=license bits6-7=reserved(0)
7       1B     reserved (0)
8       4B     evidence_root_count u32
12      4B     license_len u32 (0 when presence bit5 clear)
16      32B    source_manifest_kappa digest (zero-filled when absent)
48      32B    geometry_digest (zero-filled when absent)
80      32B    tokenizer_adapter_digest (zero-filled when absent)
112     32B    attention_operator_digest (zero-filled when absent)
144     32B    dense_operator_digest (zero-filled when absent)
176     ...    license bytes (ASCII SPDX expression, license_len bytes)
        ...    evidence roots: evidence_root_count × 32B κ, strictly ascending, no duplicates
```

PROV stores identity **digests**, not payloads — the sidecars/manifests that already own each component's canonical bytes (e.g. `TokenizerAdapter::adapter_digest`) are unchanged by this freeze.

### Scope

- `uor_r4_graph_format::Prov<'a>` — borrowed, no_std (core-only) parser with full structural validation.
- `uor_r4_graph_format::build_prov` — canonical builder, `alloc`-gated (mirrors `ArtifactBuilder`/`build_route_attention_instance`'s existing gating convention).
- New `FormatError::Prov*` variants + `NotAProduct` conversion (R5-sanctioned error surface).
- `docs/transformerless/R4G1.md` — §3 table row and §5 bullet updated with the frozen layout.
- 17 new tests (`crates/uor-r4-graph-format/tests/prov_637.rs`): round-trip, builder canonicalization (sorts unsorted input, rejects duplicates, rejects non-ASCII license), and parser negatives for every check (bounds, order, duplicate, tamper).

### Non-goals (this PR)

- No wiring into `stage2::validate`'s mandatory-section completeness pass (phase 2).
- No producer propagation — nothing in the compiler calls `build_prov` yet (phase 2).
- No consumer/era adoption, bundle/API agreement, or legacy-unbound compatibility handling (phase 3).
- **No compiler write-path change and no artifact/CID/κ re-pin** — this PR cannot move any existing artifact's bytes since nothing reads or writes an actual PROV section yet. The TLA7 κ fixture is untouched.

### Tests and gates run locally

- `cargo build/test --workspace --offline` — pass, 0 failed.
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` — clean.
- `cargo fmt --check` — clean.
- `cargo check -p uor-r4-graph-format --no-default-features` / `--features alloc` — clean (parser is core-only; builder is alloc-gated).
- `cargo run -q -p xtask -- audit-limits` (R5) — pass.
- `python3 scripts/check_claim_wording.py` — pass.
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib` — pass.

### Known deferrals

- Phase 2: emit/preserve the full identity record through cover, score, converter, and cached-cover paths; prove same-input byte-stability and changed-geometry-changes-CID.
- Phase 3: bundle/API/evaluation fail-closed agreement, explicit legacy-unbound compatibility for pre-PROV artifacts, affected R4G1 identity updates.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

