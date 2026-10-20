# Retained geometric relation spans — #1139 / #1140

## Implementation — 2026-09-07

The [accepted source-span operator](native_geometric_span_context_1139.md)
selects complete values from current query occurrences, but persistent relation
records retain only their first word. The writer commits when that first word
finishes, before later words arrive. Its former equality check also treats
`New York` and `New Jersey` as the same value because both anchors are `New`.

This continuation reuses the accepted signed-H4 Continue/Finish operator during
input ingestion. An artifact-bound optional `relation_spans` component names
the complete `419ba3a7` parent. `Model::with_retained_relation_spans` adds this
execution capability without fitting or changing any parent parameter. Removing
that component reconstructs the parent; loading validates that relationship.

A fixed pending relation holds the selected owner, first-word anchor, write
action and accepted extent. Exact adjacent words extend it only when the same
learned operator admits them, up to 28 bytes. Consumed continuation words do not
receive a separate writer decision. A non-continuation or explicit response
boundary finalizes the value once. Previously committed records remain immutable.
Only forward writes whose value is the newest completed word can defer and
extend. A reverse write completing at its owner binds its earlier single-word
value immediately: the existing writer supplies no intervening phrase endpoint.
Pending and committed-span snapshots enforce owner-before-value order. This
scope restriction uses selected roles, without a hardcoded relation word.

The committed record keeps its original `WordAtom` identity, position, geometry
and predecessor context. A separate optional span retains complete exact bytes,
word count and terminal-word provenance. Versioning and conflict comparison use
the complete value. Explicit revision can clear a conflict. A multiword value
cannot silently join a dependent lookup through its first word alone; a final
retained value can be emitted whole by the existing causal copy cursor.

Snapshot validation checks activation, payload capacity/padding, anchor prefix,
terminal metadata, contiguous source positions, word count, learned edges and
consistency with source words still visible. Pending writes must be inactive and
consistent with the latest completed word. Evicted source information remains
declared state; these checks do not authenticate unavailable original text.
Legacy absent optional fields retain their prior serialization and behavior.

The matched retained-memory control is the complete parent artifact. The existing
`SourceSpanDisabled`, `SourceSpanContextDisabled` and `SourceSpanPairDisabled`
controls affect frozen-query continuation; retained ingestion uses the full
accepted operator and committed payload emission does not rerun those controls.
They are not whole-model ablations of retained phrase memory.

The serving addition uses bounded integer/table operations and exact byte copies.
Work counters include span edge checks, copied bytes and geometric routing. This
is reuse of a previously learned geometric operator, not a new angular-versus-
equality learning result or a general phrase-boundary qualification.

## Execution contract

The cumulative model ceiling remains **4,290 seconds**, with **60.695 seconds**
available at entry. The new cycle has a **60-second model cap**, **900-second
engineering cap**, **48 MiB storage-growth cap**, one local model process, one
Cargo build job and a 4 GiB model RSS target. Projection: 50 seconds model work,
650 seconds engineering, 12 MiB artifact, 34 MiB compiler transients and 2 MiB
lean evidence. All retries and stops remain charged. No external compute or
cumulative allocation increase is authorized by this cycle.

The combined Rust run compares the parent and new artifact, checks construction
and open sessions plus existing preservation, records the decision, then opens
a fixed fresh session panel without another fit. Sessions exercise repeated
assertions, same-prefix revisions and contradictions, unrelated memory and
isolation after more than 512 padding tokens. Every generated step is compared
with checkpoint restoration. A separate actual-artifact check covers a pending
write, atomic commitment, malformed extent rejection and allocation counts.

## Initial result and correction

The initial runtime returns all 6/6 construction, 6/6 open and 6/6 separately
authored fresh session answers, versus 0/6 parent construction answers. Exact
write/version counts and new-session isolation pass. All 663 retained
construction answers and earlier source-span panels pass. However, long-window
preservation falls to 26/28: `Xarven holds ravnel` and `Zorlan holds torvek`
incorrectly retain the complete clause as the value. Their original writer
already selected the right single-word anchor; replaying intervening words
introduced the regression. Selection was false before opening the fresh panel.
The negative reports and exact initial runtime/probe sources are preserved.

The correction above removes that unsupported reverse extension. The learned
artifact and all parameters remain byte-identical; initial versus corrected
runtime source identities distinguish the two experiments. The artifact CID
alone does not identify the runtime that executed it. The corrected replay
reuses all panels as open preservation evidence; the formerly fresh six turns
are not a new held-out result, and no fitting takes place.

On the initial runtime, the actual-artifact test passes pending-write restore,
malformed extent rejection, source eviction, every-step prediction replay and
zero allocations during measured ingestion/emission. Its 12 uncached
predict-plus-observe samples have median 417 ns and maximum 127,916 ns, including
initial selection, excluding loading, encoding, ingestion and checkpoints.
This forward path is unchanged by the correction, but the result is scoped to
the initial runtime. Energy and end-to-end latency remain unmeasured.

## Corrected result

Retain `blake3:0b12b6044e4a04124fa07a69496d9c5da65fc6a9d6ad95072fc46a9590916762`
with the corrected runtime, at bounded forward retained-value scope. Construction
and open sessions pass 6/6 each; all six previously exposed fresh turns replay
correctly. Every turn restores checkpoints before each prediction; exact expected
write/version counts and independent-session isolation pass. These count checks
do not independently compare every owner/value/action against an expected write
sequence. Parent construction remains the retained 0/6 matched control.

All 663 earlier construction answers pass. The prior span panels pass 14/14,
6/6 and 9/9. Long-window preservation returns to 28/28 with 28/28 exact write
witnesses; dependent reads pass 48/48, earlier sessions 5/5 with isolation, and
all other numeric, literal, identifier, name, role and chain preservation passes.
No fit or artifact mutation occurred during the correction.

The corrected release probe and test binaries compile; three focused relation
state tests and the expanded integer-kernel source guard pass after the change.
Two existing source-span tests and the architecture policy check also passed in
the cycle. The final replay takes 14.718 seconds, bringing total model work to
54.294 seconds and cumulative use to 4,283.599/4,290 seconds. The initial actual-
artifact test is not rerun after correction; final CLI, Studio, broad release
QA and end-to-end energy measurements are `NOT_RUN`.

**Next:** learn role-aware value endpoints for both source orders using the
existing learned geometric operator and selected owner/value roles. The reverse
writer's single-word boundary, unseen connectors and wider gaps remain explicit
limitations. Do not fit on the opened fresh panels. A successor must project its
complete resources against the remaining cumulative allocation.

At evidence capture, metered engineering commands total 551.248 seconds.
Retained sampled growth is 14,856,192 bytes; peak sampled growth is 33,886,208
bytes, within 48 MiB. Peak sampled model-process RSS is 957,579,264 bytes.
The artifact is 11,631,461 bytes. No cleanup or external compute was needed.
These storage/RSS observations are sampled, not exact physical peaks.
