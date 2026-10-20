# Whole-phrase geometric query updates — #973

**PASS_WHOLE_PHRASE_QUERY_UPDATE.** The UOR-R4 Geometric Language Model experimental path now carries a selected one-to-three-word phrase into a later query and read. All 864 new answers, paths, intermediate queries, canonical query encodings, selected spans and byte bounds pass. All 72 matched families and reloads pass. The existing learned artifact is unchanged; no fitting was needed. [Exact evidence](evidence/native_geometric_phrase_update_973.json). Normal model 15baec48 remains retained and unpromoted.

## Shared mechanism and environment

The previous experimental reader can select a complete terminal phrase, but dependent_language/runtime.rs::splice and updates admit only one lowercase word. The new explicit Phrase payload window admits one to three words, at most 16 bytes each and 50 bytes total, with one ASCII space between words. It retains the 128-byte query and 16-word bounds. The historical Word window preserves previous behavior and negative-control outcomes.

The same learned updater still selects the replacement occurrence. The full payload replaces that occurrence, and lexical::Query::new re-encodes the complete resulting query with the existing ordered canonical geometry. No new updater, span-selection, scheduling or writer parameters are fitted. The exact selected occurrence/source/byte extent remains separate from update-input intervention. Whole phrase transport and word identity do not themselves establish semantic geometry.

completion::generate_routed_payload shares the existing completion loop and scheduling executor. Normal phrase execution passes selected bytes through unchanged. UpdateFirstWord truncates only the update input, leaving the observed selected span unchanged. StalePayload uses payloads from actual matched baseline traces, indexed by the current clause; no expected answer or path is supplied to serving. UpdateDisabled suppresses committed query replacement. Old entrypoints use their unchanged Word window.

Rust prepares 864 authored rows: two name cohorts, two role orientations, two styles, four record rotations, three payload lengths, three placements and three source variants. Placements test a phrase at the first update, at the second update, and at both. Competing branches share the first phrase word and differ at the last component. Active source edits change that last component; inactive edits change the unused branch answer. An independent raw-language oracle validates expected full queries, paths, byte bounds and final answers before artifact loading. Each of 72 families contains four rotations and three variants. Vocabulary and grammar are familiar and overlap prior corpora; explicit question punctuation and shared role orientation along a chain remain. There is no fit or independent final holdout in this step.

## Actual output and controls

With records `alice did guide ruby amber.`, `ruby amber did help helen.`, `ruby birch did help oscar.`, and `felix did trust bruno.`, the supplied question `who did alice guide? who did they help?` expands its second query to `who did ruby amber help?` and produces `helen` plus EOS. Changing only the first source value to `ruby birch` expands that query to `who did ruby birch help?`, selects the other branch and produces `oscar` plus EOS. The first phrase word is identical in both cases. These are actual saved outputs, not oracle substitutions.

| Control | Complete answers | One-word intermediate cases | Multiword intermediate cases | Runtime errors |
|---|---:|---:|---:|---:|
| Full | 864/864 | 288/288 | 576/576 | 0 |
| LegacyWordOnly | 288/864 | 288/288 | 0/576 | 576 |
| UpdateFirstWord | 288/864 | 288/288 | 0/576 | 0 |
| StalePayload | 576/864 | 192/288 | 384/576 | 0 |
| UpdateDisabled | 0/864 | 0/288 | 0/576 | 0 |
| ReadDisabled | 0/864 | 0/288 | 0/576 | 0 |
| ExactIdentity | 864/864 | 288/288 | 576/576 | 0 |

StalePayload preserves all 288 baseline and 288 inactive cases and loses all 288 active-source cases. Its payloads come from actual baseline trajectories. UpdateFirstWord preserves all single-word cases and loses every multiword case while leaving the observed selected span intact. The legacy word-only admission produces 576 Shape errors on the multiword cases, which count as failures. Full has no errors. These controls test the frozen policy under intervention, not separately trained alternatives. ExactIdentity is the read comparator and equals Full; no new geometric metric advantage is claimed.

The first, middle and both intermediate placements each pass 288/288. Lengths one, two and three each pass 288/288. Both-placement cases use different familiar components for the first bridge and the later aliases, isolating transport from the separate overlapping-word selection question. Final answers in the new corpus are one word; prior multiword terminal answers are retained through the same new path.

## Retention, validation and lineage

The new Full path preserves 5,440 earlier full traces and completion decisions exactly: 1,728 prior terminal-span cases, 1,536 mixed one/two-read cases, 1,280 three-read cases and 896 completion cases, including 512 typed unresolved outcomes. Separately, all 19,008 prior span-control responses replay identically through the retained Word entrypoint, including earlier error outcomes. Earlier legacy controls remain identical at their retained adapter scope: 13,248 old outputs; 512 recurrent; 640 ordered; 1,408 language; 128 old-language through relative; 9,984 relative; 9,984 dependent; 19,968 scheduling; 19,968 output-credit; and 19,200 depth rows. Legacy replay and new Full transfer remain separate claims.

One optimized offline build, 47 focused tests (three new and 44 retained), and one explicitly executed sealed report pass. There was no fit, failed attempt, retry or old fitting campaign. Formatting, claim wording and diff checks accompany delivery. The queue status names are compatibility acknowledgements, not test execution. All 54 previous sealed roots / 1,334 files, three predecessor source/binary versions and both original dirty checkouts remain preserved. The new report adds 27 sealed files, for 55 roots / 1,361 files in total.

Artifact SHA256 44c0ebf482d5807354a2e2b4c3fd6ceb310dd076ce199536f801f6cad472264b is exactly the previous span artifact. Span selector [327682], context roles, updater [3], completion actions and writer parameters are unchanged. The artifact retains its original training provenance; this new serving-source version, corpus and executable are separately bound by the report and source-freeze.json. Executable SHA256 3dbfb12931e9f03f00a212b8b7237970db095fca659acce95b61c6335befbe6b.

Model/build/data/evaluation/control charge 298872/1800000 ms; parent 9515917/11310000 ms; shared 129266490/132950000 ms. Peak sampled process-tree RSS 3056238592 bytes, below 6 GiB. Two build threads, one model process, 384 MiB new-storage allowance and 128 MiB stop margin. Initial final receipt records 63037440 bytes added storage at cumulative high water; later delivery and notes are accounted in resource-final.json. Before execution, standing authorization extended the parent by 900,000 ms to 11,310,000 ms and by 128 MiB to 4,026,531,840 bytes. Shared ceilings are unchanged. External cost zero; no cleanup.

Local source snapshots, report, resource and restart receipts: /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/language-phrase-update-1/. Serving remains geometric routing/state with integer/table operations, without transformer, serving matrix products or a runtime provider. This establishes bounded whole-phrase reference transport over authored familiar grammar. General prose, arbitrary discourse reference, context-sensitive lexical roles and complete-path energy advantage are unqualified. Independent final holdout is NOT_RUN.

**Next:** Test overlapping lexical occurrences between the known query phrase and the answer using the unchanged artifact first. Freeze matched disjoint/overlapping phrase, repeated-word, role-reversal and source-change cases; record exact query-to-source occurrence matches, span features, admitted candidates and actual one/two-read outputs. The current span reader marks every source word matching any query word as a payload barrier, so shared words may erase the distinction between the known endpoint and answer. This is a source-level prediction, not yet a measured failure. If reproduced, preserve an occurrence-specific match correspondence before considering refitting; do not add a name exemption or train on unchanged collapsed features. Retain whole-phrase transport, prior answers and typed unresolved behavior; no new depth ladder or broad research restart.
