# REPORT: m6-causal-ablation-instrument, antigravity-15

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Initial synthetic `Relation` C2 container fixture in `m6_ablation::tests::build_relation_c2_bytes` failed `IntegerDiscreteModel::from_bytes` validation:** The initial fixture set `read_initialization: "seeded_v1"` and `gates.len() = 4 + 7 * 2 = 18`, which `IntegerDiscreteModel::from_bytes` rejected (`unsupported read_initialization` and `discrete.gates length mismatch`) because `discrete_d11.rs` validates `read_initialization` against `{"zero_gates_v1", "active_leaf_v1", "address_structured_v1"}` and defines `EDGE_TYPES = 5` (`total_gates = 4 + 5 * 2 = 14`). Setting `read_initialization: "active_leaf_v1"`, `gates.len() = 14` (`gates[2] = 1, gates[6] = 1`), and distinct E8 orbit-root cells for the 5 tokens resolved the validation and isolated `SequentialSuccessor` state transport cleanly.
- **No M6 research notebook file exists and `docs/research/notebook/` is outside the item's `scope`:** [`docs/roadmap/items/m6-causal-ablation-instrument.md`](../../../../roadmap/items/m6-causal-ablation-instrument.md) defines `scope: [crates/uor-r4-training/**, crates/uor-r4-integer/**, docs/mechanisms/exact-memory.md, docs/mechanisms/geometric-stack.md, docs/model/architecture.md, docs/labs/m6-causal-ablation-instrument/**]`, so the lab record and 1-page pre-registration specification are recorded in [`docs/labs/m6-causal-ablation-instrument/README.md`](../../../../labs/m6-causal-ablation-instrument/README.md) and cross-linked from [`docs/mechanisms/exact-memory.md`](../../../../mechanisms/exact-memory.md), [`docs/mechanisms/geometric-stack.md`](../../../../mechanisms/geometric-stack.md), and [`docs/model/architecture.md`](../../../../model/architecture.md).

## What changed

This task closes #2863 by building the M6 intermediate-state causal ablation instrument on the served integer path (`uor-r4-integer`), verifying its four-arm factorial knockouts on finite CPU controls and zero-multiply native/WASM audits, and registering the M6 causal ablation specification and proposed effect-size bar:

- **State-transport knockout hook ([`crates/uor-r4-integer/src/discrete_d11.rs`](../../../../../crates/uor-r4-integer/src/discrete_d11.rs)):** Added `state_transport_knockout: bool`, `IntegerDiscreteModel::with_state_transport_knockout(&self) -> Self`, and `IntegerDiscreteModel::is_state_transport_knocked_out(&self) -> bool`, projecting the read vector $q \in (\mathbb{Z}^8)^B$ onto the current leaf token's own static E8 orbit-root blocks $v(x_{L-1})$ before running the identical `rank_writer` shortlist and tie-break.
- **M6 causal ablation instrument and integer addressed memory ([`crates/uor-r4-integer/src/m6_ablation.rs`](../../../../../crates/uor-r4-integer/src/m6_ablation.rs), [`crates/uor-r4-integer/src/native_geometry.rs`](../../../../../crates/uor-r4-integer/src/native_geometry.rs), [`crates/uor-r4-integer/src/serving_root.rs`](../../../../../crates/uor-r4-integer/src/serving_root.rs)):** Implemented `PrimeFactorKey`, `IntegerAddressedMemory` (sorted-merge GCD sieve admission $\gcd(q, A_c) > 1$, `[Initial, Previous, Current]` versioning, and `Evicted { overwritten_by_turn }` vs. `Absent` separation), `AblationArm::{None, KnockoutStateTransport, KnockoutAddressedMemory, KnockoutBoth}`, `M6TaskFamily::{Compositions, AlteredConstraints, CodeTasks}`, `M6OracleKind`, `M6PairedMcNemarContrast`, `ceil_two_sqrt_int` (via `math::isqrt_u64`), `evaluate_m6_row_under_arm`, and `evaluate_m6_ablation_panel`.
- **Lab record, pre-registration specification, mechanism docs, and roadmap render ([`docs/labs/m6-causal-ablation-instrument/README.md`](../../../../labs/m6-causal-ablation-instrument/README.md), [`docs/mechanisms/exact-memory.md`](../../../../mechanisms/exact-memory.md), [`docs/mechanisms/geometric-stack.md`](../../../../mechanisms/geometric-stack.md), [`docs/model/architecture.md`](../../../../model/architecture.md), [`docs/roadmap/items/m6-causal-ablation-instrument.md`](../../../../roadmap/items/m6-causal-ablation-instrument.md), [`docs/roadmap/README.md`](../../../../roadmap/README.md)):** Recorded the design, finite controls, and 1-page pre-registration (`TEST FITNESS: PENDING`), flipped `m6-causal-ablation-instrument` to `status: done`, and re-rendered `docs/roadmap/README.md`.
- **Size by class:** ~680 lines of integer library code and ~360 lines of unit tests in `crates/uor-r4-integer`, plus ~160 lines of documentation and task records across 10 files.

## Measurements

