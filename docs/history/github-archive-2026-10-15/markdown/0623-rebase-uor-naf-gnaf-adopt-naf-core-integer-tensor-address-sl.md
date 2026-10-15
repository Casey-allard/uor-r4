# #623 — rebase(uor-NAF/GNAF): adopt NAF core+integer+tensor+address slice (dual-label sha256/blake3); GNAF claim-class vocabulary — GNAF layers on NAF, does not supersede

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-11T23:02:35Z
- **Updated:** 2026-08-12T01:43:43Z
- **Closed:** 2026-08-12T01:43:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/623
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #589 (ecosystem rebase, step 5: uor-NAF + uor-GNAF). Both are gist specs, no impl repos: `uor-naf/1-draft.6` (2026-08-08, 4047 lines) and `uor-gnaf/1-draft.1` (2026-08-10, 3227 lines). Full spec-read completed 2026-08-11; findings below cite spec line numbers.

## Does GNAF supersede NAF? **No — orthogonal layers.**

GNAF §20 (L3086–3089) names NAF an *informative* "motivating specialization"; NAF's digit algebra is "not axioms of UOR-GNAF". GNAF assigns **no wire format at all** (§14.1) and never mentions κ. NAF is the concrete byte/addressing layer; GNAF is the abstract semantic-quotient + certified-optimality theory. Adopt independently.

## Key compatibility findings

1. **Hash axis is NOT pluggable — hard sha256 requirement.** NAF κ labels MUST be exactly `sha256:` + 64 lowercase hex (71 bytes; L1506–1507, L1244–1249: "algorithm agility requires a new outer version"). r4's blake3 κ cannot be a NAF κ. Resolution the spec itself blesses (§15 L4032–4035, "profile relation"): **dual-label** — blake3 stays r4's internal store key; sha256 NAF κ emitted only at the interchange boundary; the two typed so they can never be compared (§2 L123–137 requires exactly that discipline).
2. **No NAF home for r4's core artifacts.** Domain tags are closed at 01–05 and execution traces are explicitly outside v1 (L760–762). Graphs/routes/witnesses need a registered state/operator domain (Adapter C) — which demands a canonicalizer **completeness proof** we'd supply ourselves (L685–700, NAF-OBL-020). That's the expensive path; defer.
3. **All NAF κ are provisional until freeze.** The draft id is baked into every manifest preimage; at `uor-naf/1` freeze every κ changes and full regeneration is mandated (L3966–3968). Don't pin NAF κ into anything long-lived yet.
4. **GNAF pruning-safety warning applies to our router today** (adopt as review lens even before any GNAF claim): dropping route candidates on observed cost under one machine/cost-model is not extension-safe (§6.5 L801–817; fixture GNAF-VEC-07 "current-cost pruning is unsafe" is a *normative rejection*). Also L1890–1891: minimum op-count/weight alone does not bridge to "faster" — a cost-model bridge theorem is required. Both bear directly on how we word router optimality claims.
5. Neither spec mentions R4, geometry, tropical, E8, matmul, or blake3 anywhere — zero free alignment; all mapping is by hand.

## Minimal first-adoption slice (per the specs' own capability algebra)

- [ ] **NAF `core` + `integer` + `tensor`** (deps: none→core→integer; all uniqueness/minimality obligations already proved in-document, NAF-OBL-001..008): exact bigint/tensor canonical coefficients — the coordinate-reduction primitive with proofs attached. Storage-tier split gives "same semantic κ, different artifact κ" for width changes (L1509–1513) — exactly our determinism-vs-representation axis.
- [ ] **NAF `address` + `address-binding(integer|tensor)`** (pure §8.2 manifest/hash transform; golden vectors L3534–3605 + mandatory rejection corpus L3784–3823 imported as tests) — with the dual-label decision above recorded first.
- [ ] **NAF 6-state error taxonomy** (`invalid|unresolved|unsupported|unadmitted|commitment-failure|accepted`, NAF-DOM-006) + `policy-out-of-domain` as distinct outcome — cheap, prevents a conformance-failure class later.
- [ ] **GNAF claim-class vocabulary + dual status reporting** (§12.4: 13 classes incl. `best-known`/`measured-best-among-tested`; §12.5: ExecutionStatus ⊥ OptimizationStatus) as r4 witness labels — pure discipline, zero code, immediately upgrades certificate honesty. Note: unscoped "optimal" labels are *nonconforming* (L2306–2307).

## Deferred (recorded so the option isn't lost)

Adapter B (Atlas words; only wanted for prefix-interval sharding — not a dependency of integer/tensor); Adapter C domain registration for R⁴ states (needs completeness proof); `uor-naf-plan` (no registered wire exists, NAF-OBL-027); `execution`/`optimality` capabilities (24+22 requirements; we cannot discharge coverage today); GNAF `gnaf-state(G,U)` — **naming trap**: calling any r4 object a "GNAFSpace" alone triggers the full obligation set (L1896–1898); GNAF `interchange` (nothing to conform to in Draft 0.1).

## Verification for any adoption PR

Four gates + suite green; NAF golden + rejection vectors pass; blake3 store κ byte-stable (dual-label is additive); no runtime-kernel change; claim-wording gate extended to reject unscoped optimality labels.


## Comments (2)

### comment 5260881523 — 2026-08-12T01:07:19Z — Casey-allard

## Delivery record (PR #631 in queue)

The pre-registered adoption slice ships as the non-shipped crate `uor-r4-naf`: **core + integer + tensor + address + address-binding(integer|tensor)** per the spec's own capability algebra (§11), with:

- §12.2 golden chains reproduced **byte-for-byte** as tests — both integer-3 and tensor (2)/(0,−1) chains, all four κ labels, exact manifests and payload hex. §12.4 mandatory rejection corpus verbatim (every tag). §3 core laws swept ±1000 + wire round-trips.
- Arbitrary-precision normalization (math-int/1 genuinely unbounded — no i128 ceiling), strict decoders with frame bounds, and the full artifact decode with mandatory embedded-semantic-κ reconstruction; tamper tests prove `CommitmentFailure` ≠ `Invalid` ≠ `Unsupported` land in their own lanes.
- **Dual-label rule enforced in types**: `NafLabel` is sha256-only and unconstructible from strings; blake3 store keys cannot flow in (§2 commitment separation; the #623 decision).
- GNAF §12.4/§12.5 vocabulary in `uor_r4_naf::claims` — labels only; wiring into certify witness output deliberately left with #515 governance.

Honest conformance statement this delivers: `uor-naf/1-draft.6` — capabilities core, integer, tensor, address, address-binding(integer), address-binding(tensor). No Atlas, no domain(D), no execution, no optimality (deferred with reasons in the issue body). **All NAF κ are provisional until `uor-naf/1` freezes** (the draft id is in every preimage; freeze mandates regeneration) — nothing long-lived should pin them yet. Closing when #631 merges.


### comment 5261102656 — 2026-08-12T01:43:43Z — Casey-allard

PR #631 merged — the pre-registered NAF slice + GNAF vocabulary shipped as uor-r4-naf with the golden chains and rejection corpus as tests (record above). Deferred items and the κ-provisionality warning stay recorded in the issue body and crate docs.


---
_Comment/review ids in this file: 5260881523, 5261102656_

