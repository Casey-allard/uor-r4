# #1613 — Connect geometric NoRead to native attention and Q24 reduction

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T11:28:20Z
- **Updated:** 2026-10-09T13:17:27Z
- **Closed:** 2026-10-02T11:39:21Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1613
- **Merged:** 2026-10-02T11:39:21Z
- **Merge commit:** 1fc30e9c13387c7a88bfd8c787e93210f6d64773
- **Base / Head:** main / UOR-Foundation:codex/geometric-no-read-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native geometric reader still obtained its NoRead score from a floating hidden-state map. This change adds a learned token/retained-H4/category/held-state scalar with packed signed four-bit coefficients, a source-bound compiler/reloader, and an explicit raw-Q24 branch before legacy read normalization. Offline answer loss reaches its hard-forward shadow coefficients through the shared occurrence/null denominator. The served scalar selects derived tables and adds integers; the surrounding model still has unresolved float and coefficient-width boundaries.

Validation: four integer cases, nine training cases across preserved attempts and the focused repair, raw-Q24 precision and actual StackModel integration pass. Both actual saved query-credit parents complete zero-update construction comparisons on512 original/stress rows: source/native answers99/104 and128/128, with no source/native prediction differences and unchanged legacy parent predictions. ARM64 release score inspection finds no multiply/divide/float or allocator call in the scalar symbol; this is not a whole-model certification. Formatting and claim-wording checks pass. Test-only private-helper and F64/F32 failures are preserved with focused fixes.

These are connected construction/compilation results, not learned attention, semantic absence or language qualification. Next is a separately declared answer-only NoRead fit on frozen producers; fixed query-token/validity collinearity requires geometric attribution to be measured separately. Claude's GroundedSession files are untouched.

References #1512, #973.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5951545518 — 2026-10-02T11:38:27Z — Casey-allard

# Independent final delivery review — geometric NoRead

**Verdict: APPROVED for protected delivery of exact head `c77bf3ba19ff9737ffbc241486f29c93345ba8fe` against base `c43fe076e864c8e8e05106d80d24195359c2173a`. No material source, arithmetic, artifact-binding or reporting blocker remains in the reviewed scope.**

Reviewer: `/root/principal_mathematician`, a nonauthor of these twelve changes. This review read source, committed blobs, preserved execution receipts/logs, the saved scalar assembly and all 512 comparison rows. It ran no Cargo, training, model inference or new numerical campaign and made no source/Git changes. The only new artifact is this exclusive review note. CI/protected merge status is separate from this approval.

## Exact source and narrow repairs

The worktree was clean at review and all twelve files below match their committed blobs. The evidence receipt is SHA256 `f5a9b32729a991e5ef47863a1b3a76c157f8f42cdf7a0aefff6cf6d98375a48f`. Its eight source byte lengths and SHA256 values match both working files and committed bytes. Every embedded execution JSON and both embedded comparison reports exactly match their retained original JSON.

The prior static review remains `cbd6d84ad6aa14690cc5d8540a69d94ee98a3104b2b4e6fe49232d4fec700f68`. The integer numerical kernel, compiler/admission helper and raw-Q24 reducer are unchanged from that review. I reversed the two documented fixture-only corrections in memory: the learner `0f32` annotation exactly reconstructs the prior `a03f41ff6d97ffaf6683fb6047bcf9e507d725fffec29e335bfe628518174359` hash; replacing the checked public `rows.get` fixture access with the former private `row` calls exactly reconstructs `ba5ba6e2addbd898024360299dc9183e7552d82240bacecfd9dc2b86c25d4d0f`. Thus those execution-driven repairs did not change production arithmetic.

The final example retains complete actual-position null traces and bounded episode/query handling. Reversing only the final console correction (`let verified = report_output::verify(...)` plus its former count print) exactly reconstructs executed example SHA256 `304db44b23c4433ce0865625a74ceac2d38fa5f6d8e2e1e2de77b865265b6d45`. The final example SHA256 is `0151b71042501f02678a6d4dd70f120a499149512112d54cc1704c2fe894f446`; its second release build succeeded. The misleading historical “verified 0 files” console line referred to the verifier's empty unlisted-file result. It did not indicate an empty report or skipped verification. Actual comparisons are not represented as rerun on that logging-only rebuild.

The prior mathematical approval therefore carries forward: bounded signed q4 coefficients at fixed quarter-nat scale, artifact-pinned Q25 observation basis, signed ties-away per-root Q24 lowering, all actual retained lanes and typed old-held validity, null in the shared denominator with zero payload, raw-Q24 preservation and strict source/dependency regeneration. The learned STE is a declared biased surrogate, not a derivative of discrete rounding. Scalar factor rounding and the F32/real-softmax reference remain distinct from native LUT/Q24/Q16 arithmetic.

## Executed checks and preserved failures

