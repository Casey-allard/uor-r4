# Observed-text memory answers through one reusable session boundary

> **Principal correction, September 22 UTC:** This is the preserved original submission account. The [source/evidence review](observed-text-session-review-2026-09-22.md) supersedes its fresh-final, multiword, target-free entry, bound-resume, fixed-depth and later-fact interpretations. Original 24/24 token answers are retained, on a repeatedly exposed symbolic-token panel. Original bindings are placeholders; the wrapper accepts the expected goal; completed EOS frames fail its validator. The corrected runtime and separately sealed replay are identified in the review and checks. Original numbers and historical text below are not silently rewritten.

September 21, 2026. Executes the [observed-text session brief](deepseek-observed-text-session-step-2026-09-21.md)
and the [relational-session review](relational-session-review-2026-09-21.md), based on the **reviewed
head `6fc34c1f`** of PR #1341. That PR was still queued when this work started, so protection was
preserved and the reviewed head was used directly as the implementation base rather than my original
`84ba72ce` or an older `main`. Isolation worktree `.worktrees/observed-text-session`; owner checkout
untouched. New module `learner/observed_text_session.rs` and mode `--mode=observed-text-session`.
[Evidence](../evidence/observed-text-session-2026-09-21.json).

## The task

The serving input is **observed tokens plus declared scaffolding only**: a tokenized document of
statements, an observed question clause, a declared clause delimiter and a declared entity registry.
No gold semantic role, subject/object/answer extent, follow pointer, relation id, depth or target
reaches the served path.

Statements are `subject marker object`, with **disjoint** marker vocabulary for the office assertion,
office redirect, project assertion and project redirect. A question clause is `subject marker` with no
object, so the marker denotes the goal. Worlds are **coherent**: for each goal the persons are
partitioned into ordered chains, so every person has exactly one statement per goal and following
redirects always terminates.

What is learned from declared development text:

| Component | Learned from | Served as |
| --- | --- | --- |
| marker roles | token votes over gold marker extents | marker run location and role argmax |
| boundary trimming | gold entity-span edges | per-token positive/negative fillers |
| action semantics per role | declared gold actions | continue versus terminal emit |
| goal per role | declared gold goals | the goal a marker denotes |

The learned policy starts from a **declared uninformed all-emit table**: 14/24 correct before fitting,
24/24 after, on the same development units.

## Results

| Arm | development /24 | **fresh final /24** | correct depths |
| --- | ---: | ---: | ---: |
| **observed-text session (primary)** | **24** | **24** | **24** |
| registry-membership shortcut | – | **20** | 22 |
| relation-only fixed-depth-two | – | **18** | 18 |
| reads disabled | – | **0** | 0 |

The same exact request and entity answers after **one**, **two** and **three** reads in different
supplied worlds, and the three-read composition is **withheld from development** (development exposes
one- and two-read chains only). Final worlds are a fresh draw with new assignments. Every request emits
the complete selected answer phrase — multiword where the fact is multiword — followed by the declared
terminator, through the boundary.

The two shortcut controls establish the required distinctions: a terminal answer phrase that is **also
a key elsewhere** defeats registry-membership continuation (20/24), and no fixed depth works (18/24).
Disabling reads removes all answers.

## Causal controls

| Control | Observed |
| --- | --- |
| one-position first-source edit, downstream fixed | entity, selected sources (486 to 488), depth (3 to 2) and the complete answer (489,491 to 513) all change together and **match an independently derived expectation**, while the goal stays invariant across the redirect |
| request goal changed, entity and every record fixed | different evidence selected; both answers match their independent expectations |
| required later fact removed | explicit **unresolved** stop with no emission |
| redirect cycle | **exhausted** outcome, not an answer |
| pause/resume mid-session | identical on **48/48** executions; the frame refuses a foreign binding |
| unknown/absent objective | unresolved with no emission |

## The reusable boundary

`TextSession::step` is the **one** causal implementation: it consumes the pending action (`Read`,
`Continue`, `Emit`, `Stop`) and the bound observations, and returns a typed effect. Teacher-forced
scoring, generation, the lesions, resumption and timing all call it. The frame carries the active
goal, the exact query objective, the owned captured phrase with its occurrence `(segment, start, len)`
provenance, the pending action, the read count, the emission cursor and the terminal reason, and
validates against an immutable binding (model, tokenizer, world id, world version, document hash).
`RelAction`/`CapturedPayload` are the corrected predecessor's vocabulary, reused rather than
re-invented.

Span identity is exact and multiword: the captured phrase keeps all its tokens plus its occurrence
position, so a repeated word or a shared prefix cannot collapse two spans, and the same phrase
occurring twice is distinguished by position.

## Honest scope

The clause layout, marker vocabulary, entity registry and terminator are **declared scaffolding**;
only the marker roles, boundary trimming, action semantics and goal table are learned. Development and
final share the same marker tokens and the same assertion/redirect forms — the withheld novelty is the
**three-read composition** and the **new assignments**, not new wording. No geometric arm is claimed:
the marker observation is a learned categorical vote table, consistent with the principal finding that
the fixed-equality relation test ties. The boundary is reused across this mode's scoring, generation,
lesions and resumption, but it is a sibling type to the earlier relational frame rather than the same
struct. No prose, no executed Rust task, no whole-path latency or energy measurement. Energy
`UNAVAILABLE`; whole-path D0-b not claimed. Retained roots `observed-text-session-{1..8}` (0 unlisted
each), with `-8` delivered.
