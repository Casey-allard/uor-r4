# Scoped correction-aware conversation memory — returned result

> Principal qualification: this returned account is preserved as submitted. The [independent review](scoped-memory-review-2026-09-22.md) corrects aggregate language/API/terminal counts, final replay scope, shared capacity-oracle defect, raw-language child reload claim and fractured-read claim. Use the [principal checks](../evidence/scoped-memory-principal-checks-2026-09-22.json) for corrected execution. Original store/session v1 and executable remain retained; corrected store/session v2 are intentionally incompatible.

September 22 UTC, 2026. Continuation of the reviewed ordinary-form milestone
([principal review](ordinary-form-binding-review-2026-09-22.md), merge `b218058f`). Source commit
`c4bd440a`. Executed receipt: [evidence](../evidence/scoped-correction-memory-2026-09-22.json), sealed
root `scoped-correction-memory-14`, binding `scoped_memory.rs` `6bdfde36…`,
`observed_text_session.rs` `c6529239…` (unchanged principal correction) and `competitive-reader.rs`
`e008b404…`, executable `7f610230…`.

## What was asked and what was built

The [completion brief](deepseek-scoped-correction-memory-step-2026-09-22.md) asked for one loaded
native path that observes an assertion about an unfamiliar entity, answers from it, accepts an
explicit correction, answers the appropriate current/historical question, keeps independent scopes
and relation-specific versions, preserves exact version/payload ownership across dependent reads, and
survives a real save/reload.

`learner/scoped_memory.rs` separates three responsibilities that were previously conflated:

* **Learned observation and intent.** The retained joint binder extracts the subject span, cue span,
  relation-bearing cue role and optional object span. A separately fitted intent table reads the cue
  span for statement intent (`Assert`, `Correct`, `NonAsserting`) and question intent (`Current`,
  `Previous`, `Initial`). No gold field, host dictionary, supplied correction flag or preselected fact
  reaches this path.
* **Exact versioned storage.** Records are addressed by an injective length-delimited
  `(scope, entity lexical key, relation)` key and carry exact id, predecessor, global commit, source
  id, owned payload and the learned update action. Version order, equality, serialization and
  predecessor traversal are deterministic infrastructure; **a parse score never decides which fact is
  current**.
* **Causal read and complete emission.** One `step` selects an eligible record under one pinned commit
  view, then reuses the retained continuation vocabulary and exact owned emission.

Semantics were fixed **before** fitting, in the module documentation and the receipt:

| Case | Declared behaviour |
| --- | --- |
| Assert | appends a revision for the address |
| Contradictory bare assertion | becomes current and is marked `conflict` |
| Same-value reassertion | a new revision with the same value; history is never deduplicated |
| Explicit correction | supersedes with the stated value; never marked `conflict` |
| `previous` | the previous **assertion** (immediate predecessor) |
| previous distinct value | a separate exact query, not reachable from learned question intent |
| `initial` | the earliest revision; an evicted one is typed, never absent |
| historical dependent read | the requested view applies to the first hop; later hops resolve `current` at the same pinned view |
| pin | the read view is fixed when the answer starts |
| capacity | the oldest record is marked evicted and its payload released; traversal through it is typed |

## Measured behaviour

| Panel | Complete answers | Depth |
| --- | ---: | ---: |
| Development (open, 84 supervised clauses) | **16 / 16** | 16 / 16 |
| Exposed regression (design-informed) | **11 / 11** | 11 / 11 |
| Final (fresh, disjoint unseen entities/values) | **11 / 11** | 11 / 11 |

Learned quantities on the loaded path: binder **84/84** exact joint hypotheses and cue-role matches;
intent **48/48** statement and **36/36** question; question/statement branch **25/25** in the panels;
nonasserting inputs wrote nothing **3/3**.

Controls, each measured on the same scripts and inputs (complete answers, primary = 38/38):

