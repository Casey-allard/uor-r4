# Independent lexical-neighbor transfer — #973

**FAIL_INDEPENDENT_NEIGHBOR_TRANSFER: 2,016/2,304.** The unchanged experimental artifact 60d19679 preserves all earlier qualified outputs but fails every tested three-word name with an interior will. No model or learning parameters changed. Normal model 15baec48 remains unpromoted; #973 remains open.

## Frozen qualification and actual results

Acceptance was frozen before the source/binary freeze, OS-random seed draw, data preparation or model evaluation. Seed 240836092112233 uses six OS-random bytes interpreted as an unsigned 48-bit value and passed exactly as Rust u64. The Rust generator rejects only excluded or duplicate lexical tokens, with no model feedback. It produced 3,776 new seven-letter tokens in 3,776 draws, disjoint from 135 collected tokens and 15,584 prior raw inputs across 45 sealed data sources plus all nested artifact vocabulary. Data preparation decoded no model.

The 2,304 cases cross eight endpoint shapes, four first/second query-direction combinations, two polite-prefix states, four source positions, three source variants and three continuation outcomes. Both intermediate and final endpoints use the selected shape. Context stays at four records/two clauses, 16 words/128 bytes per record, 256 prompt bytes, three-word/50-byte payloads and 96 runtime steps. All sources use the existing today/tomorrow style. Each position stratum has its own fresh lexicon; this measures coverage across record positions, not paired rotation invariance for identical words.

| Endpoint shape | Valid | Missing-compatible | Conflicting |
| --- | ---: | ---: | ---: |
| One new word |96/96|96/96|96/96|
| Two new words |96/96|96/96|96/96|
| will + new word |96/96|96/96|96/96|
| New word + will |96/96|96/96|96/96|
| will + two new words |96/96|96/96|96/96|
| New word + will + new word |**0/96**|**0/96**|**0/96**|
| Two new words + will |96/96|96/96|96/96|
| Three new words |96/96|96/96|96/96|

Every case requires actual initial payload, rewritten query geometry, compatible source set, committed source/span/byte boundaries and exact EOS answer or typed unresolved outcome. Accepted answer text is frozen once from the report-only typed raw relation oracle and judged by answer_oracle::accepts; grammar labels never enter inference. Full and ExactIdentity structures agree 2,304/2,304, including failures; reload agrees 2,304/2,304. ReadDisabled and UpdateDisabled produce zero correct answered dependent outputs among the 768 intended answered cases. Unresolved control outcomes that stop during lookahead do not independently establish relation understanding. Active/inactive answer interventions pass 224/256 groups, with the 32 failed groups all in the interior-will shape.

Retained comparisons remain exact: 492/492 latest complete outputs, 300/300 earlier sealed outputs, 200/200 prior outputs and 6,688/6,688 actual candidate/current8055 outputs, including 512 typed unresolved. Historical adapter-control campaigns were NOT_RERUN; their earlier receipts retain their scope. The 2,976 training answers were not re-evaluated this step; their prior result remains bound to the unchanged artifact/source. The new panel was independent at first evaluation and is now exposed development evidence. Broad grammar, general prose, arbitrary discourse, coding and complete-path energy savings remain unqualified.

## Localized failure and counterexample

All 288 failures are Exhausted. Every intended first payload includes a middle will marked as source context; initial selection, rewritten lookahead query, compatible sources and final outcome are incorrect. For example, the intended first payload in today izpkgzr will call zlrkawr will ikbjwdx tomorrow. is zlrkawr will ikbjwdx. The source role key for its interior will is (center 13, left 64, right 64), where 64 means an unknown neighbor. It falls through to the inherited context default, creating a whole-span eligibility barrier. The corresponding expected query gives the same key a learned content role. This localizes the failure before the correct first payload is read; it is not merely a third-word emission error.

The retained 492 panel also contains 48 auxiliary-will source occurrences across 24 rows/four unique records with the same key, including navor will trust tavin. Here will has a different intended role. The predicate trust is absent from all three current canonical word inventories; unioning existing context words into role anchors adds no identity. The preserved source fit has 108 positive middle-name credits, but masking those neighbors alone does not recover the distinction erased by this local observation.

A deterministic assignment to this one key cannot retain both intended labels. Its actual final-output effect under a hypothetical source-rule override is NOT_RUN: downstream matching may affect the result. No word exception, literal trust insertion, copied query rule, causal override or new fit was performed. Canonical word identity is preserved; this is a collision in the finite role observation, not in canonical identity or the geometric distance calculation.