The retained logs show four passing integer cases; nine distinct training cases pass when combining the eight successes from training attempt 2 with the single repaired gradient case; the raw-Q24 and real integrated caller cases each pass. This is 15 unique focused cases, not a count of repeated runs. The nine include two retained reader/normalization cases rather than nine entirely new tests.

Training attempt 1 failed compilation because its fixture called a private trace accessor; no tests ran. Attempt 2 had eight passes and one F64/F32 fixture mismatch. Both failed logs remain preserved. The named gradient repair then passed with the explicit F32 fixture. The raw-bit test and integrated delayed-answer/typed-reload fixture passed on the final production sources. Both example builds and both no-fit comparison commands terminated successfully. Format logs are retained. These are observed logs/receipts, not execution performed by this reviewer.

The integrated source fixture tests the consequential boundaries: actual answer credit, bypass of legacy null/normalization/value maps using poisoned-and-restored weights, remaining read-output dependency, causal prefix behavior and compiled reload. No additional blanket suite or proof campaign is required by an unresolved risk here.

## Independent saved-row recount

| Seed/panel | Rows | Legacy | Source | Native | Native gains/losses versus legacy |
|---|---:|---:|---:|---:|---|
| 1 original | 128 | 98 | 99 | 99 | gains 72,102; loss 113 |
| 1 stress | 128 | 104 | 104 | 104 | none |
| 2 original | 128 | 128 | 128 | 128 | none |
| 2 stress | 128 | 128 | 128 | 128 | none |

All 512 episode identities equal the retained input and parent row records; their input/parent-row SHA256 bindings match the reports. The legacy prediction exactly reproduces the saved parent in all 512 rows. Source and native predictions agree in all 512 rows. I independently recomputed answer argmax from each retained 40-logit answer vector, counts, all changed predictions, gains and losses. The three changed rows are seed-1 original: 72 (19→23, answer23), 102 (18→28, answer28), 113 (30→22, answer30).

Both reports are complete with zero optimizer updates and all-zero packed coefficients. Every retained source null value and native Q24 null value is zero at every actual position. Each query trace has exactly two heads, causal occurrence count query+1 and sixteen payload coordinates; each positive denominator equals the sum of its occurrence weights plus its NoRead weight. All recorded weights lie in [0,2^31]. This checks the null contribution without confusing unnormalized Q31 weights with probabilities. Both actual-parent backward receipts have positive finite norms in all six parameter families for both heads. They concern one actual episode per seed, not a general gradient or learning guarantee.

The maximum saved answer-position source/native logit differences are approximately 5.62668e-5, 8.46386e-5, 5.34058e-5 and 3.71933e-5 in table order. The separately reported all-actual-position maximum is 0.00019490718841552734. The source computes that maximum over all returned actual-position logits; only answer vectors are retained rowwise, so this review distinguishes recomputed answer maxima from the source/receipt-bound full-position maximum. No global scalar bit-parity or trained nonzero-coefficient retention is inferred from the zero-coefficient comparison.

Report/manifest SHA256 identities and all comparison-row identities match the delivery receipt. Complete sealed file inventories and byte lengths match their manifests. The actual Rust verifier's successful return is preserved execution evidence; this review did not rerun it or independently recompute BLAKE3 because that Python module is unavailable. SHA256 checks on reports/manifests/rows and actual executed verifier evidence are distinguished.

## Scoped release instruction evidence

The decompressed retained construction executable matches SHA256 `e6397705def1e0c425b0c7c4223876f801ac69cff7356e3acba9a59b35f3eefd`. Retained assembly matches SHA256 `379c7bd5117e1d69e5c720d73a88beaac9f30ed627dd6da3536203fbd6d67c20`. I independently parsed 255 instructions for the exact `NativeGeometricNoRead::score` symbol. No multiply/madd/msub/divide, floating-point or dot instruction appears. The five direct calls are all `panic_bounds_check`; no allocator call appears. The source admission/private table invariants and typed valid roots/categories explain the scoped bounds-path statement.

This supports only that saved release scalar symbol. It does not certify admission, every external helper, the remaining model, total memory traffic, energy, or any unrelated coefficient family. The final logging rebuild is not claimed as a new independent instruction audit.

## Interpretation and next action

The four changed documentation/receipt files accurately report a connected trainable/native NoRead component and construction results. A zero-update change from 98 to99 is not evidence that this scalar has learned quality, abstention or a geometric advantage. The explicit remaining boundaries—wide coefficients elsewhere, float trunk/residual/read.out/output, full causal candidate access, semantic absence, language and energy—are material and retained.

The proposed next fixed answer-only scalar fit is a coherent next learning question with frozen producers. In this grammar query token34 and always-present held spans make bias/token/validity features collinear; answer restoration alone cannot identify history-conditioned geometric use. A copied-scalar feature-family intervention can isolate this later more narrowly than resetting context, which would also change address/value behavior. No additional auxiliary sweep, geometry retirement or new model computation follows from this approval.

