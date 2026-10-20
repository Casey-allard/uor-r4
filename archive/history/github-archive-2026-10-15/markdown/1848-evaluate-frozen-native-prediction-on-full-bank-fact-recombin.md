# #1848 — Evaluate frozen native prediction on full-bank fact recombinations

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T17:22:05Z
- **Updated:** 2026-10-09T13:17:42Z
- **Closed:** 2026-10-07T17:40:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1848
- **Merged:** 2026-10-07T17:40:12Z
- **Merge commit:** 702f5279fa9d2000612ba3d3d6ff2b2a36fc1b90
- **Base / Head:** main / UOR-Foundation:codex/native-frozen-transfer-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The successful native Source48/Generate64 export needs a frozen prediction test on facts outside its eight winning construction cases. This adds two core Rust examples: one prepares whole-string fact recombinations with exposure and answerability audits; the other runs the production `NativeBankGenerator` with every Source occurrence, all 4,096 legal Generate tokens, actual emitted feedback and normal EOS. Answers remain in separate evaluator files.

The evaluator authenticates the winning artifact and first checks original-eight wrapper parity against its saved outputs and native traces. The prospective 32-case panel crosses four value banks, role assignment and physical record order under two supported queries. It excludes configured complete-value/pair exposure, reports substring and BPE-length overlap, and uses a typed reference that reads the actual question and both facts. This is one frozen development discriminator, not general chat or unseen-word qualification.

The roadmap/current state record the executed 8/8 construction win, its label-informed component selection and the next frozen transfer decision. GroundedSession, model kernels and training behavior are unchanged.

References #820.

Validation at exact head 3c36bc2a2c34346fc47f3b3a379ddb20c1afec4f: touched Rust formatting, diff and claim-wording checks passed; uor-r4-core release example check, all 10 focused tests (zero ignored) and both release builds passed on two spare CPU threads of an existing Linux pod. Full-workspace formatting reports pre-existing differences in unchanged main files; the two changed Rust files pass. Independent exact-source review and the executed check receipt are linked in the self-review comment. No CUDA kernels changed, and no GPU was rented for these checks.

Original-eight wrapper replay and the fresh 32-case model transfer remain NOT_RUN until the authenticated frozen model/data are available. The tools/checks are preserved in the private HF store at commit 3e4add5101fdea45c4cf1f1a653aaa2f3cd90934, independently downloaded and SHA-256 verified. That bundle contains no model checkpoint.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6043140741 — 2026-10-07T17:24:53Z — Casey-allard

Exact-source independent review READY at head3c36bc2a2c34346fc47f3b3a379ddb20c1afec4f; evaluatorSHA2a21d5f..., preparerSHAb0c1702a... . Label flow, full4096 pool, occurrence-preserving snapshot pins, original8 native trace parity, configured exposure limits, actual-query/both-facts reference and output ancestry reviewed. No model result inferred. Attempt1: touched formatting PASS; whole-workspace formatting FAIL in inherited files; package example check stopped before compilation because cold pod cache lacked bit-set0.8.0 under --offline. This is retained setup evidence. Attempt2 permits a locked selected-package dependency fetch for that check, then returns to offline tests/build; same source and2CPU threads, no GPU use/new rental. Actual frozen wrapper replay/32 transfer remain NOT_RUN until canonical model/input volume is accessible.

### comment 6043388718 — 2026-10-07T17:38:48Z — Casey-allard

Exact-head self-review: `3c36bc2a2c34346fc47f3b3a379ddb20c1afec4f`.

The serving call receives only the complete typed Context/Source/query packet. Answers are used after own-feedback generation. All4,096 legal Generate actions and physical Source occurrences remain available; normal EOS and the128-token context bound apply. Original-eight replay compares the saved outputs, occurrence chronology, Copy components, H4 states, Generate digest, pool summaries and bridge witnesses. The offline reference reads the actual supported question and both facts. Exposure is limited to the configured donor files, with substring overlap and changed BPE lengths reported. Four banks yield correlated interventions, not32 independent samples. Independent source review found no blocking defect.

Executed on Linux x86_64 pod wp23upwpazsjes with2 spare CPU threads, GPU use disabled, from a clean `git archive HEAD`:

- `cargo check --locked --release -p uor-r4-core --example native-bank-generalization --example native-bank-transfer-panel`: PASS,34.779s. The locked online check repaired the exact missing cold-cache dependency; original offline failure is preserved.
- `cargo test --locked --offline --release -p uor-r4-core --example native-bank-generalization --example native-bank-transfer-panel`: PASS,10 executed,0 ignored;169.366s including compilation. Six new focused tests and four reused output-path tests.
- `cargo build --locked --offline --release -p uor-r4-core --example native-bank-generalization --example native-bank-transfer-panel`: PASS,3.662s.
- Both changed Rust files pass `rustfmt --edition2021 --check`; `git diff --check` and the claim-wording gate pass.
- Whole-workspace `cargo fmt --check`: FAIL in inherited files unchanged by this PR. It is recorded rather than counted as passing.

Peak child RSS3,498,208KiB; Codex root1.9GiB; no additional pod/GPU rented for these checks. Receipt and logs: `/workspace/uor-r4/codex/frozen-native-transfer/checks-3c36bc2a2-attempt{1,2}/`.

Release executable SHA256s: evaluator `646039362811a95a2735d599a417dd0284e3fdbb405390ad0b54c4b858d969cd`; preparer `b0c17a0e7a2846eef9c111b52e20eb0f4bd617d7e530e48944aadc0bb6718f24`.

Actual original-eight wrapper replay, panel preparation and frozen32-case model transfer are NOT_RUN: the successful native artifact and original512 inputs are not yet available on this host. The newly checked code changes no model kernel or training behavior; no new language/transfer/CUDA/D11 claim is made. The existing8/8 construction result remains separately bound to #1844.

References #820; ready for the protected merge queue at this exact head.



---
_Comment/review ids in this file: 6043140741, 6043388718_

