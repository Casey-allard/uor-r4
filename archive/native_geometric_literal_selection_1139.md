# Shared geometric selection for literal answers — #1139 / #1140

## Decision, 2026-09-06

**Retain `af337c28`; do not promote either literal candidate.** The extension
improves new numeric answers and complete conversations but regresses previously
correct computed-result and identifier-copy cases. Its optional code is usable
for continued development; the accepted artifact leaves `literal_answers` absent.

| Generated behavior | Accepted parent | Angular `51788aef` | Exact-code `3f31e998` |
| --- | ---: | ---: | ---: |
| New literal Copy/Add/abstention, names/numbers/fact order | 7/16 | 12/16 | 12/16 |
| New complete three-turn computations, names/numbers/order | 8/16 | 16/16 | 16/16 |
| Previously exposed independent-result transfers | 12/16 | 12/16 | 14/16 |

The unchanged 12/16 aggregate hides two lost and two newly correct cases for
angular. Both new candidates fail a previously correct request to repeat the
kira/fenn total. Angular emits9 instead of7 by adding a literal2 to the computed7;
it also emits9 instead of10 for the requested extra3. Exact-code loses one
previously correct case and has another provenance failure despite correct text.
No new angular-distance advantage over exact-code selection is established.

Angular preserves48/48 dependent responses/writes,28/28 exposed-name cases,
28/28 long-context cases,5/5 persistent turns,6/6 prior numeric cases and all
three12-case earlier role sets. However, earlier response preservation drops
from62/62 to60/62 and prior transfer from24/24 to23/24. All three losses are
identifier-copy requests: numeric answers such as `73);` replace `left` or
`right`. Full equality preservation is NOT_RUN after its own case regression.
No claim of preserved general Rust behavior is made.

All twelve numerical first-use cases pass; four abstention texts fail for each
candidate. NoOperation avoids a typed numeric write, but the remaining text path
loops or emits `coins.`. These failures remain in the16-case denominator.
They are distinct from numerical operand choice and from the identifier-copy
regressions. Nothing inserts expected answers or repairs them using an LLM.

## Implementation

The optional artifact-bound `literal_answers` flag extends the existing typed
role component to numeric candidates before any derived result exists. Old
artifacts omit the flag and retain their earlier behavior. One derived result
still uses the earlier typed selector; two or more derived results use the role
selector as before. Zero retained numeric sources leave the earlier path active.

This is one shared learned component, initialized from the accepted `af337c28`
parameters by exact word-identity remapping. Literal candidates have derivation
depth zero. Their exact numeric payloads, occurrence IDs and four lexical cues
remain in the same sixteen-record state. Query/cue matches use the existing
sixteen-word window and provenance masks; the extension adds no new semantic
metric or hidden source model. It inherits the representation's loss of words
outside the windows and the loss of parent order/multiplicity in union masks.

Two signed-H4 lanes fold learned feature codes and compare three learned action
landmarks to choose Copy, Add or NoOperation. Candidate support remains bounded;
only the selected exact integer operator executes and commits its result through
the causal output codec. Serving performs integer operations and table reads,
with no floating-point projection, matrix multiplication, dense transformer or
LLM/provider response correction. This is a bounded model experiment, not a
claim of general language, reasoning or frontier capability.

## Data and preserved failure

`literal-source.json` contains 122 construction cases: 58 prior computed-role
cases, 32 original construction cases labeled only when actual native-parent
execution reproduces the expected response, and 32 literal Copy/Add/abstention
cases across names and fact order. All 32 original cases were admitted; none
were silently skipped. They cover prose and familiar Rust current values, sums,
pre-update dependencies and abstention. Explicit action/operand labels remain
offline. Literal-only frames reject any supplied preceding prompt, response or
intermediate. Serving receives ordinary text and selects its own operation.

The 16 open literal checks reuse exposed names/order combinations. Another 16
literal cases and 16 complete three-turn cases use predeclared new names and
numbers, to be opened only after design selection. They are small authored
transfer checks, not sealed general-language benchmarks.

The first 256-feature attempt stopped before fitting. A diagnostic replay found
that `roles-fit/0/0` and `roles-fit/0/1` had incompatible targets after truncation,
although their full feature representations differ. The complete observed
vocabulary has 601 features. The source-routing configuration ceiling is now
768, and the literal fit retains the complete observed set within that bound.
This removes an identified information loss; it is not itself model-quality
acceptance. Both stopped attempts and their charged time remain preserved.
Collision errors now identify the conflicting construction cases, retained/full
feature counts and whether the full representations also collide.

## Prefix correction and resulting cost

