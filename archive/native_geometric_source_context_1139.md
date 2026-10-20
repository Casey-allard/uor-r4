# Retained source context — #1139 / #1140

## Decision — 2026-09-07

Retain `blake3:c6a98c044243a045128a023aad2605cfa478ad3b469cac7b049c08adfaa7154d`
at bounded source-owner/NoRead selection scope. The [evidence](evidence/native_geometric_source_context_1139.json)
binds source, artifact, full response reports, controls and resources. This builds
on PR #1165, merged at `640169fb`, and reconstructs its complete `e1ef0a5d` parent.

| Complete generated answers, including EOS | Parent e1ef | Retained context | Fitted router with context disabled |
| --- | ---: | ---: | ---: |
| Construction: 631 prior + 32 owner/query/order contrasts | 654/663 | 663/663 | 653/663 |
| Open development, familiar question family | 12/16 | 16/16 | NOT_RUN |
| Fresh names/places/values and owner/query/order permutations | 24/32 | 32/32 | 24/32 |
| Earlier admission set, now exposed preservation | Earlier 10/12 | 12/12 | NOT_RUN |
| Earlier literal-binding first-use set, now preservation | Earlier 16/16 | 16/16 | NOT_RUN |
| Earlier source first-use/open sets, now preservation | Earlier 20/20; 14/14 | 20/20; 14/14 | NOT_RUN |

Construction has nine gains and zero lost correct answers: the previous location
abstention plus eight new supported-owner cases. The original 631 cases now pass
631/631. Both remaining errors in the earlier admission set are repaired. The
separate exact-parent control restores all 663 parent texts and stops. These
construction and exposed-population results are not fresh generalization.

The design-selection receipt froze the candidate after construction, open and
preservation checks, before the separate fresh command. Fresh uses two new name
pairs and places, signed distractor values, both location owners, both queries,
numeric-fact orders and location placements. Supported and unsupported answers
both pass 16/16. Its question family is familiar. No fit or model change followed
fresh evaluation. This qualifies bounded source binding under these permutations;
general language, unrestricted reasoning, a full API and frontier capability
remain unqualified. No new angular-versus-equality comparison was run.

## Observed cause and reused mechanism

For the following two prompts, replace only the owner `ada` with `cyra` in the
first fact:

```text
User: ada lives in Rome. ada has 13 coins. other has 7 coins.
User: Where is the location of ada?
Assistant:
```

The sixteen completed words retained by the parent are identical by exact bytes.
Rome remains candidate 14, with `in` at 15. Its preceding `lives` and owner have
been evicted; the retained `ada` after Rome belongs to a different numeric fact.
Live traces confirm identical parent Rome features, mapped codes, H4 state and
scores. Additional geometric code learning over those same features cannot
identify the evicted owner. This is a readout-information boundary, not a result
about all possible uses of the model's other state.

The earlier [operand-provenance work](native_geometric_operand_provenance_1139.md)
and [literal continuation](native_geometric_literal_binding_1139.md) already show
why exact captured identity must reach the learned selector. The architecture
[audit](integration/architecture-2026-09/README.md) also preserves finite-state
and recurrent-code negatives: a small root summary cannot retain arbitrary
history. This change transfers the existing bounded cue-retention pattern to
word candidates instead of expanding the query window or adding a grammar parser.

Each `WordAtom` now carries four nonrecursive predecessor identities: exact
bytes/length and source token/byte endpoints. It captures them as completed words
arrive. The same sixteen candidates remain; predecessors are metadata, not new
copy candidates. `role_read::features_with_context` uses this metadata only where
an older role lies outside the shared window. Exact byte equality supplies
query-to-source matches, including unseen names; dictionary lookup supplies
existing prime identities. The original ordered two-lane signed-H4 product and
angular-rank action scoring remain the selection mechanism. No hash-distance
semantics, matrix products, transformer, expert gating or page subsystem is added.

The artifact's optional `source_context` witness activates this feature law and
stores the previous source router. Removing it and restoring that router must
reconstruct the entire e1ef parent exactly. Numeric selectors, admission,
relations, dependent reads, dictionary and emission parameters remain fixed.
`SourceContextDisabled` selects the previous router and feature law;
`SourceContextWindowOnly` retains the fitted router but removes predecessor
features. Old artifacts omit the new witness and retain their previous feature
law. The scanner carries the bounded metadata for new sessions, including sessions
of old artifacts; this has a fixed storage/copy cost even when its use is disabled.

