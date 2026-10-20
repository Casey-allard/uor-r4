# #273 — Rebase uor-r4 onto UOR-Foundation/template (claim-register conformance discipline) — phased adoption epic

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T18:48:53Z
- **Updated:** 2026-08-12T00:42:49Z
- **Closed:** 2026-08-08T22:12:02Z
- **Labels:** enhancement, status: on-hold
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/273
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
Alex published `UOR-Foundation/template` (initial commit 2026-07-28): the gate machinery with no content — claim register as single source (`model/*.toml`, R1), generated `CONFORMANCE.md`, one Gherkin scenario per conformance ID with failing-test-first ordering enforced by a meta-gate (R3), honesty levels `some-true`/`build`/`open` (R2), deferral audit (R4), sanctioned-errors audit (R5), cargo-deny (R6), `just vv` as the one acceptance gate, falsifiability table in VERIFICATION.md. Direction requested: **rebase uor-r4 onto a version of this.**

### What maps cleanly (uor-r4 already has proto-equivalents)
- `docs/formal_vocabulary.md` + `check_claim_wording.py` → the honesty meta-gate; levels map: Guarantee→`build`/`some-true`, Empirical Criterion→`open`.
- `uor-r4-proof-model`'s proof-status matrix → the ledger.
- The existing cucumber BDD suite (`features/`, `cargo test --test bdd`, 94 scenarios) → `features/suites/` with ID tagging.
- #256/#257's conformance vectors → first register rows + suites (this template is the scaffold that work was aimed at).
- `rust-toolchain.toml` (#238) already aligned in spirit.

### Policy collisions to decide BEFORE rebasing (flagged, not resolved here)
1. **R4 ("nothing is deferred — no deferral marker, no stub, no capability behind a flag") vs #249's dated-deferral record and the dormant algebra modules kept deliberately.** Under R4 as written, `cd_space`/`lie_jordan`/`bott_fock`/`quantum_cover` cannot ride along. Options: (a) dormant modules move to a separate non-rebased crate/repo until their activation gates arrive; (b) a scoped R4 carve-out for registered `open` capability claims; (c) activation is accelerated. Team decision — affects #249's settlement and #234's Bott-Fock companion.
2. **`just vv` vs the current CI shape** (five required checks, merge queue, context split from #240/#241): `vv` should become the core gate's body rather than a parallel truth; the queue/stub structure needs re-expression in template terms.
3. **Era-notes / re-pin discipline** (κ baselines, #265) has no template counterpart yet — candidate improvement to send back upstream ("feel free to make it better").
4. **Scale**: uor-r4 is ~65k LOC, 10 crates, live measurement docs (BASELINE.md) — the register migration of existing claims is the bulk of the work and should be incremental (register-as-you-touch), not big-bang.

### Proposed phasing
- **Phase 0**: team reviews the collisions above; Alex confirms whether uor-r4-specific needs (era notes, deferral carve-out or module relocation, merge-queue integration) should land in the template first ("make it better") or be handled downstream.
- **Phase 1**: adopt the skeleton — `model/`, `xtask` gates, `Justfile`, `deny.toml`, honesty meta-gate — with an initially thin register (the #256/#257 vectors + the P-4 mul-free scan as the first rows); `just vv` wired into CI alongside existing checks.
- **Phase 2**: migrate claims incrementally (formal_vocabulary Guarantees, proof-matrix rows, BASELINE Empirical Criteria) — every touched subsystem registers its claims as it changes.
- **Phase 3**: retire the superseded bespoke gates once their register equivalents have each been seen to fail (the template's own falsifiability rule).

### DoD
Phase 0 decisions recorded here; Phase 1 lands with `just vv` green in CI; a migration ledger in this issue tracks Phase 2 coverage; no regression in the existing five required checks at any point.

*Sequenced at the end of the current queue (#234–#268 work continues on the existing discipline meanwhile — the rebase must not freeze in-flight measurement-driven decisions).*


## Comments (12)

### comment 5108613103 — 2026-07-28T19:13:21Z — Casey-allard

**Phase 0 decision (Casey, 2026-07-28): dormant capability is blessed in place.**

The dormant algebra modules (`cd_space`, `endomorphism`, `lie_jordan`, `bott_fock`, `quantum_cover`) stay where they are. Under the rebase they are registered as **`open`-level claims** in the template's register — the register is the sanctioning mechanism, so a registered `open` row ("implemented, unmeasured; activation gate = …") is the template-blessed expression of what the #249 deferral table records. No relocation, no research crate, no deletion.

Remaining Phase 0 items (for Alex): confirm the audit-deferral tooling accepts registered-`open` dormant capability under this reading (if the scanner is stricter, the ask is to make the register the carve-out — "make it better" feedback rather than downstream workaround); and the `just vv` ↔ merge-queue/context-split integration story.


### comment 5112394907 — 2026-07-29T03:18:45Z — Casey-allard

Register groundwork: PR #275 adds the evidence note for the prime-router geometric-context result (mechanism refs, section-6 ablation measurements, scope limits) and drafts the corresponding `some-true` register entry (`geometric-context-carrier`, with falsifier) ready to lift into `model/*.toml` when the template structure lands. Distinct from the open #243/#244 question — this records the *context-carrier* claim only, with the generation claim explicitly out of scope.

### comment 5181903382 — 2026-08-04T16:33:52Z — Casey-allard

Status: **ON HOLD** (Casey, 2026-08-04). This rebase is a meticulous, phased effort that deserves dedicated focus rather than background attention; parking it explicitly so it reads as deliberately paused, not stale. The #393 program (measurement discipline, criterion harnesses, claim-record practice built up this week) should make several of its conformance phases easier when it resumes. Alex: no change to the epic's content or your ownership of the process side — this is a scheduling status only.

### comment 5186987578 — 2026-08-05T02:54:31Z — Casey-allard

The declared falsifier finally ran, inside #421's M-R1 harness (PR #428): routed-state anchors 7.6% free top-1 vs the same router outputs deterministically shuffled to wrong positions 3.1%. Routed > shuffled by +4.55pp — routing carries positional information; the falsifier does NOT fire. Relevant to un-holding this issue.

### comment 5187153200 — 2026-08-05T03:21:27Z — Casey-allard

Un-holding per Casey (2026-08-05): the declared falsifier ran inside #421 M-R1 (PR #428) and did not fire — routed-state anchors beat position-shuffled ones by +4.55pp on the pre-declared rule. The prime-router evidence question is live again; next step rides the #421 follow-on arms (non-oracle query context) and #422 (Hopf retrieval quality).

### comment 5187724915 — 2026-08-05T04:57:13Z — Casey-allard

Non-oracle evidence (PR #429): router anchors from DRAFTED context still beat both nulls (4.9 vs 2.1 unigram / 3.1 shuffled). The routed-state claim now holds under generation-realistic conditions.

### comment 5194543444 — 2026-08-05T16:35:26Z — Casey-allard

Correction (Casey + assistant, 2026-08-05): the two routed-state falsifier comments above were posted to this issue in error — that evidence belongs to #421's record (PR #428/#429), where it is correctly filed. This issue is the template-rebase epic; none of its phases have started and Phase 0 requires team decisions with Alex. Re-holding per Casey's original judgment.

### comment 5199339187 — 2026-08-06T01:26:30Z — Casey-allard

Correction: two comments posted here on 2026-08-05 about a router-anchor falsifier belong to the router-reconnection work, not to this template-rebase epic — misfiled by me. Please disregard them here; the records live with that track. Restoring the on-hold status this issue had before.

### comment 5205251084 — 2026-08-06T13:28:49Z — Casey-allard

Correction (2026-08-06): two comments posted to this issue on 08-05 (the #421 router-anchor falsifier results) were MISFILED — they belong to the router-reconnection measurement track, not to this template-rebase epic. This issue's scope and its on-hold status are unchanged. Apologies for the noise.

### comment 5212676213 — 2026-08-07T05:09:48Z — Casey-allard

Level-set only — status unchanged (on-hold), recorded so the parked state is legible.

Nothing has been implemented. The only artifact in the tree is `docs/prime_router_geometric_context_evidence.md` §5, which contains a *proposed* `[[claim]]` TOML entry referring to "the claim register introduced by the template rebase (issue #273)". **No register file exists** anywhere in the repo. `scripts/check_claim_wording.py` and `docs/formal_vocabulary.md` are the wording gate from the #160-#170 era and are unrelated to template conformance.

**If this is revived, phase 1 is concrete:** land the claim-register file and schema, a CI validator for it, and migrate the existing claims into it. Until someone wants that, on-hold is the honest state — nothing in the tree rots while it waits.

### comment 5228081999 — 2026-08-08T20:49:55Z — Casey-allard

## Phase-0 analysis + Phase-1 plan (drafted to unblock the team decision)

This resolves the four flagged collisions with a concrete recommendation each, then gives a Phase-1 skeleton-adoption plan. **These are proposals for Alex/the team to ratify — not decisions taken.** Grounded in the current repo state (verified): the four dormant modules exist (`crates/uor-r4-core/src/transformerless/{cd_space,lie_jordan,bott_fock}.rs`, `crates/uor-r4-graph-compiler/src/quantum_cover.rs`); CI is a single required `gates` job (`fmt / clippy / tests / no_std / κ`) running the full ladder (claim-wording, inference-contract audit, fmt, clippy, nextest, BDD, doc tests, no_std, deterministic rebuild) plus separate wasm / Kani / cargo-audit / Gate-C-trend / fuzz jobs, merged via `merge_group`; no template skeleton (`model/`, `deny.toml`, `xtask`) is present yet; `Justfile` exists.

### Collision 1 — R4 "nothing is deferred" vs the dormant modules (+ #249)

Recommend **option (b): a scoped R4 carve-out for registered `open` capability claims.** Each dormant module registers as an `open` claim carrying (i) its activation gate — the pre-declared measurement that would move it from `open` to `build`/`some-true` — and (ii) a pointer to the negative/pending record that justifies keeping it. This preserves R4's actual intent (no *silent* stub, no unregistered capability) while honoring the repo's standing discipline that measured-negative and awaiting-gate branches are kept with data (the CD term, FMM, E8, Bott–Fock all have that shape already). Relocation to a separate crate (option a) is heavier, breaks co-location with the code that would activate them, and buys nothing R4-registration doesn't. This directly settles #249's dated-deferral record: the deferral becomes a register row with a falsifier, not a comment.

### Collision 2 — `just vv` vs the current CI shape

Recommend **`just vv` becomes the body of the required `gates` job**, not a parallel truth. The queue/merge_group structure and the dual-context stub reporting (issue #240/#241) stay exactly as-is — only the job's *steps* become `just vv`. Scope for v1: `vv` = claim-wording + inference-contract audit + fmt + clippy(-D warnings) + nextest + BDD + doc tests + no_std + deterministic-rebuild (the current `gates` ladder verbatim). The heavy separate jobs (wasm, Kani, cargo-audit, Gate-C-trend, fuzz) **stay separate required checks initially** and fold into `vv` only if/when it's cheap to do so — folding Kani/audit in now would blow the local `vv` runtime and discourage its use.

### Collision 3 — era-notes / re-pin discipline (no template counterpart)

Recommend **sending it upstream** (the template's own "feel free to make it better" invitation). The κ-era-note + maintainer-gated re-pin discipline (AGENTS.md §κ-reproduction, #265) is a uor-r4 innovation the template lacks and would benefit from. Phase-0 deliverable: a short proposal to Alex to add an `era`/`re-pin` concept to the register schema, so uor-r4 adopts a template that already has it rather than carrying a downstream divergence.

### Collision 4 — scale (65k LOC; register migration is the bulk)

Recommend **register-as-you-touch**, not big-bang. First rows: the #256/#257 conformance vectors and the P-4 mul-free scan (the template is the scaffold that work was already aimed at). Every subsystem registers its claims as it is next modified; no freeze on in-flight measurement work.

### Phase-1 plan (once Phase-0 is ratified)

1. Land the skeleton — `model/` (claim register TOML, R1), `xtask` gates, `deny.toml`, honesty meta-gate — with a thin initial register (#256/#257 vectors + the mul-free scan). `just vv` wired into CI **alongside** the existing `gates` job (not replacing it yet), both required, so the queue is never left without a green required check.
2. Map existing gates to template equivalents, in place: `check_claim_wording.py` → honesty meta-gate (Guarantee → `build`/`some-true`, Empirical Criterion → `open`); `uor-r4-proof-model` matrix → the ledger; `features/` BDD (94 scenarios) → `features/suites/` with conformance-ID tags.
3. Register the four dormant modules as `open` claims with their activation gates (the Collision-1 carve-out).
4. Once each bespoke gate's register equivalent has been *seen to fail* (the template's falsifiability rule), retire the bespoke gate (Phase 3).

### What the team needs to decide (Phase-0 sign-off)

(a) carve-out (b) vs relocation (a) for the dormant modules; (b) `just vv` v1 scope — which checks fold in vs stay separate required jobs; (c) whether era-notes go upstream into the template first; (d) register-as-you-touch phasing. With those four ratified, Phase 1 is executable and I can take it on.

Leaving #273 open for that sign-off.


### comment 5260727063 — 2026-08-12T00:42:49Z — Casey-allard

Input for this template-adoption tracker from the #589 ecosystem audit (#625, UOR-Atlas-UTQC survey) — flagged for @Alex / Ari as template/process governance; filed as a menu, not a decision. Atlas is the fullest instance of the shared template lineage (r4's Justfile already marks the #273 wired slice), and its `just vv` carries gates r4 does not have yet, all cheap to lift with zero new dependencies:

- **anti-hardcode** — a grep gate forbidding literal verdict fields (`is_dense: true`-style) and `assert!(true` anywhere in the tree; catches a whole class of vacuous-test drift.
- **oracle-verify + PROVENANCE.md** — every committed external artifact (our pinned teacher/corpus κ fixtures) must match a checksum recorded beside a provenance note; turns silent fixture drift into a gate failure.
- **portability as a gate** — r4 already builds wasm32; atlas pins it (plus thumbv7em) as an explicit `just` target so a regression is a red gate, not a discovery.
- **workspace [workspace.lints] deny posture** — `unwrap_used` / `expect_used` / `panic` / `todo` = deny at workspace level (atlas also forbids `unsafe_code` outright).
- Noted but NOT recommended for r4: the LaTeX `paper-check` chain, and `atlas-pin-check` (an **online** gate — would add a network dependency to CI).

Also for awareness: the κ-injectivity census discipline from atlas tqc-vv is filed as its own r4 implementation issue (linked from #625) since it is code-level, not template-level.



---
_Comment/review ids in this file: 5108613103, 5112394907, 5181903382, 5186987578, 5187153200, 5187724915, 5194543444, 5199339187, 5205251084, 5212676213, 5228081999, 5260727063_