The complete-vocabulary first fit `923f0d8b` achieves122/122 routing targets but
only56/122 complete construction responses. All58 old trajectories fail their
first answer: their first states had not been included in the new literal fit.
The corrected source adds seven unique first prompts already present in role
construction, labeled from actual accepted-parent execution. All39 native
preservation labels are correct. The resulting129-frame source changes no
reserved transfer case. Continuing from `923f0d8b` gives129/129 routing targets,
zero hinge and121/129 actual construction responses:58/58 role trajectories,
39/39 preservation prefixes and24/32 new literal responses. The eight failures
are abstention text. Both finite four-pass fits finish without the30-second
learner cap. No further fitting follows the new-transfer opening.

The revised component contains602 features,65 dictionary words and57,630 JSON
bytes; the first complete fit had601 features. The artifact binds structural
parent `bb79456b` and initialization donor `923f0d8b`; that donor binds `af337c28`.
The matched exact-code continuation starts from the same `923f0d8b` parameters
and fits the same129 frames with the same configuration except distance mode.
This is a conditional continuation comparison, not two independent training
pipelines.

At most16 numeric records,16 query words, four cues per literal, two H4 lanes
and273 proposal slots remain configured. Each candidate has at most51 active
features. A sorted vocabulary of at most768 entries increases binary-search
work; this is not a free capacity increase. Across all16 new three-turn cases,
actual execution records1,144 numeric proposals,48 selected operator executions
(40 Add and8 Copy),48 derived writes,475,008 typed-routing comparisons,
167,832 typed-routing table reads and8,579,104 logical routing bytes read.
These are nested counters, not disjoint machine-instruction totals. The whole
path additionally records2,580 observed tokens,31,288 candidate evaluations,
2,536,578 memory-composition comparisons and722,640 score lookups. Full nested
counters, including writer, memory, output and failed responses, remain in the
bound reports and [evidence](evidence/native_geometric_literal_selection_1139.json).

The16-case complete evaluation takes56ms after model loading; the process also
pays artifact loading/validation, which is charged by the wrapper. Neither that
small authored population nor the typed subpath counts establish whole-model
speed or scaling advantage over a transformer. The actual three-turn serving
allocation check reports zero allocations/bytes and passes checkpoint roundtrip
and artifact mutation rejection. Seven focused typed tests and the kernel source
check pass. The rebuilt real CLI emits `7.\n` for reversed fenn/kira literals
with a301 distractor. New broad syntax/code generation remains NOT_RUN.

## Next implementation

Protect the accepted computed-result parameters while learning literal numeric
admission and selection against existing word-answer alternatives. The next
change should prevent numbers in a prompt from taking over an identifier or
word response. Reuse the existing NoOperation alternative, retained exact
records, cue matching, geometric routing and integer operators; use a bounded
literal-state correction rather than refitting all working response roles.
This narrower scope is justified by the measured interference above.

Inspect new query-identity codes as part of that change: `fenn` was absent from
the old role dictionary and now has a non-identity learned code, while the Add
landmark and bias also changed. These are plausible coupling mechanisms, not a
proven causal diagnosis. Keep exact operand-reference matching distinct from
operator intent, and check individual preservation cases. Do not respond to
this negative with more generic capacity or a wider random sweep. Current
first-use results become exposed evidence for any redesign; reserve new
transfers only after choosing it. Generated Rust expansion follows preserved
complete selection, not the16/16 headline alone. #1139/#1140 remain open.

## Local reproducibility

Artifacts, sources and command receipts are retained under
`/Users/casey.allard/uor-r4/.uor-models/native-typed-value-2026-09-05`.
The existing `literal-run.mjs` wrapper charges preparation, failed fits,
evaluation and replay to the cumulative model ledger. `literal-projection.json`
initially reserves300 seconds of model work,900 seconds of engineering and192 MiB of
growth. After the identified corrections, the cycle ceiling increased to360
seconds within the already approved cumulative ceiling for complete comparisons. Under standing owner authorization the cumulative model ceiling was
extended by 240 seconds to 2,970 seconds before execution. No storage increase,
deletions or external model compute were needed for this projection.

Final monitored totals are317.230s model work and663.529s engineering, within
360/900s cycle ceilings. The original210s model and650s engineering point
projections were exceeded; the model projection was revised before the final
comparisons. Cumulative model use is2921.586/2970s, leaving48.414s. Peak sampled
known storage is6,514,409,472 bytes, below the effective6,707,802,112-byte ceiling;
peak sampled child RSS is878,379,008 bytes, below4GiB. The final snapshot shows
about54.64MiB known growth from this cycle's opening snapshot. These are the
existing conservative ledger's measurements, not exact physical extent or
transient-peak guarantees. Formatting, architecture-policy and claim-wording
checks pass; broader dormant release QA remains NOT_RUN.
