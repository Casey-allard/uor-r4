# ORDER: m1-close-saved-model, antigravity-108

Lab: antigravity. Item: [m1-close-saved-model](../../../../roadmap/items/m1-close-saved-model.md). Base: `efcbd7065`.

## Goal

The item's done-when reads:

> One saved discrete model: (a) starting reach at no fewer than 4,894 of 14,287; (b) reach kept on both seeds (G-Adm); (c) more than 16 of 256 cells in use plus the D39 occupancy condition (G-Occ); (d) integer top-1 through the integer export beats the unigram table (1,226) and the next-word table (6,300) on each of two seeds, with tokens per second reported; (e) the gain holds on a fresh sealed panel.

In `antigravity-107` (`#3308`, `efcbd7065`), `IntegerDiscreteModel` (`crates/uor-r4-integer/src/discrete_d11.rs`) was extended to accept and evaluate the 19-table `uor-r4.lut-discrete-trunk/1` schema (`trunk-joint-pointer-v1-d11`), clearing **Bars (a)–(d)** through `crates/uor-r4-integer`.

However, `d11-scorer score` and `d11-scorer fresh-score` (**Bar (e)**, `#2841`) score rows on the GPU via `ArtifactTables`, `DeviceScorer`, `kernel.rs`, `host_kernel.cpp`, `scorer.cu`, and `kernel_core.h` in `docs/labs/m1-d11-artifact-scorer/instrument/`:
1. `ArtifactTables::from_bytes` (`src/artifact.rs`) extracts only the 4 base sections and ignores the 15 trunk sections (`step_candidates`, `norm_candidates`, `unigram`, and 12 sparse count tables).
2. `d11_score_row` (`src/kernel_core.h`) evaluates only the 4-table pre-rotation read vector, so `kernel::score_rows` (host C++ and CUDA `DeviceScorer`) disagrees with `IntegerDiscreteModel::evaluate_prefix` and `observe_target` (`reference_check` fails) on any 19-table `uor-r4.lut-discrete-trunk/1` artifact.
3. Also, `docs/roadmap/items/m1-close-saved-model.md` lists its own path in `scope:`, causing `loop claim` to report a false glob overlap with the idle `infra-loop-harness` umbrella draft `#3309` (`docs/roadmap/items/*.md`).

This unit:
1. Extends `ArtifactTables` (`src/artifact.rs`), `kernel.rs`, `device.rs`, `host_kernel.cpp`, `scorer.cu`, and `kernel_core.h` in `docs/labs/m1-d11-artifact-scorer/instrument/` to pack and execute the 19-table `uor-r4.lut-discrete-trunk/1` joint-pointer Hamilton gate in `d11_score_row`, achieving `0` mismatches against `IntegerDiscreteModel::evaluate_prefix` and `observe_target` (`reference_check` `agreed == checked`) and passing `planted_fault_control` (`B4`) on 19-table trunk `.d11` artifacts.
2. Drops `docs/roadmap/items/m1-close-saved-model.md` from `docs/roadmap/items/m1-close-saved-model.md`'s `scope:`.

## Cancel probe (run first)

1. `uor-harness loop claim m1-close-saved-model --lab-topic m1-close-kernel` refused on `infra-loop-harness` umbrella glob `docs/roadmap/items/*.md` (`PR #3309`, `0` overlapping files). Hand-written `ORDER.md` per PART 3 Step 2 umbrella exception.
2. Confirmed on `efcbd7065` (`instrument` classification): passing a 19-table `export_trunk_d11` artifact to `ArtifactTables::from_bytes` + `kernel::score_rows` omits the 15 trunk sections, causing `reference_check` against `IntegerDiscreteModel::evaluate_prefix` to fail (`agreed != checked`) before the `kernel_core.h` fix.

## Files in scope

- `crates/uor-r4-discrete/**`
- `crates/uor-r4-integer/**`
- `docs/labs/m1-d11-artifact-scorer/instrument/**`
- `docs/harness/prep/m1-close-saved-model*`
- `docs/labs/m1-close-saved-model/**`
- `docs/loop/tasks/m1-close-saved-model/**`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/roadmap/items/m1-close-saved-model.md`

## Done when

1. `ArtifactTables`, `kernel_core.h`, `host_kernel.cpp`, `scorer.cu`, `kernel.rs`, and `device.rs` evaluate 19-table `uor-r4.lut-discrete-trunk/1` artifacts with `0` mismatches against `IntegerDiscreteModel::evaluate_prefix` and `observe_target`, pass `planted_fault_control` (`B4`), and type-check under `--features cuda --all-targets`.
2. `docs/roadmap/items/m1-close-saved-model.md` removes its self-path from `scope:`, and all tests + `uor-harness loop gates --full` pass.

## Stop and ask if

- Any change would violate `CHARTER.md` §3 or touch an owner-only path.
