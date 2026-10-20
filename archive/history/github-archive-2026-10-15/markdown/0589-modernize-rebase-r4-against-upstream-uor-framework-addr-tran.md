# #589 — Modernize + rebase r4 against upstream (UOR Framework, ADDR, transformerless, matmul, NAF, kappa-registry) — BLOCKED until all in-flight work done

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-10T20:03:54Z
- **Updated:** 2026-08-12T01:49:28Z
- **Closed:** 2026-08-12T01:49:28Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/589
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Blocked tracking issue — do NOT start until all other in-flight work is complete

Marker for a coordinated modernization/rebase of r4 against its upstream projects, which have all advanced significantly:

- **UOR Framework** — substantial updates by Alex.
- **UOR ADDR** — substantial updates by Maura.
- **transformerless** — significant updates.
- **matmul** — significant updates.
- **NAF** (Alex) — a new repo to help with **symbolic and semantic reasoning**; evaluate adopting it.
- **kappa-registry** (Alex) — evaluate adopting it for κ/CID registration.

### Work (when unblocked)
Once the entire project is steady, update r4's pinned dependencies against the current versions of all of the above and **rebase r4 against anything those projects can contribute** — pulling in upstream capability rather than duplicating it, and aligning r4's substrate with the modernized Framework/ADDR/transformerless/matmul and the NAF + kappa-registry surfaces.

### ⛔ Blocked — start condition
**This must NOT be started until ALL other in-flight r4 work is completed** — specifically #515 and #531, and every subsequent issue that comes out of them. This issue exists only to mark and hold the work so the option isn't lost; do not begin it while any of that work is still open.


## Comments (4)

### comment 5259188746 — 2026-08-11T21:42:25Z — Casey-allard

## Unblocking + upstream audit (2026-08-11, Casey-directed)

Casey has directed this work to start. Start-condition status, checked against the live board today: #515 and #531 are **closed**; there are **zero open PRs**; the open #597–#613 program (model-source / attention / trace / compiler) is queued follow-on work, **all unassigned — nothing in flight**. Proceeding modularly: one sub-issue per upstream, smallest-delta first, no monolithic PR.

### Audit: current pins vs upstream (measured against fresh clones of every upstream HEAD today)

| Upstream | r4 today | Latest upstream | Status |
|---|---|---|---|
| **UOR-Framework** | uor-foundation / -sdk / -verify **0.5.2**, git pin `51c0138` | main HEAD = `51c0138` (2026-05-23); latest tag v0.5.2; crates.io uor-foundation **0.5.2** | ✅ **CURRENT — pin IS upstream HEAD.** Residual work is build-graph hygiene (see sub-issue). Legacy v6.x/v7.0.0 tags are the pre-reset March line, *older* than the 0.5.x line despite the numbers. |
| **uor-addr** | **0.2.0**, git pin `165b51e` | main HEAD = `165b51e` (2026-06-20) = v0.2.0+5; no newer tag | ✅ **CURRENT — pin IS upstream HEAD.** |
| **uor-prism** | **0.4.0** transitive only (via uor-addr, crates.io) | v0.4.0 = main HEAD (2026-05-24); crates.io **0.4.0** | ✅ Current transitively. Open question: adopt any prism-numerics / prism-tensor / prism-crypto surface directly? |
| **UOR-Atlas-UTQC** | not integrated | **v0.1.3** (2026-07-19); 9 `tqc-*` crates | ⬜ Evaluate: hologram substrate; possibly only partially relevant to r4. |
| **uor-matmul** | not integrated | no releases; HEAD `b13c984` (**2026-08-11**, active); 9 crates; needs rust ≥1.89 (we pin 1.97.1 ✓) | ⬜ Feature-mine. Exact coded-operand accumulation — candidate for teacher/compile-side only; **router runtime stays zero-matmul**. |
| **uor-NAF** | not integrated | gist spec `uor-naf/1-draft.6` (2026-08-08), no impl repo | ⬜ Spec-only. Exact integer/tensor coefficients + κ over canonical manifests — direct candidates for coordinate reduction + content addressing. |
| **uor-GNAF** | not integrated | gist spec `uor-gnaf/1-draft.1` (2026-08-10) | ⬜ Layered semantic-quotient + optimality-envelope spec; evaluate whether it supersedes or composes with NAF for r4's purposes. |
| **kappa-registry** | not integrated | no releases; HEAD `2929832` (2026-07-28); `/v2/` registry + kappa-macros | ⬜ Evaluate for κ/CID registration of compiled graph artifacts. |

