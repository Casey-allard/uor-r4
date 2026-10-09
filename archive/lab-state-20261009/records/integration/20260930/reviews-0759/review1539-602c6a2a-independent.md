Reviewed UTC: 2026-09-30T08:01:18.045962+00:00

# Independent correction-delta review: PR1539

**REQUEST_CHANGES — two small compilation corrections.** Exact head `602c6a2a250cb2b601caafbcd77c7a8747cdbbd4`; delta base `8844a5829fcb067c37c7177f5fb3a4a76f5d200d`. Non-author reviewer independent_systems_review, same provider/root launcher. Read only this two-file delta and the definitions necessary to assess it; no broad re-audit. No source edits, Cargo/model execution or GitHub mutation. `git diff --check` passed; Rust compilation/tests **NOT_RUN**.

## Required corrections

1. **P1 — missing PathBuf import in production source.** `crates/uor-r4-integer/src/stack/session.rs:981` adds `Option<PathBuf>`, but line6 imports only `std::path::Path`; there is no other PathBuf import or definition. Use `use std::path::{Path, PathBuf};` or qualify the argument type. This is a source-identified compile blocker, not an executed compiler result.

2. **P1 for test delivery — new fixture accesses a private model field.** `crates/uor-r4-integer/src/stack/tests.rs:974` and977 use `model.shape.pattern` and `model.shape.lanes()`. IntegerStackModel.shape is private to sibling module session; tests elsewhere already use the public shape() accessor. Change these to `model.shape().pattern` and `model.shape().lanes()` (or bind `let shape = model.shape()`).

## Four prior findings resolved at source level

- **Late-trace/reset panic:** enable_snap_trace allocates full context capacity; reset restores full capacity if needed; save_state checks the trace slice bound. The added late-enable/reset/full-stream/save/restore fixture exercises the actual trigger. Full-context trace capacity is sufficient for every successful step, since the existing position guard prevents stepping beyond context.
- **Private bound helper:** it is now pub(super), and the sibling test calls super::session::max_serialized_session_bytes. That visibility/path issue is resolved independently of the new private-shape access above.
- **Ignored directory open:** parent File::open and sync_all are both fallible and propagated. Documentation correctly distinguishes a pre-rename failure preserving the old target from post-rename durability uncertainty.
- **Ineffective collision test:** the fixture now deterministically supplies an existing sibling temp to an attempt against the actual checkpoint destination, checks failure, and verifies both the old checkpoint and colliding sentinel survive. This is a meaningful collision-path witness, independent of the global counter or parallel test order. It does not claim a mid-write fault simulation, and the source still performs complete write/file-sync before replacing the target.

The size allowance now explicitly targets the compact writer and allows24bytes per numeric element, covering i64/u64 extremal compact representations with punctuation. No new concrete capacity/serialization regression identified within the validated model-shape scope. The unchanged old results and previously resolved continuation/logit semantics are not re-audited here.

After the two import/accessor corrections, the next useful action is the already-required admitted touched-package compilation and focused persistence tests. No new model experiment or broader review is requested.