## Execution recovery, preservation and cost

The first evaluation completed all 2,304 fresh responses, then the configured storage guard stopped it while retained checks were underway. Its 619,303,063-byte response file, observations and partial retention remain unchanged. The interrupted attempt was marked INTERRUPTED_RESOURCE_LIMIT and sealed/verified with a small Rust utility using the preserved report_output implementation. It is not itself a completed model gate.

A report-only resume verifies the original manifest, source/binary/execution receipts, acceptance, data and seed. It reassesses the saved Full traces, checks saved control consistency, runs one actual reload Full per case to recover the unpersisted equality check, and finishes retention. Original ExactIdentity/read/update executions are retained; the four-control panel was not regenerated. The new attempt references the original large response file instead of duplicating it. No redraw, fitting, artifact change or acceptance relaxation occurred.

Two optimized builds each pass 53 focused tests, including deterministic environment, lexical exclusion and pre-model overlap rejection. The copied Rust sealing utility was compiled/executed separately. Total charged preparation/build/evaluation/sealing/resume work is 437,510/600,000ms; parent 15,435,682/15,810,000ms; shared 135,186,255/135,650,000ms, retaining the prior 196,620 ms reservation. Peak sampled process-tree RSS 3,428,466,688 bytes is below 6 GiB. Necessary local extensions were recorded before use: parent/shared time +300,000 ms and storage +512 MiB then +256 MiB; step storage 512→768→1,024 MiB as full trace size was measured. The 128 MiB stop margin remains. No paid compute or destructive cleanup.

All 80 prior sealed roots/1,828 files and 28 prior model source/binary versions are unchanged. This step adds three sealed roots/26 files and two model source/binary versions: 83 roots/1,854 files and 30 versions total, plus the separately receipted small sealing utility. Both original dirty checkouts and all negative artifacts remain at their original paths.

## Evidence and delivery

Evidence directory: /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/independent-neighbor-1.

- Frozen acceptance: acceptance.json, SHA256 8bb96357a1b4cd20310f24147a073cc20f5bf147283027b57a62a4c2e5b7ad21.
- Sealed data: attempt-1/data.json, SHA256 238b2adc8d231e8181adbed197cfe54377a5e08329c04d9a1dbd6e8df48e1bfd.
- Original fresh responses and failure observations: attempt-2/, sealed as interrupted.
- Completed gate: attempt-3/summary.json; resume-equality.json and response/failure reference files bind the original execution.
- Diagnostic and unrun limits: diagnostic-review.md; source-review.md; partial-analysis.json.
- Source/binaries: source-freeze.json and source-freeze-2.json; independent-neighbor-tests and independent-neighbor-tests-2. Final source matches delivered Rust exactly.
- Preservation/resources/delivery/restart: verification.json, preservation-verified.json, resource-final.json, delivery.json and CODEX_RESUME.md. Latest local receipts supersede numerical snapshots.

Artifact remains /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/unknown-neighbor-1/attempt-4/candidate.json, SHA256 60d19679347ff349c0809353e3cca48925314a880b59c277bc9977ebd816ad73. Read the [machine receipt](evidence/native_geometric_independent_neighbor_973.json) and current state for navigation. Deliver the report and result through a protected PR referencing #973, without closing its broader acceptance or promoting a model. Queue compatibility acknowledgements are not local tests.

## Next action

Freeze matched source-role controls for an unfamiliar three-word name containing interior will and a retained auxiliary will before an unfamiliar predicate. Audit existing ordered source/query occurrence correspondence and geometric matching as additional role evidence, preserving full canonical identities and orientation. Establish which contextual observation separates the intended roles and test its causal effect on the blocked first span before fitting. Do not insert exposed names/predicates or copy the query rule into the source table as a repair. If the observation is adequate, learn the smallest source/query-consistent role correction from training-only evidence, retain the current492+300+200+6688 outputs and now-open2304development cases, and reserve a separate independent final draw after design selection. A same-key label collision alone does not establish the final-output effect of an unrun override. Keep the normal model unpromoted and broader session/prose work separate until this selected correction is qualified.

Third-read and whole-phrase integration is already implemented and retained; do not repeat it as new integration work. Persistent raw-context/session access is a later useful connection, but it is not a correction for this source-role gate.
