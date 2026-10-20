# Mechanism board, 19 October 2026

There are 27 mechanisms: **10 ACTIVE**, **4 READY** and **13 PARKED**. Every status comes from its dossier (`docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/<id>.json`). Where this audit disagrees with a status, the disagreement is listed under Contradictions and the status is left unchanged. Machine form: `board.json`.

- **ACTIVE**: on the current model path.
- **READY**: built and tested, but not yet trained into a model.
- **PARKED**: the forms tried so far failed or broke a rule. The mechanism stays open (D22), and its dossier lists the conditions for reopening it.

## ACTIVE (10)

| Mechanism | What it does, in plain words | Replaces | Why ACTIVE | Next step |
|---|---|---|---|---|
| Discrete geometric learning (`discrete-learning`) | Tries every legal rotation of one table entry and keeps it only if the integer loss strictly drops. No gradients, no float. | optimizer / backprop | Learning law of the only live model, `crates/uor-r4-discrete`. It trained a pilot and four K16 runs with no language gain yet (k16-outcome-call-2026-10-19). | First trained artifact that beats its frozen control (admission-preserving A8 rerun) |
| Prime-zeta codebook (`prime-zeta-codebook`) | Gives each word a fixed, unique starting address built from primes and zeta-zero phases | embedding | P8 baseline and A8 fallback cell. Trained attribution against the nulls is NOT_RUN (prime-zeta-construction-2026-10-14) | Held attribution run against the two nulls |
| Address-structured cells (`address-cells`, A8) | Places each word next to the words that usually come before it, so a reader near the last word already sees likely next words | embedding | Raised starting admission from 127 to 1,351 of 3,847. Learning then eroded it, giving a Q1 stop (Relation 551, control 857) (address-cells-preflight, a8-dose-2026-10-19) | CUDA qualification of the merged guard (NOT_RUN), then the rerun |
| Routed read (`route-read`) | Walks a small tree of addresses and picks up the one earlier word at the end | attention | Base read of every trained discrete artifact. Its effect is unattributed: 20 proposals in 16,404 updates (attention-state-2026-10-18) | Read-off/read-on ablation with enough Route proposals |
| Flock read (`flock-read`) | Looks at a few nearest earlier positions, rotates them into its own frame and moves toward their consensus | attention | Passes 25 CUDA controls and runs inside every Relation and FrozenZero arm. Its own value has never been isolated (discrete-driver-cost-2026-10-13) | Get the C1 coupling sheet to FIT, with zero and random controls |
| Relation read (`relation-read`) | Lets the read follow links such as "the word after the earlier copy of this word" (copying) | attention scoring | 9/9 controls and two trained doses, both stopped by admission loss. No paired verdict yet (a8-dose-2026-10-19) | Paired verdict from the A8 rerun. It must beat the T2 table (6,300/14,287) |
| Admission (`admission`) | Narrows 4,096 words to those in a few opened "drawers". A word in an unopened drawer can never be chosen | softmax output head | The measured ceiling on every attention run. K=8 eroded from 5,680 to 1,920 (attention-state-2026-10-18) | Admission-preserving A8 rerun |
| Rank-margin hinge (`rank-margin`) | The loss asks only whether the right answer beats its strongest rivals by a fixed gap | loss (cross-entropy) | Integer hinge A/R is the discrete writer's loss. K16-R collapsed occupancy (admission-rank-hinge-2026-10-15, k16-outcome-call) | Occupancy-regularised rank objective |
| D11 integer serving (`lut-serving`) | Runs a finished model on a laptop or in a browser with only integers, adds and tables, and proves no multiply sneaks in | arithmetic / matmul | Export boundary under D37. Zero-mul audits green (44 native symbol ranges, 7/7 wasm). No lawful model exported yet | Export one discrete artifact and measure its export gap |
| Tokenizer (`tokenizer`) | Cuts text into 4,096 numbered pieces and back | tokenizer | Every corpus, panel and artifact is pinned to d36d3e87. A 16,384-piece trainer exists as tooling only (rust-bpe-trainer-2026-10-19) | Matched 4,096 vs 16,384 comparison sheet |

## READY (4)

