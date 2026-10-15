# #643 — operator: MSA-structured selector (Modular Structural Arithmetic role-classes + cascade orbits) as a candidate target operator — A/B vs r4-route-attention/1

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-12T16:30:04Z
- **Updated:** 2026-08-18T23:13:48Z
- **Closed:** 2026-08-16T18:29:51Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/643
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Spawned from #604 (Ari-program campaign; discovered work per Casey's directive). Casey's early r3/r4 evaluation used MSA-structured selection — Modular Structural Arithmetic (Alex's formal system: structural-preserving moduli SP(p), doubling cascades with period | p−1, three structural equivalence classes under SP(p), decidable fragment MSA₀) — and recalls it outperforming softmax scoring. #604 deliberately implemented the issue-prescribed masked-XOR/popcount relation; this issue evaluates an MSA-structured selector as a SEPARATE versioned target operator: classify candidates by modular role (table lookup — P-4-legal), route within class by cascade position (precomputed orbit tables), deterministic tie-break; period structure as work partition. Scope: register as its own (id, version) in the #602 registry with truthful fields; reference semantics + witness on the #604 template; pre-registered A/B vs `r4-route-attention/1` and the standard-attention reference under the #626-style exit rule (criterion posted before any run; negative closes with table + dormant lane). PRECONDITION: the MSA papers are private inputs — spec fields must be derivable without publishing them; coordinate with Casey/Alex on what is publishable. Filed unassigned per board rules.

## Comments (9)

### comment 5273330269 — 2026-08-12T22:06:12Z — Casey-allard

## Pre-registration input from historical-program mining (mod-12 probe chain — measured)

Nearest measured ancestor to an MSA-structured selector. Three chained probes (all with analytic backing):
1. **Representation capacity:** k=1 harmonics provably CANNOT separate quarter-phase modular classes (linear acc 0.264 ≈ chance; analytic reason: 4 interleaved equilateral triangles). k=3 separates PERFECTLY (linear acc 1.000, within-class variance 0.0 analytically, confirmed over all 343,665 states).
2. **Under training:** the k=3 advantage is helpful but neither arm solves — routing dynamics prevent exploitation.
3. **Signal preservation:** the k=3 advantage does NOT survive trajectory dynamics (mid-trajectory decodability 0.3076; no-last ablation ≈ chance).

### Consequences to pre-register for any MSA-structured selector (when unblocked)
- **Encode class/role info in the route code itself** (compile-side, frozen-carry) — never expect it to survive selection dynamics.
- **`bucket-average/1` is a k=1-style aggregator** → provably cannot represent quarter-phase-type modular classes. A projection carrying the class-collapsing harmonic is required.
- **Add a "class linear-separability under the registered projection" anti-vacuity check** before any selector run (an instrument that cannot represent the classes would produce a vacuous null).

Still gated on Casey/Alex publishability word — no MSA specification fields are published here, only the empirical constraints an eventual selector must satisfy.


### comment 5307691807 — 2026-08-16T13:33:47Z — Casey-allard

## Maintainer decision recorded — MSA publishability precondition resolved

Casey (2026-08-16, direct instruction): "The private paper I uploaded can be used to the full extent and published with our updates as it was a very early and superseded preprint that Alex did years ago that he gave to the project to help with it." Casey approves publishing MSA specification fields drawn from that paper, updated as needed for this project's use, in full.

This resolves the issue's precondition ("MSA papers are private inputs — spec fields must be derivable without publishing them; coordinate with Casey/Alex on what is publishable"). MSA specification fields may now be published directly, superseding that constraint.

