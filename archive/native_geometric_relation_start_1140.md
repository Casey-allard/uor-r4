# Learned relation-start selection — #1139 / #1140

## Implementation and projection — 2026-09-07

The [reverse-endpoint path](native_geometric_reverse_spans_1140.md) retains
complete bounded reverse values, but chooses the longest admitted prefix.
`Records show Orin Grove` therefore includes the introduction even though the
correct shorter span is already available. This is a selection failure, separate
from the known inability to preserve a two-space gap.

This continuation learns a signed-H4 score over the same admitted reverse
candidates, including the singleton endpoint. The writer-selected endpoint,
edge admission, byte transport, versions and all parent parameters stay fixed.
Seven ordered features describe the first word, next word inside the span,
predecessor, preceding gap, two shape pairs and singleton status. Shape classes
are explicit ASCII categories (absent, lowercase, title case, uppercase, mixed,
digit-bearing and underscore-bearing). They are inputs to learned codes, not a
hardcoded phrase decision or semantic interpretation of hash bits. Exact word
identity is not a new start feature. This is a bounded text-shape hypothesis;
lowercase values can remain ambiguous with introductory words.

The existing SourceRouting learner selects two-lane H4 codes and a shared
landmark using raw construction prompt/response examples. Labels identify the
complete target payload only during offline training. Inference scores admitted
starts using integer/table operations, with longest-first tie handling preserving
the parent's decision on ties. No matrix product or expert gate is added.
An outer artifact component binds the complete `321e990f` parent and training
receipts. Persistent reverse reads honor the selected endpoint even for a
singleton while its source remains visible, preventing a second query-side
extension from overriding the committed value.

The cycle projects 60 seconds model work and 600 seconds engineering, with caps
of 80 seconds model, 900 seconds engineering, 28 MiB sampled storage growth,
4 GiB model RSS, one model process and one build job. It uses the existing
82.631-second balance; the cumulative ceiling remains 4,410 seconds. Context is
512 tokens, retained records 16, payload 28 bytes and response limit 32 tokens.
The existing cumulative command guard charges all attempts; no external model
compute or broad cleanup is planned.

Eight new construction examples contrast `Notes say` / `Log tells`, bare versus
introduced phrases and multiword versus singleton values. Eight open examples
use different introduction wording, names and values. The opened `Records show`
diagnostic and former fresh panels are excluded from fitting. The same combined
Rust run checks selected earlier preservation, records a decision, then executes
eight new familiar-template fresh examples after source eviction. Old intro/gap
and a lowercase-value case remain diagnostics. A focused actual-artifact test
covers short, evicted and singleton outputs, malformed model rejection,
checkpoint prediction replay and measured allocation behavior.

## Measured decision — preserve candidate, do not promote

The one fit produced `blake3:bb6b8ba41f65b0c9d5c7ddf453e765a52b5ea8b169e50141ead05b3b5cefad79`.
Its 20 learned codes fit all eight construction frames; the learner considered
3,118 proposals and changed two code roots. The complete parent parameters are
identical after removing the new block and recomputing the outer identity.

| Complete generated answers | Parent | Candidate |
|---|---:|---:|
| New construction | 2/8 | 8/8 |
| Open development | 2/8 | 6/8 |
| Retained construction preservation | 663/663 | 663/663 |
| New fresh panel | NOT_RUN | NOT_RUN_SELECTION_FAILED |

All other selected preservation passed, including 18 prior forward turns,
18 prior reverse turns, three reverse short reads, 28 long-window and 48
dependent answers, the prior 14/6/9 source-span panels, versions and isolation.
Parent preservation values above refer to the previously retained evidence;
this run replayed the candidate preservation and directly compared both models
on the new construction/open inputs.

The two open failures are `A ledger says Pearl Cove holds tesvi` and its
single-word `Pearl` variant. Outputs are ` says Pearl Cove.\n` and
` says Pearl.\n`. Every nonidentity code is a predecessor-shape feature:
lowercase maps to roots [24,119] and title case to [119,77], with identity 119.
All first-word, next-word, pair, gap and singleton codes remain identity. Since
`says` and `Pearl` both follow lowercase words, this artifact assigns them
the same state and score; longest-first ties retain `says`. This diagnosis
follows the saved parameters and shared encode/score source, not a second model
fit. The construction examples permit that shortcut. The available first/next
and ordered-pair features can distinguish these starts, but this fit did not
learn to use them.

The selection receipt was written before the early return. No fresh-source
file, fresh response or post-selection intro/gap/lowercase diagnostic was
executed. The later artifact check uses opened construction cases, including
an evicted repetition, so it does not consume the reserved panel. Keep the
candidate and all receipts; `321e990f` remains the accepted artifact.

## Next implementation

Add independently authored construction contrasts where wrong and correct
starts have the same predecessor shape, including multiword introductions and
lowercase values. Reuse the existing ordered shape pairs and H4 learner first.
Use the opened errors as development evidence, and retain a distinct final
comparison after selection. Do not solve the examples with vocabulary rules or
silently promote partial transfer. Wider separator retention is a separate
representation task; additional paging or expert gates do not address this
observed selection shortcut.

## Verification and resources

Release probe and test binaries compiled. Two start-feature tests, five existing
span arithmetic/state tests, the scoped integer source guard and architecture
policy check passed. The actual `bb6b8ba4` artifact rejected invalid root and
parent mutations, generated opened short/evicted/singleton answers exactly,
replayed predictions from every checkpoint and made zero measured allocations
during ingestion and prediction/observation. This mechanical result does not
reverse the failed open selection.

The 41 sampled uncached prediction/observation steps had median 458 ns and
maximum 156,959 ns. This excludes loading, encoding, ingestion and checkpoint
host work. **The new start scorer runs during ingestion**, so these samples do
not measure its latency or establish a submillisecond complete model. Energy,
root CLI execution for this candidate, Studio and broad release QA are unmeasured
or `NOT_RUN`. The existing unused `NUMERIC_PROMPT` fixture warning remains.

Model command time was 60.167 seconds (44.874 combined evaluation and 15.293
artifact check), within the 80-second cap. Cumulative use is
4,387.536/4,410 seconds, leaving 22.464 seconds. The ceiling did not increase.
Peak sampled model RSS was 986,791,936 bytes; peak sampled cycle storage growth
was 20,692,992 bytes, within 4 GiB RSS and 28 MiB growth caps. Final sampled
storage growth before delivery metadata was 14,753,792 bytes. Sampling is not an
exact peak or unique-extent guarantee. No source, research, model or other
material was deleted and no external model compute was used.

The [evidence receipt](evidence/native_geometric_relation_start_1140.json)
contains artifact/source/data hashes, complete comparison outputs, preservation
counts and actual model/engineering commands. Build/test commands used one job
and the pinned offline Rust release profile. The fresh panel is preserved in
probe source and remains unexecuted. The remaining model balance cannot cover
a complete repeat at the measured cost; do not reset the ledger or silently
increase storage/model ceilings. The five automated PR/merge statuses are
transport acknowledgements only; the local commands are the test evidence.

Final local engineering commands total 276.856 seconds, including formatting and
claim-wording checks, both PASS. The final sampled growth after those logs is
14,761,984 bytes; the 20,692,992-byte observed peak above remains the larger
measurement. Artifact size is 11,634,162 bytes.