| Mechanism | What it does, in plain words | Replaces | Why READY | Next step |
|---|---|---|---|---|
| Hamilton gate (`hamilton-gate`, G2) | Turns the gate signal into a rotation and rotates the value, so no sigmoid is needed | feed-forward gate (SwiGLU) | 944 tests. RTX 5090 parity 43/0 (unnamed tests) and 18 rung-0 CUDA controls. Never trained (cuda-parity-gate-import, rung0-complete-path-2026-10-13) | D37 sheet with icosian snap, language fit and export gap |
| Geometric recurrence gates (`recurrence-gates`) | Fade rate from a table of powers of two, input opened by a clamped line, state carried by rotation | sigmoid/exp/tanh recurrent gates | Table errors ≤ 5.5e-8 and 18 rung-0 CUDA controls. The six-recurrence fit never ran (rung0-complete-path-2026-10-13) | Show the secant surrogate gives descent |
| SpiralCore/FBS projection, float form (`spiralcore-fbs`) | Each output block picks one stored block and moves it along exact E8 steps. No learned weights are multiplied | dense projections (matmul) | Device-qualified with exact fixtures. Never trained. Its integer form is ACTIVE (spiralcore-fbs-projection-2026-10-12) | Secant descent and tokens/s in a D37 sheet |
| D19 object store (`object-store`) | Keeps bytes exactly in a hashed tree, so repeats are stored once and any range is checkable | none (durable memory store) | 60 tests and a client conformance PASS. Nothing consumes it, and the M3 bars have not been run (d19-object-store-2026-10-15, uorobj01-conformance-2026-10-18) | Wire it in and run the M3 bars |

## PARKED (13)

| Mechanism | What it does, in plain words | Replaces | Why PARKED | Next step |
|---|---|---|---|---|
| Carrier credit (`carrier-credit`) | Sends learning credit back to the geometric objects that carried the information | optimizer / backprop | 437/512 fell to 0/512 in three arms. The sampled-choice producer uses softmax (unified-geometric-integration-2026-10-11) | Hard-selection producer. Needs the M2 hold released |
| Legal construction (`legal-construction`) | Picks each parameter change directly from the values the integer model can hold, protecting answers already correct | optimizer / backprop | Never scored. Every attempt stopped on solver numerics (direct-legal-construction-2026-10-09) | Solve input245 in exact arithmetic |
| E8 orbit memory (`e8-orbit-memory`) | Replaces a feed-forward layer with a few rows of a large table addressed by E8 lattice points | MLP | Half-MLP met its bar (+0.033 BPB), but learned keys beat E8 by 0.023, and the gate is a sigmoid (e8-orbit-memory-2026-10-11) | D23-clean gate plus the row-3 iteration |
| Exact addressed memory (`exact-memory`) | Writes a fact once under an exact prime address and reads it back exactly | key-value memory | Trained forms failed (19/40 vs 27/40) or used softmax. The address-as-read form was never built (addressed-memory-2026-10-10) | Build the address as the read, with an oracle first |
| Pointer key (`pointer-key`) | Copies the answer by the slot's exact name rather than by the surrounding wording | copy head | A softmax pointer. Hit rate rose from 0.377 to 0.545, but memory missed its bar (ptr-addr-2026-10-10) | Lawful oracle, then a prime entity key |
| Read binding (`read-binding`) | A teacher marks which earlier value the question asks for, and the read is rewarded for landing on it | auxiliary loss | Headline KEEP on 3 seeds (wrong-value 47 to 30/120), but on a softmax read (read-binding-powered-2026-10-11) | Port it to a Flock or Relation read |
| Hopf observation (`hopf-observation`) | Views a 4D rotation through a 3D shadow while keeping the lost angle separately | attention scoring | Telemetry only. Sector routing used 7 of 512 sectors, and the Lorentz forms were softmax or neutral (mechanism-census) | Retained-fibre test |
| VSA (`vsa`) | 4,096-bit codes glued by XOR and compared by counting differing bits | key-value memory | +0.014 to 0.023 BPB, but on the banned CPU learner. Other variants failed (vsa-native-test-2026-10-09) | GPU discrete Hamming binding |
| Spin-native weights (`spin-weights`) | Each 4×4 weight block is one quaternion, snapped to 120 icosian rotations | arithmetic / matmul | Met its bars (1.5148 BPB, level with dense), but only via dense matmul (icosian-mantissa-2026-10-11) | Owner architecture decision |
| Ternary/XOR (`ternary-xor`) | Weights of −1, 0 or +1, with similarity by bit counting | arithmetic / matmul | A dense ternary sum is still a dense sum (D25). Post-hoc export lost (7.416 vs 7.171 bits). Its parts live in discrete-learning | Owner ruling, or a geometric alphabet |
| Geometric stack (`geometric-stack`) | The old 7M to 214M float layered model with quaternion recurrence | whole model | The trained form used softmax, SiLU and matmul, and missed M1 (served 1.20–1.24 vs 0.90 BPB) (d11-serving-2026-10-10) | D37 rebuild from lawful parts |
| Native geometric engine (`native-geometric`) | A Rust-from-scratch model with prime addresses and version chains. It answered authored questions only | whole model | No prose. 437/512 fell to 0/128 on fresh rows, and its ranking uses softplus/exp (m2-fresh-437-2026-10-10) | Hinge re-registration once the M2 hold is lifted |
| R4G1 graph (`r4g1-graph`) | Compiled a teacher model's preferences into integer lookup tables | whole model | Never produced coherent free text, and needs a banned transformer teacher (free_running_reachability_840) | Owner repair-or-retire decision on the crates (D36 item 3) |

