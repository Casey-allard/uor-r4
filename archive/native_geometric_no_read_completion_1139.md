# Committed NoRead continuation — #1139 / #1140, 2026-09-06

## Cause and implementation

The accepted literal-admission artifact `c29ab982` declined arithmetic on four
unanswered location questions but failed every complete answer. Two outputs
began ` Unknown.` and continued into unrelated looping text; two selected and
copied `coins`. Native parent traces confirm actual NoRead/no-source-sentinel16 for the
first pair and Prepare/source11 followed by exact byte copying for the second. These are different failures: lexical continuation after an
abstention decision, and selection of an unsupported retained word.

This increment addresses the first failure with the existing response-entry
learner and integer token scorer. An optional `no_read_completion` artifact
component uses the existing response-entry table representation and a distinct
`uor-r4.native-literal-no-read-binding-completion/1` schema. It runs only
when an actual selected NoRead token has been observed, the response entry is
active, its committed read has no source, and at least one numeric source is
retained with no derived numeric records. Numeric NoOperation alone cannot
activate it. A selected word source continues through the unchanged byte cursor
and copied-word suffix. The complete inherited model is preserved verbatim.

Offline Rust fitting admits a document only when the frozen assembled parent
actually selects NoRead and its first token matches the supplied response.
It rejects relation-only and computed-result contexts outside that literal
training scope. It then learns remaining canonical lexical/byte tokens and EOS with the existing
sparse optimizer. Later tokens are teacher-forced during fitting; that score is
not free-generation acceptance. Whole-response and position bounds skip an
entire example rather than truncating it. A table with no positive offer preserves the full inherited prefix fallback
and its pending decision. IDs, raw prompt/response receipts,
configuration, exact parent CID, tables and resulting identity are bound.

No grammar or universal Unknown rule is added. The learned NoRead action still
chooses the first token; source selection and all numeric parameters are fixed.
This does not repair an unsupported source that the reader chooses to copy.

## Representation and cost boundary

No new session state is added. The existing word-copy prefix feature extractor
retains the query-boundary anchor, last two token identities, response-step
count, signed relative H4 pose/orientation and eight wrapped fixed-zeta phase
channels, plus up to16 candidate-relative binding-mask/preceding-word addresses.
It supplies at most32 sparse features. Known words use exact dictionary primes;
unknown identities collapse to the dictionary miss code. Binding masks retain
only their declared recent equality relationships and lose broader sentence
structure. These are reused construction-learned fields, not a new grammar parser.
The head retains learned next-token scores and postings, not a stored answer
buffer. Exact retained words and numeric values remain in their existing state.
The finite features do not retain arbitrary response history or establish a
semantic distance. Geometry can distinguish states without guaranteeing the
correct next token.

At an eligible continuation, the same bounded scorer reads at most16 candidate
tokens with four postings per matching row. The artifact cap is4096 rows and
32768 associations; this run's actual sizes and complete whole-path work belong
in the execution evidence. Existing parent scoring still executes, and its
cost must remain included. Source-model/provider access and runtime matrix
multiplication are absent from this operator. Artifact loading, validation,
encoding and checkpoint serialization remain separately measured host work.

## First candidate: continuation repair with a preservation regression

`760fc57b` used only the16 response-entry features. It repaired both looping
cases (exposed14/16 versus parent12/16), but shortened four earlier explanatory
abstentions, reducing preservation from62/62 to58/62. Other measured sets
remained48/48,24/24,28/28 and5/5. It is not accepted. Its56 teacher-forced
positions scored52/56; this was insufficient to preserve the full outputs.
The source, model, fit and complete reports remain retained.

The next revision reuses the existing32-feature word-binding prefix rather
than adding another selector. This restores access to query/source wording
that the smaller response-progress representation compressed away. A distinct
NoRead schema prevents silently interpreting the first candidate under the
revised feature law.

## Second candidate: restored short/explanatory responses, overbroad application

`235fad68` uses the32 prefix features and fits56/56 positions. It restores62/62
prior responses, retaining48/48 dependent,24/24 prior transfer and28/28 exposed
names. It is not accepted: the existing persistent relation-conflict turn emits
` Unknown` followed by repeated newlines, reducing that session from5/5 to4/5.
No fresh cases were opened during either revision.

The final correction restricts the new table to its literal-numeric training
scope. Relation-only contexts and any context with a derived numeric record
retain the entire inherited continuation. The structural check scans at most16
records, counts its metadata reads and adds no cached state. Selection within
the eligible path remains learned. This is not a general abstention model or a
claim that changing expected wording is equivalent to factual fabrication.

## Final execution

