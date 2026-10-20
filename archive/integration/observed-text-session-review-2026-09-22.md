# Principal review: contextual language is the next missing interface

September 22 UTC / September 21 local, 2026. Reviewed PR #1342 submitted head `b9eca5d2759d86664e519a9fb4c875b8127cb914`, built on reviewed PR #1341 `6fc34c1f`. At recovery both PRs were open and `origin/main` was `d9d896e8`; a reviewed branch is not a merged main. The final protected-delivery receipt owns subsequent status. Owner checkout remains preserved at `74fef088`. This review uses independent source, saved-evidence and architecture investigators, the source-linked mechanism/history synthesis, the knowledge store and a targeted primary-literature refresh.

## Principal decision

Retain the learned marker decoder, variable-depth exact reads, goal-preserving continuation and complete token-span emission. Correct the execution/restore boundary and evidence interpretation now. The next complete milestone is **learned occurrence roles and exact spans from ordinary source/request text with overlapping vocabulary**, through the corrected loaded session. Adding more disjoint marker aliases is insufficient. The [next execution brief](deepseek-contextual-text-roles-step-2026-09-22.md) gives DeepSeek substantive implementation and diagnostic discretion within that objective.

The larger goal is unchanged: a useful transformerless geometric language model, with native contextual attention, learned shared computation, durable memory and eventually prose/conversation and executed Rust. This run advances a small reusable component; it does not qualify the integrated model. Its statement interpreter does not execute the retained E/S language prior or the Q8 computation path as part of answering. Those remain complementary components requiring a learned interface, not capabilities already combined by a shared filename.

## What the original evidence establishes

The [independent original audit](../evidence/observed-text-session-principal-review-2026-09-22.json) verifies all eight original seals, complete file sets and byte sizes. Run 8 source hashes match submitted `b9eca5d2`; its original executable is preserved at `/tmp/uor-pr1342-original-competitive-reader`, SHA256 `767ddc81df9d4439e259134b3e74262a6d99e62d74883c26eff60602f05dfd3b`.

- All **24/24** retained primary answers, depths and role paths match independent reconstruction. There are twelve one-read, six two-read and six three-read cases. Three-read composition is absent from development supervision, which contains fourteen one-read and ten two-read cases.
- The **14/24 to 24/24** learning number is correctly counted on the same 24 gold statement-action units. It is a role-action-table fit metric, not an initial-versus-final end-to-end generation result. Marker/edge votes also exactly match the 48 annotated clauses.
- A single source-token edit at clause 0, position 3 changes **486 to 488**, exact selected segments **[0,1,2] to [0,3]**, depth **3 to 2**, and answer **[489,491] to [513]**. This is a useful dependent-use witness.
- The original membership arm reports 20/24, read-cap arm 18/24 and NoRead 0/24. Only primary final rows were retained; those other totals originally lacked independent row-level reconstruction.

These results are **exposed symbolic-token diagnostics**. Original primary final scores across roots 1–8 are 7,18,18,18,24,24,24,24; roots 5–8 have identical primary rows and the membership instrument changed after primary perfection. The final population cannot be described as untouched or fresh qualification. Its three-read structure is absent from the fitting labels, which is a separate, retained fact.

The inputs are directly assembled BPE token IDs with four disjoint two-token marker classes and a supplied subject–marker–object layout. Nine final outputs have two tokens, decoding as `ideations`, `ongear` or `astorm`; none of the final answers contains whitespace. Report **multi-token spans**, not demonstrated multiword phrases or ordinary prose. Main subjects are single tokens. There are no actual boundary fillers in the generator; four negative edge votes belong to marker tails already excluded during extraction.

## Corrections to the reusable execution contract

The original sibling `TextSession` is a reasonable richer representation than the scalar `RelFrame`. The systemic error was copying the shape of earlier mechanisms without retaining their already-established contracts. The corrected canonical `ObservedTextRuntime` now owns immutable loaded model/document observations and prepares actual bindings once. Its observed-question entry supplies the parsed goal; expected goals/answers stay in the evaluator. The module is the execution boundary; the runner records events and interventions around it.