## Which mechanism covers each part of a transformer

| Transformer part | Our mechanisms (status) |
|---|---|
| Tokenizer | tokenizer (A) |
| Token embedding table | prime-zeta-codebook (A), address-cells (A) |
| Positional encoding | **none.** No record names a mechanism for this |
| Attention: choosing what to read | route-read (A), flock-read (A), relation-read (A), admission (A), hopf-observation (P) |
| Attention: mixing what was read | flock-read (A) |
| Copy / induction heads | relation-read (A), pointer-key (P) |
| Context memory / KV cache | exact-memory (P), vsa (P), read-binding (P) |
| Durable memory outside the model | object-store (R) |
| MLP / feed-forward | spiralcore-fbs (R), hamilton-gate (R), e8-orbit-memory (P) |
| Activation and gating functions | hamilton-gate (R), recurrence-gates (R) |
| Recurrent sequence mixing | recurrence-gates (R), geometric-stack (P) |
| Projections / matmul arithmetic | lut-serving (A), spiralcore-fbs (R), spin-weights (P), ternary-xor (P) |
| Normalization | **none.** The discrete snap-to-orbit-root may serve, but no record says so |
| Output head (softmax over vocabulary) | admission (A), discrete-learning (A) |
| Loss (cross-entropy) | rank-margin (A), read-binding (P) |
| Optimizer / backprop | discrete-learning (A), carrier-credit (P), legal-construction (P) |
| Inference runtime | lut-serving (A), r4g1-graph (P) |
| Whole-model engine | geometric-stack (P), native-geometric (P), r4g1-graph (P) |

Family tree (full list in `board.json`): r4g1-graph → native-geometric → geometric-stack and discrete-learning. spiralcore-fbs and carrier-credit are float forms whose integer descendant is discrete-learning. route-read → flock-read → relation-read. prime-zeta-codebook → address-cells. spin-weights → spiralcore-fbs.

## Contradictions with `docs/mechanisms/README.md` (status as of 18 Oct)

1. **No rows** for address-cells, admission, route-read as its own row, lut-serving, tokenizer, hopf-observation, vsa, geometric-stack, native-geometric or r4g1-graph. Its labels (active/dormant/archived-negative) do not map one-to-one onto ACTIVE/READY/PARKED, and "archived-negative" labels a configuration, not a mechanism.
2. **Hamilton gate and recurrence gates**: the README says "no device receipt". The records show RTX 5090 parity 43/0 (cuda-parity-gate-import-2026-10-13) and 18 rung-0 CUDA controls with `gate=hamilton rec_gates=geometric` (rung0-complete-path-2026-10-13). The README is stale. READY is correct.
3. **Discrete learning**: the README says "seed-1 completion running" and "M1 quality target SUSPENDED". The pilot and four K16 runs have finished with no arm passing the bars, and the A8 dose stopped at Q1. The README is stale.
4. **Relation read**: the README lists one trained run and says the stages are NOT_RUN. Two doses have trained, with no paired verdict. The README calls the rows_v1 run "Dose 2", while the dossier and attention-state-2026-10-18 call it "Dose 1", and its attempt id is `…dose2-rows…`. One canonical name is needed.
5. **Object store**: the README says "active" and the dossier says READY. I agree with READY, because nothing consumes it and M3 has not run.
6. **Prime-zeta**: the README stresses the xorshift default and an injective-init DRAFT. PrimeZeta is now the P8 baseline and A8 fallback. Its injectivity comes from the shared quota pass, not from the prime phases.

### Where this audit disagrees with a dossier (statuses left unchanged)

- **lut-serving: ACTIVE → I would say READY.** No lawful or discrete model has ever been exported through it. Its schemas /1–/2 serve softmax through exp tables (by its own rule_compat), and a new schema may be needed.
- **hopf-observation: PARKED → partly READY.** The kept-fibre map is built and unit-tested, but its language test was never run. Only sector routing failed, and the Lorentz read was neutral, which is not a failure. Splitting the two forms would be more accurate.
- **Reopen conditions that would run banned comparators:**
  - hamilton-gate: matched comparison against SwiGLU, which uses sigmoid.
  - recurrence-gates: comparison against a "historical-gates arm" (sigmoid/exp).
  - pointer-key: forced-copy oracle on the softmax-pointer artifact.

  Each must use stored outputs or a lawful read instead.
- **recurrence-gates** has `replaces=attention`, but it replaces squashing gates inside a recurrence. The slot map lists it under gating and recurrence.