Toolchain/shared-dep notes: r4 pins rust **1.97.1** (covers matmul's 1.89 floor); blake3 pinned `=1.5.0` repo-wide — any adopted upstream code must respect that pin or bump it deliberately (Gate E κ re-pin caveat in AGENTS.md).

### Sub-issue order
1. uor-framework (filed next, smallest delta — hygiene only)
2. uor-addr → 3. uor-prism → 4. uor-matmul → 5. uor-NAF/GNAF → 6. kappa-registry → 7. UOR-Atlas-UTQC (last: least directly relevant)

Core safeguards for every sub-issue: **zero matmul in the router runtime**, no transformer gating, preserve stack-allocation/zero-copy patterns, full `cargo test` + BDD + conformance gates green before any PR.


### comment 5259688672 — 2026-08-11T22:40:26Z — Casey-allard

**Design-intent correction from Casey (2026-08-11), superseding one line of the audit comment above:** uor-matmul is not "teacher/compile-side only." It is matrix multiplication reverse-engineered *for* the transformerless architecture — exact accumulation over coded operands, no real floating-point matmul — and it is the library this project **wants to use for the mechanisms that need it**. The standing prohibition is on real floating-point matmul and transformer gating, which uor-matmul's exact coded-operand accumulation does not violate.

Note for the uor-matmul sub-issue when it's filed: r4's deployed-kernel invariant (AGENTS.md) is stricter than "no float matmul" — it bans *all* multiply/divide/float in the runtime kernel, machine-checked by the P-4 source scan. So the sub-issue must map which r4 mechanisms adopt uor-matmul (compiler, certifier, teacher-side, offline transforms are all fair game today) and whether any runtime-kernel use requires a deliberate, Casey/team-approved evolution of that invariant rather than a silent weakening.


### comment 5261102780 — 2026-08-12T01:43:44Z — Casey-allard

## Campaign complete — all seven sub-issues closed with records (2026-08-12)

| Sub-issue | Outcome | PRs |
|---|---|---|
| #618 uor-framework | Pin was already upstream HEAD; lock deduped 2→1 copies of uor-foundation/-sdk via `[patch.crates-io]` (direction flip recorded) | #619 |
| #620 uor-addr | Current; #597/#603 adopt `uor_addr::json::address_blake3` manifest roots; ring no-adopt; upstream safetensors realization requested (uor-addr#14) | — |
| #621 uor-prism | Current transitively; one axis candidate evaluated+rejected with reason; uor-matmul owns exact matmul | — |
| #622 uor-matmul | Tropical path P-4-legal (memo); dev-only parity dep landed; E8 no-swap standing decision; teacher cblas sites recorded; pin discipline set | #627 |
| #623 NAF/GNAF | `uor-r4-naf` crate: core+integer+tensor+address(-binding) slice, golden chains byte-for-byte, typed dual-label rule; GNAF claim vocabulary | #631 |
| #624 kappa-registry | `cargo xtask kappa` client, E2E-verified vs a locally built registry; hosting decision left with the team | #630 |
| #625 Atlas | Discipline ported, crates rejected; κ-injectivity retrofit filed (#628); template gate menu posted to #273 | — |

**Measured result the campaign produced:** #626 (pre-registered, from #622's legality memo) — NEGATIVE, decisive: the compiled forward-transition seat carries no route-composable token signal at any hop on the current cover (floor reproduced; both tropical arms ~5.9 bits below it; deranged null ordered correctly). Registered as `tropical-route-dormant` (#629) with an explicit activation gate. The max-plus kernel (P-4-legal, witness-bearing, parity-pinned to uor-matmul) is retained as mechanism.

**New open items filed by the campaign:** #628 (κ-injectivity census retrofit, unassigned), uor-addr#14 (upstream safetensors realization), #503 (pre-existing). Follow-up decisions recorded but not filed as issues: kappa-registry hosting (#624 record), teacher exact-GEMM swap (measure-first, #622 record), NAF κ re-pin at spec freeze (#623 record).

main at campaign close: post-#631. Zero-matmul runtime invariant untouched throughout: no shipped crate gained a dependency; every gate green on every PR. Leaving this tracker open for Casey's close.


### comment 5261137667 — 2026-08-12T01:49:27Z — Casey-allard

Closing on Casey's direction (2026-08-12): campaign complete per the summary above — all seven sub-issues closed with records, PRs #619/#627/#629/#630/#631 merged, main 2bfd360. Follow-ups live in #628 (starting now), uor-addr#14, and the recorded decisions (registry hosting, teacher exact-GEMM measurement, NAF κ regeneration at freeze).


---
_Comment/review ids in this file: 5259188746, 5259688672, 5261102780, 5261137667_

