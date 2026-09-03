# Independent source and result review — #1082

**Disposition: accept `TOKEN_EXPOSURE_DESCRIPTIVE_COMPLETE` within the frozen construction-only scope.** The result supports strongly role-dependent attended exposure. It does not support substantial pooling cancellation as a common explanation for retained answers. Large measured role displacements coexist with historically retained answers, but the diagnostic cannot identify the downstream mechanism that retained them. Preserve #1079's `LANGUAGE_R4_PRESERVED_CONTROL_WEAK` terminal.

The next useful step is the bounded #1085 interface specification, beginning with externally supplied clause segmentation. No further model experiment or downstream diagnosis is required to choose that specification work.

## Reviewed evidence and independent checks

Admission: [frozen #1082 run/replay contract](https://github.com/UOR-Foundation/uor-r4/issues/1082#issuecomment-5520228872).

Local evidence root: `/Users/casey.allard/uor-r4/.uor-models/research/issue-1082-token-exposure`.

| Bound object | Verified CID |
| --- | --- |
| Preparation | `blake3:c8a4a56de77767cbbe8fd31edc83251b42a6f729e182d065caa47d981f248741` |
| Implementation, 316 files | `blake3:1451084b66531dbe5eb0cb1d3b3e60e5f0aa3f4bd59fffbf582a18d4dfc9380e` |
| Result | `blake3:e88501f05d4c58249806b2c9c5dabddd84eecd71256de9d4001c378cf8b9be03` |
| Evidence shared by run/replay | `blake3:fcd2a0c740dc23d0dc89066fef70683e2eace0e308e05020947c38fde996d4e4` |
| Replay | `blake3:3629ea8327a3e3cfc3a35d17da7d78de81ffe95f65b53266f18fac865a8240cc` |

Independent read-only validation recomputed the three canonical JSON envelope CIDs, evidence CID, every one of the 316 bound source-file CIDs and the implementation tree CID. It verified the preparation/result/replay/runtime links and distinct process IDs, 38020 and 38088. Both raw `[10240,14,7]` little-endian f64 metric files have the recorded 8,028,160-byte lengths and matching BLAKE3 CIDs. All raw values are finite; triangle bounds and stored D/A values hold. Recombining supported/unknown summary means agrees with raw all-row means within `1e-12(1+abs(mean))`. Both views have zero exact-zero M or A cases.

The recorded execution reproduces all eight historical checks: reader attention, coherent roles, controlled roles, and actual changed-matrix count for each view. The reviewed source rejects a batch if its reconstructed f32 pools differ from the unchanged #1079 helper; the replay rejects any raw metric-byte or canonical summary-evidence difference. The successful fresh-process replay is corroborated by its envelope and external exit receipt. This independent review did **not** rerun a model; historical tensor reconstruction and replay execution remain results of those two admitted processes, whose records and source were checked.

The frozen diagnostic source uses original f32 softmax weights promoted to f64; it does not renormalize A or D. It compares actual matrices, excludes padding, measures the fourteen consumed roles, preserves true local encoding, and shifts only the transport source. Recorded answer masks enter after measurement. Source hooks reject downstream core/head forwards; before/after core and reader state CIDs match. There are no new answer predictions, fits, development tensor evaluations, controls, or geometry changes.

Run/replay took 15.617840 / 15.541262 seconds internally and 16.785677 / 16.873878 seconds externally; both exited zero without the 120-second timeout. Maximum reported RSS was 1,061,994,496 bytes, below 3 GiB. Output at independent inspection was 26,336,012 bytes, below 256 MiB. Detailed read-only check output is in [the independent verification receipt](r4_token_exposure_1082_review.json).

## What the distributions establish

The table reports **ranges of per-role means across the four fact positions**, restricted to all supported rows. M fraction is dimensionless; D uses original embedding-coordinate units. The query rows report the owner/object range. No threshold or grouping was fitted.

| View and used roles | Mean changed-attention fraction | Mean net D | Mean D/A |
| --- | ---: | ---: | ---: |
| 0: fact owner | 0.9999935–0.9999940 | 0.45954–0.46045 | 0.999915–0.999917 |
| 0: fact object | 0.9999780–0.9999790 | 0.45737–0.45821 | 0.9999994 |
| 0: fact location | 0.00003455–0.00003578 | 0.00001424–0.00001468 | 0.70997–0.71302 |
| 1: fact owner | 0.99999925–0.99999929 | 0.64988–0.65117 | 0.999926–0.999929 |
| 1: fact object | 0.00003861–0.00003996 | 0.00002404–0.00002511 | 0.59262–0.60009 |
| 1: fact location | 0.9999765–0.9999775 | 1.20819–1.21059 | 0.9999968 |
| Both: query owner/object | 0.9999847–0.9999959 | 0.45748–0.46021 | 0.999942–0.999973 |

1. **Exposure is nearly absent for particular role families, not globally absent.** View 0's fact-location role receives only about 0.0035% changed-matrix attention; view 1's fact-object role receives about 0.0039%. Their D values are tiny relative to coherent role norms, approximately 1.21 and 0.458 respectively. Other used roles receive almost all attention on changed matrices and have large D. Counting changed matrices or rows alone therefore does not establish comparable intervention across roles or renderings. These are near-zero, not exact-zero observations.

2. **Cancellation is material only within those already tiny displacements.** Their D/A means of about 0.71 and 0.60 describe partial cancellation. Strongly exposed role families retain at least approximately 0.9999 of individual displacement in their mean ratios. This weighs against broad pooling cancellation as the common account. It does not measure later normalization, projection, binding attention, logits, or decision margins.

3. **Retained answers coexist with substantial operational displacement.** Supported answers changed/retained in 2,353/5,839 cases in view 0 and 4,224/3,968 in view 1. Those are the historical 28.7231% and 51.5625% changes, not new predictions. In retained rows, mean fact-owner D remains 0.45988–0.46047 in view 0 and 0.64941–0.65084 in view 1; view 1 fact-location D remains 1.20770–1.21158. Changed-answer rows have similar scales. The retained/changed grouping is descriptive outcome conditioning and cannot identify why an answer survived or changed. Unknown rows retain 84/2,048 and 241/2,048 answers; these likewise do not become new generalization or abstention qualification.

4. **Rounding does not explain the orders-of-magnitude separation.** The maximum absolute difference between measured f32 used-pool displacement and f64 D is `2.3163e-8` / `2.8445e-8`; reported f64 closure errors are `1.6100e-16` / `2.7756e-16`. These are observed numerical checks, not a floating-point refinement proof.

## One bounded successor for #1085

Support [#1085's planned interface work](https://github.com/UOR-Foundation/uor-r4/issues/1085): specify how an ordinary text context becomes the fixed four fact clauses plus question accepted by the preserved learned reference. The first limitation to remove is **externally supplied clause segmentation**. Keep the accepted core/reader, tokenizer/vocabulary, known entity lexicon, query form, fact count, output semantics, and current R4/control verdict fixed. Explicitly define accepted syntax, boundaries, malformed/ambiguous-input refusal, context bounds, and the adapter's output schema; it must not manufacture role labels or answers.

A later, separately frozen child can compare the adapter with supplied segmentation on an independently authored, partitioned set that varies only clause-boundary surface form. Predeclare boundary correctness and end-to-end agreement with the supplied-segmentation reference, including typed refusal cases. A positive result admits this bounded raw-text input path; a negative result preserves the supplied-segmentation requirement and directs repair to the adapter boundary. Freeze metric floors, partition policy, and cost before evaluating; this review does not choose them from #1082 outcomes or authorize fitting, population generation, or evaluation.

The role-selective result does not establish general language/context transfer and does not force a new downstream mechanism experiment before writing this contract. Pursuing a causal explanation of retained answers would be a separate research question requiring a separately justified intervention; it should not replace the immediate interface definition or trigger control tuning. #973 remains open and #954 blocked.
