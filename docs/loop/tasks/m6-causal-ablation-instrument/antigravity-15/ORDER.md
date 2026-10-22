# ORDER: m6-causal-ablation-instrument, antigravity-15

Lab: antigravity. Item: [m6-causal-ablation-instrument](../../../../roadmap/items/m6-causal-ablation-instrument.md). Base: `d7d9d8d61`.

## Goal

The item's done-when reads:

> An ablation instrument registered before any run, able to knock out state transport and addressed memory and report the effect on the frozen panel. Registering the effect-size bar is part of this item (step 4).

Estimated diff size: ~1,200 lines across `crates/uor-r4-integer/**` (~680 library + ~360 unit tests), `docs/labs/m6-causal-ablation-instrument/README.md`, mechanism/architecture docs, and roadmap files.

## Cancel probe (run first)

Ran `ls crates/uor-r4-integer/src/` and `cat docs/harness/panels/m6-reasoning.json` on `d7d9d8d61`:
- `discrete_d11.rs` evaluates D11 discrete models (`Route`, `Flock`, `Relation`), but no M6 intermediate-state causal ablation instrument (`m6_ablation.rs`) exists on `origin/main`.
- `docs/harness/panels/m6-reasoning.json` has `"frozen": false` and `"bars": []`; `#2863` has 0 implementation comments.
- Result: work is not already done on `main`; continue.

## Files in scope

- `crates/uor-r4-training/**`
- `crates/uor-r4-integer/**`
- `docs/mechanisms/exact-memory.md`
- `docs/mechanisms/geometric-stack.md`
- `docs/model/architecture.md`
- `docs/labs/m6-causal-ablation-instrument/**`
- `docs/roadmap/items/m6-causal-ablation-instrument.md`
- `docs/roadmap/README.md`
- `docs/loop/tasks/m6-causal-ablation-instrument/antigravity-15/**`

## Done when

1. Design note posted on `#2863` and effect-size bar registration decision card posted on `#2028` (CHARTER §5.2).
2. `crates/uor-r4-integer/src/m6_ablation.rs` and `discrete_d11.rs` implement the four-arm M6 ablation instrument (`AblationArm::{None, KnockoutStateTransport, KnockoutAddressedMemory, KnockoutBoth}`, `IntegerAddressedMemory`, `M6TaskFamily`, `M6PairedMcNemarContrast`, `evaluate_m6_ablation_panel`) with finite CPU controls (`no_knockout_arm_is_bit_identical_to_unhooked_path`, `sorted_merge_gcd_sieve_and_eviction_versioning`, `four_arm_causal_ablation_separates_state_transport_and_addressed_memory`) and passing native/WASM zero-multiply audits.
3. Pre-registration and documentation updated in `docs/labs/m6-causal-ablation-instrument/README.md`, `docs/mechanisms/exact-memory.md`, `docs/mechanisms/geometric-stack.md`, `docs/model/architecture.md`, `docs/roadmap/items/m6-causal-ablation-instrument.md` (`status: done`), and `docs/roadmap/README.md`, with `uor-harness loop gates --full`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` green.

## Stop and ask if

- Any file outside the scope list above would need to change.
- Any GPU panel-scoring run is needed (requires frozen `m6-reasoning.json` from `#2866`, M1 trunk `#2845`, non-author `TEST FITNESS: FIT` review, and Casey's typed D40 release).
