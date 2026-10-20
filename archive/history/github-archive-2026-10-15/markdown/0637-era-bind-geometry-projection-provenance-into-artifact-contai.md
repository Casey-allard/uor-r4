# #637 — era: bind geometry-projection provenance into artifact container bytes (TLA8 / R4G1 PROV) — maintainer re-pin decision

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-12T10:07:46Z
- **Updated:** 2026-08-16T19:21:40Z
- **Closed:** 2026-08-16T19:21:40Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/637
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Spawned from #600 (campaign: Ari program). #600 made the 576→288 projection explicit and versioned, with its record in compile-report + observation-manifest provenance. Binding it into artifact CONTAINER bytes was measured to be impossible additively: R4G1's PROV section id is defined but never emitted (emitting breaks existing readers per the RFC's mandatory-section rule), and any container-byte addition moves κs of otherwise-identical fresh compiles — exactly what the κ-reproduction/rebuild fixtures pin. Doing it requires a new era (TLA8 or an R4G1 optional-section allocation) + maintainer-approved fixture re-pin with era notes (#600's own compatibility clause). Filed unassigned per board rules; input needed from Alex/Ari (format governance) + Casey (re-pin approval). Until then, projection identity rides the provenance sidecars (report/manifest), which #600's tests pin.

## Comments (10)

### comment 5299398241 — 2026-08-15T00:11:37Z — Casey-allard

## Format audit and narrowed decision record

The live-code audit corrects one premise in this issue: R4G1 PROV is already the known mandatory section 0x08. Current readers recognize it and retain its bytes; adding it to new outputs does not trigger the unknown-mandatory rejection rule. What is actually deferred is the PROV payload schema, semantic validation, writer emission/preservation, and mandatory-section completeness for the major-0 draft line.

No prior #600 comment, PR review, or repository-wide decision record approves either TLA8 or a PROV design. A fresh governance decision is therefore still required.

### Recommended decision

Use the existing R4G1 PROV section and define a canonical bounded PROV/1 envelope. New provenance-aware graph artifacts emit it; existing major-0 artifacts without PROV remain readable and are explicitly classified legacy-unbound. Do not create TLA8. Re-pin only affected R4G1 graph/report/bundle identities, with era notes; the TLA7 κ fixture remains unchanged.

This finishes the successor-format contract instead of extending the legacy TLA line, and it creates one extensible owner for #597 source identity, #601 tokenizer identity, #600 geometry, and the #602/#704 operator era.

### Alternatives

- TLA8: extends and re-pins the legacy container while still leaving normative R4G1 PROV unfinished.
- Defer until stable R4G1 major 1: retains sidecars as the sole binding and leaves this issue open.

### Proposed decomposition after approval

1. Format-only: freeze PROV/1, borrowed no_std view, canonical builder, bounds/order/duplicate/tamper tests, and RFC update.
2. Producer propagation: emit and preserve the full geometry record through cover, score, converter, and cached-cover paths; prove same input is byte-stable and changed geometry changes graph CID.
3. Consumer/era adoption: bundle/API/evaluation fail-closed agreement, explicit legacy-unbound compatibility, affected R4G1 identity updates, docs, and full gates.

@afflom @auser: please approve or object to the R4G1 PROV/1 format direction. Casey approval is also required for the affected R4G1 identity re-pin. No wire changes will be submitted before that decision.

### comment 5307691397 — 2026-08-16T13:33:42Z — Casey-allard

## Maintainer decision recorded — proceeding without further format-authority sign-off

Casey (2026-08-16, direct instruction): explicit approval to proceed on the previously-recommended direction now, without waiting on @afflom / @auser to reply. Casey is exercising final decision authority as maintainer; this supersedes the "no wire changes before that decision" hold from the prior comment.

**Decision now in force:** use the existing R4G1 PROV section (0x08), define a canonical bounded PROV/1 envelope, no new TLA8 era. Affected R4G1 graph/report/bundle identities get a maintainer-approved re-pin with era notes; the TLA7 κ fixture is unchanged.