- **Finite CPU controls (`crates/uor-r4-integer/src/m6_ablation.rs`):**
  - Command: `~/.cargo/bin/cargo test --locked -p uor-r4-integer --lib` and `~/.cargo/bin/cargo test --locked -p uor-r4-integer --features serving-wasm --lib`
  - Result:
    - `sorted_merge_gcd_sieve_and_eviction_versioning`: `ok` (`shared_primes = 2, shared_pairs = 1` selects key `[3, 5, 11]` over `[5, 11, 17]`; `Current = [2, 3]`, `Previous = [2, 4]`, `Initial = [2, 4]`; capacity eviction returns `Evicted { overwritten_by_turn: 4 }`; disjoint prime returns `Absent`).
    - `no_knockout_arm_is_bit_identical_to_unhooked_path`: `ok` (`AblationArm::None` produces bit-identical `emitted_tokens = [2, 4, 2]` and `step_traces` to direct `model.evaluate_prefix` and `mem.lookup`).
    - `four_arm_causal_ablation_separates_state_transport_and_addressed_memory`: `ok` on the 60-row synthetic M6 panel (`20` `Compositions`, `20` `AlteredConstraints`, `20` `CodeTasks`):
      - `AblationArm::None`: `60 / 60` (`20/20` per family).
      - `AblationArm::KnockoutStateTransport`: `20 / 60` (`0/20` `Compositions`, `0/20` `AlteredConstraints`, `20/20` `CodeTasks`; overall discordant drop $b - c = 40 - 0 = 40 \ge \max(16, \lceil 2\sqrt{40}\rceil) = 16$).
      - `AblationArm::KnockoutAddressedMemory`: `20 / 60` (`0/20` `Compositions`, `20/20` `AlteredConstraints`, `0/20` `CodeTasks`; overall discordant drop $b - c = 40 - 0 = 40 \ge 16$).
      - `AblationArm::KnockoutBoth`: `0 / 60` (`0/20` per family; overall discordant drop $b - c = 60 - 0 = 60 \ge \max(16, \lceil 2\sqrt{60}\rceil) = 16$).
- **Native and WebAssembly zero-multiply / zero-float serving audits:**
  - Command: `python3 scripts/audit_zero_matmul_serving.py target/release/uor-r4-stack --stack` and `python3 scripts/audit_zero_matmul_wasm.py target/wasm32-unknown-unknown/release/uor_r4_integer.wasm --serving --call-graph --tap`
  - Result: `44/44 PASS` on native `uor-r4-stack` and `1..7 ok` (`mul=0, div=0, float=0, unclassified=0`) on `uor_r4_integer.wasm`.

## Checks run at the head

- `~/.cargo/bin/cargo test --locked -p uor-r4-integer --lib`: `ok. 48 passed; 0 failed`.
- `~/.cargo/bin/cargo test --locked -p uor-r4-integer --features serving-wasm --lib`: `ok. 299 passed; 0 failed`.
- `python3 scripts/audit_zero_matmul_serving.py target/release/uor-r4-stack --stack`: `44/44 PASS`.
- `python3 scripts/audit_zero_matmul_wasm.py target/wasm32-unknown-unknown/release/uor_r4_integer.wasm --serving --call-graph --tap`: `1..7 ok` (`mul=0, div=0, float=0, unclassified=0`).
- `~/.cargo/bin/cargo run --locked -p xtask -- roadmap check`: `roadmap check: 88 items OK`.
- `~/.cargo/bin/cargo run --locked -p xtask -- links --baseline /private/tmp/base-links.json --renames-from "$(git merge-base origin/main HEAD)"`: `new broken: 0`.
- `uor-harness loop gates --full`: `6 gate result(s) GREEN`.
- `uor-harness loop report --check`: `PASS`.
- `uor-harness loop pr-check --local`: `verdict: PASS`.

## Skills used

- `uor-research-architect`: followed for pure-integer geometric knockout design on the served D11 discrete evaluator (`uor-r4-integer`), sorted-merge GCD sieve admission, and paired McNemar pre-registration.

## Still open

- Running the four-arm M6 causal ablation on a trained trunk (`m6-qualify-reasoning-coding` `#2865`) requires freezing `docs/harness/panels/m6-reasoning.json` (`m6-reasoning-panel-freeze` `#2866`), training and exporting the M1 discrete trunk (`m1-trunk-assembly-and-training` `#2845`), owner confirmation of the proposed effect-size bar on `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6341710686`), a non-author `TEST FITNESS: FIT` review on [`docs/labs/m6-causal-ablation-instrument/README.md`](../../../../labs/m6-causal-ablation-instrument/README.md), PM dispatch, and Casey's typed D40 release.

## Plain summary

To verify that reasoning and code generation come from the model's intermediate geometry and exact addressed memory rather than from memorized surface templates, this change builds the M6 four-arm causal ablation instrument (`crates/uor-r4-integer/src/m6_ablation.rs`) on the served integer discrete model (`IntegerDiscreteModel` in `crates/uor-r4-integer/src/discrete_d11.rs`). The instrument evaluates every row across four paired arms—no knockout (`None`), state-transport knockout (`KnockoutStateTransport`, which projects multi-hop context-tree, flock, and relation transport back to the current leaf token's static E8 orbit-root blocks), addressed-memory knockout (`KnockoutAddressedMemory`, which replaces the sorted-merge prime-factor GCD sieve lookup with the empty-lattice outcome `KnockedOut`), and joint knockout (`KnockoutBoth`)—and computes paired McNemar discordant drops ($b - c$ vs. $\max(16, \lceil 2\sqrt{b + c}\rceil)$) per task family (`Compositions`, `AlteredConstraints`, `CodeTasks`) using pure integer arithmetic with zero multiplies and zero floats.
