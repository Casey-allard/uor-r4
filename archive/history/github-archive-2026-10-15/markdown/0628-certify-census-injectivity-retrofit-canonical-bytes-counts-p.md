# #628 — certify/census: κ-injectivity retrofit — canonical-bytes counts primary, κ-equality asserted, two-path κ certificate

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-12T00:42:47Z
- **Updated:** 2026-08-12T02:15:54Z
- **Closed:** 2026-08-12T02:15:54Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/628
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Spawned from #625 (atlas discipline ports; campaign #589). Filed unassigned per board rules — assign on start.

## What

Retrofit the κ-injectivity census discipline (UOR-Atlas-UTQC tqc-vv `exact.rs` pattern, upstream's own HEAD-commit hardening) onto r4's distinct-count sites:

1. Wherever we report "N distinct X" by counting distinct κ labels — cover-sweep censuses, region/graph counts in `uor-r4-graph-certify` and `cover_sweep.rs` — make the **canonical-bytes count primary**, compute κ separately, and `assert_eq!` the two. The count becomes hash-independent and collision-freeness becomes a verified property per run instead of a standing assumption.
2. Add the **two-path κ certificate** where an artifact κ is produced: an independently-constructed canonical serialization must reproduce the production addresser's κ (atlas `canonical_kappa_certificate` shape). Cheap second witness for the determinism invariant.

## Scope

- Inventory the count sites first (grep for distinct-κ counting; the #488 phase-log work enumerated the census surfaces) and list them in the first comment before patching.
- The retrofit adds asserts, not new counts — census numbers must be byte-identical before/after (that IS the verification).
- No new dependencies; no atlas crates (rejected in #625 — tqc-substrate drags the hologram graph).

## Verification

Four gates + full suite green; a deliberately-corrupted κ in a test must FAIL the new assert (the instrument must be able to fail); census outputs unchanged.


## Comments (2)

### comment 5261142247 — 2026-08-12T01:50:11Z — Casey-allard

## Inventory (first comment before any patch, per the issue terms — measured 2026-08-12 on main 2bfd360)

**Item 1 target search (distinct-κ counting): ZERO sites found.** Every census in cover_sweep.rs / graph-certify counts distinct objects by canonical structure (BTreeMap keys, region ids, store keys), never by κ label — r4 already has the hash-independent-counts property the atlas discipline enforces. The retrofit's precondition does not exist here.

**Item 2 targets (two-path κ certificate):** κ producers are single-writer — ser.rs (head_cid/artifact_cid), cover_sweep.rs:511, runtime_corpus.rs, model-source. BUT: view.rs already re-derives and rejects on parse (verify-on-read), format tests carry 28 corrupt/tamper checks, and CI runs deterministic-rebuild + κ-reproduction gates. **The one genuine gap:** deterministic-rebuild reruns the SAME writer twice — it proves determinism, not that the canonical form is what the spec says. An independent second serialization (atlas's canonical_kappa_certificate sense) does not exist.

Decision points going to Casey now; patch follows his call.

### comment 5261304392 — 2026-08-12T02:15:53Z — Casey-allard

## Closing with record (PR #632 merged, Casey-confirmed)

**Item 1 — already satisfied by construction:** inventory found zero distinct-κ counting sites; every census counts by canonical structure, which is the hash-independence property this discipline enforces. Recorded, nothing added (adding κ to censuses would have introduced hash-dependence to code free of it).

**Item 2 — delivered:** `two_path_kappa_628.rs` — an independent serializer written from the header documentation alone reproduces the production writer byte-for-byte and κ-for-κ, with a mutation arm proving failability. It caught a real doc-reading drift (u64-vs-u32 table fields) during its own construction — the exact class it guards. Census outputs unchanged (test-only patch); no new deps; all gates green.


---
_Comment/review ids in this file: 5261142247, 5261304392_

