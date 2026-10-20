# DeepSeek execution prompt — learned geometric selection of exact token occurrences

**Executed historical prompt:** PR #1314 delivered this attempt. Read the [correcting audit](occurrence-reader-audit-2026-09-20.md) and [current successor](deepseek-memory-utility-step-2026-09-20.md) before resuming; do not rerun this dated task by default.

You are a research contributor on **UOR-R4 Geometric Language Model**, `UOR-Foundation/uor-r4`, working with the principal architect. Read [the principal review](occurrence-reader-review-2026-09-20.md) and this prompt completely, recover the live project, and implement, learn, exercise and deliver **one bounded exact-occurrence reader integrated with the current frozen token predictor**. You may improve the proposed design when source inspection, mathematics or a cheap diagnostic gives a better reasoned choice. The objective is useful contextual access, not mechanical compliance with a particular sketch.

## Objective and the decision already supported

PR #1312 merged as `0bca39b541e0ee56cc58bfff65a625a7854d5433`. Its frozen S attribution is useful negative evidence: retaining the individual older-history row predicts worse than deleting it on both evaluated panels. The query-only residual preserves the local gain. All generations remain repetitive. Stop refining that frozen single-register reader; do not repeat the full attribution matrix, output projection or width/precision search.

The selected next hypothesis is: **a small learned geometric selector can choose useful exact prior token occurrences, with an explicit NoRead alternative, and improve contextual generation through the same 4096-token predictor.** Geometry should guide selection and transport; exact occurrence identity should retain the payload. A 120-state root must not stand in for an arbitrary 4096-way token value.

This is a constructive learning and integration task. A new module, a handcrafted lookup demonstration, lower loss alone, or a proposal is insufficient. Deliver the resulting artifact, actual output/control evidence, complete cost scope and a reasoned next decision, including a negative or partial result if that is what the experiment produces.

## Recover context, then exercise judgment

Read AGENTS.md, DECISIONS.md D0-b/D1/D2/D3, execution policy, README, project-track, current-state, model-direction, PROJECT_MAP, the review above, and the #1312 result with its correction banner. Refresh origin/main, #973/#820/#963/#964, open PRs, the shared resource JSON, storage receipt and worktrees. Dated schedules do not override current authority. Use the project knowledge MCP and targeted original-source/literature retrieval where useful; verify snapshot claims against source. Discover context beyond this list when it would change the decision, rather than treating a missing item as forbidden.

Offline Rust training may use floats, gradients and matrix multiplication. D0-b serving permits bounded <=4-bit learned coefficients executed through adds/subtracts/shifts/table reads, without multiplier or floating/transcendental numerical kernels. Accumulators, identities and exact geometry tables have their declared separate types; calling an unrestricted learned score a table does not evade the weight contract. Keep prime/ordered occurrence identity, signed R4/S3/H4 transport, exact Z[phi]/icosian witnesses and fixed zeta channels in their actual implemented roles. No hidden transformer/provider response or Python model implementation. Not every geometric channel needs to participate in this small experiment; report active, preserved and missing roles honestly.

You have latitude to choose the reusable seam, small feature set, offline optimizer/credit estimator, compact quantizer, candidate admission bounds and curriculum. Before opening fresh evaluation, write a short design note with the equation, causal data flow, estimated cost, likely failure and falsifier. If a cheaper or better supported mechanism changes the sketch, record why and proceed within the same objective and authority. Ask the owner only for a genuine unresolved goal/constraint conflict, destructive uncertainty or external spend. Routine reversible implementation decisions do not need confirmation.

Use one primary learned candidate, with cheap shared-artifact controls. A meaningful failure can justify a focused correction and another version within recorded resources. Avoid three undertrained architectures or indefinite identical retries. Keep every version and its result. Do not move goalposts silently: a near miss can remain promising with a new prospective decision, without relabeling the original criterion PASS.

## Inputs and preservation

Use an isolated full `codex/…` worktree from refreshed origin/main. Preserve the owner's original checkout, historical worktrees, research, negative candidates and sealed roots. Root:

`/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/`

| Input | SHA256 |
| --- | --- |
| `head-projection-3/corrected/empirical.cpl2` (E) | `565cf98a01273af38425687adb522704b09a89bd7ab976b58b20bfbdaccfa0bf` |
| `s-attribution-3/corrected/separable_older_query_read.cpx3` | `ed9affd638ee93670559c626fd6dab91dc2a3c7faa5db66ec9e5a486bcdbdfad` |
| `s-attribution-3/result.json` | `994a1a15080ecc0c524aaa6ae6d93428c01f6fe4c5fd4d18ee80394d32a92165` |
| `attempt-1/prior_realtext.ckpt` (historical exposure witness, not a new reader checkpoint) | `1c3dfedb2248ec76c21f320b7ecec585c7a522a2d47e35ed5dc9eaca2454bff2` |
| source tokenizer | `9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c` |

