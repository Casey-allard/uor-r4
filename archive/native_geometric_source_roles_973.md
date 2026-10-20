# Native source-role refinement for #973

Status: **retained at bounded source-selection scope**. The [evidence receipt](evidence/native_geometric_source_roles_973.json) binds actual artifacts, commands and resources.
This record concerns a bounded source-selection correction in the UOR-R4
Geometric Language Model. It does not qualify general instruction understanding,
general prose, reasoning, alpha capability or energy savings.

## Observed failure and selected change

Retained parent `91ede422` can copy a retained word or place and append the
learned endings ` is the place.\n` and ` is the stop.\n`. Four absent-owner stop
requests nevertheless select somebody else's fact. For example:

```text
Record: tovin in Lodov. Where is velra? Explain the stop in a sentence. Answer:
```

The required response is ` Unknown.\n`; the parent emits an unrelated place.
The paired present-owner prompt replaces `tovin` with `velra` and must continue
to emit ` Lodov is the stop.\n`. The source selector already admits NoRead;
this failure does not require a new abstention action or an output exception.

The position diagnostic separates this read failure from the writer problem
below. Across 40 authored Cedar/Amber cases, the parent fails eight absent-owner
cases: four stop-format requests and four unrelated six-word insertions. One
unrelated-word example is:

```text
Record: tovin in Cedar. Where is velra? farnel vopra duskin zemer pelka norvi. Answer:
```

It emits ` Cedar.\n` despite the absent owner. Moving the same six words before
the question also reproduces the error. These controls support a source-position
bias diagnosis; the stop-instruction meaning is not necessary for the failure.
The unrelated words are diagnostic inputs only and are not serving rules.

The retained source router has exactly one nonidentity kind-6 recency code:
feature `(kind=6, a=10, b=0)`, code index 137, with H4 roots `[86,119]`.
The geometry identity is 119. The bounded offline learner changed this code to
`[119,119]`. It removed the existing positional bias while preserving the
owner-sensitive source features and the already-admitted NoRead choice.

## Implementation and identity boundary

[Source-role refinement](../crates/uor-r4-core/src/native_geometric/source_role_refinement.rs)
constructs training alternatives using the existing candidate admission,
ordered feature extraction and source/action order. `Preserve` labels the exact
parent source occurrence and action; `NoRead` labels the existing no-source
action. Labels are offline supervision, not observations supplied to serving.
Persistent/dependent paths that bypass this direct selector are reported as
skips for preservation; a NoRead-labelled bypass is an error.

Only parent kind-6 codes that already contain a nonidentity root are mutable.
The mask and artifact validator agree on that rule. Existing neutral recency
codes cannot acquire a new positional rule. Code identities, other roots,
landmarks, biases, ranks, training metadata and the serving configuration of the
source router remain exact. All other model components, including the writer,
word-emission dictionary and parameters, numeric operators and geometry, remain
bound to the parent.

The optional `source_role_refinement` witness stores the complete previous
source router, parent CID, actual fit configuration and document receipts. Its
loader restores the entire source router, checks the reconstructed parent CID,
validates the parent recursively, then checks the permitted current changes and
current identity. It precedes the word-emission witness in validation.
`SourceRoleRefinementDisabled` restores the previous source router with the same
context feature law; this is the matched parent-parameter control.

Inference continues through the shared two-lane H4 table router and signed
angular scoring, followed by exact occurrence copying or the inherited NoRead
response. Prime identities remain addresses, not semantic distances. Fixed
geometry, orientations, zeta identities and other artifact-bound mechanisms are
preserved; this experiment does not separately establish their predictive
advantage. No matrix-product inference, transformer, provider, instruction
parser, phrase exception or filler-word patch is introduced.

## Executed qualification

Candidate CID: `blake3:82662f856908e548a9cccbb78809daf4821d243781ad497b661f5ee6d94c2032`; SHA256 `70a5f8867d00a358b2d64556be1ea038c844c7926dfb6388ab01ceb5746ba0ae`; 13,432,797 bytes. Retained locally at `.uor-models/native-typed-value-2026-09-05/source-role-refinement/model.json`.

The fit used 761 documents and 505 direct-selector frames, with 256 explicitly skipped bypass/ineligible contexts. Decision-frame correctness rises from 497/505 to 505/505 with one accepted update over 479 proposals; fitting stopped normally. The fitted frame result is separate from actual generated behavior below.

