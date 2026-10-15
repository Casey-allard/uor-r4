# #1789 — Step 7a: read-key probe, key-content arms (conv8, carrier, rot) and the pair pilot; REJECT at 96M (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T04:41:36Z
- **Updated:** 2026-10-09T13:09:06Z
- **Closed:** 2026-10-06T15:57:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1789
- **Merged:** 2026-10-06T15:57:34Z
- **Merge commit:** 801d219fa77495de8d4ce89dbee2233ea2d1adcb
- **Base / Head:** main / UOR-Foundation:claude/step7a-read-key
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Step 7a (read-key content): the diagnostic, three opt-in key-content arms and the pilot task. **Result: REJECT at 96M** (issuecomment-6009471195). Every option defaults to off, and the off path is bit-identical. The PR is kept so the negative result can be reproduced.

## What it adds

**Diagnostic: `binding-probe key_probe=LAYER[,LAYER]`.** It uses `StackModel::read_query_key`, which returns one read layer's per-head queries and keys after every projection, plus the NoRead logits, the age table and beta/offset. `ReadQueryKey::row` recomputes the fused read's per-row scores and weights from them. At the classified query, the probe reports for the binding head:
- the score, content/age split and q·k cosine at the expected and distractor values and at their key words;
- the cosine between the two value keys;
- the signed token gap from each value to its key word;
- a counterfactual that swaps the asked and distractor key words.

Phase 1 numbers: issuecomment-6007700083.

**Key-content arms (`ReadLineage`).** All three start as the plain read bit for bit, are saved in `config.json` (`read_lineage`) and are restored by `load`.
- `LearnedConvWide{taps}` (`conv8`): a learned depthwise causal conv of the keys over lags 0..taps−1.
- `KeyCarrier` (`carrier`): a gated, decaying per-head key carrier, `a_h = 1 − 2^−(1 + h mod 4)`.
- `KeyPhase{snap}` (`rot`, owner amendment): per-head unit-quaternion phases from learned maps, left-multiplying every 4-channel block of keys (`P_t`) and queries (`P_q`). With `snap`, the phases are straight-through snapped to the 120 unit icosians.

The other research lineages are still refused by `save`. No arm has a served or exported form yet; the serving costs are stated in the pre-registration.

**Training entry points.**
- `dialogue-train read_lineage=conv8|carrier|rot` adds the arm to an `init=` model. It refuses a silent change or drop, and a resume must agree.
- `mqar-bench lineage=conv8|carrier|rot`.

**Pilot task: `mqar-bench layout=fact pair=1`.**
- Two facts per sentence: `k1 gap1 v1 conj k2 gap2 v2`.
- Gaps up to 16.
- Each query asks one key of a pair, and its sibling value is the in-sentence distractor; a sibling-swap rate is reported.
- A crowded window redraws its placement.
- With `pair=0` (the default) windows are unchanged.

## Checks (exact head 5d5bf2bd, on a 2× RTX 5090 pod)

- `cargo test --release -p uor-r4-training --features cuda --lib -- step7a read_query_key read_lineages read_span_probe binding_probe the_qk_lineage read_key_shift`: 14/14.
  - `read_query_key_recomputes_the_fused_read_weights`: equal to `read_span_probe` within 1e-5 for L2, Lorentz and Dot, with and without the shift.
  - `step7a_key_arms_start_plain_learn_save_and_stay_causal`: plain at init; causal; every lineage variable learns; finite-difference gradients for conv8 and carrier; save/load bit-exact; plain configs keep their bytes.
  - `step7a_phase_binding_preserves_norms_and_shared_phase_scores`: norm preservation; a shared phase leaves inner products unchanged; Hamilton product equals a scalar reference; snapped phases lie in 2I; finite-difference gradient of the free phase.
  - `step7a_key_arms_reach_their_declared_window`: conv window 3, carrier unbounded, phase 1.
- `cargo test --example mqar-bench`: 15/15 with the #1017 tokenizer present. This includes the Step 2 window test unchanged, the pair-window structure and 60,000 placed training windows.
- `cargo test --example geometric-stack -- key_shift dialogue`: 2/2.
- `UOR_REQUIRE_CUDA=1 cargo test --test cuda_stack_ops_parity -- step7a`: CPU/CUDA loss, logits and lineage gradients for conv8, carrier, and `rot` with the snap on and off. The bootstrap's whole `cuda_stack_ops_parity` suite also PASS.
- **Off path on real data:** `dialogue-train` with no new option reproduces the Step 6c C-s1 model (`model_sha256` b754d656…). With the new binary, `mqar-bench` none-s1 reproduces pilot-a2 exactly.
- **Formatting:** `cargo fmt` was applied to the touched files. `fmt --check` on the crate still fails on main's `relation_compiler.rs`, which this PR does not touch, and rustfmt is absent on the pod.

## Results

**Pilot** (1.8M parameters, 3 seeds, mean full accuracy over cells):

| arm | mean |
|---|---|
| none | 0.290 |
| F2 | 0.597 |
| conv8 | 0.586 |
| carrier | 0.460 |
| rot | 0.400 |

F2 and conv8 pass the frozen rule; carrier and rot fail it on min-seed.

**96M** (2 seeds each; C = Step 6c's 24 / 21):

| arm | v4 memory |
|---|---|
| F2 | 18 / 18 |
| conv8 | 20 / 20 |

Both are rejected. Binding swaps are unchanged, and the key-lineage counterfactual barely moves.

Archives are under `/workspace/uor-r4/step7a/` on the canonical volume. Pod cost was about $4.9.

Do not merge without owner review; the arms are a negative result at 96M.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6020141519 — 2026-10-06T15:56:04Z — Casey-allard

Exact-head self-review at `2cad3da248b2c348adb3612a9d0a06b673d05216` (owner rule: self-review allowed). Merging as a kept negative result (Step 7a REJECT on #820); every arm is opt-in (`read_lineage=` default none), so default training is unchanged.

- Merged origin/main twice (no rebase; after #1798/#1800 and after #1796). Conflicts were independent additions on both sides: kept 7a's `read_lineage_arg`/`apply_read_lineage` and main's `check_gate_supervision`/`check_read_binding_supervision` and lineage keys; kept both CUDA parity tests (7a lineage, 7d read supervision) — no duplicate fn names. Head commit is empty (tree = `786eb06c`) to run checks on the ready PR.
- At this head: `cargo check -p uor-r4-training --lib --tests --example geometric-stack` → 0 errors; rustfmt --check clean on the touched files; lib tests `step7a` 3/3, `key_runs_gaps` 1/1, `the_qk_lineage` 1/1, `read_query_key` 1/1, `read_binding_tracks` 1/1, `read_supervision` 3/3; bin dialogue-recall-corpus `binding_labels_point` 1/1.
- Not run: the `cuda`-feature parity tests (no CUDA on the laptop; both pods are busy with Step 8). APPROVE.


---
_Comment/review ids in this file: 6020141519_