| Original defect | Required correction and evidence |
| --- | --- |
| Model/tokenizer/document binding fields all literal `bound` | Actual model bytes, tokenizer identity, document content and world namespace/version; immutable control identity; reject foreign restore |
| Wrapper receives gold Goal, constructs question, checks extraction against gold and starts with gold | Observed question is passed into runtime; runtime alone extracts subject and goal; evaluator owns expected labels |
| Plain serde model load with clamped/unchecked fields | Validated versioned model load; invalid domains, duplicate tables and invalid action/goal combinations rejected |
| Restore omits capture/EOS/cursor/phase consistency; empty Stop emits EOS | Exact owned span provenance, valid phase and query goal, emitted-prefix/cursor consistency, complete nonempty phrase before Stop; explicit typed EOS contract including terminal restore |
| Trailing fillers counted as leading trim displacement | Derive slice and exact start from the same leading/trailing bounds |
| Read effect reports its next Continue/Emit action | Performed Read and selected next action are separate; save complete before/effect/after events |
| Fixed host iteration cap silently yields partial output | Session-driven bounded completion, explicit Exhausted when an actual cap is reached |
| Read-disabled synthesizes successful Stop; fixed-depth label misleading | NoRead yields Unresolved without EOS; comparator named maximum-two-read control with explicit Exhausted |
| Later-fact removal actually removes a direct first fact | Remove a verified terminal source after two successful redirects, preserve remaining segment identities, retain full changed inputs |
| Goal invariant checks first action rather than goal | Check active and query goals on actual before/after frames |
| Inline resume retains host state and compares only final output | Restore independently from saved bytes at each phase, including mid-emission and completed EOS, and compare complete resumed events/final frame |
| Only 24 primary rows retained | Save all evaluated arm/world/request rows, observed inputs, separate oracle labels, exact events and snapshot bytes/suffixes |

The [executed checks and resources](../evidence/observed-text-session-principal-checks-2026-09-22.json) and [corrected independent audit](../evidence/observed-text-session-corrected-replay-audit-2026-09-22.json) own the actual repaired counts and validation. These are a corrected replay of exposed data, not a new final evaluation. Original sealed roots remain unchanged; historical source/results are annotated rather than silently replaced.

Immutable document binding is not a mutable document service: this runtime rejects mismatched source snapshots. Durable current/previous/correction handling and owned captures across explicit source mutation remain later integration work. Snapshot JSON and vectors allocate; whole-path D0-b, cache residency, optimized latency and physical energy are unqualified.

## Corrected execution result

Final exposed root `observed-text-session-principal-2` preserves **24/24 development and 24/24 exposed primary answers**; membership **20/24**, maximum-two-read **18/24**, NoRead **0/24**. Independent recount covers **120 arm/request rows**, **470 row transitions** and **590 exact checkpoint/resumed-suffix comparisons** across every phase, including completed EOS. The primary-only checkpoint subtotal is 267/267; 590 includes controls. All 24 original primary outputs/depths/role paths are preserved. The corrected later-fact intervention removes the terminal source after two successful redirects and returns Unresolved with no emission; cycle returns Exhausted. Source edit keeps the independently expected segment/depth/answer change. The audit verifies five sealed report files and the actual model/source/executable identities. The model is 394 bytes, excluding tokenizer, observations, snapshot/event storage and other loaded assets. Eight library and 21 touched-runner tests (one explicitly ignored) pass. Both corrected roots and the initial corrected executable are preserved. See the linked audit/check receipt for exact hashes, costs and remaining scope.

## Why contextual observations, rather than more vocabulary

Current marker score is the sum of nonnegative token-role votes over an interval. For an interval `I` and a voting token `x`, `score(I union {x}) >= score(I)`. The chosen interval therefore can grow into a name containing a familiar marker token. Role voting is a bag sum: permutations of the same marker tokens have identical scores. Boundary handling uses a global sign `edge(token)`, so it cannot treat the same token as syntax in one occurrence and entity content in another. These are representational losses; additional fitting of the same observations cannot restore the missing distinction.

A suitable next hypothesis is a candidate-conditioned contextual span/role score. Keep proposed exact intervals and their ordered surrounding tokens; learn subject/object/marker/goal roles from declared offline supervision; let selected observations drive the existing read/update/emit boundary. Candidate admission, learned ranking and output correctness remain separately observable. Ordinary language can share cue words across functions and put the same entity in different grammatical positions. Do not choose a test where every semantic role is secretly a unique marker ID.

