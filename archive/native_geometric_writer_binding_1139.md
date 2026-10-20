# Learned relation-writer binding — #1139, 2026-09-06

This continues protected PR #1150 (`fae76313`) toward useful transformerless
geometric language processing. The existing dependent reader, source router,
relation directory, version semantics and copy operator remain unchanged.
The owner authorized this writer repair and necessary incremental storage.

## Representation and learned operation

The native tokenizer already uses reversible lexical pieces and byte fallback.
Tokens have prime identities and fixed signed-H4/zeta state. The writer receives
completed exact words from that stream. Its old dictionary recognized `now` but
collapsed `Now` and `Question` to the same zero key as unfamiliar names.

The writer revision has its own construction-bound prime cue dictionary.
Words labeled as owner/value payloads anywhere in its construction examples are
excluded from that cue vocabulary; inherited reader vocabulary is not imported.
This is an offline supervised vocabulary decision, not a serving entity parser.
It intentionally erases payload spelling from writer scoring. Exact spelling,
case, source endpoints and current versions remain in relation records, and the
reader retains its own unchanged dictionaries and learned H4 codes. Unseen cues
still collapse to zero. A word that serves both as a payload name and as a cue
is not given context-dependent lexical meaning by this implementation.

The existing writer margin learner fits sparse integer weights over candidate
owner/value-relative cue keys, ordered interior H4 products, signed orientation
and fixed-zeta phase bins. It jointly chooses NoWrite, assertion, explicit
revision or contradiction, with exact source endpoints for the selected pair.
This fits weights over existing geometric/context features; it does not learn
new continuous embeddings or establish a new angular advantage. Both word
orders are construction-supervised. Questions receive NoWrite supervision;
there is no hard-coded serving rule for `Now`, `Question`, cities or answers.

The artifact stores one replacement writer plus the entire frozen parent for
lineage. Only the replacement writer scores at runtime. Parent reconstruction,
cue geometry, rows, dimensions and UOR/content identity are validated on load.
Existing exact NoWrite caches may be reused only after every cached signature
is verified against the replacement scorer. Otherwise they are bypassed.
Exact signature checks remain mandatory; geometry never authorizes merging
unequal writer inputs. Reader parameters are compared unchanged in the fit report.

The writer retains the eight-word candidate window: at most fourteen directed
owner/value pairs, each involving the latest completed word, and three actions
per pair. Each pair supplies nineteen active features to the existing binary
row lookups. The fixed feature buffer has sixty-four slots. At full capacity,
that is 42 scored choices and 798 feature queries per completed word before
NoWrite admission. The model permits at most 16,384 writer rows and 256 cue
words. Existing counters account for dictionary/row comparisons, path/table
work, phase operations and record/directory work. They are logical-operation
counters, not a complete machine-instruction or memory-bandwidth census.

There are still sixteen exact relation records, one untyped value per owner,
32-byte word payloads and the inherited 512-token native context. No new
persistent session state, transformer, matrix operation, provider or dense
fallback is added. Startup validation still uses floating point. Fixed zeta,
exact signed geometry/Z[phi], paired-H4 representation and UOR identity retain
their declared roles; this change does not separately qualify each mechanism.

## Development history and evaluation boundary

All artifacts and reports are retained under
`.uor-models/native-typed-value-2026-09-05/writer-binding-*` in the original
checkout. The initial 172 construction documents preserve 112 earlier relation
examples, add 48 dependent examples, eight revision-order variants and four
question-only examples. Write labels are offline byte endpoints; serving sees
raw tokens. The fixture label adapter lives only in the existing evaluation
example, outside the native inference library.

1. `212dba62`: adding all construction names to inherited writer context gives
   23/48 dependent answers and 4/48 exact write sequences. It repairs explicit
   revisions but invents premature writes such as `casket -> in`. Construction
   is 1112/1118 distinct decision frames; the vocabulary has 127 words.
2. `e1e66e2d`: excluding newly added labeled payloads gives 43/48 answers and
   42/48 writes, fitting 551/551 frames. Its 90-word dictionary still inherits
   `Bath`; all remaining write mismatches involve that known-versus-unseen
   spelling distinction. Both attempts preserve 62/62 earlier responses and
   24/24 transfer.
3. `f9a2a273`: using only construction non-payload cues gives 48/48 answers and
   writes, preserving 62/62, 24/24 and 28/28 long-context answers/writes. It fits
   106/106 frames with 23 cues and 2343 rows, but only 4/5 persistent turns pass.
   A conflicting assertion following previous question turns is not written.
   Generated response bytes do not enter the writer source stream; fitting
   lacked the resulting question-to-next-assertion adjacency.

