# M6 Causal Intermediate-State Ablation Instrument (`m6-causal-ablation-instrument` #2863)

- **Status:** Built and verified on CPU finite controls (`crates/uor-r4-integer/src/m6_ablation.rs`, `crates/uor-r4-integer/src/discrete_d11.rs`); GPU qualification (`m6-qualify-reasoning-coding` #2865) is pre-registered below (`TEST FITNESS: PENDING`) and remains HELD pending panel freeze (`m6-reasoning-panel-freeze` #2866), M1 trunk training (`m1-trunk-assembly-and-training` #2845), owner bar confirmation (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6341710686`), non-author `TEST FITNESS: FIT` review, and Casey's typed D40 release.
- **Milestone:** M6 (`#2034`). **Item:** [`m6-causal-ablation-instrument`](../../roadmap/items/m6-causal-ablation-instrument.md) (`#2863`). **Lab:** antigravity.
- **Design note:** [`#2863` comment](https://github.com/UOR-Foundation/uor-r4/issues/2863#issuecomment-6341710004). **Bar decision card:** [`#2028` comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6341710686).

## 1. Crate Placement and Trunk State Hook Points

Evaluating `crates/` (`uor-r4-core`, `uor-r4-discrete`, `uor-r4-eval`, `uor-r4-EXPORT-ONLY-UNUSED`, `uor-r4-harness`, `uor-r4-integer`, `uor-r4-object-store`, `uor-r4-Select`, `uor-r4-ser`, `uor-r4-tokenizer`, `uor-r4-training`) shows that the served D11 discrete trunk lives in [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../crates/uor-r4-integer/src/discrete_d11.rs) (`IntegerDiscreteModel`, exported from `uor-r4-discrete` by [`m1-d11-exporter-discrete` `#2838`](../../roadmap/items/m1-d11-exporter-discrete.md)). Placing the M6 ablation instrument in [`crates/uor-r4-integer/src/m6_ablation.rs`](../../../crates/uor-r4-integer/src/m6_ablation.rs) evaluates the exact integer artifact on the served product path (and under `--features serving-wasm`).

Every reasoning or code-generation query passes through two explicit integer intermediate-state hook points:

1. **Geometric state transport (`IntegerDiscreteModel` in `discrete_d11.rs`):**
   - `evaluate_route_read`: recursively transports 8D integer block states across the ternary context tree (`ContextTree::from_window`) using `route_couple_block` (antipodal cancellation + nearest E8 orbit root snap via `snap_orbit_block`).
   - `evaluate_flock_read`: couples kept context neighbors via `flock_couple_block` gated by `level_gates`.
   - `evaluate_relation_read`: transports block states along the 5 structural relation edges (`SameCell`, `Antipode`, `SequentialSuccessor`, `SharedFirstBlock`, `SharedLastBlock`) across `read_sweeps` sweeps gated by `level_gates` and `relation_gates`.
2. **Exact addressed memory (`IntegerAddressedMemory` in `m6_ablation.rs`):**
   - Prime-factor and ordered-pair keys (`PrimeFactorKey`) over capacity-bounded versioned slots (`[initial, previous, current]`) with sorted-merge GCD sieve admission ($\gcd(q, A_c) > 1$, i.e. `shared_primes >= 1` via linear two-pointer sorted slice scan), deterministic lexicographic tuple ranking `(shared_primes, shared_pairs, turn_index, Reverse(slot_index))`, and explicit `Evicted { overwritten_by_turn }` vs. `Absent` separation.

## 2. Geometric Knockout Operations (Zero Squashing, Zero Matmul)

The four-arm factorial ablation (`AblationArm::{None, KnockoutStateTransport, KnockoutAddressedMemory, KnockoutBoth}`) applies two orthogonal geometric knockouts:

| Arm | State transport (`IntegerDiscreteModel`) | Addressed memory (`IntegerAddressedMemory`) | Purpose |
|---|---|---|---|
| `AblationArm::None` | Active (`state_transport_knockout = false`) | Active (`lookup` via sorted-merge GCD sieve) | Unhooked baseline anchor (bit-identical to unhooked path) |
| `AblationArm::KnockoutStateTransport` | Knocked out (`with_state_transport_knockout()`) | Active (`lookup` via sorted-merge GCD sieve) | Isolates causal contribution of context-tree / flock / relation state transport |
| `AblationArm::KnockoutAddressedMemory` | Active (`state_transport_knockout = false`) | Knocked out (`MemoryLookupOutcome::KnockedOut`) | Isolates causal contribution of exact prime-addressed memory |
| `AblationArm::KnockoutBoth` | Knocked out (`with_state_transport_knockout()`) | Knocked out (`MemoryLookupOutcome::KnockedOut`) | Measures residual unigram / local-token template floor |

- **Why state-transport knockout is purely geometric:** `IntegerDiscreteModel::with_state_transport_knockout()` projects the read vector $q \in (\mathbb{Z}^8)^B$ onto the current leaf token's own static E8 orbit-root blocks $v(x_{L-1})$ (`token_blocks(prefix[L - 1])`), zeroing multi-hop tree, flock, and relation edge transport while keeping every block on the exact 16-root E8 orbit lattice ($\|b\|_1 = 4, \|b\|_2^2 = 8, \|b\|_\infty = 2$) and running the identical `rank_writer` shortlist and tie-break.
- **Why addressed-memory knockout is purely geometric:** Under `AblationArm::KnockoutAddressedMemory` and `AblationArm::KnockoutBoth`, the memory hook returns `MemoryLookupOutcome::KnockedOut` (the empty-lattice element $\bot$ of the retrieval lattice), forcing `M6StepQuery::AddressedMemoryOrTrunk` to fall back to the trunk's `evaluate_prefix` on `fallback_prefix`.
- **Zero squashing / zero matmul compliance (CHARTER §3.1–§3.3, §3.10):** All scoring uses exact integer $L_1$/Hamming distances, sorted-merge prime-list intersection, and integer restoring square-root `ceil_two_sqrt_int` (`math::isqrt_u64`). Both the 44-symbol native stack zero-multiply audit (`audit_zero_matmul_serving.py --stack`) and the 7-symbol WebAssembly serving audit (`audit_zero_matmul_wasm.py --serving --call-graph --tap`) pass with `mul=0, div=0, float=0`.

## 3. Finite CPU Controls (`crates/uor-r4-integer/src/m6_ablation.rs`)

1. **`sorted_merge_gcd_sieve_and_eviction_versioning`:** Verifies sorted-merge prime-factor admission (`shared_primes >= 1`), ordered-pair disambiguation (`(3, 5)` vs. `(11, 5)`), `[Initial, Previous, Current]` version selection across overwrites, capacity eviction returning `MemoryLookupOutcome::Evicted { overwritten_by_turn }`, and disjoint primes returning `MemoryLookupOutcome::Absent`.
2. **`no_knockout_arm_is_bit_identical_to_unhooked_path`:** Verifies that `evaluate_m6_row_under_arm(..., AblationArm::None)` produces the exact same `emitted_tokens` and `step_traces` (`top1_token`, `best_cell`, `best_cell_distance`, `admitted_count`) as calling `model.evaluate_prefix` and `mem.lookup` directly without the ablation harness.
3. **`four_arm_causal_ablation_separates_state_transport_and_addressed_memory`:** On a synthetic 60-row M6 panel (`20` `Compositions`, `20` `AlteredConstraints`, `20` `CodeTasks`) over the 5-token, 2-block `Relation` C2 model (`build_relation_c2_bytes`):
   - `AblationArm::None`: `60 / 60` (`20/20` per family).
   - `AblationArm::KnockoutStateTransport`: `20 / 60` (`0/20` `Compositions`, `0/20` `AlteredConstraints`, `20/20` `CodeTasks`; overall discordant drop $b - c = 40 \ge \max(16, \lceil 2\sqrt{40}\rceil) = 16$).
   - `AblationArm::KnockoutAddressedMemory`: `20 / 60` (`0/20` `Compositions`, `20/20` `AlteredConstraints`, `0/20` `CodeTasks`; overall discordant drop $b - c = 40 \ge 16$).
   - `AblationArm::KnockoutBoth`: `0 / 60` (`0/20` per family; overall discordant drop $b - c = 60 \ge \max(16, \lceil 2\sqrt{60}\rceil) = 16$).

---

## 4. Pre-Registration Specification for M6 Causal Ablation Qualification (`#2863` / `#2865`)

### Bar

- **Proposed M6 causal ablation effect-size bar (posted for owner/PM confirmation on [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6341710686), CHARTER §5.2):**
  1. **State-transport causal dependence (`AblationArm::None` vs. `AblationArm::KnockoutStateTransport`):** Paired McNemar discordant drop $b - c \ge F_I = \max(16, \lceil 2\sqrt{b + c}\rceil)$ (`meets_causal_bar == true`) across `Compositions` + `AlteredConstraints` in every seed (where $b$ is the count of rows passing under `None` and failing under `KnockoutStateTransport`, and $c$ is the reverse).
  2. **Addressed-memory causal dependence (`AblationArm::None` vs. `AblationArm::KnockoutAddressedMemory`):** Paired McNemar discordant drop $b - c \ge F_I = \max(16, \lceil 2\sqrt{b + c}\rceil)$ (`meets_causal_bar == true`) across memory-grounded `Compositions` + `CodeTasks` rows in every seed.
  3. **Joint ablation collapse (`AblationArm::None` vs. `AblationArm::KnockoutBoth`):** Paired McNemar discordant drop $b - c \ge F_I = \max(16, \lceil 2\sqrt{b + c}\rceil)$ and aggregate panel accuracy drop of at least $50$ percentage points relative to `AblationArm::None`, refuting memorized-template answering.

### Base artifact

- **Evaluator & ablation instrument:** `crates/uor-r4-integer/src/m6_ablation.rs` (`evaluate_m6_ablation_panel`, `IntegerDiscreteModel::with_state_transport_knockout`, `IntegerAddressedMemory`).
- **Candidate trunk artifact:** D11 discrete artifact (`uor-r4.lut-discrete/1`) exported from the M1 trunk (`m1-trunk-assembly-and-training` `#2845`) via `uor_r4_discrete::export_d11_discrete_bytes` (`m1-d11-exporter-discrete` `#2838`), pinned by `artifact_sha256` before scoring.

### Data split

- **Frozen panel:** `docs/harness/panels/m6-reasoning.json` once frozen by `m6-reasoning-panel-freeze` (`#2866`), SHA-256-pinned before any GPU evaluation run, partitioned into `M6TaskFamily::{Compositions, AlteredConstraints, CodeTasks}` with deterministic `M6OracleKind` graders (`ExactTokenSequence`, `ConstraintCheck`, `ExecutionCheck`, `HonestAbstention`) and zero training/development overlap (CHARTER §3.10).

### Steps

1. Freeze `docs/harness/panels/m6-reasoning.json` under `m6-reasoning-panel-freeze` (`#2866`) and pin its SHA-256 digest.
2. Export the candidate M1 discrete trunk (`#2845`) to a D11 container (`uor-r4.lut-discrete/1`) and record its `artifact_sha256`.
3. Obtain a non-author `TEST FITNESS: FIT` review on this specification, owner confirmation of the effect-size bar on `#2028`, PM dispatch, and Casey's typed D40 release.
4. Run the four-arm ablation evaluation (`AblationArm::{None, KnockoutStateTransport, KnockoutAddressedMemory, KnockoutBoth}`) across $\ge 2$ seeds on the GPU via `scripts/pod/uor-pod` (`m6-qualify-reasoning-coding` `#2865`) and seal the resulting `M6AblationReport` JSON artifact.

### Stop conditions

- Any failure of `no_knockout_arm_is_bit_identical_to_unhooked_path` or any non-zero multiply/float count in `audit_zero_matmul_serving.py` / `audit_zero_matmul_wasm.py` stops the run before GPU dispatch.
- If `AblationArm::None` fails to meet the headline M6 task accuracy bar on the frozen panel, stop and report `MechanismOrTrunkBelowBar` before claiming causal attribution.

### Cost and minimum test power

- **Cost:**
  - `preparation`: $\$0.00$ (host-only Rust implementation and finite CPU controls).
  - `run`: $\$0.00$ in this PR; GPU panel qualification (`#2865`) is a single RTX 5090 evaluation pass (`< 0.5` GPU-hours, $\le \$0.50$) within the 4-pod / $\$8.00$/h fleet cap.
  - `controls`: included in the 4-arm paired sweep (`None`, `KnockoutStateTransport`, `KnockoutAddressedMemory`, `KnockoutBoth`).
  - `evaluation`: deterministic integer oracle grading (`M6OracleKind`).
  - `retries`: $0$ (single pre-registered evaluation per seed).
  - `storage`: $< 1\text{ MiB}$ JSON report (`M6AblationReport`).
- **Minimum test power (D22 §3):**
  - $\ge 2$ trained trunk seeds evaluated across $\ge 200$ frozen panel rows spanning `Compositions`, `AlteredConstraints`, and `CodeTasks`, with paired McNemar discordant floor $F_I = \max(16, \lceil 2\sqrt{b + c}\rceil)$.

TEST FITNESS: PENDING