**Authoritative source tokenizer digest:** `9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c`. Derived raw digest: `a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f`. Use the verified HfBpeTokenizer derivation. Pinned corpus `inputs/docs` is from `e9c04e80`; do not reconstruct it from the changing checkout. Existing split is 355 fit / 38 tune / 36 dev documents. All reused dev panels are open development, not final held-out data.

Read the exact manifest for artifact basenames and validate identities before loading. Claim new report roots exclusively immediately after argument validation; bind inputs, source/configuration and executable; seal complete attempts without mutating old roots. Keep report artifacts compact. The retained attribution binary at `.worktrees/geometric-query-read/target/release/query-read-attribution` is preserved for provenance, not the serving benchmark for this task.

## Proposed minimal mechanism

Freeze E and **only S's local query contribution**, including its existing absence behavior, tokenizer, quantization and tie rules. Define the local baseline unambiguously as `z_local = z_E + u(b)` where the old mask permits it, otherwise E. Call it the **S-query-only local baseline**: the old label Qonly already includes E and is not the earlier jointly transported arm Q. Do not add E twice or silently retain S's history row.

Reuse or extract small helpers from `native_geometric/memory_runtime.rs`, `memory_types.rs`, `role_read.rs`, `source_routing.rs`, `word_copy_runtime.rs` and the current `learner/{query_read,prior_learning,prefix_artifact,group_table,lowbit}.rs`. The old memory implementation stores sequence/slot references, rejects overwritten slots, indexes observed successors and compares source-local/query-local paths. Its older Model/word-token artifact is not interchangeable with this BPE artifact. Reuse contracts and code where practical, with an explicit adapter; do not import its authored grammar or claim old weights qualify the new token path.

A reasonable starting design is:

1. A fixed exact token-occurrence ring, initially about 128 tokens, storing token ID, sequence/slot validity and compact source-local geometric metadata. Write only observed tokens; perform the prediction before observing its target. Reset between independent documents/sessions. Never carry memory across shuffled or noncontiguous windows, even when document IDs match: reset each window or reconstruct its true causal preceding prefix, and bind that policy in data/configuration. Bound postings, scratch and candidate visits as well as live records.
2. A small candidate set, initially about 16–32 references, admitted by explicit exact token/ordered-cue postings. Preserve full token identity through collisions and duplicates. Admission must use only the causal prefix; it cannot inspect the answer/next target. A bounded scan is acceptable if cheaper to implement and measure at this size. Do not pre-optimize an unmeasured routing bottleneck.
3. One shared learned score `s_theta(query, occurrence)` with candidate-relative geometric interactions and an explicit NoRead action. Avoid another separable sum of query-only and history-only offsets. Use transported local paths/signed geometry, not numeric hash distance. A simple finite feature/landmark score is a legitimate starting point; learn parameters from data instead of writing query-family rules.
4. Read the chosen **exact observed payload or successor token** through a validated occurrence reference. Do not decode a vocabulary token from its H4 root, choose the gold reference at inference, or expose unseen target bytes to the selector.
5. Integrate the read with the frozen local scorer. One candidate is a bounded learned sparse copy residual `z(v)=z_local(v)+a_theta(query,source)*1[v=payload(source)]`, with NoRead giving zero residual. Choose a compatible small integer amplitude/scale and justify the actual objective/export. An existing shared Copy/NoRead operation is acceptable if it fits the same token lifecycle more cleanly. Prevent a forced copy on unsupported contexts. Before fitting, check the copy score units and achievable residual against representative covered fit/tune margins `max_v z_local(v) - z_local(target)`. Permit a declared bounded shift scale chosen on fit/tune so a correct source can actually win; report an output-amplitude limit separately from selection failure.

The exact ring/candidate sizes are initial proposals, not a capacity sweep requirement. Inspect admission coverage before fitting: if useful sources cannot enter the candidate set, fix admission rather than training the ranker to compensate. Report covered/selected/output-correct separately. Record when an occurrence's cue or payload was evicted; an unfollowed posting is not proof of absence.

No dependent multihop composition or durable cross-session memory is needed in this first integration. Keep an API seam for those later. Continue/Reset can be a future segment-maintenance feature or a cheap diagnostic if you find a compelling need; it is not the primary addressability mechanism here.

## Learn a new capability, not another isolated fixture

