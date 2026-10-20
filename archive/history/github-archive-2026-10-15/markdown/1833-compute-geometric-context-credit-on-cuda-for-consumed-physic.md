# #1833 — Compute geometric context credit on CUDA for consumed physical pairs

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T06:35:08Z
- **Updated:** 2026-10-09T13:17:55Z
- **Closed:** 2026-10-07T06:37:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1833
- **Merged:** 2026-10-07T06:37:40Z
- **Merge commit:** 5bc6f31ea32cb680f9eb6814051190dd485e50a7
- **Base / Head:** main / UOR-Foundation:codex/selected-context-credit-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The composed geometric learner reads the last query against the complete admitted Source bank, but its frozen-Potential context adjoint previously built utilities for every causal pair in the full sequence. This change computes that adjoint only for the physical query/Source pairs consumed by the learner and constructs the finite H4/category utilities on CUDA. It retains the existing dense CPU path as a reference.

The selected API preserves duplicate requests, shared queries, distinct occurrences with equal codes, and the special self-position derivative. All 120 H4 roots and 33 category choices remain available. Integration keeps chronological context replay, every Source candidate, native integer anchors, the existing Potential-coefficient graph, and the full Generate/Copy alias loss. It changes offline credit computation; final serving, the GroundedSession seam, candidate admission, and the six running fits are unchanged.

Host work remains for authenticating and expanding the fresh coefficient snapshot, native selected hard anchors, and code/index staging. This is not a fully resident GPU learner or a language-quality result. On the retained construction layout, the consumer requests 119,416 pairs versus 16,996,836 causal pairs per head across 6,664 teacher positions; that 142.33 ratio is a count of work requested, not a speed measurement.

Validation at `c0949cef01ecc48fc65da0a6180cedb152c7fbb5`:

- Release CUDA library check and release library test-binary build passed on Linux x86_64.
- Four focused CPU tests passed, including signed/repeated/shared-query adjoints against the existing dense backward, physical self semantics, actual composed-bank parity, and export/rebinding guards.
- Changed-file rustfmt and `git diff --check` passed. `cargo fmt --check` fails in eight files unchanged from main; those failures are retained rather than changing other labs' files.
- Three exact CUDA checks passed on a leased RTX 5090: independent dense-CPU parity, composed whole-bank state/Potential-coefficient adjoints in frozen and learned modes, and zero-update retained-artifact full-bank credit substitution. Each actually ran one test with zero ignored; the driver exited zero after 34.23 seconds. The pod was then released and deleted.
- Exact-source mathematical/adversarial review is ready; it is source review, separate from executed checks.

References #820.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6032437533 — 2026-10-07T06:36:32Z — Casey-allard

Exact-head self-review: **READY** at `c0949cef01ecc48fc65da0a6180cedb152c7fbb5`.

Reviewed the consumed-pair integration, causal physical-position validation, fresh authenticated snapshots, duplicate/shared-query accumulation, distinct-occurrence endpoint roles, and the self-position categorical/root counterfactual. Mathematical adversarial source review separately found no blocker. Every admitted Source candidate and the existing coefficient-credit graph remain in the full Generate/Copy objective. Runtime serving and running fits are not modified.

Actual checks at this head:

- `cargo check --release --locked --offline -p uor-r4-training --features cuda --lib`: exit0.
- `cargo test --release --locked --offline -p uor-r4-training --features cuda --lib --no-run`: exit0.
- Four exact CPU tests: signed/repeated/shared-query selected-versus-dense credit; self-root/joint-category credit; composed-state native alias/context credit; export/rebinding/frozen-inventory rejection. Each ran one test, passed, zero ignored.
- Three exact CUDA tests, each one test passed and zero ignored: `selected_potential_cuda_matches_independent_dense_cpu` (explicit `--ignored`); `composed_cuda_reloaded_source_full_bank_native_and_adjoint_parity` (frozen and learned Potential coefficients, both adjoint policies); `selected_context_retained_bank_cuda_zero_update` (explicit `--ignored`). GPU driver exited0 after34.23s. No CPU fallback and no optimizer update.
- Changed-file rustfmt and `git diff --check origin/main...HEAD`: exit0. Full `cargo fmt --check` fails in eight files whose blobs are unchanged from main; the failure and main comparisons are retained. No unrelated formatting changes, blanket suite or claimed Clippy result.

GPU: Linux x86_64, RTX5090, driver570.195.03, CUDA12.8 runtime, `CUDA_VISIBLE_DEVICES=0`. Executable SHA256 `36e9520f0590cf5aa90d04c85d6da8440b7fc9ea4556957374ad5d265b09469a`. Canonical check roots:

- `/workspace/uor-r4/codex/selected-context-credit/checks-c0949cef-attempt1/`
- `/workspace/uor-r4/codex/selected-context-credit/cpu-c0949cef-attempt1/`
- `/workspace/uor-r4/codex/selected-context-credit/gpu-c0949cef-attempt1/`
- Job `/workspace/uor-r4/jobs/codex/20261007T063434Z-selected-context-parity-gpu0.log`, `# exit=0`.

Retained-artifact check authenticates completed fit root `fit-1001-clipped-attempt1`, report SHA256 `49c78d15029486c07c0ffca052fe1410bb4ff3f52424227e5d844490722e3b57`, manifest `5bbf00b73c86e72150c501fcbeed0d109aa77c5f049fb97fe1581f0d56efe3b4`, and checkpoint receipt `76cfdbfc7504180f325cac2aa6250fb117529f3ab99d99a815900682847531e3`. Its producer remains the original fit source; the test executable head above is not a new fit. It checks empty/teacher-prefix contextual-credit substitution in the full bank, including native complete-pool anchors and unchanged masters. It does not establish fresh autoregressive success or complete-fit speed.

The printed forward timings are single, fixed-order diagnostics with preparation/synchronization; they are not a benchmark or end-to-end claim. The validation pod was released and deleted immediately after the terminal checks. All source/binary/receipts are on the canonical volume. Queue acknowledgements run no tests and are not counted here.


### comment 6032475251 — 2026-10-07T06:39:28Z — Casey-allard

Post-execution adversarial evidence review: READY at c0949cef01ecc48fc65da0a6180cedb152c7fbb5. Independent read of all three CUDA logs and independent binary SHA check confirms each exact test actually ran 1 passed, 0 failed, 0 ignored. Retained coverage is four conditions (rows 0/8, steps 0/3) under two policies, not eight independent cases. Full-bank test covers frozen and learned Potential coefficients. Timings are fixed-order single-shot diagnostics; the first selected forward was slower than dense, and the small fixture has no consistent gain. No robust or end-to-end speed claim. Merge 5bc6f31ea32cb680f9eb6814051190dd485e50a7 is on main, and all three code blobs match the tested head.


---
_Comment/review ids in this file: 6032437533, 6032475251_

