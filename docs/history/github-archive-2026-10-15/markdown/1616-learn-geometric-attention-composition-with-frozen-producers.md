# #1616 — Learn geometric attention composition with frozen producers

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T13:27:34Z
- **Updated:** 2026-10-09T13:17:19Z
- **Closed:** 2026-10-02T13:28:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1616
- **Merged:** 2026-10-02T13:28:43Z
- **Merge commit:** 25b7a86990816003450ade4e9aed70f6b28a7a8f
- **Base / Head:** main / UOR-Foundation:codex/geometric-composition-fit-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The connected H4 attention composition initially lost most retained answers. Add a Rust driver that learns only its categorical left/right H4 choices and four-bit gains from ordinary answer loss, with the producer, reader, NoRead, base and tail frozen.

Both fixed640-update seeds complete and independently reload: source/native answers102/109 and128/128 on the original/stress panels, with512/512 answer agreement. Accuracy improves or ties the retained parent, but CE is worse in all four panels; preserve all candidates and changed rows without a geometric-advantage or whole-serving claim. Update current state and roadmap to end this dose and address packedq4 value-producer coefficients next.

Validation: two focused driver tests, release build, both two-B8 zero-update diagnostics, both actual640-update fits, source/compile/native reload and frozen dependency checks; cargo fmt, claim wording and diff checks pass. Actual fit workers193.721/184.426s, about210MB RSS. Independent source/math and result reviews retained in the evidence root. Checkpoints preserve exact RNG/source but omit AdamW moments. References #1512 and #973.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5953544326 — 2026-10-02T13:34:15Z — Casey-allard

# Independent completed-fit result review

Reviewer: `/root/adoption_review_architecture`, 2026-10-02. I did not author the fit driver, training learner, native compiler, result documentation or receipt. I authored the underlying integer composition/reducer modules, so this is independent review of the driver, completed results and reporting, not an independent approval of my own kernels. No Cargo, model execution or Git operation was performed during this result audit.

**Verdict: approve the scoped result and delivery documentation at the root-supplied PR #1616 head `b620e728e8e93106b8d2a9322c188fb17864687d`.** I read the local result document, evidence receipt and corresponding current-state/project-track changes. Their measured claims and next action agree with the completed artifacts. The commit identity is supplied by the coordinating root; this read-only review did not independently resolve Git. No invalidating result or reporting defect remains.

## Independently recounted outcomes

Each panel has 128 rows. Source and compiled-native greedy predictions agree on all 512 rows after independent reload; this is not all-logit bit parity.

| Seed/panel | Initialized bank | Frozen parent | Learned source/native | Gains/losses vs parent | Parent mean CE | Native mean CE |
|---|---:|---:|---:|---:|---:|---:|
| 1 original | 28 | 98 | 102 | 7 / 3 | 0.938978624 | 1.099174772 |
| 1 stress | 23 | 104 | 109 | 9 / 4 | 0.894337295 | 1.091401843 |
| 2 original | 0 | 128 | 128 | 0 / 0 | 0.286855787 | 0.470687125 |
| 2 stress | 9 | 128 | 128 | 0 / 0 | 0.281315631 | 0.458133707 |

Totals are 467/512 learned, 458/512 parent and 60/512 initialized. There are 16 gains and seven losses against the parent. All four learned panels have worse saved-logit CE than the parent despite improved or retained aggregate accuracy. This supports retaining useful learned composition, not claiming a uniformly better or preserving replacement.

Seed-one original lost row indices are 17, 100 and 123; stress lost indices are 38, 109, 110 and 116. Their correct-source head-zero masses are respectively 0.99911435, 0.99929122, 0.99936463 and 0.99979931, 0.99641219, 0.99442216, 0.99829586. The source weights are unchanged. These failures occur downstream of strong correct-source selection in the composition/frozen-tail interface; they do not establish whether representation, optimization or decoder compatibility is the unique cause. Seed-two retains every parent-correct row. Parent-relative changed predictions, including wrong-to-wrong changes, remain in the raw rows.

The maximum reported source/native actual-position logit deltas are 0.0002825260, 0.0007262230, 0.0003114939 and 0.0003257990 in table order. Distinct float-reference and integer-reduction arithmetic remains explicit.

## Execution, learning and integrity scope

