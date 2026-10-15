# #510 — Template gates R4/R5/R6: wire audit-deferral (with the dormant-module carve-out), audit-limits, and cargo-deny

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-08T21:51:42Z
- **Updated:** 2026-08-10T19:56:39Z
- **Closed:** 2026-08-10T19:56:39Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/510
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up from #273 (the template-register adoption + BDD migration, which landed the R1/R2/R3 gates green). The remaining template gates need uor-r4-specific work before they can go green and move from advisory to required:

- **R4 `xtask audit-deferral`** — the template's R4 ("nothing deferred: no marker, no stub, no capability behind a flag") fails as-written on uor-r4's four dormant modules (`cd_space`, `lie_jordan`, `bott_fock`, `quantum_cover`). #273 registered them as `open` ledger claims (the carve-out). This issue: teach `audit-deferral` to accept a capability that is registered `open` with an activation gate, so R4 enforces "nothing deferred **and unregistered**" — then wire it into the register CI job.

- **R5 `xtask audit-limits`** — "no bound that cannot be traced to a parameter" over the shipped crates. uor-r4's ~65k LOC was not written to R5; bringing it into conformance is incremental (register-as-you-touch). This issue tracks that migration.

- **R6 `cargo deny`** — `deny.toml` is staged but its `[sources]` only allows crates.io, while uor-r4 pulls `uor-addr` and `uor-foundation` from git. Allowlist those git sources (deliberately, per the deny.toml discipline), reconcile licences/advisories over the real graph, then add `just deny` to the gate.

When all three are green, promote the `register conformance` CI job to a required check and fold the audits into it. The register itself (RF-01..RF-29 + the dormant-module carve-out) is already the single source; this is finishing the gate coverage on top of it.


## Comments (5)

### comment 5228691944 — 2026-08-08T23:23:40Z — Casey-allard

## Progress — R4 and R6 wired (PR #513); R5 assessed and remaining

Investigating the actual tree, the issue premise was slightly off: the four dormant modules (`cd_space`, `lie_jordan`, `bott_fock`, `quantum_cover`) do **not** trip R4 — they compile and are exercised. The real R4 blockers were elsewhere, and are resolved in #513.

**R4 — green.** `audit-deferral` now honours the carve-out: a marker fails only if its line does not cite an `open` ledger claim. Falsifiable (unregistered fails, cited passes). Resolutions:
- removed dead `graph.rs::build_graph` (`unimplemented!`, no call sites);
- registered the `packed_kernels` #159-Phase-2 placeholders (`evaluate_routing_program`/`evaluate_typed_transition`, unreferenced by serving) as a new `open` claim `packed-routing-dormant` and cited it — **kept, not deleted**, since the ROUT/typed-transition geometric-routing surface is a live direction;
- reworded two "for now" comments to present tense;
- fixed a false positive on test bytes `b"XXXX"`.

**R6 — green.** `deny.toml` allow-lists the two pinned git sources; MIT declared on our crates; BSD-2-Clause + BSL-1.0 added for the six permissive third-party crates that carry them; internal/git deps versioned so the wildcard ban stays meaningful. `cargo deny check` passes advisories/bans/licences/sources.

**R5 — remaining.** `audit-limits` reports **352** shipped `Result` returns over unsanctioned error types — a genuine incremental (register-as-you-touch) migration, not a one-shot. The `register-conformance` CI job therefore stays **advisory**; promoting it to **required** is gated on R5 finishing.

Keeping this issue open for R5. Suggest either continuing R5 here, or splitting R5 into its own tracking issue so #510 can close once #513 lands (R4+R6). Your call.


### comment 5228785908 — 2026-08-08T23:39:23Z — Casey-allard

Revision (per maintainer direction — keep every mechanism, align by registration not removal): the earlier note said `graph.rs` was *removed*; it is now **kept** instead. `build_graph()` is restored and registered as a new `open` claim `graph-construction-dormant`, with its `unimplemented!` marker citing the claim so R4 sanctions it — same kept-and-registered pattern as `packed-routing-dormant`. No mechanism is deleted in #513. A broader follow-up (align *all* ruled-out/dormant mechanisms with the gates, and evaluate feature-flag modularization so they can be toggled) is worth its own issue.

### comment 5228896661 — 2026-08-09T00:05:33Z — Casey-allard

## R5 (audit-limits) — full-rearchitecture march plan

Decision: **full rearchitecture to total operations** (the purist R5). End state: no shipped crate returns an error the model does not sanction. Sanctioned surface only: `NotAProduct` (the requested object does not exist, reported at view construction), `ObservedBound`, `KappaError`.

