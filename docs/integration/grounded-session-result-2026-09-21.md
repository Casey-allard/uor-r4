# Grounded computation and a supplied dependent session — reviewed result

September 21, 2026. Executes the [grounded dependent-session brief](deepseek-grounded-dependent-session-step-2026-09-21.md) and
the [shared-transition review](shared-transition-review-2026-09-21.md) on merged `85df1739` (PR
#1339, verified equal to the merged head). Isolation worktree `.worktrees/grounded-dependent-session`;
owner checkout untouched. **Principal corrections are incorporated below; the original sealed reports remain unchanged.** See the [principal review](grounded-session-review-2026-09-21.md) and [corrected replay audit](../evidence/grounded-session-corrected-replay-audit-2026-09-21.json). New module `learner/grounded_session.rs` and mode `--mode=grounded-session`.
[Evidence](../evidence/grounded-session-2026-09-21.json).

## Accepted corrections

- The reversal failure is **computational/learning failure in that population**, not reader
  confounding: every development and reversal example selects the correct source. The earlier
  "confounded by the reader" attribution is withdrawn.
- The development probe was value-index parity with every program in both halves, i.e. supervised
  value-disjoint fitting, not sequence-disjoint validation. Not reused as evidence here.
- The whole-program cache was not a competent transition baseline. The competent ordinary comparator
  is the shared recurrent table, and it is now the control.
- The `remaining == 0` stop table is **explicit program completion**, not learned semantic
  continuation.

## Part A - constructive factorization of the observed transition graph

The declared development labels give one permutation of the typed outcomes per observed primitive.
Those permutations are bijective, distinct and closed (order spectrum 1 x 1, 1 x 2, 6 x 4 -> Q8). An
exhaustive search finds an isomorphism into a **witnessed** quaternion subgroup of the served table;
outcome coordinates come from a reference-state orbit and initial coordinates from
`inverse(A[p]) * Z[first outcome]`, checked across every observed primitive. The reference outcome
fixes the common-right frame freedom; subgroup automorphisms remain. This routine currently factors a fully observed regular eight-outcome Q8 action. It is not arbitrary group discovery.

```text
Z[next outcome] = A[observed primitive] * Z[current outcome]        holds by construction
E[selected payload] = inverse(A[p]) * Z[first outcome of payload under p]
```

1,440 per-arm events were saved from the scoring pass. Every one of the **672 declared development
transitions is reproduced exactly**, and the factorization matches the competent shared finite-state
comparator on all three panels. Artifact sizes below use actual serialized bytes; binary versus JSON is not an encoding-normalized comparison:

| Arm | dev /288 | unseen length 4 /128 | unseen reversal /64 | artifact bytes |
| --- | ---: | ---: | ---: | ---: |
| newly fitted shared recurrence | 248 | 76 | 24 | 370 |
| competent finite-state control | 288 | **124** | **64** | 1668 |
| **grounded factorization** | **288** | **124** | **64** | **137** |

The review's diagnosis is confirmed: the earlier learner had not recovered structure already present
in its supervision, and a constructive geometric factorization recovers all of it. This is a
shared-factorization result on supplied intermediate supervision, not language capability. The original 102 was a parameter estimate, not the serialized 137-byte file. Common geometric tables add 15,480 bytes and E/S/selector assets add 454,788/53,555/4,408 bytes before runtime state. The ordinary table can be packed more tightly than its 1668-byte JSON.

The recurrence is a **new fit**: all 288 development items, no probe, two restarts and seed 0xC0F40001. It is not the previous 186/48/10 artifact, whose fitting split/restarts differed. The principal replay exports and actually uses independently loaded objects for all three arms; submitted scoring used originals despite constructing two loaded objects. The corrected instruction boundary derives the declared span from the observed prefix.

## Part B - an owned, resumable session frame

`SessionFrame` carries an immutable owned-source snapshot (sequence, absolute position, copied payload), retained computed state/outcome, response phase, emissions and typed terminal status. Typed read/apply/emit/stop operations and RLSF serialization are retained. **The copied payload remains usable after its origin is evicted.** A separate origin diagnostic checks the exact occurrence against its owning ring; it is not global session identity or a borrowed-live-pointer guarantee.

Principal corrections save model/geometry/tokenizer-bound checkpoint envelopes after first computation and after second computation. Restore continues the remaining second-hop work or emission, respectively. The first source snapshot is captured before the second read. The host still supplies the fixed two-hop protocol: these checks qualify its restoration, not an independently learned controller. Focused tests cover actual same-payload ring wrap, owned-copy continuation, nonidentity state restoration, invalid state/terminal rejection and independent frames. See [executed checks](../evidence/grounded-session-principal-checks-2026-09-21.json).

## Part C - a genuinely dependent read

Evidence is a set of `(role, key, value)` records. The first key is the request key; the second hop's
keys are the **typed outcome labels**, so the first computed result forms the second query. The query
is issued as `(role, key)` with no declared previous context, so candidate admission is driven by the
key alone.

| Measurement | Result |
| --- | ---: |
| first hop correct | 4 / 4 |
| second occurrence selected correctly | 4 / 4 |
| complete answer correct | 4 / 4 |
| post-first-compute resume / post-second-compute emission | 4 / 4 chains; 8 saved checkpoints |

Causal controls:

| Control | Observed |
| --- | --- |
| changed first source | principal control edits only payload position 2, freezes the second-hop bank and independently checks both expected answers; original control also edited positions 14 and 26 |
| added irrelevant record | first outcome, second selection and answer all preserved |
| **required record removed** | principal controls remove the first and second required records separately and require terminal `NoRead`, no answer |
| reader disabled | `read = false`, terminal `NoRead` |
| unknown primitive / payload | typed `UnknownPrimitive` / `UnknownValue` |

The first primitive 489 is the identity action, so its disabled-computation control may correctly be unchanged; it does not establish necessity of nonidentity first computation. The four cases share one authored 36-token bank and a supplied two-primitive programme. Second-source correctness in the revised instrument checks occurrence identity as well as payload. The original 4/4 positions independently recount correctly even though the original aggregate checked value only. These are exposed component examples, not a fresh held-out dependency population.

The final answer is a typed outcome label - a disjoint alphabet from every record payload - so it is
**absent from all source payloads**: the answer is derived, not copied.

## Disagreements and deviations

- I did **not** adopt the brief's illustrative `U[op] = h^(hidden index)` or any fixture-rule
  seeding. The isomorphism is found from declared labels and the gauge is fixed by a reference
  outcome, which is representation normalisation rather than a semantic label.
- The brief's non-invertible control operators are kept separate: only the ordered product is a group
  operation; read, commit, emit, exhaustion and the unknown/absent outcomes are typed transitions.
- A useful deviation: rather than treat the finite table as a rival, it is retained as the
  **equal-supervision control**, and the factorization is evaluated against it. Where the table wins I
  say so; here the factorization matches it on the same supervision. The 137-byte binary versus 1668-byte JSON sizes are serialization-specific and do not establish whole-engine compression.

## Not established

Supplied record layout and a supplied primitive program; the instruction span is declared structure,
not a learned parser. Completion is explicit program exhaustion, not learned continuation. The second
hop is exact-match key retrieval over typed outcome labels, not semantic query formation. No prose,
no executed Rust task, no whole-path energy measurement. Energy `UNAVAILABLE`; whole-path D0-b not
claimed. Retained original roots `grounded-session-{1,2,3,4}` (0 unlisted each), with `-4` primary; corrected exposed root `grounded-session-principal-1`. Original source/release executable and negative candidates preserved. Next: [learned relational access and content-dependent control](deepseek-learned-relation-control-step-2026-09-21.md).