| Control | Complete | Effect |
| --- | ---: | --- |
| NoRead | 0 / 38 | a read returns a typed no-read terminal |
| UpdateDisabled | 1 / 38 | writes rejected; the store stays empty |
| Unpinned | 38 / 38 in the replays | no mid-answer update occurs there; the dedicated intervention separates it |
| Unscoped | 33 / 38 | collapsing the address lets scopes alias |
| ParseScoreAuthority | 19 / 38 | choosing by source parse score picks stale records |
| capacity 2 | 16 / 16 (development) | typed eviction / no-history instead of silent absence |

Interventions through the same loaded runtime, all matching independently derived expectations:

* **Pinned read view.** `Cedar` resolves through `Oren`; a correction of `Oren`'s office arriving
  between the two hops is invisible to the in-flight answer (**`Cedar Annex`**), while the unpinned
  control returns the post-correction **`Office Park`** — a splice that existed in no single view.
* **Owned capture.** A captured payload survives replacement of its origin; `origin_is_live` is true
  before and false after, while the emitted answer remains the pre-replacement value.
* **Physical storage order.** Reversing the record vector leaves every answer identical.
* **Changed source.** Changing one correction in the script changes the dependent answer
  (`Cedar Annex` → `Office Park`) and keeps full oracle agreement.
* **Cycle.** Two mutually-following relations produce a typed `Cycle`, not a silent answer.
* **Save/reload.** The committed store written to disk and reloaded in-process reproduces every
  answer, and a **separate process** (`--reload-check`) loading the same files reproduces the same
  emitted tokens and terminals.

## The obstruction this step actually resolved

The first fresh-final draw (root `scoped-correction-memory-12`, retained) scored **9/11**: one unseen
three-subword-token entity name was cut by one token in its clause-initial occurrence, so that
occurrence's exact lexical key differed from the same name's clause-internal occurrence and the
dependent answer was `Unresolved`. Measurement localized the cause to the **observation boundary**,
not to identity, admission or version eligibility: the segment-length feature outcompeted the
neighbour-token feature because the neighbouring token was unseen, and the clause-initial form has
fewer supporting features than the internal form.

Two changes followed, and the informed panel is reported **exposed**, not as a fresh win: (1) the
development supervision was widened from 42 to 84 clauses with entity names spanning one to four
subword tokens and five office/project values, so boundary variation is actually supervised; (2)
correction and nonassertion cues were made **copula-free** (`became`, `might be`), because a cue
ending in `is` shares its prefix with `is <value>` and could absorb the copula plus the value's first
token. The fresh final population (new entities `Una/Pia/Soren/Kestrel`, new values
`Cobalt/Mica/Ridge/Delta/Basalt/Dune`) was drawn after those changes and scored 11/11.

## Geometric contribution actually measured

**None in this milestone's serving path.** The retained H4 artifact from the ordinary-form milestone
is unchanged and was not re-swept: its descriptor code search is still censored by an exact-hit
criterion that is already saturated, so another sweep would be uninformative. The scoped store is
exact deterministic infrastructure, and the brief is explicit that version order, equality and
predecessor traversal do not need to be approximated geometrically. The retained geometric tools
(binary-icosahedral `2I`, signed transport, retained Hopf fiber, paired-H4/icosian bridge) remain
candidates for a *witnessed* orientation, interference, transfer or compactness problem, with a
declared margin or a matched-cost objective as the discriminating test.

## Limitations

* Small authored domain and authored clause forms; scope is host-authenticated, never inferred.
* The observation boundary remains the fragile layer: widened supervision fixed the measured case but
  is not a proof of general boundary robustness on arbitrary unseen strings.
* Learned intent covers three statement and three question intents; the previous-distinct query and
  the exact historical views beyond them are API-level, not learned language.
* One deterministic process-local store; no replication or durable multi-writer semantics.
* Energy `UNAVAILABLE`; whole-path D0-b, broad language and general reasoning remain unqualified.

## Next architectural decision

Consume one Q8-derived computation result through this same owned session so a changed computation
changes a subsequent read or complete answer (milestone 2 in the canonical plan), then E/S language
prediction and executed Rust through the same artifact. Scope-separated retained state is now a
concrete structural-memory mechanism: unrelated observations no longer erase a relationship as time
advances.
