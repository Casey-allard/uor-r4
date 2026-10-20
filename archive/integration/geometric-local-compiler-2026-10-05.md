# Local H4 turn compilation: learning and transfer remain separate

The source-only compiler now observes current and adjacent word states plus
ordered H4 transport. All six fits learn the training set. None qualifies a
trained checkpoint on the declared native development criterion. This is a
bounded component experiment toward the geometric compiler/store/emitter path;
it is not geometric chat or a geometry-family rejection.

## Mechanism and causal comparison

Original text is independently tokenized by word with the frozen R64 encoder.
Act/relation heads sum current/predecessor/transport observations over actual
word occurrences. The span head adds the next-word observation. Missing boundary
observations are masked; present identity roots remain observations. Native
scores add the bias once, preserve repeated root counts and use packed Q4 tables
with checked integer additions. Exact original UTF8 spans drive store writes.
No supplied selected record, target span, relation or answer enters prediction.

`LocalRelative` uses inverse(previous) × current; `LocalProduct` uses previous ×
current. They share information, slot counts and parameter shapes, but use two
versus one group-table operations per present lane. This is not a cost-matched
transport comparison. The local heads have27,849 Q4 coefficients versus12,489
for the older Endpoint heads: the historical comparison is not capacity matched.
Aggregated observations preserve counts, not an injective ordered sequence.

Offline Rust training uses hard packed-Q4 forward scores, a declared continuous
STE and Adam. Final numerical scoring uses integer/table/additive operations;
feature/adapter allocations remain explicit. Neither instruction-level complete
serving nor laptop energy is qualified. Legacy Endpoint artifacts retain their
mode/policy and zero initialization when new optional metadata is absent.

## Declared and completed experiment

Source `abb20ef7a9cfa30b08d0d207678e2a01e7e9df0d`, release executable SHA256
`a93c58cb657741e687a5d90d9e5a44c82840bc470a262e2fe7ae00f44680f35b`.
Both local modes ran actual head-initialization seeds1001/1002/1003: three
distinct initial packed payloads, paired identically across modes. They share
one frozen R64 carrier, so these are not independent encoder/chat lineages.

Each fit uses the unchanged152 training/44 development rows,64 full-batch
updates, learning rate0.03 and checkpoints0/16/32/48/64. Earliest strict native
development CE selects the checkpoint. The44-row new fresh panel was frozen in
committed source before prediction; its wording and four values were excluded
from the old compiler train/development/fresh panels, not claimed absent from
all historical encoder corpora. It is now exposed.

All six select step0. Every selected model has0/32 exact development writes,
0/32 fresh writes,0/4 exact-store answers on each split, and0/4 complete native
selected-record or all-bank replies on each split. Erroneous predicted query
acts make1–9 native calls per run across both splits; this does not isolate
reader quality. Selected prose false writes range1–4/8 on development and0–3/8
on fresh. Initialization is not useful memory safety or a deployment candidate.

All six retained step64 models get152/152 training actions and128/128 writes.
Their crossed diagnostics are useful, but were not the selection criterion:

| Transport / seed | Familiar wording, new values /64 writes | New wording, familiar values /64 writes | Joint development /32 writes | Repeated values /8 writes |
|---|---:|---:|---:|---:|
| Relative1001 |14|38|0|6|
| Product1001 |12|40|0|8|
| Relative1002 |18|26|2|4|
| Product1002 |14|31|0|8|
| Relative1003 |10|42|2|8|
| Product1003 |16|32|0|4|

The previous Endpoint step64 achieved1/64 and13/64 on the same crossed diagnostic
panels. Local results support retaining the mechanism, with the capacity/seed
confounds above. There is no consistent transport winner or joint transfer gain.
Familiar-wording/new-value write-act accuracy is only14–30/64; new-wording/known-
value span accuracy is47–60/64. The failure is broader than span truncation.

## Instrument and evidence boundaries

The authored reference compiler passes44/44 development and44/44 fresh actions,
including32/32 writes, with no false prose writes. The earlier independently labelled reference control executes store4/4 with
native selected-record2/4 and all-bank0/4, as the preceding audit records;
these six runs do not repeat reference-store execution. The current reference
is an action/span instrument positive control, not a learned mechanism.
Actual learned predictions use original tokenization/casing, natural questions,
original statement cues, serialize/reload after writes, and separately report
selected-record versus all-bank admission. Gold labels score outputs only.
No new consumer fit or shared session edit occurred. The previous consumer's
supplied-record and authored-cue curriculum findings remain in the
[native compiler audit](geometric-native-compiler-2026-10-05.md).

All ten build/test/model phases exit0; all six have five complete checkpoints.
Incomplete/mismatched attempts are not included. Saved artifacts, float shadows,
rows, native traces and receipts are retained. Three integer, seven training and
two driver tests pass, including bias/mask/count boundaries, real seed identity,
canonical noncommuting transport, continuous-STE finite differences, fresh
exclusion and positive-control execution. Release build, touched-file formatting
and diff checks pass. The finite-difference test is for the declared continuous
extension, not a derivative of discontinuous Q4 rounding.

CPU-only approved Runpod Linux x86:48-thread cap, build860.733s, model workers
13.421s, terminal wrapper874.820s, maximum sampled process-tree RSS4,015,316,992B.
Metal tests are unavailable and not counted. Owned pod storage is8.0GiB, results
35MiB, with no active Codex job. Same90-minute card charged once; cumulative
1,261,635,028/1,262,400,000ms. Preparation projection300→1200s was corrected before
build; source work had exceeded the initial300s projection, which is retained
explicitly. No new GPU/paid host or local heavy build was used.

Full returned tar SHA256:
`1c9cef52c4472bc851f604980ce4f62e2d652151cf39bf22b41f30394e0dbc1a`.
[Machine evidence](../evidence/geometric-local-compiler-2026-10-05.json) binds the
six summaries, receipts and independent saved-artifact review limitations.
The independent saved review passes1,421,046 checks: all30 shadow-to-native Q4
payloads, selected artifact equality, seed initialization and complete/matched
inputs. It does not rerun native inference, BPE, encoder, optimizer/backward,
BLAKE3 validation or recompute development CE from raw head scores.

## Next causal change

Keep the local compiler and independently vary wording and arbitrary values in
Rust-authored training. Reuse the existing153 job/home paraphrase rows from the
two retained corpora, with independent value pools across roles and frames;
the existing helper cycles the old value pool and must not be reused unchanged.
Balance assert/correction/query/prose so value identity cannot substitute for
intent. Freeze development conjunctions and new fresh wording/value combinations
before updates. Continue the same original-byte-span and actual predicted-store
instrument with three genuine head seeds and baseline-inclusive selection.

This is a curriculum intervention, not another unchanged dose or a claim that
curriculum is proven to be the sole cause. A capture register remains conditional
on correct acts/starts with measured span truncation. Natural statement/query
cue learning is a separate later reader boundary; authored gold cues cannot
repair it by decree. Shared GroundedSession remains untouched and requires#1552
coordination. General Generate, prose, reasoning, full-history attention and
laptop performance remain programme obligations.