**Decomposition to execute (as previously proposed):**
1. Format-only: freeze PROV/1, borrowed no_std view, canonical builder, bounds/order/duplicate/tamper tests, RFC update.
2. Producer propagation: emit/preserve the geometry record through cover, score, converter, cached-cover paths; same-input byte-stability + changed-geometry-changes-CID proofs.
3. Consumer/era adoption: bundle/API/evaluation fail-closed agreement, explicit legacy-unbound compatibility, affected identity updates, docs, full gates.

@afflom @auser: flagging for visibility/objection even though this is no longer gating — if either of you sees a format-correctness problem with PROV/1 before phase 1 merges, please say so here.

Queued behind the currently in-flight #655-C0 slice; will file the phase-1 sub-issue and start its worktree next.


### comment 5307879665 — 2026-08-16T14:17:41Z — Casey-allard

## PROV/1 byte layout (phase 1, format-only)

Grounding this in existing precedent rather than inventing a new shape: mirrors FWDA's (issue #399) MAGIC+VERSION+checked-bounds+canonically-sorted-list convention (`crates/uor-r4-graph-format/src/fwda.rs`) for the variable evidence-root list, and HEAD's fixed 32-byte CID-slot convention for the identity digests. Binds exactly what the RFC's existing PROV row promises ("source model κ, corpus CIDs, license refs, evidence roots") plus the extensible identity set from my prior comment (#597/#600/#601/#602/#704), each already digest-addressable today (`TokenizerAdapter::adapter_digest` etc.) — PROV stores the digest, not the payload, keeping the section small and the sidecars (which remain byte-identical) authoritative for content.

```
offset  size   field
0       4B     magic = "PRV1"
4       2B     version u16 (=1)
6       1B     presence bitmap: bit0=source_manifest_kappa(#597) bit1=geometry(#600)
               bit2=tokenizer_adapter(#601) bit3=attention_operator(#602)
               bit4=dense_operator(#704) bit5=license bits6-7=0(reserved)
7       1B     reserved (=0)
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

**Validation (mirrors FWDA's `parse` exactly):** magic/version check; reserved byte and unused presence bits must be zero; a presence bit clear requires its digest slot all-zero (tamper/consistency — a "declared absent but nonzero" slot is rejected, not silently accepted); `license_len`/`evidence_root_count` bounds-checked with `checked_*` arithmetic against remaining section bytes; evidence roots strictly ascending lexicographically with no duplicates (same `last >= current` rejection FWDA uses for both conditions at once); total consumed length must equal section length exactly (no trailing bytes).

**Scope of this PR:** `prov.rs` module (borrowed `Prov<'a>` view + canonical builder), error variants, unit/integration tests (bounds/order/duplicate/tamper — one negative test per check above, plus round-trip), and the RFC update (§3 table row + §5 bullet, matching how FWDA/EXCT are documented). **Not in this PR:** wiring PROV into `stage2::validate` mandatory-section completeness, wiring the builder into the compiler's actual write path, or any re-pin — those are phases 2/3 per the decomposition.

