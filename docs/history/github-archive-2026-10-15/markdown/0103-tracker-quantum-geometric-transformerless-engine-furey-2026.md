# #103 — TRACKER: Quantum Geometric Transformerless Engine (Furey 2026)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-23T15:48:26Z
- **Updated:** 2026-07-23T19:53:21Z
- **Closed:** 2026-07-23T19:53:21Z
- **Labels:** graph-compiler
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/103
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Master tracker for evolving UOR-R4 into a local, UOR-based, prime geometric transformerless LLM based on N. Furey (2026) 'Standard Model Symmetries and the Nested Embeddings of R ⊂ C ⊂ H ⊂ O' (arXiv:2607.18450v1).

Root architecture spec: `docs/r4_furey_quantum_geometric_plan.md`

Chain of Work Items (in dependency order):
1. #104: Cayley-Dickson 16D vector space substrate and nested inclusions (Phase 1)
2. #105: Endomorphic multiplication algebra End_R(V) ≅ Cl(0,8) operator store (Phase 2)
3. #106: Lie-Jordan splitting and allocation-free universal product kernel (Phase 3)
4. #107: Bott Periodic Fock Space O(1) context abstraction engine (Phase 4)
5. #108: Von Neumann maximum-entropy quantum density operator and cover induction (Phase 5)
6. #109: Façade, CLI subcommands, Wasm integration, and context scaling benchmark suite (Phase 6)

---

## Definition of Done
- All linked phase issues (#104–#109) are completed, verified with machine-checked invariant scans, 100% clean unit tests, and published PRs.
- The 4-way Gate C quality table and Bott Periodic Fock (1)$ context scaling benchmark numbers are attached.

## Acceptance criteria
- [ ] All sub-issues closed with DoD evidence.
- [ ] Gate A, B, C, D, E, F quality gates pass clean.

## Comments (2)

### comment 5062522925 — 2026-07-23T19:31:41Z — auser

## Tracker status — 5/6 phases landed

| Phase | Issue | State | Evidence |
| :--- | :--- | :--- | :--- |
| 1. Cayley-Dickson 16D substrate | #104 | ✅ closed (#112) | cd_space module + isomorphism checks |
| 2. End_R(V) ≅ Cl(0,8) operator store | #105 | ✅ closed (#113) | endomorphism.rs, L/R maps, Clifford generators |
| 3. Lie-Jordan + universal product kernel | #106 | ✅ closed (#115) | lie_jordan.rs, allocation-free u8 kernel, BDD suite |
| 4. Bott Fock O(1) context | #107 | ✅ closed (#116) | bott_fock.rs, Gate F numbers below |
| 5. vN density operator + cover induction | #108 | ✅ closed (#118, BDD #119) | quantum_cover.rs, analytical entropy + criterion BDD |
| 6. Façade/CLI/Wasm/benchmarks | #109 | 🔄 PR #117 (checks green) | in merge queue path |

### DoD attachment 1 — 4-way Gate C quality table (pinned fixture)

| Rule | top-1 | bits/token |
| :--- | ---: | ---: |
| legacy_sum | 0.2863% | 76.9643 |
| rule1_chain | 1.2185% | 71.5219 |
| rule12_precedence | 31.7086% | 9.8612 |
| tla3_baseline | 31.7086% | 11.8781 |

### DoD attachment 2 — Bott Periodic Fock O(1) context scaling (Gate F, release)

| N (context tokens) | ns/append |
| ---: | ---: |
| 1,000 | 157 |
| 10,000 | 162 |
| 100,000 | 148 |
| 1,000,000 | 146 |

Per-append spread 146–162 ns (≈1.1×) — flat across three orders of magnitude; store size fixed (256 i16 cells + counter, zero heap).

### Gates A–F

- A (build integrity): `cargo test --workspace` green on every merged PR ✅
- B (static invariant): P-4 source scans machine-checked in CI; #107 carries its own self-scan ✅
- C (quality baseline): table above, trend alarm green ✅
- D (allocation census): green ✅
- E (κ reproduction): green on every merged PR ✅
- F (quantum context): table above ✅

Remaining: #109 closure via PR #117, then this tracker's acceptance criteria are fully met.


### comment 5062713388 — 2026-07-23T19:53:20Z — auser

## Tracker complete — all six phases landed

| Phase | Issue | Merged via |
| :--- | :--- | :--- |
| 1. Cayley-Dickson 16D substrate | #104 ✅ | #112 |
| 2. End_R(V) ≅ Cl(0,8) operator store | #105 ✅ | #113 |
| 3. Lie-Jordan + universal product kernel | #106 ✅ | #115 |
| 4. Bott Fock O(1) context | #107 ✅ | #116 |
| 5. vN density operator + cover induction | #108 ✅ | #118 (+ BDD #119) |
| 6. Façade/CLI/Wasm/benchmarks | #109 ✅ | #117 |

**Acceptance criteria:**
- [x] All sub-issues closed with DoD evidence on their threads
- [x] Gates A–F pass clean: workspace tests green on every merge (A); P-4 machine scans in CI (B); Gate C 4-way table at the pins — 31.7086% / 9.8612 bits vs baseline 11.8781 (C); allocation census green (D); κ-reproduction green (E); Bott Fock context flat at 146–162 ns/append for N = 10³…10⁶ (F)

The required DoD attachments (4-way Gate C table, Gate F scaling numbers) are in the status comment above. The Quantum Geometric Transformerless Engine arc is complete. Closing.



---
_Comment/review ids in this file: 5062522925, 5062713388_

