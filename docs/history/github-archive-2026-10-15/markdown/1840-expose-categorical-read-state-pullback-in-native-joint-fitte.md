# #1840 — Expose categorical read-state pullback in native joint fitter

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T11:14:19Z
- **Updated:** 2026-10-09T13:17:20Z
- **Closed:** 2026-10-07T11:48:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1840
- **Merged:** 2026-10-07T11:48:48Z
- **Merge commit:** 5d771817952865ac7a864168f62d7547a869ef55
- **Base / Head:** main / UOR-Foundation:codex/geometric-categorical-joint-fit-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native geometric joint fitter could use only the legacy read-state pullback, so the categorical bank option merged in #1837 could not be tested in a matched full fit. Add the explicit serialized `read_state_pullback` option (`legacy` or `categorical`), defaulting to legacy for existing configs. Each categorical batch authenticates the frozen map against its expected digest while rebuilding current Context/Potential/Generate preparation.

Both arms keep selector credit, the same native forward path, optimizer groups, clipping, rates, projections, data ordering and independently reloaded native own-prefix evaluation. Receipts identify the option throughout admission, training, checkpoints and final/failure results. Old sealed zero-update baseline reuse remains limited to identical initialized native artifacts and masters; each arm still executes its own graph/native admission.

Validation at `90e3e27aea125a7086e7b9ad3e51eb8a73d22bc3`: independent source and prospective-config reviews READY; changed-file Rustfmt and diff checks PASS. Release CUDA-feature package check PASS on two pod CPU threads with GPUs masked. Eight total focused tests and the release CUDA-feature build PASS with GPUs masked. Actual legacy/categorical CUDA admissions PASS on one RTX5090/CUDA12.8:119positions per arm, native forward exactness, fresh512-row integer evaluation per arm, and independently verified554-file seals. Exact-head execution/self-review receipts are posted on the PR. Whole-repository `cargo fmt --check` was executed and reports inherited formatting differences solely in untouched main files; it is not reported as PASS.

This is an offline learning option, not a fit-improvement or language capability claim. Runtime integer geometry and the frozen marker are unchanged. One purpose, one file, 80 additions/14 deletions, based directly on main; no run output is committed.

References #820.


## Timeline (5 entries; chronological: conversation comments, reviews, review comments)

### comment 6036750084 — 2026-10-07T11:14:55Z — Casey-allard

Independent exact-head source review: **READY** at `90e3e27aea125a7086e7b9ad3e51eb8a73d22bc3`, based directly on `b5f1750156b899a68d5676ac0ee0bd15d64a8f5d`.

Verified the commit changes only `crates/uor-r4-training/examples/geometric-frozen-map-fit.rs` (80 additions, 14 deletions). Its SHA256 is `1824697ddce27e5b1a0a3a3ea997847812dce787a9bae9db02e027fc6ba26c33`, identical to the independently reviewed frozen source. No blocking source defect found.

- The option defaults to legacy through a per-config field; no process-global switch is introduced.
- Each categorical batch prepares current Context/Potential/Generate snapshots and authenticates the unchanged frozen marker. Tokenizer/protocol/lanes/device admission remains appropriate as Source metadata changes.
- Both arms retain selector credit. Active parameter groups, shared clipping, fresh Adam, rates, projections, data/order, checkpoint export/reload and native own-prefix evaluation are unchanged. Frozen marker variables remain excluded from optimization and checked for mutation.
- Admission, batch, update, checkpoint and terminal/failure receipts identify the selected pullback. Authenticated old zero-update baseline reuse excludes this non-forward field while preserving initialized master/artifact identities and fresh arm-specific graph/native parity.

This is **source review only**. I did not execute Cargo checks, tests or CUDA work; those execution checks are pending/separate evidence. The new pullback is an offline conditional geometric surrogate, not a derivative of hard argmax or a guarantee of discrete-loss descent. This delivery does not establish fit improvement, attention capability, chat or generalization.


### comment 6036750431 — 2026-10-07T11:14:56Z — Casey-allard