Starting implementation now in a fresh worktree (`issue-637-prov1-format` off current main, which includes #655-C0).


### comment 5308099527 — 2026-08-16T15:09:38Z — Casey-allard

## #637 phase 1 (PROV/1 format freeze) closed out

**PR:** #732 (References #637, not Closes — phases 2/3 keep this issue open).
**Merge commit:** `633f9d499c1c013fcd2e14e889e45aaebb279137` on `origin/main`. Verified: `crates/uor-r4-graph-format/src/prov.rs` present on main (347 lines), full merge-queue ladder (fmt/clippy/tests/no_std/κ, D2 differential, register conformance, cargo audit, Kani, Gate C, fuzz smoke) passed.

**Acceptance evidence:** PROV/1 wire format frozen — `Prov<'a>::parse` (borrowed, core-only reader) and `build_prov` (alloc-gated canonical builder) in `prov.rs`, new `FormatError::Prov*` variants, `docs/transformerless/R4G1.md` §3 table row + §5 bullet updated with the byte layout. 17 tests covering round-trip, builder canonicalization, and every bounds/order/duplicate/tamper case. No compiler write-path change, no artifact/CID/κ re-pin — nothing calls `build_prov` or reads a PROV section yet, so no existing artifact moved.

**Still open on this issue:**
- Phase 2 — producer propagation: emit/preserve the full identity record through cover, score, converter, and cached-cover paths; same-input byte-stability + changed-geometry-changes-CID proofs.
- Phase 3 — consumer/era adoption: bundle/API/evaluation fail-closed agreement, explicit legacy-unbound compatibility for pre-PROV artifacts, affected R4G1 identity updates, docs.

Staying assigned to this issue since phases 2/3 remain.


### comment 5308216529 — 2026-08-16T15:34:50Z — Casey-allard

**Phase 2a: opt-in producer propagation (PR pending)**

Scope: wire the frozen PROV/1 section (phase 1, PR #732, merged 633f9d49) into the real R4G1 producer path, additively.

- `uor-r4-graph-format`: `parse_digest_hex` — parses `blake3:<hex>`/bare-hex digest strings into PROV's 32-byte wire form.
- `uor-r4-graph-compiler`: new `induction::emit_r4g1_with_provenance(..., provenance: &ProvComponents<'_>)`. `emit_r4g1`/`emit_r4g1_with_tokenizer_cid` now delegate to a shared private inner emitter with `provenance: None` — their output is byte-identical to today, so **no pinned κ fixture moves**.
- `uor-r4-graph-cli`: `cover` gains an opt-in `--emit-provenance` toggle (off by default) binding whichever of `--source-manifest-kappa` (#597) / `--geometry-projection` (#600) / `--attention-operator` (#602) / `--dense-operator` (#704) the caller already passes into a real PROV section on the artifact itself, not just the JSON report.

Non-goals for this slice (explicitly deferred, still tracked here on #637):
- #601 tokenizer-adapter digest — the typed `TokenizerAdapter` record lives one layer above this call site (graph-cli, not graph-compiler); needs its own plumbing seam.
- A license / evidence-root CLI surface — no producer for either yet.
- Threading provenance through the score / converter / cached-cover paths (issue text's other named call sites) — separate slices.
- Flipping `cover`'s **default** output to always carry PROV. That changes every future artifact's κ and requires the maintainer-supervised re-pin procedure in AGENTS.md (`dump_baseline_kappa` → review diff → adopt → `TLESS_REPIN_WRITE=1`), which is a distinct, higher-risk operation from an additive opt-in flag. Deferred to phase 2b/3 with an explicit re-pin review.

Files: `crates/uor-r4-graph-format/src/{prov,lib}.rs`, `crates/uor-r4-graph-compiler/src/induction.rs` (+ `tests/induction.rs`), `crates/uor-r4-graph-cli/src/lib.rs`.

Evidence: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` across all three crates (439 tests, 0 failed), `cargo run -p xtask -- audit-limits` (R5), and a no_std build of `uor-r4-graph-format` (`--no-default-features`) — all green locally.

Cost: ~1 focused session (design carried over directly from phase 1's frozen layout).

Next action: opening the PR now (branch `issue-637-2-prov-propagation`), then the merge queue.


### comment 5308312380 — 2026-08-16T15:56:44Z — Casey-allard

**Phase 2a merged: opt-in PROV/1 producer propagation**

PR #733 merged: `d57a0e13725245b1071da4a58c4a84b602ab3621` (on `origin/main`).

Acceptance evidence:
- `uor_r4_graph_format::parse_digest_hex` — parses `blake3:<hex>`/bare-hex digest strings into PROV's 32-byte wire form (6 unit tests, no_std-safe).
- `uor_r4_graph_compiler::induction::emit_r4g1_with_provenance` — builds and appends a real PROV/1 section from a caller-supplied `ProvComponents`. `emit_r4g1`/`emit_r4g1_with_tokenizer_cid` are unchanged/byte-identical (verified by test), so **no pinned κ fixture moved** in this merge.
- `uor-r4-graph-cli`'s `cover` command gained an opt-in `--emit-provenance` flag (off by default) binding `--source-manifest-kappa`/`--geometry-projection`/`--attention-operator`/`--dense-operator` into that section when the caller asks for it.
- All CI gates green on merge: fmt/clippy/tests/no_std/κ, Kani proofs, cargo-audit, register conformance (R1–R6), Gate C, fuzz smoke, wasm-pack, D2 canonical differential.

Deferred follow-ups (phases 2b/3, still open on this issue):
- #601 tokenizer-adapter digest — its typed record lives one layer above the `cover` call site (graph-cli), needs its own plumbing seam.
- A license / evidence-root CLI surface.
- Threading provenance through the `score` / converter / cached-cover producer paths.
- Flipping `cover`'s **default** output to always carry PROV — requires the maintainer-supervised κ re-pin procedure (AGENTS.md: `dump_baseline_kappa` → review diff → adopt → `TLESS_REPIN_WRITE=1`), a distinct, higher-risk step from this additive opt-in slice.

Leaving Casey-allard assigned since phases 2b/3 remain open.


### comment 5308653759 — 2026-08-16T17:16:46Z — afflom

Sorry. I'd need to discuss it. Too much context here for me. 

### comment 5308908868 — 2026-08-16T18:06:34Z — Casey-allard

@afflom noted — pausing phase 2b/3 (producer propagation into score/converter/cached-cover, and the eventual default-flip + κ re-pin) here rather than proceeding, since that's exactly the kind of format-governance judgment call this project treats as a maintainer/reviewer decision, not something to push through unilaterally.

To make it easy to pick up whenever you have bandwidth: phases 1 and 2a are both merged, additive, and off by default — the frozen `PROV/1` section (#732) and the opt-in `--emit-provenance` `cover` flag (#733) haven't changed any existing artifact's κ or bytes, so there's no live migration pressure forcing a decision. What's actually gated behind your sign-off is phases 2b (wiring provenance into the score/converter/cached-cover *default* paths) and 3 (the consumer/era adoption + the R4G1 identity re-pin that follows).

If it'd help narrow the discussion: the specific things I'd want your read on are (1) whether the PROV/1 byte layout itself (posted above — magic/version/presence-bitmap/digest-slots/evidence-roots) looks right to you as a format matter, independent of when/whether it becomes non-opt-in, and (2) whether you see a problem with the *direction* (extend R4G1's existing PROV section vs. a new TLA8 era) versus just needing more time/context to review either one. Happy to set up a shorter, more scoped discussion (even async in a doc) if the full thread here is too much surface area at once.

Moving on to other open issues in the meantime; #637 stays assigned to me and I won't touch phase 2b/3 until this is resolved one way or the other.


### comment 5309135705 — 2026-08-16T18:59:23Z — Casey-allard

## Phase 3: default-flip PR opened

**PR:** #738 (References #637, not Closes — phases 2b consumer/era-adoption work named earlier in this issue can follow separately if still wanted).

Casey (2026-08-16): explicit sign-off to proceed with the default flip now, without waiting further on @afflom / @auser, exercising final maintainer decision authority. To make the exact decision unambiguous for anyone catching up on this thread:

> **The decision:** should every future `cover.r4g1` artifact always include a small provenance-identity section (PROV/1) — even an empty one when no identity info is passed — instead of only including it when a caller opts in with `--emit-provenance`? Casey said yes, go ahead.

**What changed:** `cover`'s emission path always calls `emit_r4g1_with_provenance` now (previously gated behind the opt-in `--emit-provenance` flag from phase 2a, PR #733). When no identity flags are passed, the artifact still carries a present-but-empty PROV/1 section (presence bitmap zero, zero evidence roots) — this is a deliberate, valid PROV/1 record per that function's own documented contract (already proven by `emit_r4g1_with_provenance_binds_a_prov_section_without_disturbing_the_unbound_path` in `crates/uor-r4-graph-compiler/tests/induction.rs`), not a legacy omission. `--emit-provenance` itself is retired as a gate but kept as a harmless no-op flag for backward CLI compatibility.

**Re-pin:** turned out not to be needed. I checked before assuming AGENTS.md's `dump_baseline_kappa`/`TLESS_REPIN_WRITE=1` Gate E procedure applied here — it doesn't; that re-pins a different, unrelated fixture (the `kappa_reproduction` test's llama2.c reference model in `uor-r4-core`). No fixture in this workspace hardcodes `cover`'s R4G1 bytes or κ literally, so I ran the full `uor-r4-graph-cli`/`uor-r4-graph-compiler`/`uor-r4-graph-format` test suites (unit + integration, including the presence-gated real-GPT-2 `gpt2_cover_certifies_source_parity_rows` canary) against the change and got zero regressions — that's the "sanity check it ourselves" Casey asked for.

Evidence: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, full test suites above, `cargo run -p xtask -- audit-limits` (R5), `cargo run -p xtask -- check-model` (CM-01) — all green.

@afflom @auser: still flagging for visibility — if either of you spots a format-correctness problem with PROV/1 itself, please say so here; happy to follow up.

Sending to the merge queue now.

### comment 5309231570 — 2026-08-16T19:21:36Z — Casey-allard

## Phase 3 merged — closing this issue

**PR:** #738, merged as `146a976e09c012cc673e566dcb131c0fde067adb` on `origin/main`.

**What landed:** `cover`'s emission path now unconditionally calls `emit_r4g1_with_provenance` — every future `cover.r4g1` artifact carries a PROV/1 section by default. When no identity flags (`--source-manifest-kappa` / `--geometry-projection` / `--attention-operator` / `--dense-operator`) are passed, the section is still present but empty (presence bitmap zero, zero evidence roots) — a deliberate, valid PROV/1 record, not an omission. `--emit-provenance` is retired as a gate and kept only as a backward-compatible no-op flag.

**Re-pin:** none needed. No fixture in this workspace hardcoded `cover`'s R4G1 bytes or κ literally — verified by running the full `uor-r4-graph-cli`, `uor-r4-graph-compiler`, and `uor-r4-graph-format` test suites (unit + integration, including the presence-gated real-GPT-2 `gpt2_cover_certifies_source_parity_rows` canary) against the change with zero regressions. AGENTS.md's `kappa_reproduction`/`TLESS_REPIN_WRITE` Gate E procedure re-pins an unrelated llama2.c reference fixture and was untouched.

**Evidence:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, the test suites above, `cargo run -p xtask -- audit-limits` (R5), `cargo run -p xtask -- check-model` (CM-01) — all green. All merge-queue CI (fmt/clippy/tests/no_std/κ, register conformance R1–R6, Kani proofs, cargo audit, Gate C trend alarm, fuzz smoke, wasm-pack, all-features build) passed.

**Recap of the full arc:**
- Phase 1 (#732): froze the PROV/1 wire format.
- Phase 2a (#733): opt-in producer propagation (`--emit-provenance`).
- Phase 3 (#738, this PR): default flip — PROV/1 always emitted, maintainer sign-off (Casey, 2026-08-16) to proceed without waiting further on collaborator review, since the format direction was already laid out clearly and this workspace had no live fixture at risk.

Everything originally scoped on this issue (format freeze, producer propagation, default adoption) is done. Closing.

@afflom @auser: still open for a follow-up if either of you spots a format issue with PROV/1 after the fact — happy to file a fresh issue for it.


---
_Comment/review ids in this file: 5299398241, 5307691397, 5307879665, 5308099527, 5308216529, 5308312380, 5308653759, 5308908868, 5309135705, 5309231570_