The first 28 renamed cases scored 28/28 answers and exact writes, but a shell
continued into that evaluation after the session-preservation guard failed.
No selection receipt was written. That population is therefore explicitly OPEN,
not post-selection evidence. The boundary error is retained in
`writer-binding-first-use-boundary.json`. It is not used for fitting.

The continuation source adds 28 construction examples with prior question
prefixes, preserving exact source labels with shifted byte endpoints and the
same runtime representation. A replacement reserved vocabulary is checked
against the supplied source and kept out of training. Fresh evaluation must
follow a successful separate selection check. This remains familiar grammar
with changed names, not sealed broad language or frontier qualification.

## Selected functional result and cost regression

Retain `8dbf136752b6b5f2e52a528dfff300a703503cd5d49fd78b4e66910c16efcc84`
as the functional development artifact. Preserve `8070c006` as the frozen
working reader and cost comparator; this is not a serving-speed promotion.
The 200-document continuation fit reaches 128/128 distinct construction frames
with 23 cues and 2505 writer rows. All reader parameters remain unchanged.

| Check | Result |
|---|---:|
| Original dependent development, complete answers | 48/48, versus parent 40/48 |
| Same development, exact owner/value/action writes | 48/48 |
| Earlier response preservation | 62/62 |
| Earlier transfer | 24/24 |
| Raw-window eviction cases, answers and writes | 28/28 both |
| Persistent session turns, including conflict | 5/5, with restore and isolation |
| Replacement reserved vocabulary, after selection | 28/28 answers and writes |
| Unchanged generated Rust functions | 4, compiled and 12 semantic assertions rerun |

The replacement reserved set was evaluated after the separate successful
`writer-binding-selection.json` check. No parameters changed after opening it.
Its eight replacement names are absent from the supplied prior source; its
construction forms remain familiar. The former accidentally opened set retains
its OPEN status. The actual CLI now answers ` Zurich.\n` after
`casket in elvin. elvin in Bremen. Now elvin in Zurich.` and a dependent
location question. The broader red-box/owner question still returns `Unknown`.

**The new writer loses effective NoWrite reuse on the longer-context cases.**
The inherited signatures remain safe under startup rechecking, but use the old
cue namespace. All 21,907 admission queries fall through, versus 21,795 skips
for the parent. On the same 28 prompts, writer row comparisons increase from
580,944 to 226,101,330; feature queries increase from 44,688 to 17,392,410.
Both artifacts commit the same 80 records, including 12 revisions and 12
conflicts, with the same expected answers. Recorded generation subtotals are
34,500 microseconds for the parent and 767,387 for this writer, around 22 times
higher. These are descriptive samples from different runs, not a controlled
wall-time speedup/slowdown benchmark. The exact work counters expose the
mechanism of the regression independently of timing noise.

The artifact is 11,045,747 bytes, including a 159,892-byte serialized writer
block and its frozen parent. The relation state remains 2840 bytes with
168-byte records. The actual loaded-artifact ingest/select/observe/copy test
reports zero allocations and zero allocated bytes; loading is outside that
census. Existing full-path counters and generated traces remain in the reports.
No whole-model efficiency advantage or expanded natural-language capability
is claimed.

## Next implementation and delivery

Within #1139, recompile the existing bounded exact NoWrite admission metadata
against this writer's actual cue dictionary and scorer. Bind and validate it
at that operator boundary, including exact guards; do not simply trust or
rename old unknown-key signatures. Reuse the existing compiler/cache mechanism.
Keep writer weights, reader parameters and exact payloads fixed. Require the
current answer/write/session results and a restored reduction in actual scoring
work on the same prompts. This is repair of a measured compute regression,
not a new cache search or benchmark programme.

Then resume the planned learned read/operator/derived-write composition using
existing typed operators. Full #1139 joint learned-block qualification and
#1140 broader multi-operation qualification remain unmet. Tokenization is
already reversible; broader learned lexical/phrase abstraction is still a
separate capability question.

Local verification compiled the release native path, ran five focused relation
state/geometry/cue tests, exercised the actual artifact's zero-allocation path,
and ran actual CLI and unchanged generated Rust code. Formatting, architecture
policy and claim checks accompany protected delivery. The machine-readable
[evidence](evidence/native_geometric_writer_binding_1139.json) records exact
resources and commands. CI/merge status belongs to the protected PR and live
GitHub; compatibility acknowledgements are not broad QA evidence.


## Subsequent NoWrite repair — 2026-09-06

The [writer-scoped cache repair](native_geometric_writer_admission_1139.md) now
retains `2600b95b` with this writer's parameters unchanged. It restores
21,799/21,907 exact skips and reduces the same long-context writer comparisons
from 226,101,330 to 539,448. The measurements above remain the history of the
uncached writer artifact; they are not overwritten by its successor.
