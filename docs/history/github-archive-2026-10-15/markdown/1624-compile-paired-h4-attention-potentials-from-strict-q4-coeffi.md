# #1624 — Compile paired-H4 attention potentials from strict q4 coefficients

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T16:22:23Z
- **Updated:** 2026-10-09T13:17:28Z
- **Closed:** 2026-10-02T16:29:20Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1624
- **Merged:** 2026-10-02T16:29:20Z
- **Merge commit:** 7f2642b124d8fe75ac26b66f6823cc8cf52632d4
- **Base / Head:** main / UOR-Foundation:codex/geometric-potential-q4-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained paired-H4 attention potential still depended on unrestricted learned coefficients. This change adds a standalone fixed q/4 signed-four-bit source for its existing seven families, independently regenerated integer score tables, strict reload/admission, and ordinary answer credit through the actual age/NoRead denominator and frozen native values/composition. Directed geometry, occurrence identity and legacy artifact admission are preserved.

Both saved-parent conversion runs finish: **478/512 correct answers versus479/512 parent**, with0gains and1loss retained. All512 source/native answers and packets agree and1,883,664 actual causal Q24 score pairs match the independently loaded native scorer. Four B8 diagnostic backwards reach all seven families with finite positive gradients; no optimizer update occurs. Context/reducer/value/NoRead/bank numerical files remain unchanged after explicit rebinding. Coefficients and the canonical-F32 Q25 observation basis change together, so this is not coefficient-only attribution.

Validation:12 focused Rust cases, release driver build, both actual saved-artifact runs, cargo fmt --check, claim-wording gate and diff check. One initial training compilation failure was repaired by explicit product::<usize>() and retained in the receipt. Static source and independent saved-result reviews are recorded. Required GitHub statuses are queue compatibility acknowledgements and execute no tests.

The result record and current state retain all parents, changed rows and complete cost. End construction without automatic potential-fit dose; next lower context transition/observation coefficients. Event/age coefficients, selected access, float trunk/output, natural inputs and full language/energy qualification remain open. Claude’s scale/Metal/session work stays concurrent; stack_grounded_session.rs is untouched.

References #1512
References #973


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5956582373 — 2026-10-02T16:22:47Z — Casey-allard

# Independent strict-potential construction result review

Status: completed read-only result review, 2026-10-02. **The reported component-construction result is supported by the retained rows.** Both workers exited0 and completed all128 original plus128 stress episodes per seed, with zero optimizer updates. No new inference, Cargo, model run, Git operation or source mutation was performed for this review. This is not independent authorship review of the integer codec, which I authored; the earlier source-integration review covers the other authors' components.

## Recounted behavior

| Seed/panel | Parent | Q4 source | Q4 native | Gains/losses | Changed predictions |
|---|---:|---:|---:|---:|---:|
|1 original|112/128|112/128|112/128|0/0|0|
|1 stress|113/128|112/128|112/128|0/1|1|
|2 original|126/128|126/128|126/128|0/0|0|
|2 stress|128/128|128/128|128/128|0/0|0|

Total native478/512 versus parent479/512; zero gains, one loss, one changed prediction. I recomputed each answer argmax and correctness from the saved40-way answer logits, joined every episode and parent answer-logit vector to the pinned preceding q4 value rows, and reproduced the reported counts. Source/native answer choices agree on all512 rows; frozen packet, source/native packet and frozen NoRead comparison flags are true throughout.

Recomputed mean answer CE (parent -> native): seed1 original0.965844027947 ->0.966068537384; seed1 stress0.930197403326 ->0.936022260943; seed2 original0.389765765096 ->0.393299652620; seed2 stress0.380356955942 ->0.380557639509. Thus all four native mean CEs worsen slightly; unchanged aggregate accuracy alone would hide this. Source/native CE differs slightly, consistently with their declared different reduction arithmetic. The reported maximum actual-position source/native logit difference is0.0002620220184326172; this is not parent/native drift.

## The single lost row

Seed1 stress row99, pair408, condition reversed_order; four facts, write/query gaps8, correct source occurrence24 and answer-query position89. Expected token22 becomes token30 in both source and native outputs.

- Head0 correct-source mass decreases from0.92311848120037 to0.9065539110965474. It remains the highest-weight occurrence before and after conversion. Runner-up occurrence66 increases from0.06935081778586363 to0.08532588586103342.
- Head0 null mass decreases0.00024400173848670255 ->0.0001530270432637842. Its new source potential is exactly13nat; source-plus-frozen-age8.686792850494385nat equals the reported row maximum. The old maximum was8.23833853006363nat.
- Head1 is mostly null:0.993680291903117 ->0.9890446751556312. Its source mass changes0.00007160521647507113 ->0.00011187780897821322; its occurrence ranking also changes while null remains dominant.
- Target22 minus competitor30 logit margin changes+0.1477036476135254 ->−0.4928734302520752 native (source−0.49285435676574707).