**Status:** publishability is unblocked, but the source paper itself is not yet available in the active working session — it was uploaded in a different context than the one currently implementing. Implementation (registering the MSA-structured selector per the issue's pre-registered A/B plan) will start once the paper content is available to work from. Not starting speculative implementation without the source material to avoid mis-transcribing the modular role-class / cascade-orbit definitions.

Next action: request the paper be attached to the active session, then execute the registration + reference-semantics + witness + pre-registered A/B plan already scoped in this issue.


### comment 5308346262 — 2026-08-16T16:04:38Z — Casey-allard

**Starting slice 1: register the MSA-structured selector operator + reference semantics + witness + pre-registered exit rule**

Both preconditions are now resolved: publishability was recorded above, and the source paper ("Modular Structural Arithmetic (MSA): A Formal System") is now available to work from.

**Scope (this slice):**
- Register `msa-structured-selector` as a new `(id, version)` pair in the #602 `AttentionOperatorSpec` registry (`crates/uor-r4-model-source/src/attention.rs::operator_spec`), with truthful `permitted_operation_class`/field values — table-lookup role classification + table-lookup cascade-position routing, no runtime multiplication/division/modulo (P-4-legal, matching the existing `r4-route-attention/1` entry's class).
- The concrete modulus is **p = 11**, directly from the paper's own worked theorems: MSA7 ("The 11-Theorem") proves `DP(11)` with role anchors `mod_11(γ)=2, mod_11(μ)=4, mod_11(ε)=8`; Theorem M4 proves the doubling cascade `(2,4,8,5,10,9,7,3,6,1)` has maximal period 10 = p−1 in ℤ/11ℤ. Both are reused as precomputed, provable compile-time tables — no runtime modular arithmetic, no unpublished content beyond what's approved above.
- Reference semantics (compiler-side, mirroring `RouteAttentionReference` in `uor-r4-graph-certify`, per the #604 template): classify each candidate by modular role via the 3-entry anchor table, route within class by its position in the 10-entry cascade-orbit table, deterministic tie-break `(role rank, cascade position, candidate id ascending)`.
- Witness (mirroring `RouteOpCensus`): closed-form operation-count census (table reads, compares) for replay verification, same pattern as `r4-route-attention/1`'s witness.
- A pre-registered A/B exit rule vs `r4-route-attention/1`, posted here **before any run**, per the #626 convention (positive/negative numeric criteria stated up front; a negative result closes with the recorded table + a dormant-lane entry in `model/ledger.toml`, not silent abandonment).

**Non-goals for this slice** (explicit follow-ups):
- The packed R4G1 runtime lowering (mirrors `route_attention.rs`'s two-stage compiler-then-packed pattern in #604 — reference semantics lands first).
- Running the actual A/B evaluation — that needs real corpus/training infrastructure and must not start before the exit criterion below is posted and stands unchallenged.
- Anything from the paper beyond the two already-published, already-proven theorems (11-Theorem, cascade periodicity) — no additional MSA content is needed for this slice.

Files: `crates/uor-r4-model-source/src/attention.rs`, new reference-semantics module in `crates/uor-r4-graph-certify/src/`, tests.

Next action: implement + test, then post the pre-registered exit rule as a follow-up comment here before opening the PR.


### comment 5308422785 — 2026-08-16T16:22:19Z — Casey-allard

Slice 1 is up: #734 (`model-source/graph-certify/#643: MSA-structured selector (reference + witness)`).

**Preconditions cleared:** publication of the MSA paper's content was already approved (Casey, prior comment on this issue). The paper itself (`Modular Structural Arithmetic (MSA)_ A Formal System.pdf`) was located and read in full.

**Design gap found and resolved before writing code:** the paper's MSA7 "11-Theorem" only proves role classification for 3 of the 11 residues mod 11 (2→Gen, 4→Med, 8→Man, via `mod_11(γ)`/`mod_11(μ)`/`mod_11(ε)`). Rather than guessing an extension for the other 8 nonzero residues, I asked directly. Casey's decision (2026-08-16): **cascade-position mod 3** — each residue's role rank derives from its position in the doubling-cascade orbit `(2,4,8,5,10,9,7,3,6,1)` (proven maximal-period by Theorem M4), taken mod 3. This is called out explicitly, in the code and tests, as this operator's own design extension — not a paper theorem.

**What's in #734:**
- `msa-structured-selector/1` registered in the #602/#604 operator-identity registry (`uor-r4-model-source::attention`).
- Compiler-side reference semantics + a replayable witness in a new `crates/uor-r4-graph-certify/src/msa_selector.rs`, built to be plug-compatible with the #604 `r4-route-attention/1` operator's conventions (same top-M selection, saturating-scoreq aggregation, lowest-index tie-break) for a future shared A/B harness.
- 17 tests in `msa_selector_643.rs`, including a "Pre-registered exit rule (issue #643 — binding)" module doc per the #626 convention.
- Verification: fmt, clippy `-D warnings`, full `cargo test -p uor-r4-model-source -p uor-r4-graph-certify` (zero regressions), and the R5 `audit-limits` gate — all clean.

**Deferred to a follow-up slice (issue stays open, I'll stay assigned):**
- Packed R4G1 lowering in `uor-r4-graph-runtime` (the #604 TARGET-operator template's second half — P-4-scanned packed source).
- The actual A/B evaluation run against `r4-route-attention/1` (no numbers exist yet; this PR only ships the operator and its reference/witness machinery).

Will post again once #734 clears the merge queue, and again once the A/B run has been executed with results.


### comment 5308539742 — 2026-08-16T16:49:47Z — Casey-allard

Slice 1 merged: #734, merge commit `d039c9df732e5920d5f01c7653c1b4e1a2297cdf` on `origin/main`.

**Landed:**
- `msa-structured-selector/1` registered in the operator-identity registry (`uor-r4-model-source::attention`), grounded in MSA7's 11-Theorem + Theorem M4's 11-Cascade Theorem, with the mod-3 role-rank extension for the 8 unproven residues explicitly labeled as this operator's own design choice (per your 2026-08-16 decision), not a paper theorem.
- Compiler-side reference semantics + replayable witness (`crates/uor-r4-graph-certify/src/msa_selector.rs`), built plug-compatible with the #604 `r4-route-attention/1` conventions for a shared A/B harness.
- 17/17 tests passing, R5 `audit-limits` clean, zero regressions across `uor-r4-model-source`/`uor-r4-graph-certify`.

**Still open on #643 (staying assigned):**
- Packed R4G1 lowering in `uor-r4-graph-runtime` (P-4-scanned, no float/multiply/divide/modulo) — the #604 TARGET-operator template's second half.
- The actual pre-registered A/B evaluation run against `r4-route-attention/1`, with a comparison table and closure verdict per the #626 convention already drafted in `msa_selector_643.rs`'s module doc.

Starting the packed-lowering slice next.


### comment 5308632361 — 2026-08-16T17:11:43Z — Casey-allard

Slice 2 is up: #735 (`graph-format/graph-runtime/graph-certify/#643: packed R4G1 lowering for MsaStructuredSelectorV1`), continuing directly from slice 1 (#734, merged).

**What's in #735:**
- A shared canonical instance substrate for `msa-structured-selector/1` in `uor-r4-graph-format::msa_selector` (the `route_attention` precedent: this crate has no dependency on either the certify or runtime crate, so it's the only place both can share one wire format).
- Classification (`role_rank`, `cascade_position`) is **precomputed at instance-build time** and baked into the row bytes, because the packed lowering is P-4-scanned (no runtime `* / %`) and cannot recompute `candidate_id mod 11` — this is exactly what the registry record's own `permitted_operation_class` field already promised ("deployed-integer-table-read-compare-add-no-runtime-modulo").
- The packed kernel itself in `uor-r4-graph-runtime::msa_selector` (P-4-scanned, now covered by the source-scan test alongside `route_attention.rs`).
- `MsaSelectorReference` (slice 1) refactored to build through this shared substrate instead of its own bespoke bytes, so the reference and the packed kernel now run on byte-identical instances — differential tests (`reference_and_packed_agree_bit_for_bit_on_the_pinned_fixture`, `..._across_the_shape_grid`) are a true comparison of two independent implementations, not two encodings that happen to agree.
- Test suite grew from 17 to 26 tests, all passing; full workspace-relevant test run (5 crates) clean; R5 `audit-limits` and the P-4 arithmetic scan both clean.
- `msa-structured-selector-dormant` registered in `model/ledger.toml`, mirroring `r4-route-attention-dormant`, restating this issue's pre-registered A/B run contract as the claim's activation gate.

**Still open on #643 (staying assigned):** the actual A/B evaluation run against `r4-route-attention/1` — no held-out evaluation harness wires either target operator into a shared loop yet. That harness is the next concrete step.

Will follow up once #735 clears the merge queue.


### comment 5308823436 — 2026-08-16T17:51:01Z — Casey-allard

Slice 2 merged: #735, merge commit `9c9bdfba` on `origin/main` (the CI failure reported earlier was an unrelated docs-consistency gate — `CONFORMANCE.md`/`model/ledger.toml` drift from the new `msa-structured-selector-dormant` claim, fixed in-PR via `cargo xtask check-model --write`).

**Landed:**
- Shared canonical instance substrate for `msa-structured-selector/1` in `uor-r4-graph-format::msa_selector`, with classification precomputed at build time (never recomputed on the P-4-scanned packed read path).
- Packed R4G1 kernel in `uor-r4-graph-runtime::msa_selector`, P-4-scanned, differentially verified bit-for-bit against the slice-1 reference on the pinned fixture and across a shape grid.
- 26/26 tests passing, R5 `audit-limits` and the P-4 arithmetic scan both clean, zero regressions.
- `msa-structured-selector-dormant` registered in `model/ledger.toml`, mirroring `r4-route-attention-dormant`.

**Still open on #643 (staying assigned):** the actual pre-registered A/B evaluation run against `r4-route-attention/1` — no held-out evaluation harness exists yet that wires either target operator into a shared loop. Starting that harness next.


### comment 5309007931 — 2026-08-16T18:29:50Z — Casey-allard

Closing with the pre-registered A/B result.

**PR #736 merged**, merge commit `2ea108df` on `origin/main`: `crates/uor-r4-graph-certify/src/msa_ab_harness.rs`, a harness that drives `msa-structured-selector/1`'s deployed packed kernel through the SAME synthetic held-out corpus and whole-model scope #605's replacement ladder measures `r4-route-attention/1` against, reusing `route_fit_report::scope_metrics` verbatim (no reimplementation), plus a `#457`-convention unigram floor computed over the same corpus.

## Measured result (2026-08-16 fixture)

```
                                              top-1       bits/token
r4-route-attention/1  (fitted, whole-model)  0.990234    4.971638
msa-structured-selector/1 (position-only)    0.642578    5.532591
unigram floor (this corpus)                  0.035156    5.854087

gate: msa.top1 >= route.top1 + 0.02   -> FALSE
gate: msa.bits <= route.bits          -> FALSE
gate: msa.bits <  unigram.bits        -> TRUE

verdict: NEGATIVE
```

## Reading the result

`msa-structured-selector/1`'s classification is purely positional (`candidate_id mod 11`, MSA7's 11-Theorem + this project's own cascade-position-mod-3 extension) — it carries no information about the synthetic teacher's actual content-directional attention. It clears the trivial unigram floor by a wide margin (a real, non-vacuous predictor), but falls well short of the fitted, content-aware `r4-route-attention/1` arm on both pre-registered margin gates. This is consistent with the mod-12 probe-chain warning pre-registered on this issue before implementation: representation capacity for a modular class does not, by itself, survive selection dynamics without content-aware encoding.

Per the exit rule pre-registered on this issue before any run: **NEGATIVE closes #643** with this comparison table. No further code change is required — `msa-structured-selector-dormant` is already registered in `model/ledger.toml` (added in #735) with an activation gate that restates exactly this contract, so the disposition is already correct and stays unchanged.

## What shipped across this issue

- `msa-structured-selector/1` registered in the #602 operator registry, grounded in MSA7's proven theorems with the mod-3 extension explicitly labeled as this project's own design choice (#734).
- Compiler-side reference semantics + replayable witness (#734).
- Packed R4G1 lowering, P-4-scanned, differentially verified bit-for-bit against the reference (#735).
- The pre-registered A/B harness and the result above (#736).

Thanks for the paper and the design-decision calls along the way (cascade-position-mod-3, publishability). Closing.


### comment 5335360410 — 2026-08-18T23:13:48Z — Casey-allard

**Declared contract amendment (S1/S3 shared corpus):** the pre-registered A/B's trace corpus is amended to the same bundle as #605's real arm — `smollm2-360m-broad` re-observed under #603 `full/1` (layers [0,4,9,13,18,22,27,31], S=8), #804 measurement-only Accelerate backend recorded in provenance. Full amendment text with gates-unchanged declaration: https://github.com/UOR-Foundation/uor-r4/issues/605 (latest comment). The A/B runs only after a positive #605 ladder, per its own pre-registration.


---
_Comment/review ids in this file: 5273330269, 5307691807, 5308346262, 5308422785, 5308539742, 5308632361, 5308823436, 5309007931, 5335360410_