## Committed file bindings

| Path | Git blob | SHA256 |
|---|---|---|
| `crates/uor-r4-integer/src/geometric_no_read.rs` | `c38f88ce64c8e400667925bc93fa797501f050c3` | `09d589fb212a7ce20c77ba69a7c23433c10c2a2b396e15ce4c6cb8c3f4d5e0c3` |
| `crates/uor-r4-integer/src/lib.rs` | `735429c4085b574525a0acc7d09421d7c76fa99f` | `e3eb7d7065eb13dff2311deea310b44816fecb9469dbd3ed75fa406fc9d6415d` |
| `crates/uor-r4-training/examples/attention-geometric-no-read.rs` | `cd75438f068960811cfbede35a386d9d826b570e` | `0151b71042501f02678a6d4dd70f120a499149512112d54cc1704c2fe894f446` |
| `crates/uor-r4-training/src/geometric_no_read.rs` | `a477872bf6c35d1e45ef94b61b3c285cddc1519a` | `11455af6551061531381ef8fb897cb06db32e723c2723d3a5af2c56f1918b93c` |
| `crates/uor-r4-training/src/geometric_no_read_native.rs` | `bbf8a6e0ea961a21d9ccb31781a51a2c1a1978b2` | `3b5d706c9481d724a0239540e388558a834a0ecdde5b43f155d2c57a5b480b8c` |
| `crates/uor-r4-training/src/geometric_read_native.rs` | `caa1908073ee9db2a2bb1d9a070b53eb27ed5471` | `43625d67b644e01e39eca370c8691170a43fd0796d62ad7e58a5c36fb9180d4d` |
| `crates/uor-r4-training/src/geometric_stack.rs` | `62a9c67a128262483637cb56947b6c5fc38e2044` | `a189af33138edd20d27dca716635bc0f192f31b7dbc22ddf1a6cb12fcf624d2f` |
| `crates/uor-r4-training/src/lib.rs` | `14bedda973bb72cacdafef5d8c83bd90eb2e9aff` | `181c15667c4257b05a936e00f2949349270bf36fbec543c10c8104f458d95f33` |
| `docs/evidence/geometric-no-read-2026-10-02.json` | `9572c0e1629517d72068aacfbb9b3e083805d86b` | `f5a9b32729a991e5ef47863a1b3a76c157f8f42cdf7a0aefff6cf6d98375a48f` |
| `docs/integration/current-state.md` | `9a438efd9828da7b464ddd0dcdbf8cbce72ee4af` | `19a2bc9c397cd48df1ee67452eedf5c526747b2f2c26a096355046ee4a873f88` |
| `docs/integration/geometric-no-read-2026-10-02.md` | `c7a02cb04560ae0d91c0ce30b3f48d1ff74826d3` | `49db8e2f06298135daedcb6cd8780e549570c510d51233531714ffb09bb7b2b1` |
| `docs/integration/project-track.md` | `d4052d9253742634e42e083241b3223a3d0c8b19` | `a91df07686ca49829caa573cb89752e28bbd4b0c424276badd524a1571338eac` |

## Retained log bindings

| Log | SHA256 |
|---|---|
| `integer-tests-1.log` | `7cb936a7cfd5391de9a35de21e6cecd623dda8c7af7e8fa92d624c6906071807` |
| `training-tests-1.log` | `ac1044bebdcf0b560d0ed42ed27916fa7dffc54c56f5ade4be02f85f85eda08f` |
| `training-tests-2.log` | `62e10abdbf1f7ba704e5c9a6bb52b7b5294c0da964ca642fd50a848aca7469ff` |
| `gradient-test-3.log` | `a0d21574399c27233f5ce12678caa76cb737e4e813105655706658c00cb96e72` |
| `raw-test-1.log` | `a4ed7726f137a6621c06da7d6e84029dbd5e0ee6a7e55e1284e3d3e10ef0582e` |
| `integrated-test-1.log` | `f7b6b239d5274d162178518ae0622ac1e5522cbb4a1089a2a2e02b04c3721e82` |
| `example-build-1.log` | `52c078bad8031c4f0ba713273dd72bbd7863223e07cdf83a3dfcc11af9e4fc13` |
| `example-build-2.log` | `584c0631469aa080a5434bfcf23fddcce8ed548c966d009a850ea205ef8ac6cd` |
| `compare-1-s1.log` | `a6bb4f319f75d43bb6ebe681bc544c65bb251d69beb610536580feae8d0666dc` |
| `compare-1-s2.log` | `6990b1cf3c482154295776b26a07c46ca29812ae656edb5101a2934d4898e310` |
| `fmt-check-1.log` | `0eac9da5cc44c9769d43a4595738dd2e090a66d572d922dea01eec955853430e` |
| `fmt-check-2.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |



---
_Comment/review ids in this file: 5951545518_

