# #728 — observation/compiler: atomically bind recorded-corpus generations to execution provenance

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-15T23:17:22Z
- **Updated:** 2026-08-16T08:08:08Z
- **Closed:** 2026-08-16T08:07:39Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/728
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Discovered while integrating the accepted GPT-2 dense arithmetic era in #704. The dense owner itself is complete; production adoption now needs one durable rule that prevents observation bytes, attention identity, and dense identity from being assembled from different filesystem generations during resume, derivation, or managed serving publication.

## Problem

`gpt2-source-dense/2` changes offline teacher outputs without changing the source-byte kappa. Path-level sidecars alone are insufficient when a writer crashes between members or a reader observes a corpus during a cooperative update. A dense-present corpus must never be accepted as dense-absent, cross-era, or mixed-generation evidence.

## Scope

- Add canonical `recorded_corpus_binding.json` schema `/1`, published last, binding exact filename, typed presence, byte length, and BLAKE3 CID for manifest, attention operator, dense operator, metadata, records, and optional hidden rows.
- Preserve observation identity-bundle `/1` bytes when dense is absent; dense-present observations retain `/2` semantics from #704.
- Add one alias-safe cross-process producer guard and explicit observation/compile roles. Cover raw/text/batched observation finalization, Hugging Face source compile, compile-recorded, managed server refresh, and any supported derivation writer.
- Make crashes fail closed and exactly retryable with non-authoritative staging, canonical recoverable temporaries, and a compile-attempt marker before the first compile-role mutation.
- Keep markerless legacy and attention-only corpora readable under an explicitly weaker compatibility contract. Require the canonical binding for dense-present generations.
- Provide bounded-memory verified readers: large records and hidden members are streamed through retained no-follow handles and jointly rechecked; status/provenance inspection must not allocate the corpus body or require write permission.
- Hold or revalidate the final logical-root generation across managed server staging and atomic publication so server-specific locks cannot race repository CLI/observation writers.
- Keep the binding CID distinct from source kappa; it is a derived corpus-generation identity only.

## Definition of done

- Every repository-supported producer participates in the same guard, role, crash, and binding-last protocol; unsupported writers fail closed before mutation.
- Every dense-sensitive cover/compile/evaluate/certify/API/serving reader consumes one verified generation and preserves exact attention+dense pairing.
- Production-sized GPT-2 hidden streams are verified without multi-GiB materialization when the consumer does not use them.
- Tests cover fresh/resume/crash boundaries, malformed or special residue, role conflicts, alias/BUSY behavior, read-only inspection, bounded streaming, legacy absence, dense-required binding, and server stage replacement.
- README, lifecycle/configuration/observation docs, the #704 record, and formal claim wording remain truthful.
- Focused tests plus workspace fmt, Clippy, tests, no_std, wasm, and claim-wording gates pass.

This work stays in `uor-r4`; it proposes no upstream `uor-matmul` API and waits on no external repository. References #704 and #655.

## Comments (25)

### comment 5304703847 — 2026-08-15T23:19:41Z — Casey-allard

## Starting implementation state (2026-08-15)

The complete pre-review WIP is preserved locally at `0461aaf1` on `issue-704-certified-dense-v2`; the earlier expanded safety snapshot is `af66569a` on `codex/i704-expanded-snapshot`. Neither has been pushed.

Already implemented and previously focused-green before this issue was split out:

- canonical schema `/1` with six typed members and literal CID pin;
- binding-last `.writing` -> `.tmp` crash publication;
- dense-present binding requirement plus markerless legacy compatibility;
- alias-safe producer guard, observation/compile roles, and compile-attempt marker;
- exact observation identity `/1` preservation and `/2` dense ordering;
- joint dense/attention validation through the #704 downstream seams.

Open work that this issue now owns:

1. finish the bounded stream reader contract and direct race/read-only/BUSY tests;
2. remove full opaque-hidden materialization from every production consumer that does not use the hidden values;
3. complete all official writer participation and deterministic crash retry;
4. hold/revalidate the managed server final-root generation across staging and atomic replacement;
5. run the full gate ladder and independent frozen-hash review.

No ignored timing or corpus-scale run is part of this issue.

### comment 5304739435 — 2026-08-15T23:30:38Z — Casey-allard

WIP Phase-B checkpoint (2026-08-15; not an acceptance claim):

