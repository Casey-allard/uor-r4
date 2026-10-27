# ORDER: m4-lawful-likelihood-bpb, antigravity-113

Lab: antigravity. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md). Base: `7746eec32a854394c2cd03c6edd4efa04f277c89`.

## Goal

Complete the registration of the potential-ratio lawful likelihood (`PRLL`, `crates/uor-r4-integer/src/lawful_likelihood.rs`) in [`docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md`](../../../harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md) and [`docs/labs/m4-lawful-likelihood-bpb/README.md`](../../../labs/m4-lawful-likelihood-bpb/README.md) by folding in (1) the owner's recorded answer to Prerequisite 2 (`#2028#issuecomment-6379430093` / `#2861#issuecomment-6379432987`, `"Yes to both (Recommended)"`) and (2) the exported two-seed 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `uor-r4.lut-discrete-trunk/1`) base artifact pins from `m1-close-saved-model` (`#2837` / `#3312`: `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` and `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`, seeds `0x5eed_0001` and `0x5eed_0002`, clearing Prerequisite 1), obtain a fresh-context non-author `TEST FITNESS: FIT` review, update [`docs/research/open-questions.md`](../../../research/open-questions.md), [`docs/model/serving-and-export.md`](../../../model/serving-and-export.md), and [`docs/research/notebook/m4-serving.md`](../../../research/notebook/m4-serving.md), and transition [`docs/roadmap/items/m4-lawful-likelihood-bpb.md`](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (`#2861`) to `status: done`.

## Cancel probe (run first)

Ran `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --offline -p uor-r4-integer --lib lawful_likelihood --no-fail-fast`: `9 passed; 0 failed` (`tiny_alphabet_is_hand_computable`, `worst_case_minimum_keeps_one_unit_and_forty_eight_bits`, `uniform_power_of_two_vocabulary_is_exactly_uniform`, `non_power_of_two_remainders_differ_by_at_most_one`, `ties_are_keyed_by_the_lowest_index`, `order_and_argmax_are_preserved`, `distinct_scores_permute_masses_exactly`, `uniform_count_baseline_scores_exactly_log2_vocabulary`, `a_zero_count_is_unscorable_not_infinite`). Inspected `docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md`: Prerequisite 3 (`se_n = 0.014525 * sqrt(512/n)` and 2048-window `INCONCLUSIVE` rule) was resolved in `deepseek-59` (`#3099`), Prerequisite 2 was answered by Casey (`"Yes to both (Recommended)"`, `#2861#issuecomment-6379432987`), and Prerequisite 1 (exported `UORLUT01` base artifact SHA-256 and training seeds) is now available from `m1-close-saved-model` (`#2837` / `#3312`). Classification: `instrument`. **Not cancelled.**

## Files in scope

- `docs/model/serving-and-export.md`
- `docs/research/findings.md`
- `docs/research/open-questions.md`
- `docs/labs/m4-lawful-likelihood-bpb/**`
- `docs/harness/prep/m4-lawful-likelihood-bpb*`
- `docs/loop/tasks/m4-lawful-likelihood-bpb/**`
- `docs/research/notebook/m4-serving.md`

## Done when

- `docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md` pins the two-seed exported `UORLUT01` base artifacts (`04652a3d…` and `1b49e8bd…`, seeds `0x5eed_0001` and `0x5eed_0002`), records Casey's `"Yes to both (Recommended)"` answer (`#2861#issuecomment-6379432987`), passes `uor-harness check` (`prep.sheet_shape`), and carries a fresh-context non-author `TEST FITNESS: FIT` review posted on `#2861`.
- `docs/labs/m4-lawful-likelihood-bpb/README.md`, `docs/research/open-questions.md`, `docs/model/serving-and-export.md`, and `docs/research/notebook/m4-serving.md` are updated consistently.
- `docs/roadmap/items/m4-lawful-likelihood-bpb.md` is set to `status: done`, `docs/roadmap/README.md` is re-rendered (`xtask roadmap render`), and `REPORT.md` says `Closes #2861` (explicitly noting that the GPU stream-scoring run itself belongs to `m4-export-uorlut01-cli` `#2859`).

## Stop and ask if

- Any change to `CHARTER.md`, `§3` technical constraints, the 0.90 BPB M4 bar, or GPU caps would be required.
- Any file outside the declared scope would need to be modified.
