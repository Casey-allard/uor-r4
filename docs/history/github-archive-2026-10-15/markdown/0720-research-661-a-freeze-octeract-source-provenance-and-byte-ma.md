# #720 — research/#661-A: freeze Octeract source provenance and byte-map conformance

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-15T00:19:24Z
- **Updated:** 2026-08-15T01:13:40Z
- **Closed:** 2026-08-15T01:12:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/720
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Parent issue #661 asks whether an 8-bit binary Kaprekar ("Octeract") quotient has a useful, falsifiable seat in R4 route attention. Before any trace comparison, fitting, or packed prototype, the source map and byte algebra must be frozen independently so later measurements cannot change the candidate after labels are observed.

The two supplied PDFs are out-of-band research inputs. They contain no detected license or redistribution grant. This child records their identity and claim boundaries without committing the PDFs, prose, or figures.

# Goal

Produce the source-provenance record and exhaustive executable conformance layer that #661-B can consume. Correct source claims where necessary, distinguish proved finite facts from open categorical/formal claims, and leave every runtime/operator/default byte unchanged.

# Scope

1. Add the source/claim/algorithm map to `docs/octeract_attention_661.md`:
   - exact title, local filename, byte size, SHA-256, supplied-out-of-band provenance, and audit date;
   - `license: NOASSERTION` and `redistribution: not authorized` unless a maintainer supplies an explicit grant;
   - page/section mapping for every implementation-relevant definition;
   - classification as `directly-applicable`, `adaptable-hypothesis`, `already-covered`, or `incompatible`, with the owning R4 component.
2. Implement an independent scalar sort/subtract oracle and a separate closed-form implementation for the 8-bit map.
3. Freeze exact semantics for:
   - endpoints `0` and `255`;
   - complement symmetry;
   - one-step idempotency;
   - the five-shell quotient `{0, 127, 189, 217, 225}`;
   - canonical folded class `min(d, 8-d)`;
   - oriented fold and its relation to ordinary per-block weight.
4. Exhaust all 256 byte inputs and all 65,536 full-mask byte pairs. Add deterministic representative/property coverage for partial masks and 288-bit block composition.
5. Record corrections explicitly. In particular, test rather than assume injectivity, unrestricted endpoint weight identities, or that the paper's complement-order is the route-attention operator itself.
6. Bind source digests and the conformance implementation in the #661 evidence record with deterministic test evidence.

# Non-goals

- No trace-derived metric, fitting, candidate promotion, packed instance, serving path, or default change.
- No Lean dependency or claim of machine proof. Any theorem not proved in the repository remains open and excluded from conformance claims.
- No redistribution of either PDF and no copied extended prose or figures.
- No numeric use of the raw anchor integers as similarity scores.
- No mutation of `route-fit/1`, `r4-route-attention/1`, historical artifacts, or kappa pins.

# Dependencies

- Parent #661 owns the run contract and final disposition.
- #603-#606 provide the later trace, baseline, and certificate seams but are not needed to establish the finite byte algebra.
- #647 supplies the matched-frame guard for the later screen.
- #653 is not available; this child does not create a competing proof registry.

# Acceptance criteria

- [ ] Both source files are bound by title, filename, size, SHA-256, provenance, audit date, license state, and redistribution state.
- [ ] No source PDF, extended quotation, or figure is committed.
- [ ] A direct digit-sort/subtract oracle and an independently coded closed form agree for all 256 inputs.
- [ ] Published byte examples and all five outputs are reproduced exactly.
- [ ] Endpoints, complements, idempotency, shell identity, anchor relabeling, ties, and oriented-fold reconstruction are pinned by tests.
- [ ] All 65,536 byte pairs pass the full-mask relation checks; partial-mask and 288-bit composition tests are deterministic and non-vacuous.
- [ ] The evidence record identifies false/overbroad/open source claims and excludes them from implementation guarantees.
- [ ] Current V1/operator/runtime/artifact bytes remain unchanged.

# Verification

- Focused graph-certifier/compiler tests for the new conformance surface.
- Deterministic repeated test/report bytes where a record is serialized.
- `cargo fmt --all --check`.
- `cargo test --workspace --offline`.
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`.
- Graph-format no-std/alloc ladder and wasm only if touched by the final diff.
- `python3 scripts/check_claim_wording.py` and `git diff --check`.

# Compatibility and claim status

This is additive, certifier-side research evidence. It defines a finite executable conformance oracle; it does not establish attention quality, a categorical adjunction for the deployed operator, transformer equivalence, production readiness, or a promotion result.

# Follow-up order

After this child merges, file/activate #661-B for the predeclared V1/weight9/fold/oriented/prefilter/lower-bound/null collision and reachability screen. Create a one-head child only if that ceiling and the #647 frame control clear their locked gates.

References #661.


## Comments (1)

### comment 5299747832 — 2026-08-15T01:13:39Z — Casey-allard

#720 is complete via PR #721, merged through the queue as `598cba902c6d16fb9d7687601744728856a4197f`.

Delivered:

- exact identity/license/provenance records for both supplied PDFs without redistributing either source;
- an independent digit-sort/subtract oracle and separate closed-form implementation;
- R5-compatible bounded value types for byte weights, block distances, full-byte shells, and canonical oriented classes;
- exhaustive 256-byte and 65,536-pair conformance, all 45 canonical oriented states, deterministic partial-mask coverage, and a 288-bit composition check;
- append-only source corrections and a strict certifier-only boundary: no operator, runtime, artifact, default, or kappa mutation.

Pinned evidence includes basin counts `2/16/56/112/70`, `17,920` equator pairs, `47,616` non-equator fold collisions, `213,862` strict partial-mask lower-bound cases, and the deterministic 288-bit totals `146` exact/reconstructed versus `106` folded.

Verification:

- local workspace fmt, clippy, tests, no-std/alloc, wasm, R1-R6 audits, cargo-deny, claim wording, and diff checks passed;
- canonical κ reproduction was non-vacuous and passed 3/3 against `/tmp/ref/out/model.bin`;
- merge-group run https://github.com/UOR-Foundation/uor-r4/actions/runs/31855114773 passed the full queue ladder, including nextest, all-features, κ, Gate C, fuzz, wasm, and conformance.

The next action is the separately gated #661-B trace collision/reachability screen. It will preserve a typed canonical `UNAVAILABLE` record if the required pinned real `full/1` trace remains absent; synthetic fixtures cannot manufacture an empirical PASS/FAIL.



---
_Comment/review ids in this file: 5299747832_

