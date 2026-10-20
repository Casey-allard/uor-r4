# Corrected-writer NoWrite reuse — #1139, 2026-09-06

## Mechanism and scope

This repairs the compute regression recorded in
[native_geometric_writer_binding_1139.md](native_geometric_writer_binding_1139.md).
The corrected writer `8dbf1367` remains frozen: its 23 cue words, 2505 scoring
rows, H4/zeta role context, reader, source router, tokenizer and exact
payload/version semantics do not change. No training occurs in this step.

The existing compiler observes only the 200 construction prompts. At each
completed-word boundary it retains the length and ordered prime addresses of
up to eight words, using the replacement writer's dictionary. It counts exact
signatures, proposes other phases of exactly periodic full windows, certifies
NoWrite with the unchanged writer, and retains certified signatures up to a 256-entry
replacement-writer ceiling (the legacy limit remains 64). If the eligible set
exceeds that bound, construction frequency orders the selection. Proposed phases carry zero observed frequency and gain admission only after
exact certification. The rule requires an eight-word window with an exact
period of 1, 2 or 4; arbitrary windows gain no new phase. Responses and write labels do not select
the cache. Frequency is measured from construction text; this is compilation of
learned decisions, not new predictive learning or learned semantic geometry.

The writer now owns an optional admission field. Its parent identity binds the
complete corrected artifact with that field removed. Load validation checks
that identity, signature bounds/order/uniqueness, expected geometric metadata,
and every entry's NoWrite result under the bound writer. The old nested cache
and its lineage remain unchanged. An absent new field serializes identically
to the earlier format. A changed writer cannot silently inherit this cache.

Serving translates completed words with the same cue dictionary and consults
one cache. The replacement uses the previously selected sparse exact lookup,
with at most 256 entries and a length-plus-eight-prime equality guard. A miss
runs the unchanged writer. Legacy geometric/collapsed controls remain available;
geometry alone never authorizes skipping a different signature. Unknown words
still map to zero in writer features; their exact bytes remain in records.
Zero is safe to reuse only because the current writer decision consumes the
same signature, not payload spelling or evolving global state. This is no
claim that unknown words are semantically equivalent.

A hit costs dictionary lookup, nine signature field writes and a bounded
binary search with short-circuit exact comparisons. A miss additionally costs
up to 42 writer choices, each with 19 active feature queries and geometric
feature construction. Existing counters retain admission comparisons/metadata
bytes, writer dictionary/row work, H4/phase work, record updates and output work.
These are logical counts, not a machine-instruction or memory-bandwidth census.
No per-session storage is added; the existing relation state remains 2840 bytes
with sixteen 168-byte records. Artifact storage and load certification costs
are reported separately below. The executed inference kernel adds no matrix
operation, floating-point scoring, provider or allocation. Startup geometry
validation retains its previously documented floating-point boundary.

## Evaluation and decision

The first 64-entry artifact `59d71142` preserves 48/48 dependent answers and
writes, 62/62 earlier responses, 24/24 transfer and 28/28 long-context answers
and writes. It skips 11,151/21,907 long-context queries, leaving 10,756
fallbacks and 111,001,800 writer row comparisons. This is a partial repair,
not the selected final artifact. Its reports remain preserved.

Construction has 155 distinct signatures, 140 certified as NoWrite. The
initial diagnosis attributed the residual work to the 64-entry cutoff.
Artifact `3535134e` therefore retains all 140 within a 256-entry ceiling. It
preserves all response/write/name/session checks, but only increases skips to
11,159/21,907, leaving 110,918,808 writer row comparisons. This disproves the
cutoff as the main cause of the remaining repeated-work bottleneck. The actual
140-entry serving path measures zero allocations on its dependent-copy check.

Metadata inspection identifies the missing alternate phase of the exactly
periodic `[79,73,79,73,79,73,79,73]` construction signature. The four repetitions
in short construction produce one full phase; longer repetition also reaches
`[73,79,73,79,73,79,73,79]`. That key is absent from the construction cache,
rather than merely ranked below the cutoff. The compiler now proposes periodic
phases generically and certifies them individually. There is no serving rule
for `quiet sky`, no answer label, and no approximate NoWrite inference. The
proposal is a structural hypothesis; the exact unchanged writer determines
whether it may enter the cache.

**Retain `2600b95b` as the repaired development artifact.** It adds one certified
periodic phase, retaining 141 exact signatures. All learned parameters are
unchanged from `8dbf1367`. The complete artifact is 11,086,742 bytes, an increase
of 40,995 bytes including construction receipts; the 141 entry structs occupy
6,768 bytes, excluding spare vector capacity and receipt/allocator overhead. No session state grows.

