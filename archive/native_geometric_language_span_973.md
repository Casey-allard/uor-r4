# Learned contiguous language spans — #973

**PASS_TYPED_LANGUAGE_SPAN.** The UOR-R4 Geometric Language Model experimental path selects and emits contiguous one-to-three-word terminal answers after one, two or three reads. All 1,728 development answers, source paths, word spans and byte bounds pass; all 48 matched families complete. The corrected path also preserves 3,712 earlier Full answer and typed unresolved outcomes. [Exact evidence](evidence/native_geometric_language_span_973.json). Normal model 15baec48 remains retained; no promotion.

## Mechanism and bounded investigation

The source audit found a representational boundary: each retained relative-language candidate carried one word. The new candidate enumerator preserves source and first-word identity separately from an exclusive last-word index and exact byte extent. It admits every contiguous one-to-three-word interval up to 50 bytes within the existing four-record input bounds; no answer length, source path or gold boundary enters serving.

Candidate intervals contract to one vertex for the retained 18 match-topology predicates. A nineteenth predicate asks whether the interval is a complete local payload run bounded by query-matched words, learned context words or record edges. The selected rule is a conjunction over these predicates. Word matching and context lookup reuse signed ordered-H4/Hamming comparisons and canonical word identities. ExactIdentity is a comparator; this step does not establish a metric advantage.

The recurrent completion executor is shared. Only the reader is injected at its existing current-route and lookahead seam; the learned query updater, Read/Emit/Stop/Unresolved policy and byte/EOS writer remain artifact-identical. The writer emits the entire selected byte interval, including internal spaces, through its existing cursor. Intermediate query substitution remains one word; this step evaluates multiword terminal answers after one, two or three reads.

Rust prepares 576 new direct-question training rows and retains 2,048 earlier direct training rows. Development has 1,728 rows: 576 each at one/two/three reads, with one/two/three-word answers, both subject/object orientations, two familiar authored styles, rotated record positions and baseline/active/inactive source variants. Forty-eight development families each contain 36 rows. Active multiword edits change a later component while retaining the first word; inactive edits change unused source text. An independent raw-language oracle checks the authored paths and byte bounds, which remain evaluation-only. Vocabulary and grammar overlap, and method selection uses open development. Independent final holdout is NOT_RUN.

The first attempt stopped before fitting: 976 training rows had incorrect candidates that dominated every output-compatible candidate in the fixed feature representation. No positive-conjunction selector in that representation could uniquely choose the correct interval. The second attempt added the local boundary predicate and learned finite context roles from all training-record vocabulary minus answer vocabulary. It removed all structural conflicts and fit 2,624/2,624 training rows with rule [327682], but only 624/1,728 development answers and 2,096/3,712 retained Full transfers passed. Unused records incorrectly supplied context evidence for helen and oscar.

The third attempt attributes boundary evidence only to words outside every source span that can actually produce the training answer and EOS. It intersects evidence across all compatible latent occurrences, then removes any word used in a training answer. No name exemption or development label is used. This preserves candidate payloads, ordering, extents and actual output-compatibility labels, and keeps rule [327682] frozen. It re-evaluates training and development without another rule search. Context roles remain finite corpus-derived lexical roles, not general grammar or context-sensitive polysemy.

## Actual generation and controls

Training passes 2,624/2,624 and all 1,728 development trajectories reproduce after artifact reload. The unchanged one-word parent passes 576/1,728 on this new corpus. The final candidate retains rule [327682] and all completion-parent parameters. The refined context operator contains 15 words.

| Control | One-word answers | Multiword answers | Total answers | Runtime errors |
|---|---:|---:|---:|---:|
| Full | 576/576 | 1152/1152 | 1728/1728 | 0 |
| ReadDisabled | 0/576 | 0/1152 | 0/1728 | 0 |
| ScorerDisabled | 0/576 | 0/1152 | 0/1728 | 0 |
| CursorDisabled | 0/576 | 0/1152 | 0/1728 | 0 |
| StopDisabled | 0/576 | 0/1152 | 0/1728 | 0 |
| SingleWordOnly | 576/576 | 0/1152 | 576/1728 | 0 |
| PayloadFirstWord | 576/576 | 0/1152 | 576/1728 | 0 |
| ExtentDisabled | 288/576 | 0/1152 | 288/1728 | 0 |
| BoundaryContextDisabled | 288/576 | 576/1152 | 864/1728 | 288 |
| LocalBoundaryDisabled | 0/576 | 0/1152 | 0/1728 | 0 |
| ExactIdentity | 576/576 | 1152/1152 | 1728/1728 | 0 |

