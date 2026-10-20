# Endpoint-bound reverse relation spans — #1139 / #1140

## Implementation — 2026-09-07

[PR #1169's retained-value path](native_geometric_retained_spans_1140.md) keeps
complete forward values, but reverse statements commit only the writer-selected
word. Replaying words after that selected value previously included the linker
and owner and regressed two long-window answers. That negative remains intact.

The source provides a more useful boundary: the learned reverse writer chooses
a value word before its owner. This continuation treats that selected word as a
hard terminal endpoint. The existing H4 Continue/Finish operator checks bounded
contiguous starts before the endpoint, using each candidate's original source
cue consistently across every edge. The longest admitted start is retained.
The linker and owner after the endpoint never enter the value. No new fit,
lexical linker rule, transformer, matrix product or learned expert gate is added.
This reuses an already learned edge operator with a typed endpoint; it does not
establish a new learned general phrase-boundary model.

The selected `WordAtom` keeps its bytes, geometry, predecessor context and
position. A reverse span carries its earlier `start` atom separately, with its
terminal equal to that selected anchor. Complete payload comparison works across
forward and reverse anchors, preserving immutable versions and conflicts.
Reverse payloads remain eligible for persistent reading while their terminal
anchor is still visible: the recent reader starting at that final word cannot
recover the earlier bytes. Existing dependent-read first-leg span rejection
continues to prevent joining a phrase through one constituent word.

`Model::with_reverse_relation_spans()` adds an optional complete-parent CID to
the artifact. Removing that field reconstructs the entire retained-span parent;
loading validates both identities. Snapshot checks bind the opt-in, exact start,
terminal, ordering, payload, learned edges and source words still visible.
Reverse spans cannot be pending forward writes. Evicted source fields remain
declared state, not authentication of unavailable original text. The complete
parent artifact is the control; existing query-span ablations do not disable
persistent ingestion using the full learned operator.

## Execution contract

The owner's continuation instruction follows the disclosed 6.401-second balance.
This cycle explicitly adds 120 seconds to the cumulative ceiling, from 4,290 to
4,410 seconds, without resetting the 4,283.599 seconds already spent. Cycle caps:
120 seconds model work, 900 seconds engineering, 40 MiB new sampled storage,
4 GiB model RSS, one model process and one Cargo build job. Projection: 70 seconds
model work (35 comparison/artifact construction, 15 actual-artifact checks,
20 correction reserve), 600 seconds engineering; 12 MiB artifact, 26 MiB compiler
transients and 2 MiB reports. No external compute or broad cleanup is planned.
Context remains 512 tokens, retained relations 16, value payload 28 bytes and
response limit 32 tokens. The existing cumulative guard charges all attempts.

One combined Rust probe compares parent and candidate on six construction turns,
then six open turns and three short reads. It checks complete outputs, exact
version counts, checkpoint prediction replay and session isolation. Existing
forward sessions and numeric/source/dependent preservation follow. A decision is
recorded before six separately authored fresh turns and two boundary diagnostics
(unseen plain introduction and a two-space gap). No fitting uses those panels.
The actual-artifact test checks short/evicted reads, terminal-anchor preservation,
parent and malformed-start rejection, every-step replay and allocation counts.

## Result

Retain `blake3:321e990f218899aa2668c2530caed5589c614cc58d1bb1de0b7145a64aee9563`
at bounded reverse-value retention scope. The complete `0b12b604` parent and
all its learned parameters reconstruct exactly; no fit or correction retry was
needed. Construction improves 3/6 to 6/6, open development passes 6/6 and short
reads pass 3/3. Six familiar-template fresh turns first executed after selection
pass 6/6, including three-word values. All panels preserve exact version counts,
independent-session isolation and per-step checkpoint prediction equality.
Counts are not an independent expected owner/value/action sequence comparison.

All 663 retained construction answers pass. Prior source-span panels remain
14/14, 6/6 and 9/9; all 18 earlier forward session turns pass. Long-window answers
and write witnesses remain 28/28, dependent reads 48/48, and previous persistent
sessions 5/5 with isolation. Other selected numeric, literal, name, role,
identifier and chain preservation passes.

Both boundary diagnostics fail. `Records show Orin Grove holds nelra` yields
`Records show Orin Grove`, because longest admitted start is not a learned
semantic start boundary. `Orin  Grove holds nelra` yields `Grove`, because the
exact one-byte separator representation cannot reconstruct two spaces. These
post-selection diagnostics are retained and were not used for fitting or a retry.
The earlier reverse linker/owner overextension stays fixed by the hard endpoint.

The release probe and test binaries compile. Five focused relation-span tests,
the integer source guard and architecture policy pass. The actual artifact
passes malformed parent/start rejection, terminal-anchor preservation,
short/evicted reads, every-step checkpoint replay and zero allocations during
measured ingestion/emission. Twenty-eight uncached predict-plus-observe samples
have median 375 ns and maximum 151,916 ns, including initial selection but
excluding load, encoding, ingestion and checkpoints. This is a small kernel
measurement, not an end-to-end latency or energy result. Root CLI, Studio and
broad release QA are `NOT_RUN` for this artifact.

Model work is 43.770 seconds (29.272 comparison/construction, 14.498 actual-artifact
check). Cumulative use is 4,327.369/4,410 seconds, leaving 82.631 seconds. All
source, parents and negative results remain retained; no cleanup or external
compute was needed. The evidence receipt records exact resources and identities.

**Next:** learn a phrase-start decision over admitted starts instead of selecting
the longest passing prefix. Use exact owner/value context and the existing H4
operator with independently authored construction/open contrasts; keep these
opened diagnostics as preservation only. Wider separator retention is a separate
representation limitation. A new run must project all work against the cumulative
balance. This step does not establish general language, reasoning, angular
superiority, frontier capability or energy savings.

At evidence capture, metered engineering commands total 269.887 seconds.
Retained sampled growth is 14,749,696 bytes; peak sampled growth is 17,833,984
bytes, within the 40 MiB cycle cap. Peak sampled model-process RSS is
967,245,824 bytes. The new artifact is 11,631,560 bytes. These observations
are samples rather than exact physical peak measurements.
