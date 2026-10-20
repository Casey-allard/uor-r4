# Text-to-clause specification and separate comparison — #1085

**Status: `CLAUSE_SEGMENTATION_SPECIFIED`; implementation, population preparation,
fitting and evaluation are `NOT_RUN`.** This is the deliverable for
[#1085](https://github.com/UOR-Foundation/uor-r4/issues/1085), based on
`a7f62b025c707640058e48721ef4971f8be789c5`. Protected delivery freezes the
interface and empirical decision below. [#1094, the parked empirical child](https://github.com/UOR-Foundation/uor-r4/issues/1094), must bind
implementation, independently prepared inputs and execution identities before
observation. This specification is not executable run admission.

## Decision and retained evidence

Remove one caller obligation: supplying five presegmented token clauses and
their lengths. `R4TextToClausesV1` accepts one bounded text buffer containing
four facts and one question and produces exactly the existing reader input.
It has zero learned parameters and performs no semantic answer lookup.
Punctuation and a restricted grammar still supply boundary cues: this is a
deterministic controlled-language adapter, not learned sentence segmentation.

Keep #1077's reader and #1073's core, their full soft role mixtures, null entry,
five binding scores and full 4096-token head. #1079 preserved 25,600 answers and
all 156 primary criteria; its valid token-frame control met the strong-drop
floor in only 3/6 views. Preserve `LANGUAGE_R4_PRESERVED_CONTROL_WEAK`, the strong
fact control, and earlier negatives. #1082's role-selective exposure is
descriptive; it does not establish segmentation as the cause of that difference.

No new lexical items, query forms, semantic combinations, fact counts, ordering
campaign, temporal updates, geometry, fitting, native export, generation or
lowering belongs to this task or its first comparison. #973 remains open and
its paragraph/conversation/bounded-global terminal unmet. #954 remains blocked.
The old route loop's qualified scopes do not transfer to this learned reference.

## Accepted identities and actual interface

These file/state identities are distinct from structural IDs. Verify the full
linked manifests and closures before future model access.

| Object | Fixed identity / source |
|---|---|
| Reader file / state | `blake3:c11d21817bff818fa242f653279e9e0c12d21641ff63df3a5f7a6680bcc732a7` / `blake3:7c659422df2e65a0ce24c08738dc9f08dca99775de1702251097a0fc6483404e` |
| Core file / state | `blake3:9c055cc6ea09548bf960e37288276535b30515b94a50a96aa929b5e55afea3c4` / `blake3:abbdbcaafc2d9eb36543ce75fbb0101b6788119d80a6ed9c017bb9d06fbeac59` |
| Lexical/data binding | [#1077 preparation](../r4_zoology_language_interface_1077_preparation.json), `blake3:0395b826049dbeed351a647960c7b66cc4d65fc19b65eb3c522fcdd807aaad69`; vocabulary file `blake3:571d5fbc282b17c8726eebd7b23c3ae55212a3de81b35d27722a0fa5979b8c5b` |
| Native frames and source closure | [#1079 preparation](../r4_zoology_language_r4_1079_preparation.json), `blake3:d9c8ad8448365b2039276fdeda6b70da53ef63fde24e02dd1dd8dea437b546a4`; frame tree `blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c` |
| Historical preservation / replay | [#1079 result](../r4_zoology_language_r4_1079_result.json), `blake3:dee107190172afcb7637d52469662ecab217847271e4bbdb0721514fcfbdc3a5`; [replay](../r4_zoology_language_r4_1079_replay.json), `blake3:eaa17433d5cd150a2a0c52adab6104bda4c4dae26221944fcde112ef841ca597` |

The lexical mapping is defined in
[`zoology_language_interface/data.py`](../../tools/r4-softmax-trainer/src/r4_softmax_trainer/zoology_language_interface/data.py)
and imported
[`zoology_english_binding/data.py`](../../tools/r4-softmax-trainer/src/r4_softmax_trainer/zoology_english_binding/data.py)
at the baseline revision. This is the controlled 4096-entry word codec, with
reader-only aliases 52–57 for `not`, `but`, comma, `owned`, `by`, and padding.
Input accepts existing task words/punctuation only; unused entries, special
tokens, literal padding and the answer token `unknown` are not input words.

The 141,571-parameter reader consumes int64 `inputs[B,5,13]` and `lengths[B,5]`.
Right padding is ID 57. Its radius-two convolution zeros padding embeddings;
role softmax masks only padding. All 15 soft role distributions are computed,
14 consumed. Role argmax is an evaluator diagnostic only. The 286,976-parameter
core stays fixed. See [model.py](../../tools/r4-softmax-trainer/src/r4_softmax_trainer/zoology_language_interface/model.py)
and [R4 attention.py](../../tools/r4-softmax-trainer/src/r4_softmax_trainer/zoology_language_r4/attention.py).
R4 folds every valid token, including punctuation, continuously across clauses
and uses clause-end frames. Padding is excluded; cuts and punctuation matter.

## Input, segmentation and refusal

**Definition — request.** Schema `uor-r4.text-to-clauses/1` plus one `text` byte
buffer, strict UTF-8, at most 4096 bytes. Refuse unknown fields. No caller clause
array, lengths, roles, targets, semantic record, row ID, prior state, retrieval
result or provider response is accepted. The adapter reads only those bytes and
the fixed lexical/syntax policy. Artifact verification cannot choose a parse.

Lex maximal lowercase ASCII words `[a-z]+` and individual `. , ? :` tokens.
Separators are ASCII space, tab, LF and CRLF; a bare CR is invalid. Words require
whitespace between them; punctuation may touch words. No Unicode normalization,
case folding, spelling repair, delimiter insertion or truncation. Preserve raw
bytes and half-open byte spans for every token. Non-ASCII, unsupported controls,
uppercase, digits or other symbols are refused as `INVALID_ENCODING` (the tag
includes unsupported character repertoire); unknown lowercase words are
`UNKNOWN_LEXEME`. Literal special-token spellings are also refused at lexing.

Split at exactly four literal period tokens, retaining each period in its fact.
The entire suffix is query clause 5, including `? answer :`; the question mark
does not terminate this clause. Its last non-whitespace token must be `:`, with
no supplied answer. Clause spans run from first token start to last token end;
inter-clause and leading/trailing whitespace are outside spans. Retain text
order, all token IDs and actual lengths. Never sort, deduplicate, fill missing
facts or select relevant facts. Exactly five nonempty clauses, each at most 13
tokens, are padded on the right; no BOS/EOS or synthetic separator enters them.

**Definition — admitted syntax.** Use the existing four fact forms and fixed
query form, shown with spaces between lexical tokens:

```text
O , not D , put the X in the L .
in the L , not D but O put the X .
not D but O put the X in the L .
in the L , O , not D , put the X .

where is the X owned by O , not D ? answer :
```

O/D are distinct members of the frozen owner list, X a frozen object, L a frozen
location. All four facts use the same form as in the accepted per-view policy.
These are grammar metavariables, not model fields. After delimiter splitting,
an independent boolean-only recognizer checks unchanged token spans against
literal/membership predicates. It returns only acceptance. Do not call
`_parse_clause`, which reconstructs semantic assignments and gold role positions,
or re-render a canonical clause. Grammar captures, category masks, role positions,
view/group metadata and answer constraints never enter inference. Every valid
token, including both owner names and punctuation, stays eligible for every
learned role. Known grammar and disjoint lexica remain explicit scaffolding.

The wrapper verifies artifacts at startup, before admitting requests; unavailable
startup returns `UNAVAILABLE_ARTIFACT` regardless of request contents. With a
verified wrapper, refuse before any reader/core forward, in precedence order: schema, byte limit,
encoding/lexing, boundaries/counts, clause length, syntax. Within a stage use the
first offending byte when meaningful. The future corpus pins one exact tag per
row; overlapping reasons follow this precedence.

| Tag | Required behavior |
|---|---|
| `UNSUPPORTED_SCHEMA` | Unknown schema/field, including externally supplied segmentation or state. |
| `INPUT_LIMIT` | Buffer exceeds 4096 bytes or recovered clause exceeds 13 tokens; never truncate. |
| `INVALID_ENCODING` | Invalid UTF-8 or unsupported character/control repertoire, including special-token punctuation. |
| `UNKNOWN_LEXEME` | Unknown lowercase word, including disallowed `unknown`. No substitution. |
| `UNSUPPORTED_BOUNDARY` | Missing/extra period, empty clause, wrong count, missing query suffix or appended answer. No guessed boundary. |
| `UNSUPPORTED_SYNTAX` | Unsupported/mixed fact form, equal O/D or alternative query; no ambiguity resolution. |
| `UNAVAILABLE_ARTIFACT` | Wrapper cannot verify reader/core/codec/frame/source identities; no fallback. |

Surface ambiguities outside the grammar are refused; this is not semantic
conflict detection. Syntactically admissible contradictory worlds remain outside
qualified behavior. Do not merge or resolve them, or claim `CONFLICT`/`CLARIFY`.

## Output, state and identity

**Definition — adapter success.** Emit `SEGMENTED`, schema/policy revision,
`raw_text_sha256`, five ordered clause spans, token spans, `inputs[1,5,13]`, and
`lengths[1,5]`. The derived tensor identity binds dtype, shape, little-endian
IDs/lengths, codec/policy and ordered clauses. Raw text and derived-input identity
are distinct: different whitespace surfaces intentionally yield identical model
inputs. Preserve the link to original bytes; normalized tokens are not a
bijection on text. Do not label all hashes κ or use commutative composition for
ordered clauses. The following transport-independent record types are normative; #1086 may later
bind an HTTP/native transport without changing these fields. `bytes` means an
actual byte buffer, not an already-normalized string. UTF-8 is checked here.

| Record | Exact fields and types |
|---|---|
| Request | `schema: "uor-r4.text-to-clauses/1"`, `text: bytes`; no other fields. |
| Adapter success | `schema: "uor-r4.text-to-clauses-result/1"`, `status: "SEGMENTED"`, `policy_sha256: lowercase hex[64]`, `raw_text_sha256: lowercase hex[64]`, `derived_input_sha256: lowercase hex[64]`, `clause_spans: [u32,u32][5]`, `token_spans: list<[u32,u32]>[5]` for nonpadding tokens, `inputs: i64[1,5,13]`, `lengths: i64[1,5]`. |
| Refusal | `schema: "uor-r4.text-to-clauses-result/1"`, `status: one refusal tag above`, `byte_offset: u32 or null`. No token, tensor, span, partial parse or answer fields. Artifact/schema failures use null offset. |
| Model success | `schema: "uor-r4.text-binding-result/1"`, `status: "MODEL_TOKEN"`, `policy_sha256`, `raw_text_sha256`, `derived_input_sha256` (all hex[64]), `reader_file_cid`, `core_file_cid`, `frame_tree_cid` (the fixed blake3 strings), `token_id: u32 < 4096`, `token: core-vocabulary UTF-8 string`. |

**Definition — hash framing.** Raw SHA256 hashes exactly the original bytes.
Let `L(s)` be u32 little-endian UTF-8 byte length followed by the string bytes.
The derived-input SHA256 hashes, in order: `L("uor-r4.text-to-clauses-input/1")`,
`L(policy_sha256)`, `L(the fixed vocabulary file CID)`, `L("i64le")`, shape
integers `1,5,13,1,5` as u32 little-endian, then all 65 input IDs and five lengths
as i64 little-endian in fact0/fact1/fact2/fact3/query order. Spans remain raw-byte
provenance; they do not change this tensor identity. The child's immutable policy
manifest must bind this specification's commit/digest, lexical artifact, syntax,
refusal precedence, limits and surface-profile rules. `policy_sha256` hashes its
exact published bytes; receipt bytes and digest are frozen before population
preparation, with no normalization or commutative composition.

Only inputs/lengths reach the model. The wrapper returns `MODEL_TOKEN` with the
actual full-head argmax ID, **core-vocabulary spelling**, artifact and input
identities. Reader aliases must not decode outputs: core ID 52 still means
`<unused-0052>`, not reader word `not`. Legitimate model output ID 11 is `unknown`,
the existing task answer, not a general calibrated abstention policy. Unexpected
IDs remain visible; do not apply a location filter, template, provider response
or fabricated refusal. Adapter errors have no model token and zero model work.
Richer answer/abstain/conflict semantics require separate #954 qualification.

Requests are stateless and atomic: read the complete prefix through the colon,
then infer once. No answer/future token is in the input. No update/rollback,
paragraph/conversation/global memory, evidence selection or persistence exists.
A request replaces the whole four-fact context. These are unmet #973→#954
requirements, not a revision of that terminal.

## Separate empirical comparison

**Empirical Criterion — question.** Does the sole adapter recover independently
annotated token clauses and preserve actual soft outputs of the accepted R4
reader/core while refusing malformed text without model work? Compare with the
same learned R4 path supplied oracle segmentation, not canonical hard-role
lookup. Zero fitting is permitted in this child.

Before outcome access publish an immutable preparation envelope binding this
specification's commit/digest, implementation/evaluator closure, codec/grammar,
raw corpus and independent reference digests, authoring/partition provenance,
reader/core/frames, runtime, execution plan and budgets. Missing bindings stop
`UNAVAILABLE_COMPARISON_INPUT`. Counts and floors below cannot be tuned to
preparation results. Implementation/preparation is the next task, not part of
#1085 delivery.

### Independent inputs

An independent curator who does not implement the adapter prepares raw text and
references without model access or reuse of production adapter, `_clause`,
`_parse_clause`, rendering or decode helpers. Use already-observed #1073
construction worlds, preserving original five-row groups (base q0/q1, swapped
q0/q1, absent q0), fact order and distractor assignments. Independence is text
preparation/annotation, **not semantic novelty or independence from training**.

Freeze 4 authoring and 16 withheld groups, half from each existing question
family in each partition. Within each family sort SHA256 identities of canonical
ordered group descriptions; take first 2 for authoring and next 8 for withheld.
Before generating text, the curator commits the exact description serialization
and selected IDs, without model-based filtering. Variants/views/surfaces of a
group stay together. They are related observations, not independent samples.

For each five-row group use all four existing fact forms and four spacing
profiles: compact punctuation/single-space clause joins; compact punctuation/LF
joins; space-separated punctuation/CRLF joins; alternating space/tab/LF token
gaps with single-space clause joins. Freeze exact profile byte rules before
corpus generation. No profile changes a word or deletes punctuation. This yields
**320 authoring and 1280 withheld valid rows** (groups × 5 × 4 × 4). Independently
annotate token/clause byte spans, IDs/lengths and 14 diagnostic role positions;
semantic targets stay evaluation-only. Production input contains only text.

Also freeze **16 authoring and 64 withheld refusal rows**: one/four per family
respectively. The sixteen families are invalid schema/extra field; oversized
buffer; invalid UTF-8; non-ASCII; bare CR; unknown word; literal padding;
missing period; extra period/empty clause; fewer facts; extra fact; overlong
clause; missing query suffix; appended answer; unsupported/mixed fact form;
unsupported query/equal O/D. Exact strings and expected tags follow the defined
precedence and are produced only in the future child. No model is used to pick
cases. These refusal rows do not expand qualified language scope.

Only authoring text/reference inputs may guide parser repair before final source
freeze; no model forwards or fitted parameters are used during that repair.
Withheld text/annotations remain outside implementer/model access until then;
record hashes without inspecting contents. Independent review checks provenance
and group separation. Allow one withheld evaluation and one fresh-process
replay. A revealed miss preserves the negative and requires a newly frozen
boundary population for any later repair. No retry, variant search or new fit.

### Arms, controls and numeric criteria

1. **O, oracle-segmented:** curator clauses/lengths enter unchanged #1079 coherent
   R4 execution.
2. **A, adapter:** raw bytes enter `R4TextToClausesV1`; only its inputs/lengths enter
   the same R4 model. Use identical row/batch order, dtype, parameters and frames;
   report adapter work separately.
3. **Boundary negative control:** remove the first period from 16 preselected
   withheld valid rows, one per fact-form/profile cell. Require boundary refusal
   and zero forwards. Do not feed an overlong segment or invent answer-drop floors.
4. **Oracle-leak control:** A's fresh-process replay has no reference/role/target
   files available; the evaluator joins those records only after inference.
   Compare complete deterministic digests. Source review verifies the boundary.

No new frame controls are run: #1079's unchanged weak token-control finding
remains historical evidence at its original scope. This comparison asks input
fidelity, not additional geometry attribution or sensitivity to arbitrary cuts.

Require **100% valid acceptance, exact IDs/lengths and token/clause span agreement**,
separately in each of 16 fact-form/profile cells of authoring and withheld.
Require **byte-identical** A/O reader attention, all 15 soft pooled role vectors
(including unused query location), binding attention, full-head logits and answer
IDs under the same deterministic runtime. All 14 diagnostic pointer decisions
per row must agree. Report O/A supported/unknown answer and role accuracy and
complete five-row groups separately; retain oracle mistakes in denominators.
Agreement does not establish new correctness. For each refusal family and the
16 boundary controls require **100% exact tags and zero model forwards**.
Fresh-process replay must reproduce all deterministic adapter/model/decision
evidence. Timing/RSS are recorded separately, not compared bytewise.

If the bound reference/runtime cannot reproduce its tensors, stop
`UNAVAILABLE_REFERENCE_REPLAY`; do not loosen equality. Preflight checks artifact
identities, raw-only access, annotation integrity, fixed counts/group separation
and authoring input/length tensor equality without model forwards before any
withheld model access. An authoring
adapter mismatch is `CLAUSE_ADAPTER_PREFLIGHT_MISS`, with withheld evaluation
`NOT_RUN`; provenance/reference readiness failures use the unavailable tags.

### Reachability, cost and actions

**Definition — reachability.** The adapter recovers input rather than adding
semantic information. Identical tensors entering the same deterministic model
produce identical decisions; adapter fidelity cannot improve the reference's
correctness by information gain on these rows. This conditional software
argument is not a proof of parser implementation, floating-point determinism or
language capacity. Expected benefit is removing caller segmentation; measured
adapter performance is currently `NOT_RUN`.

Use the already-qualified CPU plan: one process, four intra-op threads, one
inter-op thread, Python/Torch/Apple Accelerate as bound in #1079. Freeze exact
versions/hardware and batch size 128. Drift needs a separate plan justification
before observation, not an automatic accelerator/thread search. Upper bound:
1600 valid rows × 2 arms × 2 processes = **6400 logical row forwards**. Replay
may run both arms in isolated sequential processes if needed; only A's process
must lack oracle files. Refusals/control rows remain model-free. Batching does
not mean thousands of process launches. Authoring/reference preflight compares
input bytes only and adds zero model forwards. Each arm scores each of the 1600
valid rows once per execution/replay; authoring outputs are not scored again in
a separate preflight.

The larger #1079 run/replay cost 57.0422 seconds; estimate this bounded comparison
at under two minutes, subject to cold-load cost. Hard caps: 120 seconds preparation
integrity, 120 seconds execution, 120 seconds replay, **360 seconds cumulative**,
**3 GiB peak RSS**, **128 MiB new corpus/results**, zero downloads/optimizer
updates. Compare complete tensors while streaming one batch, then retain ordered
domain-tagged per-tensor digests and counts, summary metrics and bounded mismatch
details. Do not persist all four full copies: their f32 outputs alone exceed the
128 MiB cap. Byte comparison remains the criterion; a digest is the retained
receipt, not a relaxed tolerance. Reserve at most 8 MiB for failure excerpts and
retain inputs/identities needed for independent reproduction. Immutable start/terminal receipts bind consumed time, bytes, before/
after reader/core state and forward counts. Overrun is `INCOMPLETE_RESOURCE`,
with evidence retained and no renewed budget. Inventory disk and existing
artifacts first. Independent input authoring and code review are not timed model
work; report their resource use separately. No broad QA or large run is needed.

| Terminal | Required next action |
|---|---|
| `CLAUSE_ADAPTER_PRESERVED` — every integrity, isolation, refusal, fidelity and replay criterion passes | Admit this bounded text entry to the research reference; give evidence/schema to #1086. General transfer and #954 remain blocked. |
| `CLAUSE_ADAPTER_PREFLIGHT_MISS` — authoring acceptance/refusal/tensor fidelity misses before withheld access | Preserve supplied segmentation and the authoring failure; repair on authoring inputs and independently review/refreeze implementation before any separate execution. No withheld result exists. |
| `CLAUSE_ADAPTER_MISS` — complete valid comparison misses an adapter criterion | Keep supplied segmentation as qualified entry; preserve failures and separately specify an adapter repair on fresh boundary inputs. |
| `UNAVAILABLE_COMPARISON_INPUT` / `UNAVAILABLE_REFERENCE_REPLAY` | Repair provenance/reference readiness separately; no adapter/model verdict or automatic retry. |
| `INCOMPLETE_RESOURCE` | Preserve partial evidence, stop and reassess feasibility separately; unmeasured rows stay `NOT_RUN`. |

## Remaining stages and consumer boundary

These are staged #973 requirements, not activated runs or a second backlog.
Every empirical child needs independent inputs, matched controls, numeric
criteria and divergent actions/budgets before fitting/evaluation.

| Restriction | Separate next decision |
|---|---|
| Role ambiguity / vocabulary | Qualify contextual roles beyond current disjoint lexica and known cues; bind any changed codec/reader. |
| New semantic combinations | Freeze worlds disjoint from all historical fit/development worlds by semantic identity, not just wording. |
| Variable context/fact count and evidence selection | Define support/access/work limits and qualify retained context through actual decoded decisions and matched disabled/permuted state. |
| Temporal/conflicting updates | Define prior-state identity, ordering, conflict/abstention policy and conversation/global scope. |
| #954 intake | Qualify this artifact's paragraph/conversation/bounded-global behavior and final source-free execution; then apply #954's independent correctness decision. |

#1086 may specify export/loader while parsing is pending, but cannot advertise
raw-text support from this document. #1087 owns final integer/table lowering.
Dense reference success does not satisfy either final serving or higher scope.

## Source review and delivery

Public knowledge queries led to actual project source and pinned external
sources. The [source audit](clause-segmentation-1085-sources.md) records
NEMESIS/W33/UOR provenance and relevant constraints. None supplies empirical
parsing evidence or a proof of this adapter. This document adds definitions and
empirical criteria; it adds no model result or mathematical proof.

Activated delivery checks are independent contract/source review, claim wording,
changed-document link/JSON-reference integrity and `git diff --check`. They
decide specification consistency and readiness for protected delivery. Rust
builds, model loads, population preparation, fitting, evaluation/replay and broad
QA remain `NOT_RUN`. Queue acknowledgements are transport only.

Close #1085 after the reviewed contract, roadmap/claim pointers and parked
[#1094 empirical child](https://github.com/UOR-Foundation/uor-r4/issues/1094) are delivered through the protected PR path. Explicitly ingest
accepted records with revision, digest, visibility and evidence status. Storage
review preserves the original mixed checkout, unique model/corpus/diagnostic
evidence, sealed inputs and user material.