| Same 28 long-context prompts | Frozen corrected writer | Repaired cache |
|---|---:|---:|
| Admission queries | 21,907 | 21,907 |
| Exact NoWrite skips | 0 | 21,799 |
| Writer fallbacks | 21,907 | 108 |
| Writer row comparisons | 226,101,330 | 539,448 |
| Writer feature queries | 17,392,410 | 41,496 |
| Writer candidates | 915,390 | 2,184 |
| Admission metadata bytes consulted | 764,561 | 2,986,575 |
| Committed records / revisions / conflicts | 80 / 12 / 12 | 80 / 12 / 12 |

Writer row comparisons fall **99.7614%**. The larger exact cache increases
admission metadata work; it does not erase lookup cost. The repaired writer
also uses fewer row comparisons than the earlier `8070c006` comparator's
580,944, while preserving the corrected writer's functional improvements.
The [evidence JSON](evidence/native_geometric_writer_admission_1139.json)
retains the complete nested work counters, including unchanged dictionary,
record, reader and output work. Text, bytes, token IDs, final geometric state,
copy traces and response-entry traces match the frozen writer on all 28 prompts.
Evaluation elapsed samples are 1632 ms for the parent and 62 ms for the final
combined check; report construction differs, so these are descriptive and
are not a controlled wall-time speedup claim.

Final preservation is **48/48** dependent answers and exact writes, **62/62**
earlier responses, **24/24** transfer, **28/28** exposed-name answers/writes,
**28/28** long-context answers/writes and **5/5** persistent turns with restore,
forged-commit rejection and isolation. Four generated Rust functions are
byte-identical to the retained semantic-test source; they are recompiled and
all twelve assertions execute successfully. Eight focused relation/cache tests
pass, as do formatting, the Rust architecture-policy check and claim wording.

The allocation census is zero allocations/bytes for the 140-entry artifact
`3535134e`, which uses the same serving code and 256-entry bound. The final
141-entry change is offline metadata construction; a dedicated final-artifact
allocation rerun is **NOT_RUN**. The CLI is rebuilt with this serving/validation
code, but a final-artifact CLI invocation is also **NOT_RUN**. Final behavior is
executed through the actual shared Rust `Model::generate` and persistent session
methods. These distinctions avoid spending another model load beyond the
remaining allowance or relabeling earlier tests as final-artifact runs.

This completes the measured NoWrite regression repair. Next, return to learned
admission of existing typed operands/operators before execution, commit the
selected derived value, and make it affect the next read/output decision.
Do not start another cache survey. Full #1139/#1140 composition and broad
language/frontier qualification remain unmet. All evaluation sets, including
the earlier reserved names, are already exposed preservation checks; no new
held-out claim is made.

## Reproduction and resources

Source, projection, artifacts, reports, monitored command logs and storage
receipts are under `.uor-models/native-typed-value-2026-09-05/` in the original
checkout, prefixed `writer-admission-`. The existing cumulative model ledger
is `.uor-models/native-joint-learning-2026-09-04/model-time.json`.


The cycle used **65.590 seconds** of model work, including both partial repairs,
retries, evaluation and generated-code checks. Its original 60-second local
allowance was explicitly revised to use the remaining inherited allowance;
the cumulative limit stayed **1800 seconds**. Final cumulative use is
**1796.744/1800 seconds**, leaving **3.256 seconds**. No complete next fit-and-evaluation cycle has a viable projection within that
remainder; establish one and obtain any genuinely missing cumulative-budget
authorization before execution. No new global allowance is implied by a new issue or turn.

Monitored engineering commands used 1117.187 seconds including the final documentation checks,
within the revised 1200-second cycle projection. Initial 64-entry and 140-entry
attempts, their diagnoses and all outputs remain preserved. A necessary
64 MiB storage increment was recorded under the owner's standing authorization.
The conservative final storage sample is 6,206,554,112 bytes under a
6,727,663,616-byte cap; the tighter stop retains 165,703,680 bytes of headroom
and the 128 MiB safety margin. Peak sampled known storage is 6,244,818,944 bytes;
peak sampled model-process RSS is 844,300,288 bytes under the 4 GiB target.
Samples are not exact transient-peak guarantees. No material was deleted and
no paid external compute was used.

The combined final command is:

```sh
native_geometric_value_probe writer-binding admission-verify \
  writer-binding-continuation-model.json writer-binding-continuation-source.json \
  writer-admission-periodic-model.json writer-admission-periodic-compile.json \
  relation-admission-source.json writer-admission-periodic-preservation
```

Run from the preserved artifact root through its cumulative resource wrapper.
The command is recorded for reproduction, not authorization for an unbudgeted
rerun. `writer-admission-checkpoint.json` and the checked-in evidence bind all
actual build/test commands, source hashes and artifact identities.
