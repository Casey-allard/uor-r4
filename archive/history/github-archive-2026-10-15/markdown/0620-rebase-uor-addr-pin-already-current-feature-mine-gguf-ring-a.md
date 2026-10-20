# #620 — rebase(uor-addr): pin already current — feature-mine gguf/ring κ-addressing for #597/#603 and geometric objects

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-11T22:53:59Z
- **Updated:** 2026-08-12T00:05:23Z
- **Closed:** 2026-08-12T00:05:23Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/620
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #589 (ecosystem rebase, step 2: uor-addr).

## Audit result (measured 2026-08-11)

r4 pins `uor-addr 0.2.0` at git rev `165b51e` — **that rev is upstream `main` HEAD today** (2026-06-20). Delta v0.2.0..pin is 5 commits, **ONNX-realization only** (empty-name node ordering tie-break, digest tie-break optimizations, matching Lean proof + fixtures). r4 consumes only `uor_addr::json::{address, address_blake3}` (router lib.rs, tless_uor.rs, server.rs) — the json realization is untouched by the delta. uor-addr 0.2.0 also exists on crates.io, but the registry copy is the *tagged* v0.2.0, 5 commits **behind** our pin; no reason to move.

**Version verdict: no rebase needed.** After #619 (patch.crates-io), uor-addr's transitive uor-foundation/-sdk copies unify with ours automatically.

## Feature-mining map (the actual work of this issue)

Upstream surfaces r4 does not use yet, mapped to the open board:

1. **`uor_addr::gguf`** — typed content-addressing for GGUF v3 model files with a blake3 κ-axis (`address_blake3`), SHA-256 canonical leaf commitments. Direct candidate for #597 (versioned source snapshot manifest) and #603 (content-addressed observation shards): κ-label the model artifacts we ingest instead of hand-rolled hashing. Gap to note: no Safetensors realization upstream (#598 is Safetensors-shard work) — evaluate whether to (a) file an upstream issue for a safetensors realization, or (b) address the shard manifest as canonical JSON via `uor_addr::json` (already in our dep graph, zero new code).
2. **`uor_addr::ring`** — ring-element addressing under Framework Amendment 43 §2 (`header(k) || le_bytes(x, k+1)`, Witt level k). Candidate κ-surface for R4 coordinate/lattice states if we want content-addressed geometric objects with a spec-canonical byte layout rather than our own serialization.
3. **`uor_addr::cbor` / `sexp` / `xml` / `asn1` / `codemodule`** — additional realizations; no current r4 consumer identified. Catalog only, no action.
4. **`bindings/{npm,python}`** — upstream ships Python/npm bindings; r4's Python tooling (scripts/) could κ-label corpora identically to the Rust side. Optional; file separately if wanted.

## Deliverables

- [ ] Decision + record on gguf adoption path for #597/#603 (coordinate with those issues' owners when they activate; they are queued-unassigned today)
- [ ] Decision + record on ring realization for geometric-object addressing (this is a design question — needs Casey's call, not just a patch)
- [ ] Upstream issue filed on uor-addr for Safetensors realization iff (1a) chosen
- [ ] No version changes expected; if any land, full gate ladder + zero-matmul invariant unchanged

## Verification

Any adoption PR: four local gates green, `cargo test --workspace` green, deployed-kernel P-4 source scan untouched, κ-labels for existing artifacts unchanged unless the PR explicitly re-pins them with a record.


## Comments (1)

### comment 5260315016 — 2026-08-12T00:05:23Z — Casey-allard

## Decisions recorded — closing with record (2026-08-12, both decisions confirmed by Casey in session)

**Version verdict (from the audit, unchanged):** pin `165b51e` = upstream main HEAD; delta v0.2.0..pin is ONNX-realization-only and r4 consumes only `uor_addr::json::{address, address_blake3}`. Registry uor-addr 0.2.0 is 5 commits *behind* the pin — staying on the git pin. After #619, uor-addr's transitive uor-foundation/-sdk unify with ours via `[patch.crates-io]`. **No dependency change shipped or needed; zero-matmul runtime untouched.**

### Decision 1 — snapshot/shard addressing for #597/#603: `uor_addr::json`, not `gguf`

r4 ingests Hugging Face **safetensors** (#598 scope), not GGUF, so the `uor_addr::gguf` realization has no current call site — not adopted. The adopted path for #597's source snapshot manifest and #603's observation-shard binding: **canonical-JSON manifests addressed with `uor_addr::json::address_blake3`** (`AddressOutcome<71>`, blake3 axis — matches our repo-wide κ label format byte-for-byte). Already in the dependency graph (router uses it at three sites); zero new dependencies, zero new code in this issue. Concrete shape for #597 when it activates: per-file leaves stay raw `blake3:<hex>` digests (what `model-source` already emits at lib.rs:1183/1511), the manifest binding {repo, revision, license, adapter/compiler version, every path+bytes+digest} is canonical JSON, and the **root κ comes from `json::address_blake3` instead of raw `blake3::hash`** — spec-canonical JSON addressing for free, and registry-compatible with #624's `KappaLabel` format. Pointer comments left on #597 and #603.

### Decision 2 — `ring` realization: NO-ADOPT (Casey confirmed)

Measured mismatch: ring elements are single algebraic scalars **≤5 bytes** (Witt level 0–3 + 1–4-byte LE coefficient, Framework Amendment 43 §2) — structurally nothing like r4's 288-dim state vectors or R⁴ tangent coordinates. Content-addressing of store codewords is better served by the uor-matmul κ-manifest path (#622). Revisit **only if** r4 ever adopts Amendment 43 ring algebra for coordinate scalars; recorded here so the option isn't lost.

### Upstream gap filed (Casey approved)

uor-addr has structural realizations for JSON/GGUF/CBOR/ONNX/etc. but **none for safetensors** — the format r4 (and most HF open-weight work) actually ingests. Filed on UOR-Foundation/uor-addr (unassigned, their governance): request for a safetensors realization with per-tensor leaf commitments in the gguf module's style, referencing r4 #597/#598/#620 as the use case. Link in the comment below once created.

### Other surfaces (catalog only, no action)

`cbor/sexp/xml/asn1/codemodule` realizations and the npm/python bindings: no r4 consumer identified; bindings noted as an option for κ-labeling corpora identically from `scripts/` Python if ever wanted.

**Deliverables check:** gguf/#597/#603 adoption path decided + recorded ✓ · ring decision made with Casey's call ✓ · upstream safetensors issue filed ✓ · no version changes (none needed) ✓. All gates unaffected (no code changed under this issue).

Upstream issue: https://github.com/UOR-Foundation/uor-addr/issues/14


---
_Comment/review ids in this file: 5260315016_

