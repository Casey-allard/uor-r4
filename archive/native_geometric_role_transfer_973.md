# Unchanged occurrence-role transfer — #973

**FAIL_UNCHANGED_ROLE_TRANSFER.** The unchanged qualified occurrence-role artifact passes 120/204 new complete cases. All 200 most recent prior Full traces replay exactly. This measures a broader boundary of the retained artifact; it is not a regression caused by a model edit. Normal model 15baec48 remains unchanged and unpromoted. References #973.

| Frozen development panel | Full | CoverageDisabled | QueryRolesDisabled |
| --- | ---: | ---: | ---: |
| Optional question-word availability | 12/60 | 24/60 | 12/60 |
| Styled occurrence-role recombination | 60/96 | 24/96 | 48/96 |
| Familiar/unobserved predicate anchors | 48/48 | 36/48 | 48/48 |

The 204 cases contain four source records, two dependent questions, four record rotations and paired one-record changes. The raw authored oracle is independent of model geometry and roles. It interprets the authored `please tell me` prefix and `today`/`tomorrow` modifiers only for evaluation; inference receives unchanged original bytes. Acceptance was written before model loading. Successful generation requires exact first payload, actual next query, compatible source identities, committed source/span/byte bounds, final bytes/EOS or the expected typed unresolved result, and state continuity. Final text alone is insufficient.

## Measured boundaries

**Global availability is not query obligation.** The 12 baseline valid/missing/conflicting cases pass. Replacing only an unrelated record's name with `please`, `tell`, `me` or `who` makes all 48 interventions fail before selecting the first payload. The current coverage gate requires every query identity present anywhere in the source set to occur in each candidate witness. The unrelated occurrence therefore blocks the correct source. Disabling coverage restores the first payload in all 60 cases. It restores complete behavior for baseline and `who` cases (24/60), but the other 36 stop with `AmbiguousUpdate`.

**The updater has a separate availability shortcut.** Its unchanged rule `[3]` admits an unmatched query word with any globally matched word before it. For the second query `please tell me who did they help?`, an unrelated `please`, `tell` or `me` changes which prefix words satisfy that rule. Source inspection predicts 4, 3 or 2 admitted update sites, versus one for baseline and `who`; these counts are source-derived, not an additional generated test. The sealed actual outcomes show the corresponding ambiguous update. A coverage-only change cannot qualify this panel.

**Styled names expose role generalization gaps.** Twelve cases containing `today will will call ...` classify both occurrences of `will` as context and fail to select the name. Twenty-four cases carrying `amber will` classify its final word as context and select only `amber`. In 12 of those cases the final answer happens to be correct, but the transported phrase and next query are wrong; acceptance correctly rejects them. The combined styled panel is not a clean independent intervention on every modifier. These recorded role observations and payloads localize the failure; they do not establish a universally correct role rule.

**Predicate reuse succeeds in this scope.** All 48 cases using familiar `help` or unobserved predicate anchor `trust`, with endpoints `bruno` or `will`, preserve correct answers, source changes and paths. No general unseen-language or geometric metric advantage follows from this authored panel.

## Controls, preservation and execution

Full equals ExactIdentity on all 204 complete Generated structures, including failures. Reload equals Full on all 204. ReadDisabled and UpdateDisabled produce zero correct answered responses. UpdateDisabled preserves eight correct typed-unresolved baseline cases because lookahead stops before a read is committed. Source/query role controls are separate from coverage controls; disabling either is not a repair.

The candidate remains SHA256 `712de3158ca14241b549043f743709911ca6bbc1bf8adc271d990d8616c8df42` at its original occurrence-role attempt-3 path. All 83 compared predecessor source files are unchanged; the only prior module edit registers the new test report. The previous 6,688-trace campaign is retained, not rerun. All 69 prior sealed roots/1,671 files and 17 predecessor source/binary versions are preserved, as are both original dirty checkouts. This report adds one sealed nine-file root: 70 roots/1,680 files in total.

One optimized offline Rust build passes 34 focused tests. One frozen-artifact evaluation ran; no fit, retry or runtime change. Build plus evaluation cost 167,797 ms; peak sampled process-tree RSS 3,282,223,104 bytes, below 6 GiB. Before execution, standing owner authorization was recorded for +128 MiB to parent and shared storage allowances; model/time ceilings were unchanged. There was no external compute cost or deletion. Final held-out qualification remains NOT_RUN; no promotion or general-language claim.

## Next action

First make query participation independent of irrelevant records: learn required-evidence and dependent-update eligibility as distinct query-occurrence decisions inside the existing geometric read/update loop. Begin by verifying that full canonical center identity and local query context distinguish optional-prefix, required relation/name and reference-site observations; the current 15-context-word center code collapses too much for this new decision. Use final-output-compatible witness/update credit and matched missing-relation/irrelevant-source contrasts. Keep source/query context-role tables, exact occurrence/span identities and parent operators fixed initially. Qualify the 60 coverage cases, actual update sites and prior successful traces; neither a prefix/name exception nor simply disabling coverage is sufficient.

Then address the 36 styled role failures through the existing role-learning seam with mixed target/known-endpoint credit. Preserve all successes from this diagnostic and earlier artifacts. The separate roles, obligations and update eligibility must not silently become one Boolean. Current state owns sequencing and resource projection; this correction is a hypothesis, not implemented behavior.

[Machine evidence](evidence/native_geometric_role_transfer_973.json). Exact artifacts, source/binary receipt, acceptance, actual responses, analysis, resource ledger and restart are under `.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/role-transfer-1/`. Original sealed candidate and evidence paths remain unchanged.
