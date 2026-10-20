# Geometric routing to retained word sources — #1139

## Mechanism and scope

This revision connects learned H4 selection to the accepted model's exact
recent-word references and causal copy operator. It addresses the preceding
shared-output block's inaccessible retained content and identical eight-token
inputs. It does not add a second memory store, a tokenizer or an answer-token
classifier. The accepted parent is `067adbf0`, from the sparse relation-admission
artifact. Its reader, writer, vocabulary and response completion remain frozen.

At the existing role-reader boundary, at most 16 recent word occurrences plus
NoSource are enumerated. Candidate-relative features describe surrounding
lexical addresses, recency and exact equality between query and source-role
occurrences. These addresses are equality identifiers, never semantic distances
computed from hash bits. Offline Rust construction selects 64 features from
1,987 observed features in the initial fit, using frozen reader-weight salience
and construction frequency. This reuses an already learned feature vocabulary;
it is not discovery of semantics from unstructured bytes alone.

Each selected feature has two learned signed H4 root codes. Ordered group-table
products fold its candidate's features into two states. Action landmarks and
biases are learned jointly with those codes. Serving scores each allowed action
by the sum of two signed relative-cosine ranks plus its integer bias. The matched
exact-code mode replaces each rank with identity/nonidentity selection while
retaining the same H4 composition and parameter capacity. Ties retain the first
candidate/action under the existing newest-first order. No multiply, divide,
floating point or dense matrix operation executes in this changed kernel.

The winning result is an exact source index and existing ReadAction: copy now,
emit the learned prefix then copy, or NoRead. The existing operator keeps source
end/byte-end witnesses, commits only after matching observation, copies exact
bytes and then runs existing completion. The displayed copy-trace score is the
existing dispatch margin; it is not the new raw angular score.

The H4 fold compresses selected metadata, not payload bytes. Features outside
the learned vocabulary contribute identity. Different feature sequences can
still collide in the 120 × 120 state space. Exact spelling and source identity
survive in the retained word record through copying; the fold is not an inverse
codec. This finite candidate scan does not yet give a sublinear context index or
multiscale abstraction. It adds two roots of stack state, no persistent session
buffer and no new per-source cache.

The two channels are independent signed H4 channels, not a newly qualified
paired-icosian/E8 construction. The inherited model retains fixed-zeta phases,
exact `Z[phi]`, R4/H4 state, orientation, typed paired-H4 storage and UOR identity
at their existing boundaries. Persistent relation reads keep priority over this
recent-word selector. Numeric eligibility and derived-value operators are
unchanged. New learned access over persistent relations and a dependent second
source read remain subsequent. Startup artifact/geometry validation still uses
floating point; the kernel result is not an all-startup purity claim.

## First matched result — 2026-09-06

The existing 480 construction examples yield 320 reachable source/action frames;
160 numeric/upstream cases bypass this selector. No active target is unreachable
and no frame uses persistent-reader dispatch. Both fits finish two passes within
their 30-second limits. Angular learns 319/320 choices; exact-code learns 191/320.
They evaluate 3,869 and 3,863 proposals respectively (unchanged-value proposals
are skipped). This is matched capacity, initialization and search schedule, not
identical optimization trajectories or equal realized loss.

| Arm | Earlier exact responses /62 | Earlier transfer /24 | Fresh names /24 |
| --- | ---: | ---: | ---: |
| Frozen accepted parent | 62 | 24 | 24 |
| Angular source routing | 61 | 21 | 21 |
| Exact-code source routing | 46 | 11 | 9 |
| Angular codes disabled | Not run | Not run | 4 |

The 62/24 populations are reused OPEN development. Fresh cases replace entity
and place names with strings absent from construction, within known grammar;
they are not sealed evaluation or broad language qualification. The inherited
parent gets all fresh cases right. Complete disabled-selector replay matches
all 62 parent Generation objects except the explicit control label.

The angular preservation mismatch selects Pune correctly but omits the leading
space. Two fresh Talven cases choose NoRead/Unknown despite retained reachable
values; a third copies Talven correctly but omits the space. This is source/
action selection failure, not lost payload or failed byte execution. The fresh
Rust identifier `packet_input`, absent from construction, is copied into the
known identity-function shape through the actual CLI. Its exact generated text
is compiled without repair and checked against negative, zero and positive inputs.

Angular beats this exact-code control on this one construction/OPEN population,
while still regressing the accepted reader. It is bounded evidence for learned
geometric selection, not superiority over the existing sparse reader or a
frontier language model. General prose, novel grammar and multi-operation
reasoning do not follow from these copy results.

## Localized revision

On `Record: firden in Talven. Where is firden? Answer:`, the actual CLI returns
` Unknown.\n`. The existing `word-copy-geometry-disabled` control changes the
first source decision to the correct retained Talven record and returns
` Talven.\n`, with the learned H4 codes still executing. This control also
affects later copy completion, so only its first source decision is used to
localize the selection problem.

The versioned `role_context_only` configuration therefore excludes inherited
word-path/zeta features only at the new selector's feature boundary, in both
fitting and serving. It preserves those states elsewhere and preserves the
existing suffix behavior. Learned ordered H4 composition and angular scoring
remain active. This tests a specific interference hypothesis; it does not
demote geometric state generally or establish complete renaming invariance.

The revised angular artifact is
`55e602a026238f12c8e9026569c1d1e06239a83efa3ff0ca23b48d4e5242c001`;
exact-code is `1b38e1a4cac211d896e10c5d670d6f4c58f180685b02857c556352bfc71df42e`.
Both select 64 of 1,738 observed role features and finish two passes. Angular
gets 320/320 construction choices with zero hinge; exact-code gets 152/320.
They evaluate 3,861 and 3,844 proposals in 3.758 and 3.697 seconds respectively,
including final artifact validation within the reported fit time.