The fit retains 128 codes and admits 128 identity-initialized entries, with 256
as the configured cap, two passes, eight proposals, seed 1139 and a twenty-second
learner ceiling. Initial retained-context selection is already **428/428**,
with hinge 1. One old code update removes that margin penalty; final selection is
428/428, hinge 0, with 9,962 proposals and no time-limit stop. All new entries
remain identity, and landmarks/biases remain unchanged. The report's 13.140
seconds includes construction-frame preparation and final artifact validation.
The recovered owner feature `(kind=10,a=7,b=3)` already carries code `[84,119]` in
the parent. With retained context, the supported Rome state changes from
`[116,119]` to `[26,119]`, while the unsupported state stays `[116,119]`.
Thus the observed repair primarily comes from exposing retained information to
an existing learned binding code, rather than learning a new owner mechanism.

## Preservation and executable checks

The earlier literal populations pass 103/103 and both 16/16 populations; both
16/16 computation populations, 58/58 computed construction, 8/8 identifiers,
6/6 numeric and three 12/12 role populations pass. Dependent, changed-name and
long relation populations preserve 48/48, 28/28 and 28/28 answers and writes.
Earlier general preservation remains 62/62 and 24/24. Five persistent session
turns, restore/forged-commit checks and isolated Unknown behavior pass.

The final changed Rust example and focused unit/integration binaries compile
with pinned rustup Cargo, release mode, offline dependencies and one build job.
Seven distinct lexeme/snapshot tests pass, including word-boundary behavior,
retained-owner differentiation, invalid shapes, overlapping-byte tampering,
retained-token replay and legacy missing-field compatibility. Overlapping
predecessor metadata must agree with the original words, and available source
tokens must agree with predecessor payloads. Already evicted history remains
explicit unauthenticated checkpoint state, as with the existing payload contract.

The actual c6a9 artifact test passes exact e1ef restoration, malformed parent/code
and inherited-parameter rejection, supported/unsupported evicted-owner answers,
numeric preservation, repeated prediction, no derived write after a mismatched
observation, observation-only numeric writes, checkpoint replay and zero allocations through the measured ingestion
and generation path. The native integer-kernel source guard, architecture-policy
check, formatting and claim-wording check pass. No blanket workspace suite,
public CLI rebuild, WASM/Studio qualification or legacy release QA is claimed.
Protected CI names are compatibility acknowledgements; these checks ran locally.

Seventeen warm `predict + observe` samples from three actual-artifact cases
measure median **135,583 ns** and maximum **183,541 ns**. Loading, encoding,
initial ingestion and checkpoints are excluded from that latency measurement.
This is a small warm-path measurement, not an end-to-end or matched speedup,
energy-efficiency or laptop capability result.

## Resources and continuation

The complete pre-execution projection reserved 125 seconds expected / 150 seconds
maximum model work within the existing 158.065 seconds, 550/900 seconds of
engineering, one model worker, one build job, a 4 GiB model-process RSS limit,
512-token context, sixteen candidates and four predecessors each. Total new
storage was capped at 256 MiB under the inherited cumulative storage ceiling.
All old artifacts, controls, source material and original checkout changes remain.

Actual model work is **60.834 seconds**: 37.739 for load/preparation/fit/open/
preservation, 10.944 for fresh evaluation and 12.151 for the actual-artifact test.
Cumulative use is **4072.769/4170 seconds**, leaving **97.231 seconds**. Peak
sampled model-child RSS is 1,081,999,360 bytes. Artifact JSON is 11,626,472 bytes.
At behavior completion, sampled storage growth is 194,912,256 bytes; final
metadata and engineering accounting is in the evidence/delivery receipt. Report
storage exceeded its preliminary subestimate while compiler growth was smaller;
total growth remained below the configured hard cap. RSS/storage samples are
not exact transient peaks. No budget extension, external compute or deletion
occurred. The initial build was followed by a final build after snapshot review;
both engineering costs are retained, rather than charging only the final build.

**Next direction:** use the selected exact entity/value and committed operator
result to condition reusable learned contextual transitions and emission. The
small owned-context carrier can supply that identity; learned prime-addressed
transitions and signed geometric state must then produce useful novel prose and
Rust continuations. Reuse the existing emission path and preserve the successful
source/numeric boundary. Do not repeat these closed order fixtures as a new
capability gate, fit on the just-opened fresh set, restart the old 120-state-only
recurrent experiment, or introduce paging before a measured access bottleneck.
A fresh complete resource projection must fit the remaining cumulative allocation
before executing a successor. Native model/API quality precedes WASM/Pages Studio
integration; offline Rust training matmul remains allowed and serving matrix
products remain excluded.