### The semantic model
"Total ops" does not mean "delete all `Result`s and silently corrupt on bad input." It means the **only failure surface is object/view construction**, expressed as `NotAProduct`. Everything downstream of a successfully-constructed view is total (cannot fail). Concretely, per violation class:

- **Vestigial `Result` (never returns `Err`)** -> drop the `Result`, make the fn total; update call sites to stop matching an error. Lowest risk, done first.
- **Parse / validate at a boundary** (`parse(bytes) -> Result<Self, FormatError>`, patch validation) -> fold into a constructor returning `Option<Self>` / `NotAProduct`. The "does this parse / exist" question is answered once, at construction; the constructed view is then total.
- **I/O** (`read_file`, `load(source)`) -> lives at the load/entry boundary; a missing/unreadable source is `NotAProduct` at load, not a mid-pipeline error.
- **Arbitrary-limit guards** (e.g. `max_steps == 0 -> Err`) -> remove the limitation (make total) or, if it is a real observed capacity, express as `ObservedBound`.

### Landscape (352 sites)
By crate: graph-compiler 88, graph-certify 53, core 48, graph-cli 42, proof-model 31, format 31, router 25, api 20, model-source 9, runtime 9.
By type: `String` 113 (mostly command handlers), then `ProofValidationError` 22, `FormatError` 15, `InferenceError` 11, `RegionObjectError` 9, and ~20 more domain enums.

### Sequencing (bottom-up, so call-site ripple flows upward)
Each step is its own PR that drives `audit-limits` for that crate to zero; the gate goes green + promoted to **required** only when the whole count hits 0.

1. **graph-format** (31) — the leaf. `FormatError` parse/decode -> construction-returns-`NotAProduct`. Everything depends on this, so it sets the pattern and the `NotAProduct` vocabulary.
2. **graph-runtime** (9) + **model-source** (9) — small, mostly parse/IO/vestigial; validate the pattern above their now-total format layer.
3. **uor-r4-core** (48) — the substrate; `GeometryError`/`SemanticStateError`/etc. Largest correctness surface.
4. **proof-model** (31), **router** (25), **graph-certify** (53) — mid-graph.
5. **graph-compiler** (88) — the top of the library graph.
6. **api** (20) + **graph-cli** (42) — the command surface; the 113 `Result<_, String>` mostly live here. These become total dispatch over already-constructed views; a bad CLI arg is a non-existent view (`NotAProduct`), printed once at the boundary.

### Guardrails
- Every step keeps the cucumber BDD suite and the register meta-gate green; no behaviour change beyond the error-surface reshape.
- `audit-limits` stays **advisory** (already so per #513) until the count is 0, then it joins the required `register conformance` job.
- Each PR reports the before/after violation count so the march is measurable and no crate is silently skipped.

This is a multi-PR effort; PRs land bottom-up in the order above.


### comment 5229004196 — 2026-08-09T00:34:44Z — Casey-allard

**R5 tranche 1a landed (PR #518, enqueued).** graph-format parse/CID layer moved to the sanctioned surface: `NotAProduct { object, reason }` / `KappaError` / `ObservedBound` introduced in `sanctioned.rs`, 17 parse fns + 2 CID fns converted, **diagnostic-preserving** (the `FormatError` reason is carried inside `NotAProduct`, not lost). graph-format R5 count **31 → 12**; audit-limits total **352 → 333**. std + no_std clean, all tests + register meta-gate + 101-scenario BDD green. Went with the softer-R5 read (regularize the public contract, keep internal detail) per maintainer call — the better end state for a parser.

Next per the march: **1b** = graph-format\x27s remaining 5 error enums (ScoringError, InvariantValidationError, InferenceContract*, RealizationError; 12 sites), then up the dependency graph (runtime/model-source → core → mid-graph → compiler → api/cli), each PR driving its crate to zero. audit-limits promoted to required only at total zero.

### comment 5245265907 — 2026-08-10T19:56:38Z — Casey-allard

R5 complete: PRs #524..#587 converted every shipped crate's Result error surface to name ONLY the four sanctioned types — NotAProduct / ObservedBound{observed,bound} / KappaError (uor-r4-graph-format) + SourceUnavailable (uor-r4-model-source); everything else made total (Option / bare value / report-bool / panic for self-produced|infallible|host defects). Crates driven to 0 bottom-up: core, router, proof-model, graph-compiler, graph-certify, api, graph-cli. xtask audit-limits is clean and now a CI conformance gate (#588); it exits nonzero on any reintroduced unsanctioned Result. Note: promoting register-conformance to a REQUIRED GitHub status check is a repo-admin branch-protection toggle.


---
_Comment/review ids in this file: 5228691944, 5228785908, 5228896661, 5229004196, 5245265907_

