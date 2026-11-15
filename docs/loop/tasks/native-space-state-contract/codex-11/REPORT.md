# REPORT: native-space-state-contract, codex-11

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim` refused with exit 2 because PR #3981 claims overlapping `AGENTS.md` scope; PR
  #3937 separately holds this item's `stack/**` scope. The work continued under the owner's open
  Q31 direction without touching PR #3937's files.
- The first formatter run followed child modules and touched twelve unrelated files, including
  `stack/tests.rs`. Every formatter-only file outside this unit was restored byte-for-byte from
  `origin/main`; the final diff contains only the three ordered Rust files and loop/notebook
  records.
- The first real-recurrence fixture reached an out-of-domain carry after decay plus drive. The
  shared Q31 operator correctly refused the next step. The carry boundary now applies
  `q31_saturate`, so every served carry remains in the same closed domain as training.
- The all-target integer suite stops on six pre-existing tokenizer/bundle tests whose external
  fixtures are absent from this checkout. The relevant M7 and session-codec integration tests
  pass; the missing-fixture failure is not counted as a code result.
- A wasm32 `--lib` check passes. A wasm32 all-target check still fails in existing timing-only
  binaries that import symbols gated off wasm32; this unit did not change those binaries.
- The static serving auditor is not FIT on this Linux host: its call-graph path invokes the
  unavailable macOS `otool`, and its strict ARM64 mode still treats now-allowed scalar integer
  multiply instructions (D80) as failures. The report does not claim a static-audit pass.
- Package-wide integer Clippy with `-D warnings` is red on existing unrelated warnings. A
  package Clippy run exits 0 and reports zero diagnostics in the three touched Rust files.
- GitHub assigned no runner to the first final-head policy/gate jobs. Force-canceling that
  superseded run allowed its `always()` acknowledgement job to publish a stale failure at that
  commit. A documentation-only refresh separates the final candidate from that check set; it
  changes no executable content.

## What changed

`IntegerStackModel` can now attach the exact 48-byte native Q31 state and expose those bytes
without conversion. When attached, the shared recurrence calls
`Q31ServingSession::transition` for every lane; decay, drive and the closed-domain saturation then
produce the next carry. A model without an attachment stays on the existing artifact path.

Requested tracing records every lane's raw input carry, raw Q31 transport and raw post-drive
carry. The old FBS image accessor refuses a model with an attached Q31 state, so an evaluator
cannot silently omit it and still claim parity.

Three Rust source files changed; no dependency or artifact format changed. The focused Q31 test
module grew from two to six tests. The code/test change is 561 inserted and 77 removed lines
before the loop and notebook records, 16% above the ORDER's 550-line upper estimate.

## Measurements

- CUDA qualification, RTX 4090, `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89
  UOR_REQUIRE_CUDA=1 cargo test --locked -p uor-r4-discrete --features cuda --test
  native_vector_cuda`: 4 tests passed; 16,384 raw transport words and 4,096 training records
  matched the Rust oracle; 64 shared model steps kept the same gate and epoch.
- Release qualification, same CUDA environment, `cargo run --locked -q -p uor-r4-discrete
  --features cuda --example native_vector_qualify`: gate `[2008113292, 336469910, 0, 0]`, epoch
  `64`, aggregate loss `17,179,869,184 -> 13,373,147,056`. The exact 48-byte state SHA-256 is
  `3419349b6e9b9fccb3c0587e1ebe5b519698416605c8be0187df5298034f46a3`.
- Serving replay, fixed prompt `[3, 5, 8, 13]`, synthetic two-layer width-32 FBS model: all
  `64 / 64` lane transitions and `256 / 256` raw transport words equal the shared Q31 oracle; all
  `192 / 192` carry-boundary words continue exactly from the preceding step. At least one emitted
  token/readout differs from the same model without the attachment.
- Exact restore: after a two-token prefix, a fresh session on the same attached model restored
  from `UR4SESS1` and the uninterrupted session produced identical two-step tokens, Q12 queries
  and native-Q31 traces. A model without the attachment and a model with a different Q31 state
  both refused the snapshot.
- Refusal atomicity: an incompatible snap/Q31 trace combination and an out-of-domain restored
  carry both returned errors without changing the pre-step serialized session.
- No language, capability, speed or memory result is measured.

## Checks run at the head

- `cargo test --locked -p uor-r4-native-vector` - 14 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm q31_serving` - 6 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib` - 649 passed, 1 ignored.
- `cargo check --locked -p uor-r4-integer --features serving-wasm` - passed.
- `cargo check --locked -p uor-r4-integer --features serving-wasm --lib
  --target wasm32-unknown-unknown` - passed.
- CUDA integration test and release example listed above - 4 tests passed; exact gate/epoch/loss.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --test
  m7_parity_library_cli_browser --test session_codec_interop_fixture` - 4 passed.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib` - exit 0;
  zero diagnostics in the three touched Rust files.
- `uor-harness check --only harness.native_space_contract` - 1 rule, 0 failed.
- `cargo fmt --check` by touched file with child traversal disabled, `git diff --check`, and the
  ORDER/REPORT checks - run at the final head.
- NOT PASS, instrument not FIT: full-crate integration stops on absent external fixtures and the
  Linux static auditor lacks its macOS call-graph tool.

## Skills used

- `working-discipline`: revalidated the merged head, isolated the worktree and recorded every
  failed or unavailable check.
- `uor-research-architect`: kept the deliverable finite, source-bound and separate from a model
  quality claim.
- `native-space-cycle`: required one state, one shared typed operator and raw-word parity.
- `solution-integrity`: identified the absent session call site and rejected a diagnostic-only
  adapter.
- `scope-guard`: caught and reverted formatter spill outside the ORDER.
- `test-freeze`: ensured no existing test was changed or weakened.
- `context-guard`: this checkout has no readable context meter, so each commit is treated as the
  conservative note/check seam.

## Still open

- The 48-byte Q31 state is attached at runtime; `UORLUT01` does not yet package it as an artifact
  section.
- The full-crate missing-fixture tests, wasm32 all-target timing binaries and Linux static
  auditor remain separate instrument/environment work.
- Protected merge queue, fresh-main verification and branch/worktree cleanup follow this report.

## Plain summary

The CUDA trainer writes one small quaternion model state as 48 exact bytes. Before this change,
the integer server could read those bytes, but the normal token engine never used them. This
change attaches the state to the model and makes the recurrence call the same Hamilton transition
that training used. Tests compare the raw 64-bit words, not rounded decimals, and show that the
state changes generation and survives an exact session restore. The result proves a finite
training-to-serving wiring path, not language ability or release readiness.