BoundaryContextDisabled loses all 288 styled subject multiword cases. Its 288 runtime errors are counted as failures: removing the context barriers can offer a multiword intermediate to the still one-word update operator. These are declared intervention outcomes; Full has zero runtime errors. The removal controls test dependence of this fitted policy, not separately trained alternative architectures. ExactIdentity equals Full, so there is no demonstrated metric advantage.

One actual three-read example has records `alice did guide bruno.`, `bruno did trust helen.`, `helen did trust oscar amber birch.`, and `oscar did trust ruby.` The supplied sequential question is `who did alice guide? who did they trust? who did they trust?` The candidate silently reads bruno, then helen, and emits `oscar amber birch` followed by EOS. The exact trace and geometry are retained in sample-depth3-words3-subjectfalse.json. This is authored relational phrase copying, not spontaneous prose.

## Retention, validation and resources

The new Full path preserves 3,712/3,712 earlier outcomes: 1,536 mixed one/two-read, 1,280 three-read and 896 completion cases, including 512 typed unresolved outcomes. Scheduling/depth core traces and completion responses are compared against their prior artifacts individually. Separately, unchanged-parent control replay preserves 13,248 old outputs; 512 recurrent; 640 ordered; 1,408 language; 128 old-language through relative; 9,984 relative; 9,984 dependent; 19,968 scheduling; 19,968 output-credit; and 19,200 depth rows. Legacy replay is distinct from new-path transfer.

Optimized offline compilation and 44 focused tests pass (four new and 40 retained). Three report attempts were explicitly executed and sealed: a representation diagnostic without fitting, one rule fit with a negative candidate, and one boundary-credit refinement with the rule frozen. A test-only error-conversion compile failure was corrected and its 24,001 ms charge retained. Three builds succeeded and one failed; no old fit campaign was repeated. Formatting, claim wording and diff checks accompany delivery; queue acknowledgements do not run these tests.

All 51 earlier report roots / 1,265 files, predecessor frozen sources/binaries and both original dirty checkouts verify unchanged. This step adds 7 diagnostic files, 31 negative candidate files and 31 final candidate files: 54 roots / 1,334 sealed files in total. Both rejected attempts, all three frozen source/binary versions and normal model 15baec48 remain at their original paths.

Final candidate SHA256 44c0ebf482d5807354a2e2b4c3fd6ceb310dd076ce199536f801f6cad472264b. Completion parent SHA256 f9e1f5aeef918d0fe6b463cd75986847d551aa8049c9f1c85b1d6d46c55a1e1f. Negative span candidate SHA256 ddf9bea88413c458f3ba45c4cd0f0e89eb2089b0ed84f194ede798a9c195812e. Final binary SHA256 d9810736cfbd8830d4e38c0c206052c8e860ade5cf34d189b2e4cf0521bd86e8. Artifact binds schema, geometry, source, data, context roles and training history.

Model/build/preparation/fit/refinement/controls cost 993360/2100000 ms; parent 9217045/10410000 ms; shared 128967618/132950000 ms. Peak sampled process-tree RSS 2952855552 bytes, under 6 GiB. Two build threads and one model process; 384 MiB new-storage allowance with 128 MiB stop margin. Initial final receipt records 211451904 bytes new storage at cumulative high water, with later documentation/delivery accounted in resource-final.json. Before work, standing authorization extended the parent by 600,000 ms to 10,410,000 ms and by 256 MiB to 3,892,314,112 bytes. Shared limits remain unchanged; external cost zero and no cleanup.

Evidence, resource receipts, source snapshots and restart notes are in /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/language-span-1/. Serving continues through the existing geometric routing, state and integer/table operators without a transformer, serving matrix product or provider. Final independent holdout is NOT_RUN; general prose, broad reasoning, product integration and complete-path energy advantage remain unqualified.

**Next:** Carry a selected multiword value through the existing query-update operator as an intermediate reference, so a later geometric read depends on the entire phrase. Begin with the explicit one-word guards in dependent_language/runtime.rs::splice and updates, and preserve exact span identity and ordered query encoding. Freeze a small mixed terminal/intermediate phrase corpus with a later-component-only change, changed active source, unchanged inactive source and truncated/stale payload controls. Try the current span rule, completion policy and writer without refitting first; preserve all current answers and typed unresolved cases. Multiword copying remains distinct from general prose and arbitrary discourse reference.