Train only the new source-selection/read integration. Start with a small compositional construction population through the **actual BPE token path**, then carry that exact same predictor into a bounded pinned raw-text probe. Offline supervision may use the observed next token to assign credit to compatible *past* sources; describe the objective and ambiguity handling. The target must never enter serving features or candidate admission. If a surrogate selector objective differs from token likelihood, state that and measure the resulting hard exported token predictor.

The construction population should require older content while keeping the recent suffix fixed. Include multiple bindings, distractors, varying lags and at least one ambiguity where the latest exact matching cue is the wrong source. For example, role/context distinguishes two live occurrences of the same key; vary which matching occurrence is correct so recency alone cannot solve the panel. Keep the raw format simple and disclose it. No English instruction parser or full-answer table.

Split by key/value assignments and combinations. Include valid tokenizer payload IDs never used as payloads during new-selector fitting; these IDs can remain known to the frozen parent. The copy action must generalize through exact pointers rather than memorize output labels. If later raw-text adaptation exposes those held-out payload IDs to selector fitting, evaluate that payload-transfer claim on the frozen pre-adaptation snapshot or maintain the exclusion throughout all fitting; report the later adaptation separately. Use generated fresh rearrangements after selecting the design, with a saved seed/manifest before scoring. Record any overlap in keys, templates and documents: fresh assignments are not new semantic domains.

Before the main fit, validate nonzero shared hard forward, the actual learning signal, and a small actual-disk continuation check for **the trainer you will use**. If QueryTrainer/CPQK is not used, do not complete its unrelated continuation framework as a prerequisite. Keep CPQK's deficiency open. The new path must preserve parameters/codes, optimizer moments/age if applicable, data/config/artifact identities, schedule/cursor and RNG. Compare an uninterrupted small run with process A checkpoint + process B actual file reload and the next update/export. Declare cache reconstruction; do not fake a file-resume test by passing an old in-memory vector.

Probe the complete training step, select a finite dose and checkpoint schedule that fits resources, and save a useful curve on fit/tune. Avoid interpreting route changes alone as adequate optimization. A small successful overfit is an instrument check, not the main result. Conversely, a genuine learning defect should be fixed before spending the full dose.

## Evaluation and a flexible but honest decision

Write a short prospective acceptance note before fresh evaluation. Choose a primary behavioral endpoint, sensible practical target and tolerated local regression based on the constructed task difficulty and cost. Do not inherit the previous 0.10-bit whole-panel screen as a universal requirement for this targeted memory component. A strong new counterfactual binding result with a small all-token CE gain can justify continuing; a tiny CE win without correct contextual generation does not.

Required distinctions and controls:

- **Local baseline versus learned reader** through identical tokenizer/decoder and causal input handling.
- **Fixed exact-prefix/latest-occurrence selector**, using comparable candidate admission and copy allowance. Disclose any tuning. Do not choose a deliberately weak baseline, but do not turn it into another model research campaign.
- **ReadDisabled** must recover the frozen local predictor exactly; altered source payload with fixed query/local suffix must change the selected generated payload appropriately when the source should be used.
- **Competing occurrences, distractors, stale references and session reset:** inspect selected sequence/slot as well as output. Repeated equal values cannot identify which occurrence was selected. NoRead should leave the local predictor intact; arbitrary local output is not an explicit abstention claim.
- **Learned-selector/geometry control:** compare a parameter-disabled or matched simple selector. If geometry does not improve over an exact-identity solution, report a useful exact-memory integration separately from an unestablished geometric ranking advantage. Do not impose a multi-family geometry refit campaign now.
- **Causality:** appending/changing future tokens cannot change an earlier prediction; observe-target-before-predict bugs invalidate the result, however good its score.

Capture generated token IDs and decoded bytes with source traces, not only teacher-forced probabilities. Test multi-token continuation only if implemented; otherwise name the single-token boundary and next integration need. Preserve a small representative success/failure set, including the original six free-generation prompts as observational regressions. Do not claim arbitrary prose from synthetic copying.

On a bounded raw-text panel, report all-target loss plus admitted/covered/uncovered strata, candidate recall and ranking, local-vs-reader paired differences, false copy behavior and actual continuations. Respect document splits. This can be a modest transfer probe within the run; do not launch a large corpus fit. Pair/counterfactual family or document is the sampling unit appropriate to the task; token-bootstrap pseudo-replication is not independent evidence.

Retain a promising near miss with effect sizes, failure examples and cost if useful. You may propose a new prospective criterion when the old one was a poor proxy, but show the original result unchanged. Leakage, invalid identities, hidden serving arithmetic and train/export mismatch are correctness defects, not thresholds to relax. Stop refining a valid negative when another cheap retry has no new causal rationale.

## Correct known reporting defects without restarting S research

