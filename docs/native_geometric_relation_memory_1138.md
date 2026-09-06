# Exact learned relation memory — #1138

**Subsequent result:** the [version-2 role-path correction](native_geometric_relation_role_path_1138.md)
now passes the bounded handoff. This record preserves the version-1 implementation,
its negative transfer verdicts and the resulting then-next proposal.

## Implementation and scope

The native role-reader parent is `61a24cfa4ce262fd974bc8e84f082a0489db3b58cfafb46c4c42a86e49c13184`,
delivered through protected PR #1142 at
`223aed71e770b158cebb0f0dd9a3d6be4f191829`. The first optional relation successor is
`dd85e3ed7faf9792242ae4b83b5290031b9adfdbfe09d623c10ce6955d21b2d5`, at
`.uor-models/native-typed-value-2026-09-05/relation-memory-model.json`.
The [immediate plan](integration/project-track.md#immediate-build-sequence)
requires a useful bounded relation-memory handoff before selective geometric
access and typed composition. These are capability boundaries, not issue-count
milestones. General conversation and alpha remain unqualified.

[`relation.rs`](../crates/uor-r4-core/src/native_geometric/relation.rs) implements
one learned association family, with selected owner and value words. Each
observed whole-word boundary offers at most fourteen ordered owner/value pairs
from the last eight words. The newly completed word must be one member. A sparse
integer score chooses assert, explicit revision, contradiction, or NoWrite.
There is no serving-side city parser or semantic label. NoWrite is the zero-score
alternative, with ties declining to write. The
[Rust fitter](../crates/uor-r4-core/src/native_geometric/relation_training.rs)
uses construction-only exact source-byte labels and typed actions to learn the
weights. It follows actual tokenization and per-token geometry while observing
every word completed inside a lexical piece.

The writer uses ordered local prime context, relative participant position,
signed relative H4 state/orientation and wrapped zeta-phase differences. The
reader learns a current-record/entry-action choice from exact participant
matching and local query context. Read fitting uses the labeled construction
writes; only complete generated evaluation tests the assembled learned writer
and reader. The existing #1137 recent-word selector remains responsible when
that exact source occurrence is still captured. Persistent reads feed the
existing observed entry, exact copying and learned completion path. A committed
read retains its relation version as well as source-token and byte endpoints.

## What remains exact and what is lost

| Level | Retained | Bound and information loss |
|---|---|---|
| Recent words | Exact bytes, order, source endpoints, H4 pose and phase state | Sixteen completed words; write features use the newest eight. Older syntax leaves this capture. |
| Relation versions | Exact owner/value atoms, monotone id, previous-version id, learned action and conflict status | Sixteen versions, FIFO replacement. Old versions remain addressable while retained; eviction can remove an active association or old witness. |
| Current directory | At most sixteen current-version references; exact owner comparison identifies membership | A coarse index pointing to exact members, not an averaged semantic summary. It loses old-version detail but the version store retains it within capacity. |

Assertion of a different value for an existing owner creates conflict; an
explicit learned revision resolves the active value. Contradiction marks
conflict. These are declared typed operator laws, selected by the learned
writer. Updating one directory entry does not rewrite unrelated records.
Evicting a current version forgets that association; an older retained version
must not silently become current again.

The record retains no general clause structure or separate predicate for
multiple independent attributes of one owner. This is one association family,
not general relational knowledge. Dictionary misses share prime zero in lexical
features; exact byte equality still distinguishes unseen participants. Local
context and phase binning are lossy. Neither phase-bearing records nor learned
scores establish geometric superiority. The session retains the existing
artifact-bound typed geometry and UOR model identity; record ids are local exact
references, not semantic hash distances.

Generated response bytes do not form new relation writes. Session instances own
private stores. Checkpoint validation checks bounded shapes, exact atoms,
current-version references, causal version ordering and committed copy evidence.
Old evicted source truth is supplied state, not authenticated by the checkpoint;
broader identity-scoped product import/export/forget remains #962.

## Evidence and complete cost

Construction comprises 28 raw prompts with offline write labels. The first fit
selects 215/215 distinct write frames and 28/28 read frames, exporting 2,142
writer rows and 51 reader rows. Its 64-epoch ceiling is a budget; fitting stops
at complete construction classification. These are integer perceptron-style
updates, not dense training or a differentiable serving reader. The report's
"margin updates" wording names its intended update rule; positive margin is not
an independently demonstrated claim.

The first assembled OPEN run passes 28/28 complete answers after all original
facts leave both the 512-token ring and sixteen-word capture. It covers initial
facts, explicit revision, retaining an unrelated fact, conflicting assertions,
unsupported owners, explicit contradiction and resolution of conflict. The
first preservation run retains all 62 responses. Final selection, first-use and session results follow below.

The source is prepared before fitting, with separate 28-case first-use rows and
new name/value assignments relative to supplied prior construction/development.
Grammar families are known. Acceptance was declared as 28/28 first-use answers,
62 existing preservation responses, 24 prior role-reader cases, eight binding
outputs, actual eviction and focused session/restore checks. Original source
SHA-256 is `b33877f017997c338d265baf98904569c0eff5254ebe801457bd0d22b5910211`.
This is not the final programme-wide held-out qualification.

Count observation, dictionary work, candidate construction, all writer/reader
score searches, directory reads/updates, exact-member reads, evictions and
ordinary generation. The first OPEN case spends 2,054,052 writer row comparisons
over 202 word boundaries, including 200 NoWrite decisions. Its persistent reader
uses sixteen directory probes and 168 row comparisons. This localizes the large
new cost to repeated write-candidate scoring; optimizing the small read directory
alone would miss it. Final counters add explicit source-presence and phase work.
No efficiency advantage is claimed for this storage extension.

The model JSON grows from 10,557,795 to 10,669,491 bytes (+111,696). Construction,
OPEN, preservation and final checks use the inherited cumulative monitor under
`.uor-models/native-typed-value-2026-09-05/relation-memory-*`. The complete
projection is 90 seconds model work, 850 seconds engineering commands and
384 MiB new storage, within 120/1,200 second cycle limits. The inherited
6,459,228,160-byte storage cap and 128 MiB margin suffice; no increase or deletion
was required to admit this cycle. Refresh actual totals before later work.

The example supports `prepare-relations`, `fit-relations`, `evaluate-relations`
and `verify-relations`; normal serving remains `r4 geometric`. Full-command
wall includes artifact loading/validation, report I/O and checkpoint verification.
The evaluator separately reports generation time and explicitly charges its
second source ingestion used to inspect eviction and restore. Focused checks
cover version retention, conflict handling, bounded eviction, invalid directory
references and actual multi-turn restored reads. Broader release qualification
is NOT_RUN.


## Final development decision — 2026-09-05

**RETAIN_DEVELOPMENT_RELATION_MEMORY; HANDOFF_NOT_MET. Keep #1138 open.**
The selected development artifact is
`blake3:16f4c10f6b79807868c7774872ba58776acd68a08c0c22054f49aca5206ecbeb`,
`relation-memory-coverage-model.json` in the same local artifact root. The
accepted predecessor remains #1137's `61a24cfa`; this relation experiment does
not satisfy #1139's capability prerequisite.

| Attempt | Construction | OPEN assembled answers / exact write sequences | First-use |
|---|---|---|---|
| Initial `dd85e3ed` | 28 documents; 215/215 writer frames, 28/28 reader frames | 28/28 answers; exact write-sequence metric added in the subsequent revision | 16/28 answers; twelve supported answers abstained because new-name assertions were absent |
| One new construction cohort `5df81b8b` | 56 documents; 403/403 writer frames, 56/56 reader frames | 50/56 answers; 38/56 write sequences, including the reopened initial first-use set | New reserved set NOT_RUN for this artifact |
| Three new construction cohorts `16f4c10f` | 112 documents; 773/773 writer frames, 112/112 reader frames | 56/56 answers and 56/56 write sequences | 26/28 answers; 21/28 write sequences; unchanged #1137 parent gives 12/28 answers |

All three artifacts and reports remain. The final writer has 2,172 rows and the
reader 51. The corrections add independently prepared construction names absent
from the inherited dictionary, leaving the selection law and serving operators
unchanged. The original first-use set became OPEN before the second fit. The
second source prepared a new reserved set before that fit; the third source
retained that same reserved set unchanged. The final artifact and source were
frozen in `relation-memory-coverage-selection.json` after OPEN, preservation and
session checks, before the reserved run. No fitting or selection-law change
followed its opening. Final source SHA-256 is
`579f71497eb665550c0839bf2f467412e65f27f8559f2f611b25f5952de79c2b`.
These are known grammar families with four name/value worlds, not independent
unrestricted language tasks or final programme-wide held-out qualification.

The final failures share one world: `hestin in Tavor.` is not written. Its
initial query produces ` Unknown.\n` instead of ` Tavor.\n`. A later assertion
of a different value can then look like an initial fact and produce
` Revik.\n` instead of the required contradiction abstention. All seven cases
in that world have an incorrect write sequence; five still emit the expected
answer. Answer-only scoring would conceal this causal failure. The writer has
participant-dependent neighboring prime features and participant-content H4/zeta
paths. The data establish inadequate transfer and sensitivity to construction
coverage; they do not isolate which feature causes it.

The selected development artifact preserves **62/62 earlier responses, 24/24
prior role-reader responses and 8/8 binding outputs**. Thirty-two generated Rust
case sources are byte-identical to previously checked sources; their prior
compilation/execution evidence is reused, with no new generated program or
repeat rustc claim. The actual `r4 geometric generate --json` output agrees on
every saved Generation field for the first long-context development case.
`relation-memory-preservation-verification.json` records these comparisons.

One actual session answers five successive reads correctly: Rome, revised
Perth, unrelated Cairo, contradiction Unknown, unrelated Cairo. Four exact
versions remain retained. A fresh session answers Unknown and has no shared
records. Restore after the first observed response token preserves source/version,
all causal state and subsequent tokens, and rejects a forged committed version.
A latent NoRead restore bug was fixed: response-entry validation delegates to the
joint selector when either a copy origin or a read commitment exists. Mandatory
copy validation still recomputes that choice. The previously used independent
lexical proposal is not authoritative for a joint NoRead decision.

An earlier whole-checkpoint equality check failed solely because historical /4
index reconstruction changes `work.memory_stale_rejections` (first turn 60 vs 4).
The verifier excludes only that diagnostic counter from equality and records
both actual values. It does not claim equal restore work. Every other checkpoint
field is equal. Failed checks and their exact snapshots remain available under
`relation-memory-restore-*`; the NoRead failure and corrected run are retained.

Final focused checks pass fourteen copy tests, two role-read tests and two new
relation tests. The release CLI/example and final test executable were rebuilt
and exercised. Formatting, native architecture policy and claim wording are
checked for protected delivery. Full legacy release, separate relation allocation
census and matched geometric attribution are NOT_RUN. Source-bounded arrays and
integer/table code are implementation facts, not a measured whole-host purity
or speed result.

The final JSON is 10,682,291 bytes, +124,496 over #1137. An exact relation record
occupies 168 bytes; the sixteen-record store/directory occupies 2,840 bytes.
The compiled Session is 11,512 bytes with optional storage accounted even when
inactive, besides its existing preallocated memory buffers. Complete reports
retain base model, tokenization/observation, H4/phase, candidate, dictionary,
score, exact-copy and directory work. The final first OPEN case still requires
2,054,052 writer row comparisons, 22,176 relative-phase subtractions, 12,704
dictionary comparisons and 67 write-directory probes; its persistent read uses
16 directory probes, 32 recent-source presence checks and 168 row comparisons.
These are representative counts, not a universal ratio or end-to-end speedup.

The initial 850-second engineering projection was revised after a real restore
failure and feature-unification rebuild cost. `relation-memory-restore-projection.json`
admitted the remaining build/test work inside the unchanged 1,200-second cycle
ceiling. Complete actual totals, all failed commands, storage and source identities
are in `relation-memory-checkpoint.json`; no limit was silently reset.

## Immediate successor within #1138

Change only the learned writer representation. Construct candidate-relative
local role context with participant payload identities masked, and a bounded
ordered R4/zeta role path that does not accumulate arbitrary owner/value content
as role evidence. Keep exact participant bytes, original geometric witnesses,
versioned storage, read/copy and typed update operators unchanged. This is an
unimplemented hypothesis; it must improve exact write sequences and complete
answers together. Reuse this now-OPEN failure set and the preservation/session
checks, then prepare separate first-use cases before selecting the design.
Do not enlarge storage or append another name cohort as the default successor.

A following cycle requires a fresh full projection. With the current warm cache,
allow about 650 seconds engineering (combined CLI/example rebuild, focused test
build, checks and reserve), 90 seconds model work and 384 MiB storage headroom.
The current cycle cannot fit another changed-core rebuild. Refresh the shared
ledger/cache and admit a complete cycle before execution. #1139 remains blocked
until this write-transfer handoff passes; its eventual cost work must include
writer admission/scoring, which currently dominates the new memory work.


## Charged final local checkpoint

The [compact evidence](evidence/native_geometric_relation_memory_1138.json) binds
source files, artifacts, reports and resources. This cycle charges **90.108/120
seconds model work** and **1,090.152/1,200 seconds engineering commands**, including
failed checks and corrected rebuilds. Cumulative model work is **1,228.950/1,800
seconds**, leaving **571.050 seconds**. Peak sampled direct model RSS is
789,020,672 bytes, below the 4 GiB target; sampling is not an exact peak guarantee.
The final sampled known storage is 5,674,254,336 bytes under the unchanged
6,459,228,160-byte cap. The monitor's tighter checked-growth ceiling leaves
429,568,000 bytes before stopping; preserve the separate 128 MiB margin.
Source/report metadata written after this sample remains charged at the next
snapshot; no deletion or storage-cap increase occurred. Paid compute was not used.
