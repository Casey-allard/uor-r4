# User-input access in the native bank path

## Question, scope and correction

The owner asked whether native geometry can compare back to the beginning of user input to predict the next token. **It can support that operation.** This review locates what the present M2 caller actually supplies and reads before prescribing a new mechanism. Parent source: `65f2eb0468b50004dc0482c12e5671d5f6869e0a`, including #2088 and #2090. This is a source/caller audit, not a new artifact evaluation or an experiment on a language split.

The earlier chat answer was too broad if read as saying that user input has no geometric access or that the entire project lacks all-position reads. M2 encodes every supplied context/query/prefix token and also has query-specific cue and continuation paths. Its explicit occurrence candidate population is narrower. M1's stack attention and its registered exact-lane/Hamming work are separate and must not be erased by an M2 diagnosis. The source-only candidate set is a design boundary, not by itself proof of the cause of the accepted parent's 8/512 result.

## Actual caller and information flow

All source links below are pinned to the audited revision.

| Boundary | Implemented behavior | Consequence |
|---|---|---|
| [continuation_snapshot](https://github.com/UOR-Foundation/uor-r4/blob/65f2eb0468b50004dc0482c12e5671d5f6869e0a/crates/uor-r4-training/examples/geometric-frozen-map-fit.rs#L4765) | Converts explicit chronological Source/Context packets and query IDs into a pinned bank. Requires an actual Source bank for this fixed panel. | The M2 experiment does not run a text-to-memory compiler or automatically promote query tokens to Source records. |
| [admit_bank](https://github.com/UOR-Foundation/uor-r4/blob/65f2eb0468b50004dc0482c12e5671d5f6869e0a/crates/uor-r4-core/src/native_geometric/learner/native_bank_generate.rs#L501) | Checks scope, IDs, Source identity, source-emission views and preceding nonempty actual Context. | Source provenance and caller-supplied context are separate authorities. |
| [prepare_bank](https://github.com/UOR-Foundation/uor-r4/blob/65f2eb0468b50004dc0482c12e5671d5f6869e0a/crates/uor-r4-integer/src/geometric_occurrence_read.rs#L898) | Replays all supplied segment tokens, the complete query and actual response prefix in order, retaining intermediate steps. Enumerates only Source occurrences. | Early input can affect cumulative state; Context/query positions do not independently enter this occurrence candidate list. |
| [score_indices](https://github.com/UOR-Foundation/uor-r4/blob/65f2eb0468b50004dc0482c12e5671d5f6869e0a/crates/uor-r4-integer/src/geometric_occurrence_read.rs#L1055) | Compares the final query snapshot with the causal output code at each admitted Source position through the existing geometric potential. | The direct comparison and weights are Source-position scoped, rather than an all-input self-read. |
| [cue preparation](https://github.com/UOR-Foundation/uor-r4/blob/65f2eb0468b50004dc0482c12e5671d5f6869e0a/crates/uor-r4-integer/src/geometric_cue_carrier.rs#L472) | Encodes the complete query separately, plus each Source's preceding Context, and adds directed relative H4 contributions to Source scores. | Query exclusion from Copy candidates does not imply query exclusion from geometric computation. |
| [native generation](https://github.com/UOR-Foundation/uor-r4/blob/65f2eb0468b50004dc0482c12e5671d5f6869e0a/crates/uor-r4-core/src/native_geometric/learner/native_bank_generate.rs#L552) | Uses the same Source candidates for Copy and optional bridge-donor selection; the bridge changes the state used by Generate. U additionally encodes query plus actual prefix. | Input affects both selection and output scores, but a query-only occurrence cannot independently be a bank donor. |

The admitted total is **128 tokens across segments + query + actual prefix**, checked with a recoverable error rather than hidden truncation. This number is a bound in this reader, not the project's universal context length. Recurrent state width, total admitted sequence length, source candidate count and generated answer length are different quantities.

The selected `ContinuationParent::generator` loads the bank generator with query-conditioned donor selection disabled: its bridge donor is the earliest maximum of summed Source BASE scores, before U. Cue still incorporates the query into those scores. U itself replays query plus response prefix, not the Context segments separately.

The Stack grounded session's learned turn compiler can write predicted value spans into exact storage. That is not this bank experiment's caller. Likewise `r4-native-chat` Historical/GeometricProse dispatch is not interchangeable with `NativeBankGenerator`. Neither caller is evidence that every M2 user token has been exposed as an independently addressable key.

## Consolidation boundary and decision

**KEEP** the existing all-input recurrent, query-cue and continuation influence, plus exact Source/Copy provenance. **REJECT** the proposed shortcut of relabeling the whole user query as stored Source: that would conflate reading context with authority to emit a stored value. No such code change was made.

An explicit read-only history role is a plausible missing interface for the owner's requested operation: typed event/position identity and causal state for a retained Context/query occurrence, distinct from Source record/version identity and Copy admission. Read comparison, donor selection and emitted token actions must remain separate. Existing H4 relation tables, retained states and native potentials are the reuse points; no E8 basis conversion is required merely to expose those positions. If E8/SpiralCore actions are transferred later, their basis/metric and inverse witnesses remain necessary.

**NOT YET PROMOTED:** neither that interface nor a language gain is implemented or measured by this review. Absence of a per-position key does not establish that the recurrent/cue paths lost the required information. No representation collision, broken gradient, geometric advantage or remedy for the 8/512 result follows from source inspection. The protected-margin API from #2088 and the measured protected-direction conflict from #2084 remain valid and unresolved.

## Decisive next comparison

Before replacing the current learning task with an all-input reader, use the retained actual parent and a paired earliest-query-token intervention with identical suffix, source records, actual response prefix and operator parameters. The changed early token must alter the requested answer in a prospectively specified way. Record its admitted position, cumulative/query-cue/U states, all Source comparisons, donor, complete Generate/Copy masses and actual next-token choice. This intervention changes recurrent, cue and U paths together; retain separate witnesses rather than attributing an output change to one path. A score change alone is not correct use of the evidence. A later history-reader comparison requires the unchanged parent and an explicit history-access ablation. Existing source-swaps and correction/order controls remain relevant; no fresh acceptance panel is exposed during design.

If early input is dropped by the caller, repair admission. If it is encoded but the task-relevant distinction is erased before consumed scores, implement and compare read-only occurrence access against the unchanged parent. If the distinction reaches consumed scores but the wrong token still wins, repair scoring/learning at that observed boundary, including the existing protected-margin integration where justified. These are causal outcomes, not answer-specific serving branches. After a positive construction, immediately test actual generated answers and EOS under the saved model's own prefix.

## Verification and resources

Engineering independently traced the callers; adversarial review confirmed the source finding and required the intervention attribution limits above. No model was loaded, trained, graded or generated. Existing tests were inspected, not rerun: `bank_interleaved_context_preserves_local_offsets_events_and_candidate_keys`, `occurrence_reader_repeated_tokens_keep_exact_offsets_and_exclude_query`, and `bank_generation_chronology_and_source_identity_preserved`. No Rust source changes or compile claim.

Geometry documentation and the matching flow figure distinguish the current M2 path from M1. The geometry verification executes **29/29 PASS**; all 14 figures are regenerated, including the new M2 flow diagram. The existing 600-cell SVG changes element ordering only (identical line multiset), with no altered coordinates or content. Claim-wording and diff checks pass; exact-head repeats are recorded on the delivery PR. These check mathematical connections/rendering and selected wording; they do not verify learned language behavior. Exact outputs are retained; protected-merge and preservation receipts will be recorded after actual delivery.

Projection: 30 minutes preparation, expert/source review, figure/documentation checks and protected delivery; at most 1 GiB RAM and 32 MiB retained evidence plus one owned worktree. No model training, grading, GPU or paid compute. Charge the cumulative ledger, preserve the 30 GiB disk floor plus 128 MiB margin, and remove the delivery branch/worktree after verified merge. Accepted **8/512 complete development replies** remains unchanged; M1 and M3 qualification are unchanged.

**Next:** Run the paired earliest-query intervention through the actual M2 parent under a separately admitted exact configuration, so the next repair is selected by where the answer-relevant distinction survives or disappears. Do not repeat unchanged gradient/radius runs or promote source-only Copy exclusion into an established language failure.
