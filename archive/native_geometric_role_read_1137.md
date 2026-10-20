# Role-aware source selection and causal entry — #1137

## Bounded handoff, 2026-09-05

**Accept the bounded role-sensitive binding/entry handoff.** The selected native
artifact is `blake3:61a24cfa4ce262fd974bc8e84f082a0489db3b58cfafb46c4c42a86e49c13184`,
at `.uor-models/native-typed-value-2026-09-05/role-read-local-model.json`.
It extends the unchanged `5f590f1c` development correction. Preserve that parent,
the previously selected `d095a1ab`, and the first new fit's negative result.
The [current plan](integration/project-track.md#immediate-build-sequence) still
requires exact learned relation memory, selective geometric access and typed
operator composition before broader capability qualification. This is not alpha.

## Implemented operator and retained information

`role_read.rs` replaces the independent initial prefix/copy decisions with one
learned joint choice: a retained occurrence and optional lexical prefix, or
NoRead and a lexical token. The action vocabulary comes from construction
targets. Serving receives raw text and a model, without supplied semantic roles,
an answer buffer, a city parser or an expected source index.

The scorer uses relative local word context, exact word equality at relative
offsets, ordered prime context, source recency and the existing signed H4/zeta
relations. It learns sparse scalar weights in Rust and exports quantized integer
tables. These are learned contributions from local role features, not a separate
semantic role classifier or a learned discrete role codebook. The first fit also
included pooled query identities; the selected `local_roles_only` revision removes
those features. Its identity-bound flag preserves the first fit's execution law.

The sixteen-word capture remains unchanged. Exact word bytes, occurrence end,
source-byte end, pose and phase state remain recoverable. The dictionary's prime
addresses are equality identities, not a semantic metric; dictionary misses share
zero while exact byte matching still distinguishes unseen words. Local features
retain nearby ordering and participant matches, but lose distant syntax and full
clause structure. Existing geometric phase binning remains lossy. Equal-spelling
source matches are latent fitting alternatives, so these results do not identify
the semantic role of every duplicate occurrence.

Prediction is transient. Observing the selected lexical prefix commits the same
occurrence/version before copied byte zero; direct copying commits on its first
observed byte. NoRead is also committed, preventing a second source selection.
Mismatched initial observation clears the selection; an interrupted committed
copy retains provenance but disables copying as in the existing copy contract.
Changed source provenance rejects further copying. Checkpoints validate the
joint choice, observed prefix, exact source and cursor. Existing numeric
precedence, /4 memory, copy completion and committed-byte dispatch remain.

## Measured behavior

Construction contains 480 raw prompt/response pairs: 160 upstream numeric
responses are verified unchanged and 320 joint source/entry decisions are fitted.
Both fits use 24 epochs and learning rate 0.1. Epoch selection uses the quantized
construction objective; no first-use result selects a parameter or feature.

| Evaluation | `5f590f1c` parent | Selected role reader |
|---|---:|---:|
| Existing preservation | 62/62 | 62/62 |
| Existing binding outputs | 8/8 | 8/8 |
| Earlier sixteen-case wording set, now OPEN | 13/16 | 16/16 |
| Earlier 32-case transfer set, now OPEN | 16/32 | 30/32 |
| New first-use set | 13/24 | 24/24 |

The first role fit (`ed05aea4`) selected 320/320 construction decisions but
preserved only 61/62 responses, incorrectly abstaining on the older Oslo answer.
The local-role revision retains 320/320 construction decisions, restores Oslo,
and reduces learned rows from 4,804 to 3,973. This is the observed result of
removing pooled query identity; it does not prove that identity leakage caused
every earlier failure. Both model files and their reports are retained.

The first-use source was prepared before fitting, with split and retention
assertions passing before any evaluation. The selected artifact and source hashes
were recorded before opening it. The 24 cases contain 20 prose answers (including
four unsupported queries) and four Rust arguments, with wording, reversed relation
wording, source order, updates and absent entities. Names and values are new
relative to the supplied construction and prior development material; grammar
families are known. This is a bounded first-use result, not unrestricted language
or a final programme-wide held-out qualification. No parameter or selection-law
change followed opening. Two older transfer cases still abstain incorrectly.

All four new Rust functions compile unchanged and pass seven identity inputs
each, including the i32 endpoints: 28 assertions. All 32 earlier generated Rust
sources match their previously checked bytes/hashes. An actual `r4 geometric
generate` run produces ` Merok.\n`; every Generation field equals the evaluator's
result. The CLI artifact identity matches; its additional UOR address metadata
is recorded separately.

## Complete work and resources

The operator admits at most sixteen source words, eight learned action forms,
512 feature slots and 16,384 sparse rows. This artifact has three action forms
and 3,973 rows. Feature construction still scans bounded word pairs and performs
dictionary searches; the current implementation also computes the parent copy
context before its role dictionary lookup. The inherited initial lexical offer
still executes before the joint choice. Those costs are counted, not claimed
eliminated. Selected read scores compare joint actions internally; the winning
operator's `Base + 1` score is a dispatch marker, not calibrated LM confidence.

| Same 62 responses, one pass | Parent Full | Role Full | Role dispatch disabled |
|---|---:|---:|---:|
| Complete generation time, including session setup/encoding/ingest/output | 49.267 ms | 46.517 ms | 65.037 ms |
| Ordinary score lookups | 1,503,295 | 1,470,063 | 1,772,626 |
| Memory score lookups | 291,498 | 277,122 | 427,481 |
| Copy/role score lookups | 77,151 | 44,238 | 44,238 |
| Copy dictionary comparisons | 23,104 | 14,224 | 14,224 |
| Copy equality byte comparisons | 8,351 | 8,608 | 8,608 |

All 62 outputs, token sequences and final causal states agree with dispatch
disabled. Artifact loading/validation, report serialization and process startup
are outside the generation subtotal and included in the recorded full command
wall times and cumulative budget. The parent/role timing difference is one pass,
not a robust speedup estimate. Copy-geometry suppression reaches 45/62 and copy
suppression 32/62; these are within-artifact sensitivity results. No matched-refit
geometric superiority is established.

The model JSON grows from 10,274,524 to 10,557,795 bytes. Local model work uses
57.201/120 seconds this cycle, including failed/passing fixture runs and generated
code. The cumulative ledger is 1,138.842/1,800 seconds, leaving 661.158 seconds.
Peak sampled direct model RSS is 613,810,176 bytes; engineering compiler and
unmonitored fixture peaks remain unavailable. Engineering commands have their
separate cumulative record; final totals and retained storage are in the local
checkpoint. The necessary 512 MiB preauthorized storage increase raises the cap
to 6,459,228,160 bytes, retaining the 128 MiB stop margin. No material was deleted
and no paid external compute was used.

## Reproduction and limits

The existing Rust example now supports:

```text
prepare-role-read SOURCE_V3 NEW_DIRECTORY
fit-role-read MODEL SOURCE NEW_MODEL NEW_REPORT
evaluate --model MODEL --source SOURCE --output-dir NEW_DIRECTORY --controls full
```

Use the existing cumulative monitor and a complete build/evaluation projection.
The model API is `Model::fit_role_read`; inference and checkpoint loading use the
ordinary native CLI/service interfaces. Local evidence is under
`.uor-models/native-typed-value-2026-09-05/role-read-*`, particularly
`role-read-source/`, `role-read-selection.json`, the two fit reports, preservation,
OPEN/first-use/control reports, CLI output, compiler receipts and
`role-read-checkpoint.json`. Earlier reports used the example's generic old copy
scope prose; this record supplies the actual joint-operator scope and the example
metadata is corrected. Their original measurements and outputs are preserved.

Focused checks pass fourteen existing copy tests and two new real-fit tests for
observed commitment, pre-byte-zero restore, provenance rejection, quantized row
validation and artifact identity. Broader release qualification is NOT_RUN.
The next build is #1138: learn exact associations and revisions that remain
answerable after raw-window eviction, reusing this accepted bounded reader.