Both attempts finish 640 updates with terminal exit zero. I checked all 640 history entries and saved batch identities per seed: contiguous absolute batches 1920..2559, exact u64 RNG chaining, eight episodes per batch, source/answer joins, actual and padded lengths, finite losses/norms/times, and sum/mean CE denominators. The 10,240 episodes are sampled draws, not a claim of unique histories. Actual positions total 491,104; padded positions total 571,920. Episodes range 24..78 in seed one and 24..80 in seed two; admitted context is 128.

Both nine-checkpoint chains at steps 0,80,...,640 match history RNG and parent identities. Final source bytes match checkpoint 640; compiled parameter payload digests match final metadata/history. Sampled ordinary-answer gradient norms reach gains and both selector families and are finite/nonzero at all nine recorded samples. First-to-last minibatch CE falls from 2.398585 to 1.225940 and 4.633334 to 0.620380. These different minibatches are not a fixed evaluation comparison. Recorded changes are cumulative update events, not counts of distinct final entries.

I independently checked exact outer file closure, lengths and BLAKE3 digests for 715 entries per sealed attempt, plus all nested checkpoint seals. All 46 frozen dependency file identities and 45 raw saved-base tensor identities per seed match their actual bytes. Every retained evaluation row joins its episode, prefit and parent records; saved predictions equal earliest argmax over finite 40-way logits. For retained query rows I checked exact occurrence weights, NoRead weights, denominators, maxima and NoRead Q24 against prefit/parent traces. Full-prefix preservation is covered by the executed driver's assertions and report, not independently reconstructed from unretained full-prefix trace data here.

Observed worker wall times are 193.720764 and 184.426446 seconds, total 378.147209 seconds; peak RSS is 209,453,056 and 210,092,032 bytes. These figures exclude source preparation, build, review and delivery and are not complete-path energy measurements.

Artifact SHA256 bindings:

| Artifact | Seed 1 | Seed 2 |
|---|---|---|
| report.json | `f805ea43fc8250f3dd5b549a459bdf40159361dc8e3715ec736d2b46db65d034` | `81616b9a950f31de32b6ecdf57480435eb15d0617a075d9eb75d2d07dfd3c0e8` |
| manifest.json | `5c17df5b867451ba672c560653a04637aaf62a11f3fdbd8f88b78f1837293446` | `8f61da56907f37ab88c7ffe55f7ada0d62b928460d3368e1808d9cace2681d69` |
| history | `baa5dc0f797cbfe8459293ccca05b72f1e7274525c304cb131d06acbddcf9181` | `55d843bc71d8be9d1d9bc11c1e49dd9efea7ab8dc9a4df9adb5c3a6722acc5b2` |

Executed binary: `153de9d37b123155882b50e40908e79c2eef5a0906fb130e98d0c921b8b0966c`. Driver: `b6c65eaf27a3615e21234c1b1ec9c8356df8e7dbf9464bc659bbc7bcae8f2936`; fit helper: `0d350cbe095803640296d230d5e8e69b07be33e02c91bb95130697eb432b7a84`. Execution receipts report unchanged source before/after. Exact RNG starts/ends are seed one `9347350181718662555` → `461503785751815875`, seed two `10697677911788535647` → `1220542080958524937`; these were compared as integers, not floating-point identifiers.

## Consequential next action and claim limits

End this fixed dose and retain fitted, initialized and parent artifacts. There is no matched extra-640-update parent control, so these results do not establish geometric superiority. The frozen float trunk/vocabulary tail and remaining wide learned coefficient sources prevent a complete four-bit/native-serving claim. This grammar panel does not establish natural language, general reasoning or energy advantage.

The next concrete boundary is the K2 value producer's unrestricted learned token/own-state/neighbor-state/held-span/span-valid coefficients, currently expanded into Q24 tables. Use explicit packed q4 source coefficients with independently regenerated signed-root factors; preserve 120 signed root identities, two atoms, radius bins, explicit absence/zero distinctions and the 16-coordinate-per-head interface. Freeze context, events, addressing, NoRead, learned composition and tail for the first deterministic saved-parent projection/recompilation comparison. That outcome can justify bounded answer learning if needed; perfect packet or donor-vector imitation is not a gate. A root ID is a categorical geometric selector, not a four-bit numeric coefficient, and wide derived geometric table entries must be distinguished from free learned wide coefficients. Fixed coefficient units/rounding need their own declared source contract. No further bank dose or selector/alphabet sweep is warranted by these results, and no geometry family is retired.



---
_Comment/review ids in this file: 5953544326_

