# DeepSeek execution prompt — determine what the frozen separable reader contributes

> **EXECUTED 2026-09-20** on base `8bbdb63d1c686d078292562bb879c0bf4dadde34` in the isolated
> worktree `codex/separable-attribution`. Retained root
> `.uor-models/realtext-prior-2026-09-20/s-attribution-3` (30 files, 4,390,694 bytes, sealed and
> verified); the superseded `s-attribution-1` and `s-attribution-2` are preserved sealed with
> identical numbers. Result and corrections:
> [s-attribution-result-2026-09-20.md](s-attribution-result-2026-09-20.md) and
> [the receipt](../evidence/native_geometric_s_attribution_2026-09-20.txt).
>
> Sections 1–8 are executed as specified except where noted here. Section 3: the tokenizer binding
> is repaired at the producer/export API, the legacy files load only through a restricted import,
> corrected descendants change exactly 32 bytes inside 73..105, and the inference seam is split
> with compose-call instrumentation; the evaluation-only entry point is
> `bin/query-read-attribution.rs`. Section 4: the new-position panel is frozen before scoring and
> yields 277 windows / 17,451 targets from 35 of the 36 dev documents (one offers no eligible chunk
> after the exclusions; disclosed per document). Sections 5–7: the four identity-anchored corners,
> E, the direct zero-row conditions, the offline M01 comparator, the exact-tail donor and
> older-order reversal, generation and the declared cost protocol are all executed. CPQK production
> continuation is **not** repaired here and remains required before any new fit.
>
> **Outcome:** no history lead. S's gain is a current-token query/emission calibration; the older
> row alone is indistinguishable from E, dropping it improves CE on both panels with the interval
> entirely below zero, and a content-free length-conditioned fit average beats the individual state
> by ~0.02 bits on both panels. The sealed `result.json` carries a conservative "inconclusive"
> label from a two-branch rule keyed on S01; the review records the corrected classification (this
> prompt's second branch). No optimizer update, reset/write fit, capacity expansion, new corpus,
> projection campaign or decoder change was performed.

Work on **UOR-R4 Geometric Language Model**, `UOR-Foundation/uor-r4`. Refresh origin/main beyond PR #1310, merge `7ea3744ce190c6acd708275e4121a6354b301023`. Read this prompt and [the principal review](query-read-review-2026-09-20.md) completely, followed by current authority. Deliver **one evaluation-only frozen S attribution experiment** with the metadata and inference/evaluation repairs needed to make it reliable. No optimizer updates, reset/write fit, history-capacity expansion, new corpus, projection campaign or decoder change is part of this task.

The finding to resolve: S scores 7.032227740 versus E 7.170815718, but its saved conditional older-prefix penalty is −0.002298261 [−0.014071554,+0.008581473]. Its positive query-neutralization penalty does not demonstrate older-history utility. Identify which frozen components contribute before choosing the next learned state-maintenance mechanism. Complete the replay, actual generation, cost and protected delivery; a module or proposed analysis alone is insufficient.

## 1. Authority, inputs and preservation

Read AGENTS.md, DECISIONS.md D0-b/D1/D2/D3, execution policy, README, project-track, current-state, model-direction and PROJECT_MAP. Offline Rust floating point/gradients/matrix products are permitted. D0-b allows bounded <=4-bit/ternary serving maps through add/subtract/shift/table operations, with no multiplier or floating/transcendental numerical kernel. Geometric routing remains the priority. Keep frozen R4G1 guarantees separate. Use Rust for model preparation, evaluation, artifacts and inference; Python may edit documentation or summarize saved observations, not implement a model.

Use an isolated full `codex/…` worktree. Preserve owner checkout, all source/research, sealed attempts and negative candidates. Refresh #973/#820/#963/#964 and existing project items; assign actual active work. Claim new roots exclusively immediately after argument validation and before model loading, seal complete outputs and verify the member set. Do not place derived material inside an old sealed root.

Root `/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/`. Verify these SHA256 identities:

| Relative path | SHA256 |
| --- | --- |
| `head-projection-3/corrected/empirical.cpl2` | `565cf98a01273af38425687adb522704b09a89bd7ab976b58b20bfbdaccfa0bf` |
| `query-read-2/artifacts/query_conditioned_older_read.cpx3` | `341f07211bb8695604c5cae50f3a529a1178def83d4d431f9031b2636a78a3fc` |
| `query-read-2/artifacts/separable_older_query_read.cpx3` | `9f8cda09bfb562221b5718a8330cc4a296215b8e94d3e0c2b8abe2ce31fae851` |
| `query-read-2/artifacts/local_only_read.cpx3` | `b0e5088794c2dedc87adabfe988cf2530fe5605863ca33ad3ce8078924b372a4` |
| `query-read-2/panel.json` | `b58d104083164dba30bc8251f0957ac1da9b3bec7d9c3b6ea299d6a9c7e1f79f` |
| `query-read-2/result.json` | `4aa6ed817f1e277007101ac06a56065c7050e2a281e36346ecdb07c2e58b42fc` |
| `attempt-1/prior_realtext.ckpt` | `1c3dfedb2248ec76c21f320b7ecec585c7a522a2d47e35ed5dc9eaca2454bff2` |

Verify the whole `query-read-2` manifest and corresponding saved vectors/panel, and preserve `query-read-1` plus the probes. The actual raw derived tokenizer digest is `a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f`; source tokenizer digest `9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c`. Reuse the verified HfBpeTokenizer derivation and pinned corpus `inputs/docs`, commit `e9c04e80`. Reuse corrected indices/exposure and the CPCK permutation witness; never reconstruct from the changing source checkout.

Population: 429 unique documents, 355 fit/38 tune/36 dev, 38987 fit windows, 4096 consumed windows/257113 targets. Reconstruct the exact old 36-document/288-window/17342-target panel and donor map. Use source `learner/{query_read,prefix_artifact,prefix_state,realtext_support,prior_learning}.rs` and `bin/query-read.rs`. Reuse shared scorer/panel/report helpers; avoid another copied learner or independent training stack.

## 2. Complete resource projection before execution

Refresh `/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json` and the established storage accounting. Snapshot **176138565 /178100000 ms**, remaining **1961435 ms**. Preserve the 5900000-ms mixed measured/estimated debit; do not refund or charge it again.

Propose **3600000 ms**:900 s source repair/build/focused tests;450 s metadata re-export, pinned-panel and old-vector reproduction;450 s fit-only means and new-position panel;900 s attribution/controls/generation/cost;900 s derived reporting, retry/checkpoint reserve and delivery. Before consuming the allowance, record the standing-authorized **+2400000-ms limit increase to 180500000 ms**, giving 4361435 ms headroom at this snapshot. Refresh actual values and revise before use if necessary; this prompt is not itself an applied ledger change.

One worker, <=4 Cargo jobs, <=8 GiB peak RSS, <=256 MiB new retained/temporary model/report data plus <=1 GiB incremental reused build output, 128 MiB storage stop margin. Probe actual evaluation throughput, bound parent-cache entries before insert rather than checking only after a batch, and include scratch memory. Record actual nonoverlapping monotonic phase durations and UTC boundaries, including failed builds/replays. Charge actual work once, not the reservation. No destructive cleanup, paid/external compute or downloads. Stop and retain evaluation progress at configured limits; use a pre-recorded standing-authorized extension if the complete necessary work needs one.

## 3. Repair reusable metadata and the directly exercised inference seam

All old CPX3 tokenizer fields, bytes 73–104, are zero. `QueryTrainer::hard_core` supplies `[0;32]`; `bin/query-read.rs` separately hashes the hexadecimal digest string and only prints that second hash. Correct the producer/export API to require and propagate the actual raw 32-byte tokenizer identity. Require real identity in normal production export; zero must not silently become a valid binding. Add a focused nonzero-digest round-trip/mismatch regression. Do not use a placeholder, hash-of-hex, or only a report-side value.

Allow the exact hash-pinned legacy zero-digest files through a restricted repair/parity import; normal production loading/export must require the correct raw digest. Re-export Q/S/L into the new claimed root by reconstructing their exact retained numerical fields with the verified raw digest. Preserve originals. Verify identical size, byte differences restricted to73–104, every other byte/field unchanged, and full old-panel logits/reader rows/generation IDs identical. Report old/new hashes; use corrected reloaded artifacts thereafter. No master weights or CPQK reconstruction is needed.

The CPX3 parent hash identifies a separately loaded E. Validate its raw bytes, table/palette/scale dimensions and correct tokenizer relationship before inference. Keep CPX3 reader 4/output 3 scales and CPX2 semantics unchanged. Reject malformed lengths/codes/scales and wrong input identities without panics. The old arbitrary-token clamping should not silently substitute another token in the new public replay interface: validate token IDs and nonempty prompt/index bounds at that boundary.

Separate inference row selection from training-chain construction. For valid older-present positions, Q/S should fold the older prefix once, L should use only its local pair, and the donor helper should consume the supplied validated state without rebuilding a conflicting recipient training chain. Preserve the training path and exact chronological table convention. No new learned parameters, new representation or blanket allocation guarantee. Verify parity of Q/S/L rows and integer logits across the complete old panel, adversarial identity/absence/future-token fixtures and the six retained generations. Instrument actual compose-call counts in a focused fixture to catch unnecessary L/history or duplicate Q/S folds; do not test only that the source contains a particular loop.

Add an evaluation-only entry point/mode that loads retained artifacts and cannot fall into fitting. It must allow derived evaluation/reporting without repeating training. Bind the actual source revision (and dirty status), relevant source-file hashes, executable SHA256, arguments/configuration, artifact/input hashes and timings. `source_rev=92240a86` is only the old base; `git_rev=unset` is not executable provenance. Mark old missing binary identity unavailable rather than manufacturing it.

CPQK full production continuation remains an explicitly open prerequisite **before another fit**. The old test writes a file but resumes the same in-memory bytes; it does not recover the data cursor or run separate processes. Correct documentation to the actual parameter-state parity scope. Do not spend this evaluation-only task building a full continuation framework or running optimizer updates. The two projection-source repairs already delivered in #1310 need no new calibration run.

## 4. Reproduce and freeze the analysis populations

First reproduce all old E/Q/S/L own vectors, read-disabled equality, S's identity-query vector and donor/reversal vectors within the existing 1e-8-bit tolerance; require exact integer pre/post repair parity. Recover and preserve all original occurrence IDs and donor indices. Old micro means: E 7.170815717556,Q 7.141505532244,S 7.032227739515,L 7.101049448943. A mismatch must be explained before new interpretation.

Reaggregate query-neutralization on `older_len>0` (16766 targets); retain the original all-target result separately. Reproduce historical bootstrap seeds per metric, then explicitly use the common prespecified seed below for new comparisons. Do not imply all old intervals used one seed: own gains used 0x12345678, donor 0xD1B54A32, query 0x9E3779B9.

Before computing new-position losses, create and seal a small selection manifest from the same 36 existing dev documents. Reconstruct original document-local 64-token chunk indices from the old spread-window selection; `Rec.win` is a global panel index, not a document-local chunk index. Exclude every old selected chunk and any exact duplicate token window matching an old selected window. Rank remaining windows by SHA256 of `uor-r4-S-attribution-v1|doc_sha256_hex|decimal_local_chunk_index`, lexicographic digest order, index tie-break; take up to 8 per document, without replacement. Short final chunks with at least 3 tokens remain eligible in the candidate pool and follow the same top 8 ranking. Save local/global indices, token ranges, token-window hashes and per-document shortfalls; never substitute tune/fit documents, relax exclusions, redraw or select by score. This produces at most 288 windows and uses no new corpus.

This is a **new-position open-dev replication on the same documents**, not independent-document or final-held-out qualification. Treat documents with no available chunks as unavailable for that panel and disclose reduced support. Build a donor map separately for it using the same exact `(prev,cur,older_len)` strata and existing deterministic algorithm; bind seed and indices before scoring. Keep target/local pair fixed and use S's own A map. Do not pool old/new populations to hide a disagreement.

## 5. Evaluate the exact frozen decomposition

For S define `u(s)=128*sum_j Wcode[v,j]*Rcode[s,j]` per vocabulary row; multiplication is notation for the ternary add/subtract path and shift7. E denotes its frozen integer logits. At older-present positions evaluate:

| Label | Integer scores | Question |
| --- | --- | --- |
| E | E | Original local reference |
| S11 | E+u(q)+u(b) | Full retained S |
| S01 | E+u(e)+u(b) | Neutralize individual older state, preserve identity offset |
| S10 | E+u(q)+u(e) | Neutralize current query; existing saved control |
| S00 | E+2u(e) | Both neutralized, learned constant offset retained |
| Qonly | E+u(b) | Drop the complete older-row contribution |
| Honly | E+u(q) | Drop the complete query-row contribution |

At absent-prefix positions **all conditions equal E**, including constants. An existing older prefix whose product is e remains present. Use exactly one validated shared integer scorer; retain the two-read bound 4096 and correct signed scaling. Qonly/Honly are diagnostic conditions of the frozen artifact, not refitted models or newly promoted artifact formats. Existing S10 is not Honly. S01 is not Qonly. Do not confuse the Qonly label with the independently trained historical joint-Q arm.

Verify per vocabulary row `S11+S00=S10+S01` using wide integers. The identity-anchored constant/history/query decomposition is `C=2u(e), H=u(q)−u(e), K=u(b)−u(e)`. Loss differences do not satisfy this logit identity; report the CE interaction separately. If reporting two-player Shapley loss contributions, label their anchor/population and use the exact two-order average, never call it unrestricted causal importance. No Shapley framework is required.

Add one fixed **offline-only** history-mean comparator M01. From the recovered 4096 consumed fit windows, compute `mu[len,j] = sum R[q,j] / N_len` over each older-present occurrence of a given older length, using frozen S's A and R. Preserve exact integer sums and counts and verify total coverage; no labels, tune/dev windows, code optimization or refitting enters the means. Let `M01=E+128 W(mu[len]+R[b])`. Use stable f64 diagnostic logits for this rational comparator, without rounding it into a purported serving artifact. A mean of logits is not a mixture of probabilities. It preserves fit-average history calibration by length while removing each observation's specific older state. If an evaluation length has no fit support, report it and exclude it from that matched comparison; do not choose a replacement after viewing losses. Current 64-token n−1 panels should use lengths 1..61; generation may reach 62 and is not evaluated with M01.

Save full per-occurrence losses, document sums/counts, q/b/read IDs, length/mask and exact means/provenance for both panels. Produce micro/macro CE for every condition on all targets and older-present targets, plus the donor-supported subset. Save integer-logit parity/mixed-difference counts. Reuse E/Q/L old references by verified identity; no need to evaluate their full new-position matrix.

## 6. Matched interventions and decision rules

For S11 retain exact-tail donor exchange, query-neutralization and older-order reversal. Donor comparisons use only eligible recipients; report document coverage, eligible/changed/unchanged counts, singleton strata and no-history exclusions. Require S01/S00/Qonly and M01 to be exactly invariant to older-prefix exchange and reversal because they depend on no older content; E is invariant too. These are instrument checks. Identity q/b and row coincidences require separate tests so constants are counted correctly.

For new paired intervals use 2000 document-bootstrap draws, seed 0x12345678, ratio of resampled loss sums/counts and the existing quantile convention. Bootstrap documents once per draw with exact alignment; zero-support resamples must have explicit handling, not NaN silently sorted. Describe intervals as nominal open-dev uncertainty. Keep original unmodified intervals and newly reaggregated populations distinct.

The primary new dependence comparison is **CE_M01−CE_S11 on older-present positions**; it asks whether varying history beats fit-average history at the same length. S01 versus S11 is an anchor sensitivity check, Qonly versus S11 includes offset removal, and the conditional donor supplies a different content-reliance check. Freeze this hierarchy before seeing results; do not select the most favorable baseline.

Use **0.01 bits per eligible target** as a newly declared component-level practical margin, reported separately from zero-exclusion. This is not the project's 0.01 BPB threshold and does not alter Q's historical 0.10-bit screen. Report effect sizes/CIs even when below the margin. A positive history lead requires the primary mean-history comparison and conditional-donor penalty to be >=0.01 bits with positive lower bounds on the new-position panel, with no old-panel comparison showing harm of at least 0.01 bits with its entire paired interval below zero. It remains a bounded lead for another experiment, not model promotion or a global history theorem. If support is insufficient or anchors disagree, report that ambiguity rather than widening strata or fitting an extra model.

If a local-only condition preserves or improves S numerically, identify it explicitly as a frozen local calibration/query result. Do not infer local-only equivalence from a nonsignificant difference: use the prespecified +/-0.01-bit band and paired interval if making an equivalence statement. The offline M01 condition is not eligible as a serving incumbent. Full S remains retained regardless of this diagnostic; any later adoption or stripping requires a declared artifact and proper output/cost evaluation.

## 7. Actual generation and cost

Generate the same six prompts for 64 greedy tokens from E and corrected S11/S01/S00/Qonly/Honly using the shared integer path. Keep tokenizer, lowest-ID ties and decoder unchanged; save token IDs and bytes. Reproduce S11 old output exactly. Report repetition directly. S11/Honly use the complete bounded ring for a sufficient-state cycle certificate; local-only conditions use their true local inputs plus the absence-mask state. A repeated local pair cannot certify an S11 cycle. Do not report absent full-state repetition as proof of good language. M01 is offline-only and is not generated.

Measure optimized E, old/new full-S inference and S01/Qonly with one declared identical prompt/token/repeat protocol. Use the original row-selection helper as the reference inside the evaluation-only harness; do not invoke the old training runner to obtain a baseline timing. Include the loaded parent and residual artifact bytes, state/scratch buffers and actual complete numerical path. No fit-time cache in serving timings. For microbenchmarks use `std::hint::black_box` on inputs and outputs, a varying pinned input set, explicit actual prefix lengths and consume results. Check correctness before timing. Record min/median/spread and distinguish whole-process RSS from steady inference allocations. Old discarded-result nanosecond ratios remain historical, unqualified estimates. Physical energy remains UNAVAILABLE without a measurement.

## 8. Finish one decision and update the project

No new learning is authorized by this prompt. The result should identify whether S's gain is best described as local query/calibration, useful older-content reliance, or unresolved at this support. Recommend **one** next step from those observations:

- If useful history is supported, preserve the reader and specify a selective-maintenance test with fresh behavioral criteria.
- If history is unnecessary or harmful while local query is useful, preserve that local result and target the state-maintenance problem directly; do not advertise the old group product as useful memory.
- If the diagnosis is inconclusive, identify the smallest missing causal/measurement condition; do not default to a width/precision/corpus sweep.

A possible later mechanism is a four-bit choice of Continue(gamma_k) or ResetTo(gamma_k), keeping exact 2I state. This is a noninvertible update monoid, not a larger register, arbitrary FSA or exact keyed memory. It is **not implemented/fitted in this run**. Exact occurrence/version access, shared composition, useful conversation and executed Rust on one artifact with complete M1 cost remain the larger roadmap. Reuse the historical native-memory work rather than starting a disconnected product.

Run rustup-managed Cargo formatting, touched-package offline checks and focused identity/arithmetic/metadata/input/causality/row-parity tests for the actual edits. Do not launch the broad learner suite or optimizer/resume campaigns. Save executed commands/results; queue compatibility statuses are not tests. Run `python3 scripts/check_claim_wording.py` on changed claims.

Seal a complete report with source/binary/input identities, corrected descendants, old/new hash and numerical parity, both frozen panel manifests, fit-mean counts, all losses/controls/generations, exact decisions and phase/storage receipts. Separate analysis from fitting in its schema. Add derived fields from retained artifacts/vectors without repeating the full experiment; claim a new root for corrections and preserve superseded attempts.

Update README, current-state, project-track dependencies, model-direction, PROJECT_MAP, CONTINUE, EVIDENCE and the resource ledger. Add an executed/superseded notice to this prompt only for work actually done. Preserve historical raw receipts and correct the old history, checkpoint, tokenizer, source, timing and cycle assertions in the interpretation layer. Update live issue bodies and existing project items, preserve historical issue fronts, and keep #973/#820/#963/#964 broad acceptance open. Stage named paths, push a `codex/…` branch, create/attach protected PR, use ordinary merge/queue and verify the complete merged tree/content. Return precise outcome, surviving artifacts, unrun boundaries, remaining resources and one comprehensive next handoff.
