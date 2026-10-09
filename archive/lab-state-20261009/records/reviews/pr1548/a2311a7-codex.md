# PR1548 scoped successor review

Reviewer: Codex `/root/second_council_review`, delegated by `/root`, non-author of the PR. Date: 2026-09-30. Exact reviewed head `a2311a7fc83f962c74a9fcf7f81552d02f8f5eaa`; live GitHub base `18e4bd893a76ba516a6537437fc7613b14d48741`, OPEN/non-draft when checked. Delta baseline for the two earlier findings: `652890305d181450eb27c2e3bf34430b1a8f03c4`, previous review comment5914726031. No edits, builds/tests, model jobs or GitHub writes; this local receipt only.

**Decision:** both previous caller defects are resolved in source. One concrete test-compilation blocker appears in the inspected successor delta and requires a minimal fix. This is neither a whole-PR review nor executed test approval.

## Earlier F1 resolved: snapped D11 comparator refuses pointer models

In `crates/uor-r4-training/examples/geometric-stack.rs:2889`, `d11_evaluate_snapped` now uses `load_raw_logit_comparator(model_dir,"d11-evaluate")` before transport snap, token loading or float forwarding. The helper at763–766 loads the model and immediately calls `check_raw_logit_evaluation` on its actual saved configuration. The real snapped-artifact dispatch at2742 still reaches this function. Thus the previously unguarded pointer comparator cannot silently be scored through raw logits.

Added regression at4388 saves/loads a pointer model through that same helper and checks the named refusal; it also verifies the helper accepts a model without the pointer head. This is useful helper-level coverage with actual save/load, while the evaluator-to-helper connection was statically inspected. I did not execute it or claim complete artifact-level end-to-end coverage.

## Earlier F2 resolved: ordinary train init inherits the top1 refusal

At `examples/geometric-stack.rs:1178`, `train_settings` now calls `refuse_trained_single_source` on the effective `config.pointer.select` after resolving `init_config` or fresh configuration. This precedes data settings and the output claim in the real `train` dispatch at4259–4262. `init_config` reads/validates the saved config, so the saved top1 case is not bypassed by the CLI's pointer-option allowlist. The existing dialogue-train refusal remains at3380.

The added regression at4415 supplies a saved config with top1 and calls real `train_settings`; it requires the training-refusal error. The soft-head companion reaches a later missing-train error instead. It is a focused config/caller regression, not a model training test. No tests were executed in this review.

## New concrete finding — P1: two added library test fixtures omit required fields

`crates/uor-r4-training/src/geometric_stack.rs:5276` and`:5318`, inside `target_nll_with_explicit_head_changes_when_head_modified_leaving_embeddings_fixed` and `target_nll_with_head_rejects_malformed_head_shapes`, construct `StackConfig` literals ending at `memory: None`, with no struct-update remainder. The same exact head's definition at293–323 requires `select` (318) and `pointer` (322). Rust requires both fields in a literal; their serde defaults apply only to deserialization. Consequently building the library test target will fail with missing fields before tests can execute.

Required correction: add `select: None` and `pointer: None` to both ordinary-model fixtures (or use an existing complete fixture constructor preserving those values), then compile/run the relevant focused test target. This finding is source-demonstrated; no compiler log or executed failure is claimed. It arose while checking the small `geometric_stack.rs` successor delta and does not expand this review to every PR path.

## Scope and validation limit

No additional blocker found in the two requested caller fixes. The new explicit-head API and the rest of the large PR were not comprehensively reviewed. Report source closure of the earlier findings separately from this new fixture blocker and from executed validation. Provider independence is procedural within the same Codex lab; no whole-PR source/evidence approval is issued.