| Actual behavior check | Candidate | Parent or matched control |
| --- | --- | --- |
| Open position contrasts | 40/40 | 32/40 with parent router restored |
| Original absent-owner requests | 10/10 | 6/10 on retained parent |
| Post-selection fixed-form names/values | 32/32 | 20/32 with parent router restored |
| Retained construction | 663/663 | Preserved |
| Word construction / open / prior transfer | 132/132, 28/28, 24/24 | Exact source commitments preserved |
| Matched histories | 12/12 | Every record/version/pose/phase field equal |
| Earlier semantic/control reports | 14/14 | Exact outputs and decision identities equal |
| Native API checks | 166/166 | Includes actual answers, next independent turn, checkpoint import and corruption rejection |
| Focused tests | 29 unique passing | Includes source guard and six actual allocation/checkpoint checks |

Fresh preparation occurred after the recorded selection, with no later fitting. Its four owner/value groups are absent from this cycle's fitting prompts; the four prompt forms were already open. This measures identity transfer under fixed forms. The observed parent-router intervention restores the eight development failures and twelve fresh failures; exact source commitments remain unchanged on previously correct cases.

Actual generated Copy/Add equality checks and four prior fresh Rust identifier-return fragments were extracted, compiled and executed. General Rust synthesis is unqualified. The two known 33-step output targets remain 0/2 with exactly preserved wrong bytes and EOS behavior under the existing 32-step entry limit. Original owner-first composition is not addressed. New-artifact browser/WASM/HTTP, a blanket full suite and Clippy were not run.

The focused three-case word test measured 25.061 seconds for artifact loading and zero allocation across the measured ingestion/begin/predict/observe operations. Its warm predict/observe median was 144.083 microseconds, excluding load, encoding, session creation, BOS, checkpoints, JSON, decoding and reporting. This is scoped optimized execution evidence; complete-path latency and energy advantage remain unqualified.

The first position execution used a stale evaluator that omitted the new parent-router control. Its full-generation result is retained separately. The control list was corrected, the driver rebuilt, and the final position run executed the intended intervention. No model change or refit followed this driver correction.

This cycle used **1372.133 model seconds** and **497.487 engineering-command seconds**, including diagnostics, fitting, all controls and actual validation. Cumulative model use is **20483.859 / 22350.000 seconds**. The necessary 1,800-second extension was recorded before use; five minutes of unused fitting allowance were later reassigned to preservation without raising any total limit; the preserved 196.620-second reservation leaves 1669.521 seconds. Conservative storage is **35,905,552,384 / 37,010,698,240 bytes**, including retained material and a 16 MiB metadata reserve, with 970,928,128 bytes remaining after the 128 MiB stop margin. No storage extension was needed. Limits remain one model process, two threads, 512 context tokens, 96 output tokens, 4 GiB model RAM and 6 GiB build RAM. Sampled model/build process-tree peaks are 2,775,465,984 / 2,372,124,672 bytes. No unique deletion or paid external compute occurred. Small metadata/document/Git commands are outside the engineering stopwatch.

## Unresolved writer representation collision

The writer issue is distinct and remains unchanged. Two actual input traces are:

```text
Where is selvi? Explain in a sentence. Answer:
Where is selvi? velra in Dusk Ridge. Answer:
```

At the inspected proposal, the first input presents owner `Explain`, value `a`;
the second presents owner `velra`, value `Dusk`. The instruction and fact produce
exactly the same 21 Assert feature keys for owner-slot 2/value-slot 0. Both
receipts bind those keys to:

```text
875abdee54cf2b7b71c669e3bace99ba25ad14ba54111d749023ebde1b82d066
```

All compared proposal scores and the cache decision also match: Assert scores
2 against NoWrite 0. This is an exact local representation collision, not merely
a similar aggregate score. The diagnostic includes actual input-token state
changes separately from read-only proposal rescoring; a proposed write is not
by itself proof that the serving path committed it at that boundary.

Changing weights over these same keys cannot distinguish this particular
instruction/fact pair. The next implementation should expose a bounded,
contextual lexical or role distinction to the learned writer, then train and
measure the shared write/NoWrite choice on matched instruction/fact contrasts.
Preserve exact owner/value occurrence identity and test actual committed writes,
not only proposed scores. Any candidate must retain the present read correction
and earlier copy, abstention, conflict and numeric behavior. No hardcoded
`Explain` exclusion, sentence parser or filler-word rewrite is implied.

## Evidence and preservation

The current local evidence directory is
`/Users/casey.allard/Documents/Codex/2026-09-08/uor-r4-instruction-fact/`:
`diagnostic/`, `positions-parent/result.json`, `positions-candidate/full.json`,
`writer-collision/{instruction,fact,comparison}.json`, `fit/fit.json`, the
candidate artifact, pinned source receipts and resource records.

Preserve parent `91ede422`, its complete `2dc63b2c` lineage, all prior word-emission
candidates and negative receipts, including rejected `9e8d9cfb` and its retained
Rust/ledger regressions. The new fit does not erase historical charges or change
those results' scope. The local cumulative ledger and storage stop margin remain
authoritative; the completed evidence receipt contains the resource reconciliation.