| Revised arm | Earlier exact /62 | Earlier transfer /24 | Changed names /24 |
| --- | ---: | ---: | ---: |
| Frozen parent | 62 | 24 | 24 |
| Role-context angular | 62 | 24 | 24 |
| Role-context exact-code | 36 | 10 | 10 |
| Angular learned codes disabled | Not run | Not run | 4 |

All 62 complete parent Generation objects replay with the selector disabled,
normalizing only the control label. Final angular also preserves 28/28 answers
and exact writes on the reused longer-context relation population, and 5/5
persistent-session turns with revisions, contradiction, unrelated-memory
preservation, restore, forged-commit rejection and new-session isolation. Those
tests check compatibility with the existing relation mechanism, not learned
geometric access over its persistent references. The final CLI emits exactly
the same compiled-and-executed `packet_input` function bytes as the initial fit.

**Decision: retain `55e602a0` for the next source-routing development step, with
`067adbf0` preserved as the frozen comparator.** This establishes bounded useful
source selection under the actual learned angular kernel and beats the matched
exact-code learning control here. The control's poorer fit is part of the
measured result; it does not show every exact-code optimizer must be inferior.
Reused and renamed development cases influenced this revision, so none is
relabelled sealed evaluation.

The owner-of-the-red-box question still returns ` Unknown.\n` instead of Rome;
the add-two-then-double question returns the same instead of 14. The next
implementation connects a selected exact entity/reference to one dependent
relation read, retaining both references and using existing final copying.
Learn that source/operator choice and preserve these working one-read cases.
No wider metric sweep, source store or token classifier is justified here.
The full #1139 handoff and broader #1140 reasoning qualification remain unmet.

The final angular block is 68,714 bytes; its full artifact is 10,797,015 bytes.
On the same 24 changed-name responses with identical parent/output bytes, the
new selector records 2,532 feature lookups, 2,340 code reads, 10,994 table reads,
18,298 comparisons and 348,904 logical operand bytes. Inherited copy row
comparisons fall from 141,490 to 10,710. Payload byte reads (181), word-record
reads (21,827), equality byte comparisons (4,155), memory-composition comparisons
(1,090,425) and relation-writer row comparisons (1,877,694) match the parent.
Complete warm samples are 23.388 ms parent and 23.567 ms angular; no speedup
claim follows. The logical two-root accumulator is four bytes; peak compiled
stack and physical memory traffic were not measured.

Final release builds, final allocation and kernel-source checks, formatting and
the Rust architecture-policy checker pass. Two focused units passed before the
localized revision; the revised units run in protected native CI. Actual artifact
reload, generation, CLI and session checks exercise the revised path locally.
Final held-out evaluation is NOT_RUN. Protected delivery owns final CI status.

This cycle's model commands total 67.516/120 seconds, including both complete
fits/comparisons, controls, CLI, generated-code compilation/execution and
persistent-memory checks. Cumulative model use is 1,514.012/1,800 seconds,
leaving 285.988. Engineering through formatting/policy is 1,162.496/1,200 seconds.
Highest sampled model-child RSS is 840,744,960 bytes. Latest known retained
storage is 5,969,833,984 bytes, with 133,988,352 bytes before the tighter stop
and the preserved 128 MiB margin. No storage increase, deletion or paid compute
was used. Final checks/delivery receipts may add small engineering/metadata
costs; refresh them before a successor projection.

## Cost and reproduction

The new immutable block initially adds 68,667 bytes (angular) or 68,668 bytes
(exact-code), including 480 provenance receipts. Both full artifacts also retain
the complete frozen comparator. No total-model compression claim follows.
For N ≤ 17 candidates, F ≤ 512 features, V ≤ 256 learned codes, two H4 lanes
and A ≤ 8 actions, source scoring costs O(N F log V + N F + N A), after existing
feature construction. Role construction itself includes bounded nested equality
checks over recent words and bytes; it must not be omitted from cost claims.

On the initial 24 fresh angular responses, the new selector executes 24 times,
examines 299 sources including NoSource, performs 5,042 feature lookups, 2,754
code reads, 11,822 table reads and 35,868 comparisons, and accounts for 652,562
logical operand bytes. Existing word-copy work additionally records 22,793 word
record reads, 10,557 dictionary byte comparisons, 4,497 equality byte comparisons
and 163 payload byte reads. Its selector row comparisons fall from 141,490 to
11,472, while whole-model memory composition still costs 1,069,275 comparisons
and the unchanged relation writer costs 1,877,694 row comparisons. These are
logical operation counts, not measured DRAM traffic. Gather/commit work lives
in inherited copy counters, not duplicated in the new routing counters.

Warm complete 24-response samples are 22.816 ms parent, 22.158 ms angular and
22.869 ms exact-code. Different failed output lengths and single samples prevent
a speedup claim. Initial actual CLI wall time is 2.832–3.270 seconds including
artifact load/validation, encoding, execution and JSON output. Separate startup
phase timing is unavailable. Both fits plus full evaluation/reload/replay cost
21.112 seconds wall, with highest sampled model-child RSS 838,762,496 bytes.

All raw source, fit, model, generation, work, controls, compiler output and command
receipts remain under `.uor-models/native-typed-value-2026-09-05/source-routing-*`.
The existing `native_geometric_routing_probe` accepts `PARENT NEW_DIRECTORY
--sources DEVELOPMENT_JSON FIRST_USE_JSON`; `--sources-role-only` runs the
localized revision. It creates a new output directory and never overwrites an
earlier attempt. The [compact evidence](evidence/native_geometric_source_routing_1139.json)
binds file identities, results and final cumulative resources.