Retain `blake3:e7c14c994c57eaad6c9a765c71e7699c0f7f0c88a225aea71e6e84c8b7da0e11`
at literal-numeric NoRead-completion scope. The head is36,763 serialized bytes,
with149 rows and354 associations. Eight actual selected NoRead examples supply
56 continuation positions; all56 fit. The other95 of103 supplied construction
examples do not supply a selected, matching NoRead continuation. No rows or
associations are dropped. The final learned table/configuration equals
`235fad68` after excluding the changed operator schema: the final correction
is its state eligibility, not another parameter search.

| Complete generated scope | Parent `c29ab982` | Final `e7c14c99` |
|---|---:|---:|
|16 exposed literal cases |12/16|14/16|
|16 reserved changed-name/value cases |14/16|15/16|
|16 complete three-turn computations |16/16|16/16|
|8 identifier-return cases |8/8|8/8|
|103 construction responses |95/103|99/103|

All12 reserved numeric answers pass;3/4 reserved abstentions pass. The remaining
reserved failure and both remaining exposed failures copy `coins` as a location
answer. All four construction failures likewise select that unsupported word.
The new set includes two numeric worlds, source-order reversals and a shorter
location-question form; it is a bounded first-use comparison, not a sealed
language-capability qualification. Its cases were not evaluated until after
the final design selection.

Preservation passes48/48 dependent responses/writes,62/62 earlier responses,
24/24 prior transfer,28/28 exposed names,28/28 long-context,5/5 persistent turns,
6/6 earlier numeric, three12-case role sets,58/58 computed construction and
16/16 earlier independent-result transfer. The eight identifier-return outputs
are preserved; this increment does not claim new algorithm synthesis or new
execution of the prior24 Rust assertions. The rebuilt actual CLI emits
` Unknown.\n` for the repaired literal query and `zorin\n}\n` for the numeric-
distractor identity-function prompt, with EOS in both cases.

Thirteen focused tests pass, including actual fit/parent equality, no-offer
fallback, literal/derived/empty eligibility, selected-observation causality,
checkpoint restoration and invalid schema/parent/token/weight rejection. Both
the actual NoRead response and actual three-turn numeric path measure zero
allocations and zero allocated bytes. The entire actual parent artifact is
unchanged. The kernel source check passes. Broader dormant release QA is
`NOT_RUN`; no new angular-distance advantage is claimed.

For the same two repaired exposed prompts under the same32-token ceiling,
whole-path observed tokens fall192 to136, candidate evaluations16,871 to2,162,
and memory-composition comparisons592,402 to74,064. Word-selector row
comparisons fall19,840 to3,008; score lookups21,008 to3,532; dictionary byte
comparisons17,289 to4,081. Initial routing table reads remain1,092 in both.
These reductions follow correct early termination, not a matched per-token
speed claim. Input processing, inherited scoring, eligibility, dictionary,
state, selection and emitted-token work remain included in the retained reports.

## Resources and delivery

Model work is282.150s against the360s cycle ceiling and300s projection.
Cumulative use is3450.276/3570s, leaving119.724s; the pre-recorded cumulative
extension was+360s under standing owner authorization. All failures/retries
remain charged. Build/check work exceeded the initial500s projection; the
recorded engineering ceiling was extended to1560s for the corrections and final
CLI integration. Engineering totals1364.981/1560s. Peak sampled known storage is6,746,660,864
bytes and peak sampled child RSS is1,109,098,496 bytes (4GiB target). Details are in the
[compact evidence](evidence/native_geometric_no_read_completion_1139.json).

The CLI-only feature combination crossed the tighter storage guard and stopped;
subsequent evaluation was refused before execution. The scoped growth allowance
then increased160MiB under standing owner authorization, with a128MiB remaining-
work projection. The broad7,063,207,936-byte storage ceiling stayed unchanged;
the effective tighter ceiling is6,875,574,272 bytes. No material was deleted and
no external model compute was used. A wrong source-split argument, an EOS test-
interface correction, a stopped intermediate schema build and all candidate
revisions are retained in the command receipts; they are not model-quality
results.

Artifacts, sources, complete reports and append-only command receipts are under
`.uor-models/native-typed-value-2026-09-05/`. The accepted artifact is
`answer-entry-final/model.json`; prior candidates remain in
`answer-entry-candidate/` and `answer-entry-binding/`. The evidence binds exact
paths, SHA256 file digests, model CIDs, source code, commands and resource records.

Next revise joint source/NoRead selection for a retained word unsupported by
the question. Reuse the existing geometric source router and exact occurrence
binding, with supported-word/missing-attribute pairs and current preservation.
Keep the accepted numeric, identifier and memory paths intact. General syntax,
prose, reasoning, frontier capability and whole-model laptop advantage remain
unqualified; #1139/#1140 remain open.