- Moving files are currently limited to graph-compiler recorded-corpus/compiler integration and graph-cli consumers; model-source dense arithmetic remains frozen.
- The lower bounded stream API now supports read-only inspection, BUSY classification under an active producer, generation rechecks, markerless compatibility, and materializes hidden bytes only when they match the 288-wide compiler consumer. Opaque GPT-2 768-wide hidden data is verified and streamed instead of retained in a multi-gigabyte Vec.
- Direct graph compile, raw/text observation postbinding, cover, score, evaluation, Hugging Face source compile, and compile-recorded have been rewired to bounded snapshots. Compile-recorded streams optional hidden bytes through guarded atomic publication, verifies the source generation before binding-last, and runs deterministic inventory checks before mutation or ready cleanup.
- Current focused cargo check for uor-r4-graph-compiler + uor-r4-graph-cli with all features and offline mode is green. The lower focused suite was 30/30 green immediately before the latest consumer wiring.

Next binding checkpoint: add consumer regressions, audit remaining full-snapshot callers, run focused tests/clippy/fmt/diff, freeze exact hashes, and obtain independent P0-P2 review. No server Phase C changes or timing runs have started.

### comment 5304775041 — 2026-08-15T23:41:33Z — Casey-allard

Phase B frozen for exact independent review (2026-08-15; HEAD 0461aaf1, no server or #729 edits):

Exact moving-file SHA-256:
- graph-compiler/src/lib.rs: 3fcd975d2e82323a5d93f4faa9e7a5bf1f022d45f9d0861fb2dea0786d60e64d
- graph-compiler/src/recorded_corpus.rs: 5c8d770778587eb67e8be589a2e5eca41761921822a5b114c220757568881731
- graph-cli/src/lib.rs: e50541ebdae013e030fee6d03cdd67205108ddb23cd805a8d4a5c260d2dcf786

Outcome:
- large opaque GPT-2 hidden rows remain streamed and binding-verified rather than retained in unused Vec storage;
- direct compiler, observation postbinding, cover, score, evaluation, source compile, and compile-recorded use bounded corpus streams;
- compile-recorded atomically stages optional hidden data by declared length and BLAKE3, final-verifies the source before binding, and checks deterministic destination inventory before cleanup or mutation;
- the #729 sampler is now the only production full-snapshot caller.

Focused evidence:
- lower recorded-corpus tests: 32/32 PASS
- direct graph-compile preflight: 2/2 PASS
- graph-cli compile_recorded filter: 3/3 PASS
- guarded stream publication: 1/1 PASS
- focused two-package cargo check: PASS
- focused two-package all-target/all-feature clippy with -D warnings: PASS
- cargo fmt --check, git diff --check, claim-wording: PASS

One superseded command-level unit was intentionally interrupted after 2.5 minutes in the existing debug RVQ compiler and replaced by a deterministic seam test that passes in 0.24 seconds. No ignored tests, timing measurements, server Phase C, or #729 work ran. Exact read-only P0-P2 audit is now in progress.

### comment 5304789128 — 2026-08-15T23:45:56Z — Casey-allard

Exact-review finding on the Phase-B freeze (P1; acceptance withheld): compile-recorded sidecar publication is not yet crash-recoverable. A process death after fsync of a canonical attention/dense temporary but before hard-link publication/removal leaves a recognized temp plus the durable compile-attempt marker. Retry recognizes the temp as compile-role evidence, but deterministic inventory rejects it before the sidecar publisher can recover, making an honest crash terminal.

Repair scope is deliberately narrow: place attention and dense sidecars under the common guarded publication/recovery protocol (or add exact nofollow owner recovery for both temp families), and add injected boundary tests proving exact rerun completes with no residue. No server, sampler, model arithmetic, upstream uor-matmul, or timing work is being opened for this repair.

### comment 5304805302 — 2026-08-15T23:51:11Z — Casey-allard

Additional exact-review blockers on the frozen lower layer (all P1 within #728; frozen hashes remain obsolete until repaired):

1. Binding commit does not retain and recheck the guarded root plus every member generation through the atomic publication point. A root replacement or records-A/hidden-B transition between sequential hashes can commit a false or wrong-root binding. Repair requires commit-point root/member rechecks (directory-handle-relative operations where available) and deterministic swap hooks.
2. Several control/residue paths remain unbounded: schema records and binding/attempt staging residues can be read into unrestricted Vecs, and hash loops can chase a growing EOF. Repair requires fixed schema caps, initial-length snapshots, exact bounded reads plus EOF checks, and streaming residue validation without retaining arbitrary bodies.
3. Canonical binding parsing currently proves JSON round-trip/schema syntax but does not centrally validate every fixed slot name, typed presence/length/digest tuple, digest syntax, and role-specific hidden filename before mutation. One complete structural validator must gate stable and temporary records.

The previously logged attention/dense sidecar-temp crash-retry defect remains in the same repair batch. Independent review found no separate defect in the bounded compiler materialization/cover/score/evaluation consumers. Optional Windows/read-only-markerless hardening is being dispositioned separately rather than silently expanding acceptance.

### comment 5304851263 — 2026-08-16T00:05:24Z — Casey-allard

Replacement Phase-B freeze after exact-review repairs (2026-08-15; stopped for independent audit):

Exact SHA-256:
- graph-cli/src/lib.rs: 4c32eecbc452b2045c5ce59bc41073daf8f69ae1da8fd76a70dfdeddf9e6375f
- graph-compiler/src/lib.rs: 3fcd975d2e82323a5d93f4faa9e7a5bf1f022d45f9d0861fb2dea0786d60e64d
- graph-compiler/src/observation.rs: 61bdeada5c4b003a2192faabd59c5b8b6889a22b73fded567f961eb6a7dfe4e8
- graph-compiler/src/recorded_corpus.rs: 8aa9f9a5b9d506d33495ebbd009d6baab2a8e0ce93154d2a8e380f165c9c9ffa

Repairs:
- attention/dense sidecars now use the guarded recoverable publication protocol, with injected post-sync crash boundaries and exact rerun cleanup;
- binding commit retains/rechecks guarded root and every member generation through publication, with root/member swap hooks;
- schema/control/residue reads are capped and exact-length/EOF checked rather than chasing moving EOF or retaining arbitrary bodies;
- binding parsing centrally validates all fixed slots, presence/length/digest tuples, digest syntax, and role-specific filenames before mutation;
- observation refuses compile planned-output residue before mutation; exact-current attempt completion and residue directory sync are pinned.

Focused PASS evidence:
- lower transaction suite 38/38
- observation no-mutation 1/1
- compile-recorded 4/4, including real command orchestration via a fast fake-products backend for dense crash, attention crash, exact rerun/no residue, binding-last, and ready idempotence
- guarded stream 1/1
- focused two-package check and all-target/all-feature clippy -D warnings
- fmt, diff, and claim-wording gates

No server Phase C, #729, model-source, ignored tests, or timing runs. Independent exact-hash review is active.

### comment 5304857772 — 2026-08-16T00:07:00Z — Casey-allard

Replacement exact review: prior blockers are closed, but acceptance remains withheld for two P1 final-window defects.

1. Final metadata/provenance/binding/attempt rechecks still reopen through an uncapped capture helper. A valid small record can be replaced by a multi-GiB regular inode after initial capture, causing the bounded reader/publication verifier to reserve/read the replacement rather than fail with a bounded mismatch. Repair: allocation-free exact expected-length comparison (or cap-aware verification) plus oversized post-capture tests for manifest, binding, marker, and metadata.
2. After the last root/member generation check, binding rename and planned sidecar link/remove still mutate through paths. A root-to-symlink/replacement swap in that final window can touch an unowned directory before the later check reports failure. Repair: perform commit mutations relative to the retained nofollow root-directory handle using *at-equivalent operations, with injected hooks immediately after the last check and before binding/planned-file mutation.

The issue retains its filed nofollow/generation-bound contract; we are not weakening it to cooperative writers after implementation. Verified closed: common nine-member sidecar staging and command crash retry, fixed-slot binding validation, retained records/hidden witnesses, observation cross-role refusal, and exact-current attempt completion.

### comment 5304897965 — 2026-08-16T00:17:25Z — Casey-allard

Second replacement Phase-B freeze after final-window repair (2026-08-15; stopped for exact audit):

SHA-256:
- graph-cli/src/lib.rs: 28e76b978fdb78952f5250b94e2a276cea576f3a7d5ac1b87744c0f14c7d4098
- graph-compiler/src/lib.rs: 3fcd975d2e82323a5d93f4faa9e7a5bf1f022d45f9d0861fb2dea0786d60e64d
- graph-compiler/src/observation.rs: 61bdeada5c4b003a2192faabd59c5b8b6889a22b73fded567f961eb6a7dfe4e8
- graph-compiler/src/recorded_corpus.rs: 648f677e6697373578f98e817873f566d177548475154fc75e0e5cae40b9e9b1

Final-window repairs:
- final control-file verification compares the already-captured expected length/content without allocating according to a replacement inode; oversized post-capture manifest/attention/dense/meta/binding/marker replacements are pinned;
- binding and planned-file commits on Unix use retained-root directory descriptors with openat/linkat/renameat/unlinkat and directory sync, with explicit fail-closed non-Unix behavior;
- hooks immediately after the last generation check prove root redirection cannot mutate the replacement root.

PASS: lower 39/39; observation empty planned-scope 1/1; CLI guarded planned publication 2/2; compile-recorded 4/4; guarded stream 1/1; focused two-package check and all-target/all-feature clippy -D warnings; fmt/diff/claim. No server, #729, timing, or ignored test.

### comment 5304907344 — 2026-08-16T00:19:55Z — Casey-allard

Exact review found one remaining P1 in the same last-window class: compile-attempt marker begin/finish still stages, hard-links, removes, syncs, and rereads through paths after the retained-root check. A root replacement in that interval can mutate the replacement directory even though the guard owns the original.

Narrow repair: migrate marker staging/link/unlink/sync/verification to the retained-root APIs already used by binding and planned files; add post-last-check root-redirection tests for begin and finish; inspect the planned-residue final directory sync for the same seam. No new subsystem or scope is opened.

### comment 5304926841 — 2026-08-16T00:25:06Z — Casey-allard

Threat-model boundary for acceptance, to prevent scope drift:

#728 serializes every repository-supported producer with the same exclusive guard and requires binding-last, crash-retryable publication. Verified readers and the new commit/recovery primitives retain no-follow handles and generation evidence and fail closed on detected replacement. It does not claim that every pre-existing source-generator/tokenizer/merge path write is mutation-proof against an arbitrary external process that ignores the guard and renames the corpus root mid-system-call.

Therefore the active blocker set is the five new lower/transaction defects already enumerated: retained-parent root creation, dirfd residue reclaim, dirfd attempt-marker begin/finish, stable-idempotent binding revalidation, and exact-initial-length corpus hashing. We will not widen #728 into a wholesale rewrite of legacy source checkpoint/generator/tokenizer/raw-merge writes. Supported writers must still hold the common guard for their complete mutation interval; violations by repository code remain blockers.

### comment 5304938715 — 2026-08-16T00:28:32Z — Casey-allard

Phase-B lower transaction refrozen under the explicit cooperative-writer boundary.

Only recorded_corpus.rs moved: c884bd02f6fb2690349d8ef42e6b1aa7a6b50d22eb7afb281c59647d321a6df8. Other exact pins remain CLI 28e76b978fdb78952f5250b94e2a276cea576f3a7d5ac1b87744c0f14c7d4098, compiler lib 3fcd975d2e82323a5d93f4faa9e7a5bf1f022d45f9d0861fb2dea0786d60e64d, observation 61bdeada5c4b003a2192faabd59c5b8b6889a22b73fded567f961eb6a7dfe4e8.

All five agreed closures are implemented: retained-parent mkdirat root creation; retained-dirfd planned-residue unlink/sync; retained-dirfd attempt-marker begin/finish and exact verification; stable-idempotent binding revalidation after hook/sync; and exact-initial-length corpus hashing with EOF probe. The cooperative-writer threat boundary is documented in code.

PASS: lower 45/45, observation no-mutation 1/1, compile-recorded 4/4, guarded planned publication 2/2, focused clippy -D warnings, fmt, diff, claim wording. No server, #729, timing, or ignored tests. Exact acceptance audit is active.

### comment 5304946736 — 2026-08-16T00:30:40Z — Casey-allard

Final API-surface P1 before Phase-B acceptance: VerifiedCorpusMember validates a declared binding length, but its public Read/Seek delegates to the live file. A post-open append can make read_to_end/io::copy consume beyond the bound (and SeekFrom::End uses live EOF) before final generation rejection.

Narrow repair: cap Read to declared_length minus logical_position, define SeekFrom::End relative to declared length, and add a post-open append regression proving read_to_end returns only the original bytes while final verify rejects the changed generation.

### comment 5304958889 — 2026-08-16T00:34:12Z — Casey-allard

Public bounded-reader repair refrozen: recorded_corpus.rs 7cb0abda6ebcba9c6c726657e1ee1663e6fa70983df5799ce8edbbe6fd1f1fd9 (intermediate 0695846 is superseded). Verified members cap Read and SeekFrom::End to the binding-declared logical length; expected presence and initial length are rejected before any body hashing, including declared-absent and 1 TiB sparse mismatch regressions. Lower suite 47/47 plus focused lower clippy -D warnings, fmt, diff, and claim gates pass. Exact final audit is active; other pins remain unchanged.

### comment 5304973595 — 2026-08-16T00:38:36Z — Casey-allard

Exact 7cb0 audit verified all prior stream/transaction closures but found one residual aggregate boundedness P1: residue scanners cap each file body but can still collect arbitrarily many registry-shaped siblings into Vecs. Under the cooperative crash protocol legitimate cardinality is tiny. Repair: enforce a small fixed residue-count ceiling before body/path capture, let reader-only binding-temp checks reject the first temp immediately, and test over-limit preservation with no mutation.

### comment 5304989139 — 2026-08-16T00:43:15Z — Casey-allard

PHASE-B ACCEPTED — independent exact-hash audit CLEAN (no P0-P2 under the documented cooperative-writer boundary).

Accepted pins:
- recorded_corpus.rs 4bd4732c9b52ec7b1a84b231e9d24213942c10ce585d7aa79e3fe010badd30f1
- graph-cli lib.rs 28e76b978fdb78952f5250b94e2a276cea576f3a7d5ac1b87744c0f14c7d4098
- graph-compiler lib.rs 3fcd975d2e82323a5d93f4faa9e7a5bf1f022d45f9d0861fb2dea0786d60e64d
- observation.rs 61bdeada5c4b003a2192faabd59c5b8b6889a22b73fded567f961eb6a7dfe4e8

Final lower evidence: 48/48 tests; focused all-target/all-feature clippy -D warnings; fmt/diff/claim. Independent review verified aggregate residue bounds, retained-root commit/recovery, crash retry, exact stable-binding checks, fixed-length hashing, early expected-slot checks, bounded Read/logical Seek, canonical role/duplicate validation, and markerless compatibility. No builds or timing were run by the reviewer.

Next #728 tranche is managed-server final logical-root coordination and publication; #729 implementation remains unopened until that checkpoint is bounded.

### comment 5305010362 — 2026-08-16T00:49:50Z — Casey-allard

Phase-C pre-edit server design decision:

Use stage-first compare-and-swap, not a final common guard held through the long build.

1. Under existing canonically sorted server outer sessions, capture the final logical root as a typed exact base generation/absence (completion + binding/root identity). Build P/K and all corpus/cover/score members inside the private stage under its existing stage transaction; do not mutate the live final root.
2. Complete and validate the stage.
3. Acquire a canonically sorted final+stage common handoff token, CAS-revalidate the final base and exact stage generation, then atomically exchange/rename. If a direct CLI/observation writer committed B during the stage build, preserve B and refuse rather than overwrite it.
4. Rebind/adopt the promoted stage inode as the guarded final root and retain ownership through serving validation.

Reader side: one ordered shared pin spans precedence-root selection, completion, corpus identity, cover/report, and runtime load. Completion inventory becomes fixed and bounded: capped controls, exact-initial-length streamed hashes for large members, <=64 temporary entries, and the transient corpus compile-attempt marker excluded from bundle CID inventory.

Required tests: fresh publish; completed refresh; direct-writer CAS conflict; observation/CLI BUSY; crash prefix/resume; root replacement; 1 TiB sparse/post-open growth; >64 completion temps; exact final adoption. No server bytes move until this design boundary is accepted.

### comment 5305226357 — 2026-08-16T01:56:29Z — Casey-allard

Phase-C moving checkpoint (not a freeze/acceptance claim): root server library check passes after managed startup-pin rewrite. Focused real tests currently PASS for canonical attempt exclusion, stage-recovery BUSY, post-exchange old-marker cleanup, and bindingless legacy bootstrap with absent-precedence pins.

Six concrete server-only closures remain before hash freeze:
1. exact stage-content CID CAS across stage-guard release and final handoff;
2. acquire the stage common/session guard before prefix copy/reset, so the first stage mutation is covered;
3. exclude stale instruction-eval and other legacy derived outputs during refresh;
4. exact-initial-length plus post-EOF checks/hooks for bounded server readers;
5. a truthful legacy-only fixed inventory for compiled.r4g1 + compile_report without weakening current completion validation;
6. startup BUSY assertions and removal of superseded dead-code warnings.

One historical-witness test currently needs a fixture/assertion correction, not a production-code failure. Scope remains #728 server transaction only; no #729/model/timing/broad cargo.

### comment 5305362402 — 2026-08-16T02:37:15Z — Casey-allard

Phase-C exact freeze for independent acceptance audit (2026-08-15):

- server.rs a9e154c5d6a397464def15c6594bada96be394c7c23f75b5e5b833c1c42ee1e0
- recorded_corpus.rs e7a159fc44bb9f250746839bed1d79cb3b90e7375dd009fc6c967a1dc08346a8
- graph-cli lib.rs 0c975b2590d914da1e28e623414327af2126af02d2a8896baa834f968593a749
- observation.rs 61bdeada5c4b003a2192faabd59c5b8b6889a22b73fded567f961eb6a7dfe4e8
- graph-compiler lib.rs 3fcd975d2e82323a5d93f4faa9e7a5bf1f022d45f9d0861fb2dea0786d60e64d

Implemented stage-first three-root+stage content CAS, schema-/2 persisted base witness, managed absence-locking reader pins, guarded legacy bootstrap/migration, retained final authority through install, BUSY defer/409 behavior, binding/execution/cover/P/K reconciliation, fixed/capped streamed completion inventory, and crash-resumable private-stage construction before first mutation. Final caps: cover report 16 MiB, score report 1 MiB; generic no-clobber compares only exact expected length.

Evidence: server module 158/158; lower 59/59; root/compiler/CLI all-target/all-feature clippy -D warnings; fmt/diff; sparse report/no-clobber production regression. No ignored tests/timing/#729/model work. Exact read-only final audit is active.

### comment 5305399017 — 2026-08-16T02:48:48Z — Casey-allard

Clean handoff: current #728 transaction/server checkpoint is committed and pushed in draft PR #730 at `a902dbc2`; worktree is clean and no build is running. Exact remaining blockers are mixed-stage prefix recovery plus binding temp recovery, pre-attention/pre-dense exact P/K prefix preparation, direct cover/score common-guard participation, completion `/1` literal wire/CID pin, and legacy compile-report cap. Full hashes, evidence, and resume commands: https://github.com/UOR-Foundation/uor-r4/pull/730#issuecomment-5305398275

### comment 5305421872 — 2026-08-16T02:56:23Z — Casey-allard

Resumed repair batch on draft PR #730 (single editor; no #729/model/timing):

1. Recover lower binding writing/tmp residue under the retained private-stage guard, then require every bound-stage prefix to match the persisted typed base before reuse; pin A-to-B copy refusal, A restore, safe retry.
2. Pre-attention/pre-dense recovery requires exact requested canonical P+K; optional attempt is coordination only. Refuse attempt-only, mismatch, binding, payload, or foreign residue; never copy the attempt marker.
3. Source-update active attempt may tolerate corpus-body drift only when stable manifest/attention/dense provenance still exactly matches current sidecars and the guarded checkpoint; cross-provenance is terminal unchanged.
4. Direct cover/score and managed private-stage cover/score acquire/retain the common producer authority before completion preflight and member mutation; completed roots refuse unchanged; server re-CASes after reader-to-producer transition.
5. Completion-temp recovery and stage-marker cleanup require retained-root producer authority and dirfd operations; configured-external startup takes outer then common authority.
6. Add completion /1 literal bytes/CID, legacy report/control caps, sparse preservation, and streamed large-member hashes.

PR fast CI is also diagnosed: cargo audit passed; R5 audit-limits fails only on branch-added Result syntax and a test brace-scanner false positive. That policy repair will follow the #728 correctness freeze as a separate focused commit.

### comment 5305629686 — 2026-08-16T04:00:27Z — Casey-allard

Coherent resumed-diff audit (diff SHA 98353cdd...) found no P0 but requests one consolidated authority repair before acceptance:

- remove nonportable exclusive-to-shared lock conversion; managed roots retain producer authority through selection/load/install;
- recover stages guard-first, then recapture exact marker/temp state under authority before mutation;
- retain the original Stage-A authority through compiler validation and seal publication, closing the compiler-to-seal gap;
- public source compile must ancestor-classify and refuse any server-owned stage descendant before creating sessions/output;
- graph authority is topology-stable: canonical bundle graph outputs always use bundle-root authority regardless of mutable markers; true cross-root captures all source inputs before one output authority;
- preserve arbitrary explicit `--r4g1-artifact` support as a standalone nofollow exact artifact snapshot, never by inferring/locking `parent().parent()`; only proven canonical bundle topology uses managed bundle pins.

Raw public under-held capabilities will be narrowed/opaque where practical. Barrier tests will cover each recovery, ownership, completion-appearance, and provenance window. Completion/seal goldens and legacy bounded controls already pass review. Single editor is implementing this batch; #729/model-source remain untouched.

### comment 5305768639 — 2026-08-16T04:44:40Z — Casey-allard

Exact static review of the frozen server-recovery batch (base a902dbc2, diff 180d7cb2) returned REQUEST CHANGES with three bounded ownership findings:

- reject symlinked paths that enter a server-owned private stage before creating any child session/output;
- keep arbitrary standalone --out directories exact-root even when their basename is graph or graph-cover, while managed bundle calls pass explicit bundle-root authority;
- explicit canonical bundle startup must reject owner/seal-only private stages and fully validate the stable completion under retained authority before capture/load.

No model-source, matmul, or #729 expansion is involved. The earlier lock-conversion, recovery-recapture, compiler-to-seal authority, Stage-A seal, bounded-control, and explicit-artifact fixes passed the exact review. These three fixes plus four focused regressions are now the only active #728 batch; raw capability API cleanup is deferred as P2.

### comment 5305909965 — 2026-08-16T05:29:41Z — Casey-allard

Accepted coherent #728 checkpoint on base a902dbc2:

- exact diff SHA-256: 9fc2d36dd2c93ea62cf09998f1d3d812046c571c0203585ebb0bf5f3e8947ec0
- independent static verdict: P0 0 / P1 0 / P2 0 for the issue acceptance scope
- focused transaction/recovery/authority regressions: green
- cargo fmt, git diff --check, and no-run compilation for root, graph-cli, and graph-compiler: green

The accepted batch includes bounded corpus readers/publication, crash-recoverable binding and Stage-A sealing, three-root managed CAS/authority through install, explicit standalone/declared/provided graph authority, guarded completion recovery, explicit artifact capture, and BUSY/transient handling. The raw public capability cleanup is explicitly deferred as hardening.

This checkpoint is frozen but not yet committed or presented as PR-ready: PR #730 still has the independently diagnosed R5 audit-limits failure and three stale CLI fixtures to repair, followed by the repository gates.

### comment 5306128031 — 2026-08-16T06:30:56Z — Casey-allard

The independently accepted #728 implementation is now committed and pushed as `9aeecc2c` on draft PR #730. Full workspace tests, workspace clippy, fmt, both graph-format no_std checks, claim wording, and R5 audit all pass at this commit. Exact static review reports P0 0 / P1 0 / P2 0 for the accepted scope.

The PR remains draft only because the separately tracked #729 sampler/scale scope is still pending; #728 no longer exists only as an uncommitted local run.

### comment 5306462637 — 2026-08-16T08:08:08Z — Casey-allard

Merged via PR #730 in merge commit 830aafde4c571e6356ee920ef4cc1108eb493997 after the full merge-queue ladder passed. Recorded-corpus generations are now transactionally bound to exact attention+dense execution provenance across observation, compiler, CLI, and managed server publication/recovery paths, with bounded readers and crash/idempotence coverage.


---
_Comment/review ids in this file: 5304703847, 5304739435, 5304775041, 5304789128, 5304805302, 5304851263, 5304857772, 5304897965, 5304907344, 5304926841, 5304938715, 5304946736, 5304958889, 5304973595, 5304989139, 5305010362, 5305226357, 5305362402, 5305399017, 5305421872, 5305629686, 5305768639, 5305909965, 5306128031, 5306462637_

