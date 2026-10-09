# Shared native pooled-margin credit

## Question and causal scope

The four original-parent Prefix vectors in [#2084](https://github.com/UOR-Foundation/uor-r4/pull/2084) reduced task/reference CE but failed protected native outcomes. The [SpiralCore/native comparison](../spiralcore-live-review-2026-10-09/README.md) located a specific asymmetry: protected guard roles supply no corrective gradient, although physical rows that overlap task/reference roles retain their declared objective weight. Guards currently act at finite admission.

This deliverable establishes the shared score/credit boundary required to give those protections a differentiable margin. It does not yet select rivals, assemble protected parameter Jacobians, project joint Prefix/Generate directions or run the original-parent construction. Those remain the dependent integration task. No new serving rule, token-specific exception, geometric metric or model parameters are introduced.

## Exact artifact, data and configuration

Parent main: `4e01b93d300ab5f3382936cfe7f7a568e4289d32`. Source boundary: `crates/uor-r4-training/src/geometric_generate_learning.rs`. Input is the existing native `VocabularyActionTrace` and matching raw Generate/Copy tensors; all physical aliases remain in their native ordering. The tests use the existing eight-token authored tokenizer and integer vocabulary reducer. These are arithmetic/autodiff fixtures, not a development or held-out language split.

The new public contrast takes two explicit token IDs and the existing clipped/raw adjoint policy. Caller authority still binds model, tokenizer, causal prefix, native pool and selected protection role; tensor admission cannot establish those identities itself. The API does not automatically choose a target, runner-up, tie policy, penalty weight or optimizer.

## Numerical forward and declared adjoint

For explicit token IDs w and r, the numerical margin is `ln(Mw / Mr)` from native integer pooled masses, computed in f64 then converted to f32. Larger values favor w. It includes Generate and every physical Copy alias of each token.

The backward convention reuses the existing CE graph. Let `pN` be native pooled probability and `pS` relaxed pooled probability over the admitted clipped action scores. Each token uses the anchor `pN + (pS - stop_gradient(pS))`. The margin's adjoint is therefore `d(pS_w)/pN_w - d(pS_r)/pN_r`, equal to the existing rival CE Jacobian minus the winner CE Jacobian. A zero-forward adjustment anchors the scalar to the direct native mass ratio.

An unrelated action can receive the normalization residual `pS_j * (pS_r/pN_r - pS_w/pN_w)`. No universal smallness bound or exact off-pair cancellation is claimed. Using the difference of two unanchored relaxed log probabilities would be another backward policy; it is not silently substituted here. Clipped and RawIdentity policies retain identical native numerical forward; only their score derivatives differ. This remains a declared local surrogate, not differentiation through integer table interpolation, quantization or hard donor selection.

## Decision and limitations

**KEEP** the shared library boundary. Model benefit remains **NOT YET PROMOTED**. Accepted **8/512 complete development replies** is unchanged; no actual-nine, full512, fresh or multi-turn model evaluation is executed for this library-only change. No model training or grading runs on the laptop. No CUDA parity, serving energy or broad capability claim follows from CPU arithmetic tests.

**Next:** Consume the shared margin in the existing coupled graph, retaining authenticated Prefix occurrence incidence and the complete donor/Generate/Copy forward. Specify mechanical rival/tie selection and a finite protected-direction policy before extra gradient work. Preserve all 15 episode positions, 17 reference roles and 380 native winner checks; a local margin constraint is not a guarantee after finite quantization or donor changes. Positive construction must be followed immediately by actual-artifact own-feedback qualification.

## Resources

Prospective projection on M2: 45 minutes preparation/build/tests/review/delivery, two compiler jobs, at most 8 GiB build RAM and 6 GiB new build storage plus one owned worktree. Initial free disk 52 GiB; 30 GiB plus 128 MiB margin retained. Build output is inside the owned worktree and removed after verified delivery. No paid compute or model fit. Elapsed delivery and compilation are charged to the existing cumulative ledger; exact test/build receipts are preserved with results in iCloud.

## Executed checks and review

Source SHA256: `2dc1ee72a18af9097861849f56581581c61df289e32e22bdfe3f295c7e2f809c`. The two new public APIs are `vocabulary_log_mass_margin` (default clipped credit) and `vocabulary_log_mass_margin_with_credit` (explicit existing score policy). Existing CE now uses the same admitted physical-action probability graph. Valid-pool CE operation ordering is retained; malformed complete token tables now reject earlier, including duplicate/missing/reordered rows and mismatched Generate/Copy subtotals.

With `CARGO_TARGET_DIR` inside the owned worktree, `CARGO_BUILD_JOBS=2`, `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0`:

- `cargo test -p uor-r4-training --lib geometric_generate_learning::tests --offline`: **24 passed, 0 failed**, six newly added tests plus18 existing tests, CPU test profile. Cold compilation617 seconds; tests1.67 seconds. Five CUDA-gated module tests were not compiled in this default-feature run; CUDA validation remains NOT_RUN.
- New tests establish multiple-alias analytic derivatives, native mass-ratio forward bits, CE Jacobian equivalence with a non-grid native/relaxed discrepancy and nonzero unrelated-token credit, clipped/raw policy separation, legacy CE forward/gradient preservation, complete mass-table corruption rejection, and malformed/nonfinite tensor rejection.
- Changed-file `rustfmt --edition 2021 --check` and `git diff --check`: PASS. Claim-wording gate: PASS.
- Package-wide `cargo fmt --package uor-r4-training --check`: FAIL_PREEXISTING in `examples/demand-store-2turn.rs`, `examples/mask-scan.rs`, `examples/mqar-bench.rs` and `src/relation_compiler.rs`. Their blobs are unchanged from parent main. The complete formatting output is retained; no unrelated formatting edits are included.
- Mathematical review approves the explicitly declared CE-compatible surrogate. Independent adversarial source review found no blocking numerical, autodiff or admission regression. Neither review is an additional execution of the tests.

Build storage reached2.2 GiB, within the6 GiB projection. No actual model, original-parent protected construction, GPU, audio/geometry replay or generated-language evaluation was run. The exact-head repeat receipt is retained; final review and protected-merge receipts will be recorded on the delivery PR; CI's five historical status names remain compatibility acknowledgements, not substitutes for these tests.