READY: prospective configs for source 90e3e27aea125a7086e7b9ad3e51eb8a73d22bc3.

Reviewed /tmp/uor-categorical-fit-configs.py and read all eight generated JSON configs and receipt via read-only uor-pod SSH on zvfltaygb30wpz. Independently recomputed every config SHA256 against receipt.json. Receipt file SHA256: b792f8b804577e231c2dcdeb9d71767c3af51b660b0f92d38600a279f2a49874.

Every fit config differs from its authentic retained fit-{1001,1002,1003}-raw_identity-config-attempt2.json only by read_state_pullback and out. Thus all three matched pairs preserve donor checkpoint/saved_fit, categorical map, parent config, both panel/label paths, RawIdentity score credit, order seed, 8100-second estimate, 2-GiB report ceiling and sealed admission baseline. Within each pair only read_state_pullback/out differ. Output roots are distinct and prospective.

Admission configs intentionally differ from the old Clipped admission by credit=raw_identity, read_state_pullback and out. Both new admissions use RawIdentity, seed1001 and otherwise identical donor/data/budget conditions. No baseline is supplied in admission mode.

Source pins UPDATES=128 and BATCH=8. They are source constants rather than missing JSON keys. Training constructs the declared CUDA pullback; final development evaluation reloads the direct integer native path. Do not describe this as D19 sessions or all-CUDA evaluation. The configuration-maker receipt's same_executable_for_all_arms is a prospective statement; bind actual executable identity in run receipts before claiming it was executed.

Input/label identities and donor/baseline were authenticated in the preceding data audit. Training=development on the exposed retained512panel. Order1001/1002/1003 are data-order replicas, not independent pretrained lineages. No model execution/build/GPU/lease/lifecycle action occurred in this review.


### comment 6036816766 — 2026-10-07T11:19:11Z — Casey-allard

Exact-head execution checks at `90e3e27aea125a7086e7b9ad3e51eb8a73d22bc3`, clean committed source archive on Runpod Linux x86_64 (`zvfltaygb30wpz`), own shared target, two Cargo/Rayon threads and `CUDA_VISIBLE_DEVICES=""`:

- Changed-file Rustfmt and `git diff --check`: PASS.
- `cargo check --release -p uor-r4-training --features cuda --lib --example geometric-frozen-map-fit`: PASS, exit0,57.75s.
- `cargo test --release -p uor-r4-training --features cuda --example geometric-frozen-map-fit -- --nocapture`: all6focused tests PASS, exit0,327.70s including build. Covers option default/serialization/rejection, authenticated baseline compatibility/rejection, fixed schedule, phase weights and clipping.
- Release CUDA-feature example build: PASS, exit0,9.04s. Largest measured child physical maxRSS3,344,588KiB.

Receipt: `/workspace/uor-r4/codex/categorical-joint-fit/build-90e3e27a-attempt3/receipt.json`. Whole-repository `cargo fmt --check` was also executed; it FAILS for formatting in unchanged files inherited byte-identically from main (six addressed-attention binaries, mqar-bench.rs and relation_compiler.rs). The changed fitter passes on the pod formatter. No unrelated formatting is included in this PR; no claim of whole-repo PASS.

CUDA execution is NOT_RUN yet; these checks compiled CUDA code with the GPU masked. Both actual legacy/categorical admissions must pass before queueing. A compatible sm89 build from this same head is being prepared on spare CPU before any 4090 rental because5090 stock is unavailable.


### comment 6037242191 — 2026-10-07T11:47:02Z — Casey-allard

Independent audit of exact-head CUDA execution: **READY** for `90e3e27aea125a7086e7b9ad3e51eb8a73d22bc3`.

Audited both completed, sealed zero-update admission attempts under `/workspace/uor-r4/codex/categorical-joint-fit/`: `admission-legacy-attempt1` and `admission-categorical-attempt1`. The existing Rust `verify-report` integrity CLI returned `VERIFIED` for both, with `CUDA_VISIBLE_DEVICES=""`; this integrity check returns before model/CUDA initialization. Independently checked each complete manifest file set and byte lengths (554 files per arm).

