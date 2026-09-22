# Ordinary-form structured argument binding — returned result

**Principal qualification, September22:** preserve this returned account at its submitted scope. The [independent review](ordinary-form-binding-review-2026-09-22.md) verifies the answer counts and cross-BPE joins but corrects three interpretations: root8 replays root7's first final panel; descriptor search cannot improve an already96/96 exact-hit objective; and the original source-object edit also removed trailing `downtown`. The matched lexical-membership control and isolated source edit are executed separately in the [principal checks](../evidence/ordinary-form-binding-principal-checks-2026-09-22.json). The next milestone is scoped correction-aware unfamiliar-entity memory; consumed Q8 is sequenced after that, not required in the same change.


September 22 UTC, 2026. Continuation of the corrected draft PR #1344 on top of the merged
review/documentation delivery `38314380`. Source commits `3537280c` (implementation) and `5a6df770`
(result, evidence, resources). Executed receipt: [evidence](../evidence/ordinary-form-argument-binding-2026-09-22.json),
sealed root `ordinary-form-argument-binding-8` binding `observed_text_session.rs` `626d6fed…` and
`competitive-reader.rs` `5774a1c1…` at `git_rev 5a6df770`, executable `75ac2ba1…`.

## What was asked and what changed

The [completion brief](deepseek-structured-binding-completion-step-2026-09-22.md) asked for complete
evidence-dependent answers across ordinary sentence and question forms — interrogatives, reordered
arguments and trailing non-answer material — with exact lexical identity across real BPE boundary
differences, through the same loaded causal session, plus a competent order-aware categorical
comparator and a coherently fitted ordered-H4 hybrid on equal support.

`learner/observed_text_session.rs` now decodes and serves **one declared structured objective** over
the joint hypothesis `(subject span, cue span, cue role, optional object span)`:

- the cue **role is part of the decoded hypothesis**, so span selection and role choice are fitted and
  served by the same score; the previous staged span-then-role objective is removed;
- uncovered tokens carry **learned singleton background potentials**, so a trailing adjunct competes
  with extending an argument instead of being free;
- argument roles are **sided** relative to the cue (`SEG_SUBJECT_LEFT/RIGHT`,
  `SEG_OBJECT_LEFT/RIGHT`), so "the subject precedes the relation and the object follows it" is
  learnable structure rather than a tie between symmetric token features — the failure mode that made
  the first two attempts decode `X office follows Y` with the arguments swapped;
- every interval potential (including background) is **precomputed once per clause** before the
  bounded decode;
- bounded **ordered categorical** features (`G_POS`, `G_BIGRAM`) give the categorical arm order
  information, so a comparison with the geometric arm is not confounded with order;
- the ordered `2I` code map is fitted by a **bounded coordinate search that alternates with weight
  refitting** and retains an incumbent by the same decoded objective.

The runner (`bin/competitive-reader.rs`, `--mode=observed-text-session`) now authors ordinary readable
forms: the interrogative `What is {P}'s office?`, the possessor assertion `{P}'s office is {T}`, the
topicalized `{T} is {P}'s office` (object before subject), the trailing `{P}'s office is {T}
downtown`, and the redirect `{P} office follows {Q}`. Declared spans are located by exact token-byte
offsets, so a span boundary that is not a real token boundary is a hard error rather than an
approximate extent.

## Measured behaviour

| Panel | Order-aware categorical | H4 hybrid |
| --- | ---: | ---: |
| Development (open, 6 worlds) | **48 / 48** | **48 / 48** |
| Exposed regression (5 worlds, design-informed) | **40 / 40** | **40 / 40** |
| Final draw (3 fresh worlds, after design freeze) | **24 / 24** | **24 / 24** |

Development also reports 96/96 exact joint hypotheses (spans **and** cue role), 96/96 cue extents, and
96/96 cue-role matches; 0/96 at the declared uninformed start. Depth is correct on every exposed and
final row. 2,426 of 2,426 saved checkpoints resume to an identical suffix and final frame.

Controls on the exposed panel (primary arm): registry-membership continuation **38/40**,
maximum-two-read **30/40**, reads disabled **0/40**. Membership keeps the weaker token-vector identity
boundary.

Interventions through the same loaded runtime, all matching independently derived expectations:
a changed terminal source changes the answer on the **same** evidence path; a valid redirect to a
different existing person changes the **path**; removing the required terminal fact is unresolved
after exactly one fewer read; a cycle is exhausted; a request-goal change selects different evidence;
the goal is invariant across every step.

**Cross-BPE identity.** The first query join compares the interrogative subject surface `[' Mar','a']`
(2828, 81) with the document subject surface `['M','ara']` (61, 4075): different BPE sequences, equal
exact lexical key `Mara`. Both chain joins (`Ivo`, `Cedar`) likewise cross real BPE boundaries with
equal keys. Surface extents remain the provenance and emission source; the key never replaces them.

## Measured comparison, stated at its exact scope

On this milestone's support the H4 hybrid and the order-aware categorical arm are **identical on every
panel**. The bounded code search accepted **0** improving moves (5,880 evaluations), so the ordered
geometric features add parameters and accesses without changing any decision here. That is a genuine
tie, not a geometric win and not a geometric loss: no dominance claim is made in either direction, and
the geometric arm is retained.

## Limitations

- Small authored domain (4 persons, 3 offices, 3 projects); the forms are authored, not learned from a
  corpus. The final draw is fresh in composition and style but reuses the same vocabulary and form
  family; new lexical values and new structures are untested.
- The final panel is fresh **after** the design was frozen, but three earlier exposed failures did
  inform the development-form coverage, which is why the second panel is reported as exposed.
- The membership control retains token-vector identity, so its 38/40 has a weaker identity boundary
  than the primary path.
- Whole-target D0-b serving compliance, complete-path energy and broad language understanding remain
  unqualified. Energy is `UNAVAILABLE`.

## Next architectural decision

The observation interface is complete for this milestone. The next bounded step is durable
role/scope/correction state and consumed Q8 computation results through this same session, not another
span fixture. Broad prose, general reasoning and executed Rust remain subsequent. Structural banks,
retained Hopf fiber, signed Spin-H4, paired-H4/icosian E8, S7 and harmonic/scalar fields stay
available for a witnessed interference, orientation or sharing need rather than as a mandatory
dimensional ladder.