The geometric primary candidate should inspect the existing ordered H4 `SourceRouting::encode/score`, full relative relation ranks and occurrence-role context donors. Retain an equal-information categorical/contextual comparator, along with the present global token-vote baseline. Geometry can win through useful shared structure, parameter efficiency or quality/cost—not because the comparator is denied context. A fixed identity/central offset is equivalent to a paired categorical code; it cannot demonstrate the value of noncommutative transport. DeepSeek may select a simpler contextual finite model when an explicit development collision shows the proposed H4 encoding also erases the needed distinction. Preserve useful submechanisms even when the geometric arm does not dominate.

## Wider mechanism reuse and sequencing

1. **Now:** corrected loaded sessions plus ordinary-text occurrence-conditioned roles and full answer spans. Reuse `occurrence_role`, `span_learning`, `span_boundary` and ordered `source_routing` interfaces after inspecting their source and adapting old byte/ASCII assumptions to the pinned BPE tokenizer. Do not import old authored grammars as learned language behavior.
2. **Next:** the same engine handles corrections, multiple active entities/scopes, required evidence versus legal replacement, and grounded computed results. Reuse `query_participation`, scheduling, recurrent-text execution, exact versioned memory and owned-object responsibilities. Learned Q8 transitions remain available for actual ordered computation; arbitrary facts need not be bijective group actions.
3. **Broader language and coding:** source-separated prose/conversation training and executed generated Rust, integrating the useful local prior with read/compute/emit behavior. A copied fact answer is one capability, not completion of either track.
4. **Representation and scaling as needed:** structural role/scope banks address lifetime and interference; full relative H4/Spin/Hopf features address witnessed order/frame aliases; pages/sketches address measured admission/cost. Paired-H4/E8/S7 or harmonic coefficient memory follows a concrete additional distinction or equal-resource advantage, not a mandatory dimensional ladder.
5. **Qualification:** one artifact and actual CLI/API/session path must demonstrate useful output and complete-machine latency/RAM/traffic/energy. No geometry theorem establishes laptop energy savings without measurement.

The owner's mathematical bridges remain part of the active toolset. An E8 root code is a finite set on S7 after normalization, not 240 orthogonal registers. A point on S7 differs from a harmonic field with separately stored coefficients. Hopf projection loses fiber unless retained; common-left `q^-1 k` and common-right `q k^-1` use different conventions. Finite spin actions can be integer/table operations. Scalar compatibility is a learned score, not physical gravity or supersymmetry. Twin primes supply no semantic-distance theorem. The [mechanism synthesis](geometric-attention-mechanism-synthesis-2026-09-20.md) and [Hopf/spin direction](structural-memory-hopf-direction-2026-09-20.md) retain the detailed verified mathematics and source inventory.

## Primary research checked for this decision

- [Type Diversity Enables Transformers to Generalise Compositionally](https://arxiv.org/abs/2609.13144), September 11, 2026, under review: distinguishes diversity of structural constructors from more lexical substitutions. It motivates variation in clause/question structure; its transformer results are not native-model evidence.
- [SLOG](https://aclanthology.org/2023.emnlp-main.194/), 2023: distinguishes lexical from structural generalization. Borrow a relevant structural holdout and an honest split definition, not an unrelated full benchmark campaign.
- [Right for the Wrong Reasons / HANS](https://aclanthology.org/P19-1334/), 2019: supports explicit controls against lexical/subsequence shortcuts. Here, shared vocabulary and changed observed roles must change the required source or output.
- [Structural Generalization on SLOG without Hand-Written Rules](https://arxiv.org/abs/2604.26157) was withdrawn August 16, 2026 because the evaluated final-state proxy did not measure predicted logical-form edges. Its former performance claim is excluded. This directly reinforces checking actual emitted answers instead of internal-state proxies.

The useful inference from these sources is experimental design, not a claim that they validate our chosen geometry. Codex remains principal architecture/research reviewer; DeepSeek has substantive autonomy in mechanism choice, diagnosis and completion, with prospective resource accounting and the standing necessary-local-extension authorization.