Both reports record admission mode, zero updates and RawIdentity credit, with their respective explicit legacy/categorical option. Both actual graph/native admissions cover the same 119 positions with phases `[8,79,32]`, identical row indices and weighted native loss `7.275362825509017`, independent native parity and bit-identical Clipped/RawIdentity forward loss. The zero-admission objects agree exactly after excluding only the pullback field.

Admission source identity is the reviewed head. Initial active-master identities, original bridge masters, marker masters, tokenizer-bound categorical receipt, data/config receipts and checkpoint identities agree across arms. Independently SHA256-compared every initialized Source/native/Generate-master/original-bridge/cue/prefix tree and the Generate/quarter/categorical artifacts; all are byte-identical across arms. The frozen categorical digest remains `af5039e74c6cd92e2fa0bf4c72c315ef34f912d77b09fc607215a18f86cd1d0e`.

Each arm freshly evaluated all 512 native rows rather than reusing a baseline. The ordered evaluation receipts and initial metrics agree, including 6,664 canonical target positions. Independently checked 512 unique ordered legacy row IDs and equality with available categorical row bytes during completion; the terminal complete evaluation receipts match. Initialization still has zero correct entry tokens and no correctly reached source-dependent own-prefix continuation; this is admission evidence, not improved capability.

Terminal report SHA256:

- Legacy: `dcc5e1a40ba2e5f4aa4d4eb59a0b2353b6541e012ce851580580bdf94e59d8a7`; manifest `20e07dd4f9707705c01eb5186688d6181dd9e29039ae9484839dad69b6bd1ba5`.
- Categorical: `401e261d901018b6803e9625b415019736a5d62d020396bf26fac9d338cab784`; manifest `4b20c8289bd72c4051c5abb05c42f886034adee70456355554eba368e69e93e3`.

Reported execution elapsed is 247.8325 seconds (legacy) and 229.6422 seconds (categorical), excluding wrapper overhead. This is an independent audit of the lab's execution, **not an independent replication**. I ran no training, inference or GPU work. No learning, chat, generalization or discrete-loss-descent guarantee follows from these zero-update checks.


### comment 6037256453 — 2026-10-07T11:47:58Z — Casey-allard

Exact-head self-review: **READY to merge** at `90e3e27aea125a7086e7b9ad3e51eb8a73d22bc3` (single fitter file80+/14-, directly based on main).

Checked default compatibility, per-batch fresh snapshots/bindings, frozen-map authentication and optimizer exclusion, unchanged selector/optimizer/rates/projection/data/evaluator, receipt propagation and authenticated old native-baseline reuse. Independent exact-head source/config review and independent audit of the actual executions are posted above; no blocking scoped defect remains.

Executed checks:

- Changed-file Rustfmt and diffcheck PASS; release CUDA-feature touched-package check PASS.
- Release fitter tests PASS:8total (6fitter contracts plus2imported output-support contracts), GPUs masked. Release binary built and hash-bound before GPU rental.
- Actual CUDA legacy and categorical admissions PASS on one RTX5090/CUDA12.8, the same binarySHA `2812c337d24a982aaf14fa1a41ab2c1bdb42d04db3adf0c91a63cff126fe1010`. Both exit0, each119positions `[8,79,32]`, bit-identical native forward results and finite/device-admitted gradients. Wrapper elapsed259.95/240.34s; largest child physicalRSS1,243,296KiB.
- Each arm freshly reloads/evaluates all512native rows; initialized artifacts/masters and ordered native outputs agree. Both completed554-file seals independently VERIFIED.

Whole-workspace `cargo fmt --check` was run and FAILS for inherited formatting in untouched main files. The sole changed fitter passes the same pod formatter; no whole-workspace PASS is asserted and this PR does not include unrelated formatting.

The runtime forward path/map is unchanged; this is an offline conditional learning surrogate. Zero-update checks do not establish useful attention/chat, fit improvement, generalization or descent of the discrete objective. The next matched fits remain prospective.



---
_Comment/review ids in this file: 6036750084, 6036750431, 6036816766, 6037242191, 6037256453_

