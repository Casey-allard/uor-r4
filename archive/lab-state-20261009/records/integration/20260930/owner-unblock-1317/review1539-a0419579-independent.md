# Independent source-only correction review: PR1539

REQUEST_CHANGES: one remaining test compilation correction. Reviewed head `a04195797ccda2ab81f60137a4bfa76876f8e85b`, delta base `602c6a2a250cb2b601caafbcd77c7a8747cdbbd4`. Independent non-author reviewer, same provider/root launcher. Read-only review of the two-file delta, saved-state references, and surrounding restore/test definitions. No source edits, Cargo/model execution, or GitHub/production changes. `git diff --check` passed; Rust compilation and tests remain NOT_RUN.

## Required correction

P1 for test delivery: `crates/uor-r4-integer/src/stack/tests.rs:627` assigns `CONTEXT + 1`, whose type is usize, to `bad_pos.position`, changed to u64 by this delta. Change it to `(CONTEXT + 1) as u64`. Rust does not implicitly convert this assignment. This is a source-identified compile error, not an executed compiler diagnostic.

## Resolved and assessed

- The prior missing PathBuf import is fixed in session.rs:6.
- The late-trace/reset fixture now uses the public model.shape() accessor in tests.rs:981 and984.
- All three serialized counters now use u64. restore_state converts each with usize::try_from before checking context, expected cache lengths, and slicing. The production delta preserves validation-before-mutation and does not introduce a truncating restore conversion.
- Unsupported schema versions now consistently return StackError::Schema, and the corresponding test expects it.
- Invalid trace length and out-of-range root tests now run against a genuinely snapped model. The out-of-range fixture preserves the correct nonzero trace length and changes its roots to120, so it reaches the intended value validation rather than merely the unsnapped-model rejection.

No other concrete source regression was found in this bounded delta. The existing save/restore, late-trace/reset, compact-size, and atomic-write corrections are not reopened. After the one assignment correction, proceed to the already-required admitted compilation and focused persistence tests; no extra model experiment or broad audit is requested. Session-state persistence remains distinct from learned durable conversation-memory qualification.
