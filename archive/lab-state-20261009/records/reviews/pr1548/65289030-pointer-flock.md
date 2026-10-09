# PR #1548: bounded independent pointer/flock integration review

- Reviewer: Codex `/root/second_council_review`, delegated by `/root`; non-author of this PR, separate reasoning session within the same Codex lab/provider. Independence is procedural, not provider diversity.
- Date: 2026-09-30.
- Exact reviewed head: `652890305d181450eb27c2e3bf34430b1a8f03c4`.
- Exact base: `7b11e1d803689e48f3085182c8f662b689f22592`.
- Live `gh pr view 1548` confirmed those head/base identities during review.
- Decision: **REQUIRED FIXES in the scoped integration review**; two concrete findings below. This is not approval/rejection of every change in the approximately 12,000-line PR.
- Actions: read-only Git/GitHub/source inspection and this local receipt only. No source edits, builds, tests, model jobs or GitHub writes.

## F1 — P1: the snapped D11 evaluator still accepts and mis-scores a pointer checkpoint

File: `crates/uor-r4-training/examples/geometric-stack.rs`, lines **2876–2883**, with the raw-logit use at **2903** and misleading float-model NLL/agreement record at **2939–2955**.

`d11_evaluate_snapped` loads arbitrary `model=` using `StackModel::load`, applies the transport snap, and checks only vocabulary/context. Unlike `snap-evaluate`, `rounding-attribution` and `lut-evaluate`, it never invokes `check_raw_logit_evaluation`. It then calls `float.forward`, which the new pointer contract explicitly retains as ordinary raw logits rather than the pointer mixture. A saved pointer checkpoint with the matching shape and a valid existing snapped D11 artifact therefore gets accepted, has its copy head omitted, and produces a report labelled as the float model's NLL and D11 agreement. The export refusal does not prevent this: the CLI separately accepts artifact and model paths and does not require the supplied pointer checkpoint to have produced that artifact.

Required scoped correction: invoke the pointer/raw-logit refusal immediately after loading this comparator, naming `d11-evaluate`, and add a regression covering the snapped-artifact arm with a pointer-bearing comparator. Retain support for legitimate non-pointer comparisons. `check_export_config` may additionally be appropriate if this arm requires strict export compatibility; the essential finding is the silent pointer omission, not a request for a broad artifact-binding redesign.

Source: https://github.com/UOR-Foundation/uor-r4/blob/652890305d181450eb27c2e3bf34430b1a8f03c4/crates/uor-r4-training/examples/geometric-stack.rs#L2876

## F2 — P2: ordinary `train init=` bypasses the single-source training refusal

File: `crates/uor-r4-training/examples/geometric-stack.rs`, lines **1162–1169**; inherited config at **922–925**, saved-model load at **1598–1608**, update-path loss at **1683**.

The PR correctly refuses `pointer_select=top:1` in `dialogue_train_mode` after resolving the explicit option or saved `init=` selection (3354–3367). However, the existing ordinary `train` mode also accepts `init=`. Its `init_config` returns the saved `StackConfig` including the newly added pointer selection. `train_settings` performs only the QAT check here, so `train init=<saved-pointer-top1-model>` proceeds to model loss/backward/optimizer with the single-source pointer. Excluding pointer options from this mode's argument allowlist does not exclude a saved pointer model. For top:1 the query/key/scale gradients are identically zero, as the PR's own mathematical contract and finite-difference tests establish, while weight decay and other model parameters still change. This bypasses the stated training refusal and can silently consume training as though the pointer selector were trainable.

Required scoped correction: check the effective pointer selection in ordinary `train_settings` before execution, using the same refusal, with a saved-config regression. Evaluation must continue to permit post-hoc top:1. Low-level loss APIs may remain available for gradient tests; no blanket API ban is requested.

Source: https://github.com/UOR-Foundation/uor-r4/blob/652890305d181450eb27c2e3bf34430b1a8f03c4/crates/uor-r4-training/examples/geometric-stack.rs#L1162

## Reviewed integration that did not produce an additional concrete finding

- `geometric_stack.rs`: transformer/control attention and geometric reads pass `config.select` into `fused_read_selected`. `FusedRead::transform` at 3982–4031 builds the total causal score (including age/Lorentz components), calls canonical `crate::flock::flock_select` with exactly `scores[..=t]`, masks unkept positions to negative infinity, and retains the separate NoRead term. Forward and backward use the same tile/transform path. Full-support masking leaves scores untouched, preserving the prior position-order reduction.
- Pointer selection at 214–217 and 4854–4884 delegates to canonical flock/top-k support independently of read selection. The mask consumes positions rather than accidentally interpreting rank order as source order. Batch indexing of pointer query/key/copy support is causal within each window.
- The mixture-aware `loss`, `weighted_loss`, `target_nll`/`score_targets`, `next_scores`, dialogue development and greedy reply integrations were traced. The custom backward's zero-copy branch and copy-share algebra were inspected for their role in selected support and single-source gradient behavior; this was not an exhaustive floating-point proof over every input.
- Pointer QAT refusal is present in the CLI and `set_served_representation`; export and grid-reference boundaries invoke export checks. AERM constructors/load reject a pointer model instead of omitting its head.
- Existing test source reviewed includes the preserved prototype mask comparison (8020), full-support bitwise equality (8055), masked-reference and finite-difference checks (8163), support/NoRead/zero-gradient assertions (8308), nonfinite ranking refusal (8429), pointer's own score/selection checks (8502), pointer mixture finite differences including top:1 (8885), pointer save/reload/post-hoc selection and export refusal. Shared selector source and its existing tie/short-row tests were read. I found no concrete new selector-migration defect in this bounded scope.

## Evidence limits

This review inspected tests; it did not execute them. The PR body reports a separate CI ref with identical Rust sources, 393 passing library tests and three attributed baseline failures, plus example/integration checks. I did not independently verify that run's complete logs, tree equality, or baseline-failure attribution, and do not convert those statements into my own executed PASS. The two caller regressions above are not covered by the inspected refusal tests: the raw-logit helper test names only the three guarded modes, and top:1's CLI test exercises dialogue-train only.

No broad M-world-v2 distribution/probe/held-out audit, full custom-autodiff numerical campaign, real artifact reload, trained retrieval, useful conversation, D11 pointer serving or whole-PR qualification is asserted. Stop this review at these concrete integration risks; fix and rereview the narrow successor delta rather than launching an unrelated test campaign.
