# Geometric toolbox: mechanisms kept available, 2026-09-28

Owner decision [D12](DECISIONS.md#d12--gates-promote-never-kill-reopen-geometric-candidates-port-the-native-engines-mechanisms-keep-a-geometric-toolbox) item 5. Lab 1 (Claude main) evaluated every closed or parked mechanism in ROADMAP §5.

**The questions asked of each one:**
- Did it demonstrate a real geometric capability?
- Is that capability absent from the current main-line model?
- How novel is it?
- Where are its pieces?

**How to read this record:**
- **Numbers** keep the exact scope of their sources.
- **"Not yet promoted"** replaces DEAD, RETIRED and FAILED for mechanisms (D12 item 1).
- **Novelty notes** are Lab 1's assessment from the literature it knows. They are not literature reviews, and they claim no result.

**The current main line**, for reference, is the `rrarra` stack: quaternion (S3) transport in its recurrence, Lorentz reads, dense SwiGLU MLPs, and an exact six-view store (I1, library).

## A. Reopened as active candidates (D12 item 2)

| Mechanism | What it demonstrated | Absent elsewhere? | Novelty (assessment) | Pieces | Next step (owner) |
|---|---|---|---|---|---|
| **Finite-group tracking lanes (2I and reflection pair)** | **Exact A5 state tracking at length 4,096:** tracking accuracy 1.000, with 60-state automata that minimize exactly (`verify_exact`). 2I lanes: 8/9 after snapping. Reflection pair: 9/9. Inside the stack, the reflection pair's **six-seed mean text cost is +0.018**; one seed at +0.069 missed a per-seed 0.05 gate | **Yes.** The stack does not track a non-solvable group | **High.** A5's word problem is the standard hard case for state tracking: constant-depth transformers and diagonal state-space models are known to struggle with it (for example Liu et al. 2022, "Transformers learn shortcuts to automata"; Merrill et al. 2024, "The illusion of state in state-space models"). Exact finite-group lanes with a verifiable automaton, served in integers, are a distinctive capability | `crates/uor-r4-training/src/stack_tracking.rs`, `examples/tracking-lanes.rs`; B1 record §8; #1442, #1447 | Train them jointly with the trained-in 2I transport (S4). Lab 1 |
| **Exact relational memory (D2 / AERM)** | **1.000 on every in-distribution class,** including absent facts, previous values and recency traps. The equal-parameter dense control reached 0.51 on First and ≤ 0.07 on Absent. Writes transferred to unseen phrasings; reads did not | Partly. I1 now provides the six-view store; the learned read is missing | Moderate. The pattern resembles key-value memory; exact, versioned, typed statuses with abstention are the distinctive part | `stack_aerm.rs` (frozen probe), `stack_store.rs` / `stack_checkpoint.rs` (I1) | I1's store, with G's learned read (Lab 2) aimed at held-out phrasing. Labs 1 and 2 |
| **Geometric sparse index (T2 / D3)** | "Not qualified" in a contest that was a compression screen, and its geometric arms were **confounded by magnitude** (ruling 11). So it was never a fair test of geometric addressing | Yes. No geometric index is in the path | Moderate. Lattice and cell indices exist (for example product quantization); a fixed icosian/E8 cell index with multiplier-free decode is the distinctive part | `addressing_arms.rs`, the T2 contest code | Re-test inside G, with magnitude carried and the geometry trained in. Lab 2 |
| **2I / E8 read codes, trained in** | Only **post hoc**: 2I direction codes changed 41–46% of read decisions (D6), and the finite 2I read score harmed answers when it also dropped magnitude and swapped in an MLP (#1438; confounded). The transport's post-hoc snap costs +0.022–0.026 nats. Nothing was ever trained in | Yes | Moderate to high. Lattice codebooks are state of the art for weight quantization (for example QuIP#'s E8 codebook, Tseng et al. 2024). Trained-in finite-group read addresses are less explored | `addressing_arms.rs` (D6 arms), `canonical_h4_roots` | Inside G, trained end to end with magnitude. Lab 2 |

## B. Already in the main line (earlier "parked" or "dead" labels superseded)

| Mechanism | Earlier label | Now |
|---|---|---|
| **Quaternion rotation as a content mixer** | PARKED (−0.025 nats against Householder, 1 seed) | **Load-bearing** in the stack's recurrence: identity transport costs +0.0711 and +0.0774 nats (D1, 2 seeds). Trained-in 2I is S4 |
| **Lorentz (hyperbolic) read score** | DEAD as a score (transferred readers +0.046 and +0.013; Lorentz−Dot −0.0001 in the default stack) | **In use**: the main-line stack reads with `read=lorentz`. It is neutral against Dot at 7M on code, so it stays, as the geometry the reads are built on |

## C. Toolbox: demonstrated geometric pieces kept available

The code stays building and documented. Each entry names the problem it could solve later.

| Piece | Capability shown | Novelty (assessment) | Where | Could fix |
|---|---|---|---|---|
| **2I relative-relation table** (`inverse(q)·k`) | Exact group-relative relation between two icosians, as a table lookup. Its consumer read (#1438) was confounded, so the table itself is untested | Moderate. Rotary embeddings are continuous relative rotations; an exact finite-group relative code is rarer | #1438 (parked branch); ruling 3 | Relation-sensitive reads in G: who did what to whom, as a group element |
| **Prime / CRT / Galois exact addressing, and UOR identity** | Exact, injective, composable arithmetic addresses. As predictive features: no better than tabulation (phase 2; review §9.6) | Low to moderate as prediction; distinctive as **exact identity** (composable, collision-free by construction) | `crates/uor-r4-core/src/native_geometric/` (prime registry, ordered n-lets) | Store keys and cross-session identity: native port step 3 (D12 item 3) |
| **Fixed zeta-zero phase channels** | Artifact-bound incommensurate phases. No predictive gain as a feature | Moderate, as a positional basis with incommensurate frequencies | `native_geometric` (phase channels) | Position and phase encoding trained into the stack: native port step 4 |
| **Exact Hopf S3→S2 projection with retained fiber** (`UnitS3Q30`) | Exact Q30 observation map, used by the integer runtime | Moderate | `crates/uor-r4-integer/src/math.rs` | An exact observation of S4's 2I state, with the fiber kept explicitly |
| **E8 (240 roots) and H4 (120 icosians) fixed codebooks; product-key memory** (cycle 5) | Fixed geometric key indices with multiplier-free decode. Cycle 5 arms: NOT_RUN | Moderate (after product-key memory, Lample et al. 2019) | `stack_memory.rs`, `addressing_arms.rs`, `joint_admission.rs` | G address codes; D4 lattice weight codes (Lab 3); D5 parameter sparsity |
| **Orthant (sign-pattern) bounded admission** (`orthant64`) | A geometric partition for bounded candidate admission. Neutral against full admission: 17 of 32 answers in both arms | Low | `joint_admission.rs`, `joint_bounded_campaign.rs` | Cheap admission for G at long context |
| **Vector-symbolic binding** (Sep 19 model) | Role binding by superposition; about 0.0013 bits/byte, within noise | Low to moderate (Plate's HRR; Kanerva) | `crates/uor-r4-core` (`set-vsa-code-mode`) | Role binding for G's relation reads: D2's failure was role and phrasing generalization |
| **Golden-ratio rotation codebooks** | Structured dense rotation sets. At practical sizes, covering is no better than random, with a hole near the identity | Low as shown | review §6.4 | Finer rotation sets than 2I, if S4 needs them (the 2I×2I step) |
| **Hyperbolic curvature and horocycle position prior** | Curvature stayed flat when learned; horocycle 3.264 against 3.226 (worse) | Low as shown | phase 2 records | Long-context hierarchy, only with new evidence |
| **Cayley–Dickson exact algebra** | Exact quaternion and octonion arithmetic | Low (standard algebra), useful infrastructure | `crates/uor-r4-core/src/cayley_dickson.rs` | Exact Z[φ] quaternion products in S4's serving path |
| **Frozen R4G1 / TLA kernel** | XOR/AND/popcount/table integer kernel under its own frozen contract | — | its scoped crate (AGENTS.md) | Reference for multiplier-free kernels |

## D. Ordinary negatives (not geometric; results kept at their scope)

- **Count-prior blend:** +0.0049 against the `(prev,cur)` table.
- **Termination-weighted objective:** inert.
- **A1–A4 local selector tuning** (D8).
- **Per-product table emulation as a serving kernel:** 4.3× the energy of float, now D11-COST's problem for Lab 3.
- **#1433 legal-code choice.**
- **Exposure-only continuation of the 1.68M native model.** The model's *mechanisms* are ported (D12 item 3), and this result speaks only to exposure at that size.
- **D10 SmolLM2:** out of mission, a comparator only.
- **`uor-r4-lut`:** a frozen comparator.

## Maintenance

- Lab 1 keeps the pieces in sections A and C compiling on `main`.
- A PR that would delete one needs a written reason in its body.
- New closed mechanisms are added here, each with a capability, novelty and reuse entry.
