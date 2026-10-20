# #624 — rebase(kappa-registry): register compiled artifacts + CAS-tag κ pins via service client — zero shipped-crate deps

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-11T23:02:37Z
- **Updated:** 2026-08-12T01:43:42Z
- **Closed:** 2026-08-12T01:43:42Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/624
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #589 (ecosystem rebase, step 6: kappa-registry).

## Audit result (measured 2026-08-11)

Not integrated. Upstream: single-node filesystem-backed `/v2/` Kappa Distribution registry (axum/tokio/redb), HEAD `2929832` (2026-07-28), `publish = false`. **Maturity is early**: exactly 1 commit (squashed import, author Ari), CI = one job (fmt/clippy/test/musl build), 205 unit+integration tests, BDD is one feature file, conformance scripts not CI-wired, `auth::authorize` is a permit-everything stub. Evaluate as **trusted-network internal infrastructure**, not a hardened service.

## Fit for r4 — strong, and zero-cost at the label layer

1. **Label format is already byte-identical.** r4 emits `blake3:<64 lowercase hex>` (71 bytes) — exactly the registry's `KappaLabel` blake3 axis. Compiled `.uor-models` artifacts can be `PUT /v2/{ns}/blobs/blake3:<hex>` with no reformatting, and the registry **re-derives the hash on every put** — an independent, out-of-process witness for our deterministic-rebuild gate.
2. **Teacher κ pins → tags with compare-and-set.** `tag_set_if` (expected-value conditional) + atomic `tag_set_batch` = "a pin only moves from the value you read", which is precisely our Gate-E κ re-pin discipline, machine-enforced. `gc pin/unpin/sweep` for retention; `GET /v2/{ns}/_root` for a deterministic (ed25519-signable) namespace root to record in the ledger.
3. **Provenance edges** (`edges/`, `referrers/`, `_reconcile` set-reconciliation) can carry teacher→student and corpus→graph links; `_transaction` gives atomic artifact+sidecar publication; `_bundle`/delta bundles give offline corpus transport.
4. **blake3 compatibility**: registry declares `blake3 = "1"` (lock 1.8.5) — satisfied by our `=1.5.0` pin if graphs ever merged; digests are version-stable regardless. As a *service* there is no shared graph at all.

## Adoption slice (client-side only — no new dependency in any shipped r4 crate)

- [ ] 1. Run the musl binary as a service beside CI (`KAPPA_STORE_ROOT` on durable disk); trusted network only (auth is a stub; blobs are global CAS — namespaces are NOT an isolation boundary: GET/DELETE ignore ns)
- [ ] 2. One small HTTP client module in r4 tooling (xtask or scripts/): put compiled artifacts by existing κ, assert verify-on-put round-trip
- [ ] 3. Teacher/corpus κ pins as CAS tags + gc-pin; signed `_root` recorded in the ledger per re-pin
- [ ] 4. Only after 1–3 prove out: `_transaction` for artifact+sidecar atomicity, bundles for offline transport

## Do NOT (recorded)

Embed `kappa_registry` as a Rust lib (drags axum+tokio+redb+governor+jsonschema+3 sig crates for what is ~150 lines of label code; if we ever want in-process labels, reimplement `kappa.rs` shape locally). Trust `compose/{g2..e8}` for cross-repo agreement — it reimplements composition **without uor-addr**; unverified against our uor-addr results. Mint `sha3-256`/`keccak256` labels (parse-only; compute errors). Expose beyond trusted network.

## Verification

Round-trip test: put artifact → get → byte-equal; put with corrupted label → rejected; tag CAS: stale expected → refused. No r4 crate dependency changes; four gates unaffected.


## Comments (2)

### comment 5260881410 — 2026-08-12T01:07:18Z — Casey-allard

## Verification record + delivery (PR #630 in queue)

Client delivered as `cargo xtask kappa` (std-only HTTP/1.1, zero shipped-crate deps). **End-to-end verification against a locally built registry** (rev 2929832, `KAPPA_STORE_ROOT` on temp disk):

- `publish` of the real `tless_artifacts.bin` fixture → `blake3:8fbf3f68…54d9` accepted (201) — **byte-identical to the κ the graph harnesses print for the same fixture**, and the registry re-derived it server-side: the out-of-process deterministic-rebuild witness works as designed.
- Corrupt-label PUT (same bytes, wrong label) → **400 refused** by the server. Instrument can fail. ✓
- `tag-cas` create via If-None-Match → created; **stale expected → refused with exit 1** ("re-read and re-decide, never force"); correct expected → moved. Gate-E semantics machine-enforced. ✓
- `fetch` re-derives locally and byte-compares → identical. ✓

**Remaining item 1 (run the service beside CI) is a hosting decision for Casey/the team** — the musl single binary + `KAPPA_STORE_ROOT` on durable disk, trusted network ONLY (auth stub; blobs are global CAS). Item 3 (pins as CAS tags + signed `_root` in the ledger) and item 4 (transactions/bundles) activate once hosted. Closing when #630 merges; the client is ready the day the service exists.


### comment 5261102497 — 2026-08-12T01:43:41Z — Casey-allard

PR #630 merged — client delivered and E2E-verified (record above); the remaining hosting decision (run the registry service beside CI, trusted network) is recorded for Casey/the team and reactivates items 3/4 when taken.


---
_Comment/review ids in this file: 5260881410, 5261102497_