The principal review records a terminal-position deviation in M01 fit averaging: one no-target terminal state per fit window, 4096 extras, of which 34 affect lengths used in evaluation. Do not rely on the old M01 numbers as an exact target-occurrence estimator. A narrow derived correction is optional if needed for your design; otherwise carry the limitation and move on. No pooled-mean experiment is needed to justify moving to occurrences.

If reusing the old decision helper, fix equivalence to require the **whole CI** inside the margin, and old significant-harm detection to use the upper bound. If reusing QueryHard public helpers, reject invalid indices/states and make legacy repair authorization explicit. Repair only the interfaces actually exercised by the new work; do not expand into an unrelated cleanup suite. A source comment suggesting an injective 4096-to-120 map is mathematically false; correct it if touching that module.

## Cost and resources

Refresh the live JSON at `/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json`. Review snapshot: **178538565 /180500000 ms**, 1961435 ms remaining. Preserve the previous 2400000-ms mixed measured/estimated debit once. The principal review performs no new model/build work and does not change this balance.

Initial planning envelope: **10800000 ms (3 hours)** for 1500 s implementation/build/focused checks/continuation; 900 s data/admission/learning probe; 4200 s new-reader fitting and justified corrections; 2400 s behavioral/raw-text evaluation, generation and direct cost; 1800 s reporting/checkpoint/delivery reserve. This is a complete projection, not a command to consume it. Refresh and improve the estimate after cheap inspection. At this snapshot, record the standing-authorized **+10800000-ms limit increment to 191300000 ms before using it**, giving 12761435 ms headroom. Necessary local revisions/extensions remain authorized when reason, complete increment and new limit are recorded before consumption. No paid compute.

One training worker, <=4 Cargo jobs, <=8 GiB RSS, initial <=512 MiB new model/report data and <=2 GiB incremental build growth, with the 128 MiB protected storage margin. Read [the cleanup report](storage-cleanup-2026-09-20.md). Main and geometric-query-read incremental caches were removed, and unopened main debug dependency files were cleaned; release binaries and the latest worktree debug dependency cache were retained. Reuse the surviving compatible target rather than creating several multi-gigabyte copies. Refresh actual free space and project storage accounting; the whole-machine reserve and the per-experiment 128 MiB margin are different constraints.

For serving, time the **actual declared inference implementation**, with E computed rather than hidden behind an evaluator's token-pair logit cache. The old attribution harness warmed a ~268.5 MB parent-logit cache and precomputed residual-score tables, so its 0.53–0.80 ms timings are not direct serving evidence. Benchmark your uncached numerical path first. A separately declared bounded cache/precomputed deployment is allowed to be investigated, but include construction/loading, memory traffic, total live bytes, invalidation and cache-hit assumptions; do not switch implementations only for timing. E's baseline must not execute unused S history folds.

Include all parent parameters, metadata, live occurrence/posting/selector state, scratch and output processing; distinguish serialized bytes from resident memory, per-token arithmetic from cold load/tokenization, and whole-run RSS from serving RSS. Consume timed outputs and use sufficient repetitions. Physical energy is UNAVAILABLE unless measured. No whole-path savings claim follows from fewer multiplications or a CPU-cache-sized table by itself.

Measure actual nonoverlapping durations and capture UTC at each actual boundary, including failed builds/retries. Charge actual work once, not the full reservation. Stop/checkpoint at configured limits rather than running until the disk is full. Cleanup authority in this owner request covers identified unneeded material, not unique evidence or unknown VM/session data; do not delete more merely to avoid accounting.

## Deliver and hand back a research judgment

Run focused Rust checks for the changed arithmetic, causality, references, serialization, actual continuation and generated behavior. Use the claim-wording check for capability documentation. Queue compatibility acknowledgements are not executed tests. Stage named paths, push, use a protected PR, and verify actual merge/content equality; never direct-push main or use admin bypass. If the queue is pending, say pending.

Update README/current-state/project-track/model-direction/PROJECT_MAP/EVIDENCE/CONTINUE where their current statements change. Update **issue bodies' current front sections**, not only comments leaving old NOT_RUN instructions visible. Preserve historical bodies/records in clearly marked history. Refresh project items if they exist; none existed on #973/#820/#963/#964 at the review snapshot. Leave broad issues open unless their full acceptance is met. Keep imported/historical READMEs scoped instead of rewriting them all as current claims.

Finish with: what you changed and why; artifact/source/data identities; learned behavior and controls; comparison against the best applicable simple baseline; exact failures and limitations; real resource/cost accounting; GitHub delivery; and **one recommended next action**. Explain disagreements with this sketch when your findings support a better direction. We want your architectural contribution as well as execution.