This is a genuine downstream mixture sensitivity under the frozen coefficient/basis conversion. It is not evidence that the correct source became unavailable or ceased ranking first in head0, and it is not a source/native hard-score discrepancy. Both head mixtures change; the retained outputs do not isolate one head as the sole cause. Because coefficient-grid conversion and F64-to-Q25 observation conversion were bundled, the loss cannot be attributed solely to coefficient width. It remains a real negative, not a reason to retire the geometric mechanism or demand perfect parent imitation.

## Learning, identities and preservation

Each seed has two B8 zero-update backward diagnostics at absolute draws3200/3201. All seven coefficient-family L2 gradients are finite and positive in all four diagnostics (28 family instances). Individual unused cells remain allowed to have zero gradient. I checked the actual episode counts/positions, reported padding and answer denominator8; the recorded causal-score pair counts equal8*T*(T+1). Gradient connectivity and cost are measured here; optimizer recovery is NOT_RUN.

Both initial training RNGs equal their reported next training RNGs exactly: seed1 `17847168693577953940`, seed2 `8892160473256393877`. Diagnostic RNGs are separate (`17052478318131460056`, `14344005333449752929`). Python integer parsing was used for these identities; no lossy JavaScript Number conversion.

For every saved evaluation row I checked source-score layout2*T*T, future-zero placeholders, causal pair countT*(T+1), finite40-way logits, and both old/new query denominators equal the sum of all occurrence numerators plus the null numerator. Actual per-pair native/source score equality is checked by the reviewed driver before recording each row. I did not reconstruct a second scoring engine from the reports.

I independently compared all17 listed frozen numerical payload files per seed across rebound context/reducer/value/NoRead/bank artifacts: exact byte equality. This complements the driver's source/model bit guards and row packet/null checks. Both cheap loaded prefixes have zero logit difference, exact prefix packet/occurrence agreement and reported float-tail bit equality for the first18 of36 tokens. That finite prefix check is not arbitrary-context qualification.

Each outer manifest lists45 files; file closure and every declared size match the actual directories (36449377 and36552437 bytes excluding manifest). The executed driver sealed and called report_output::verify before its terminal0. I independently checked closure/size and SHA256 evidence/source/executable identities; I did not separately recompute manifest BLAKE3 digests because the available Python environment lacks that module. This scope does not replace the actual driver's verification with a claimed independent BLAKE3 check.

## Compile repair and reviewed source scope

`compile-repair-1.json` preserves failed training-tests-1 E0283: ambiguous iterator product comparison. The only change was `data.len() != shape.iter().product()` to `data.len() != shape.iter().product::<usize>()`. Reversing that exact annotation in the current source reproduces the previously reviewed source hash. Before1e760b7a641c23d66c11bf42b1fe8e737d852c4b76fb6d01ce46d29dd6949a62; afterfac315eb2d0d924080a07cdf02a8afc7024810009d08d76c95127a472b89256b. It changes no arithmetic, graph, schema or fixed condition; the failed build is not model-quality evidence.

Both execution receipts bind the same executable SHA256850e7fc03d9b9415354f0520209446be96983957436456ae4bb9c8fd7eb2bc50 and ten source hashes. I recomputed those source hashes and the actual executable hash; all match, and both receipts report unchanged_source_after=true. Worker wrapper times are10.229023084015353 and12.462039749982068 seconds; maximum resident sets332857344 and255606784 bytes. They are construction/replay costs, not serving-energy measurements.

Report SHA256s: seed1 d53d8133a8990a811540007581c746e3874713308921542b03dc776d8e024c99; seed2 b34ee07f41d81b3d41ada501a8441233b978c41faa99f208a4d006984ceeec43. Manifest SHA256s: seed1 1721f2727e02d2da8922d10603c866e46afd17162d4091febcb951446f85e66d; seed2 405d6b41893e4262b61e1573337a8d809bc212de703b02163a656490e1387697.

## Next decision

Retain both q4 potential candidates and the one lost row, and close this fixed-conversion construction comparison. A potential fit could be considered later for an explicitly needed behavioral/calibration improvement, but nonzero gradients and one exposed-row regression do not by themselves require another optimizer dose. The present result already resolves the source-coefficient contract and the native/source hard-score bridge.

Proceed to the next separately admitted strict geometric producer boundary, preferably the context producer while preserving its latent/observable separation, synchronous exact signed-group transitions and independently bound fixed readouts. Keep the event/span controller frozen during that change so trajectory effects can be attributed. This is coefficient-contract progress, not an assumption that current address observations are Markov sufficient or that current failures are retrieval failures. Event/age and remaining float boundaries still require their own scope. Offline donor-weight compilation remains available, and neither this authored panel nor q4 packing establishes general language, geometry superiority, whole-model native execution or energy savings.



---
_Comment/review ids in this file: 5956582373_

