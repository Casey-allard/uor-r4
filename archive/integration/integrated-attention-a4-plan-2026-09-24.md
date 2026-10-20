# A4: couple integer source selection to evidence-conditioned language

Status: prospective development protocol, recorded before A4 compilation or fitting.
This continues the [A3 result](integrated-attention-a3-result-2026-09-24.md)
inside the [integrated programme](principal-attention-plan-2026-09-24.md).
It does not replace that programme or qualify Milestone A.

## Decision and mechanism

A3 improved source admission but selected no correct first-answer source. Its
separate ranker and read gate therefore prevent the evidence-conditioned head
from operating. The next artifact replaces that split decision with one learned
integer score over `{NoRead} ∪ {actually admitted records}`. It also gives answer
tokens and terminal Stop more head-learning exposure in both treatment and control.

The action score combines causal query/state features, the observed candidate
value, unary/pair relative-code factors, and exact value-by-lane-by-relative-code
factors. Every coefficient is signed four-bit. Higher scores win; NoRead wins a
tie, and tied read candidates retain their causal admission order. Selected score
evaluation uses integer additions and packed table reads. It neither inspects
future targets nor receives record identity or provenance as a served feature.
The exact occurrence remains separately owned by the memory store.

Training directly edits the exported integer scores using a bounded structured
margin. It cancels shared coordinates, clips coefficients to `[-8,7]`, and
recomputes the actual rival after each of at most four update rounds. This is a
local perceptron-style correction, not a global convergence or descent guarantee.
The new selector starts at zero; the other quantized parameters are loaded from
the matching A3 parent. Float optimizer state is reconstructed, not resumed.

## Causal labels and coupled learning

- At the first authored answer decision, the annotated source is the positive
  action only if its actual occurrence is admitted. Missing admission skips this
  update; it is not a NoRead label. Identical served candidate observations make
  exact occurrence supervision unidentifiable and are counted/skipped.
- At every later answer token and every 256th natural-text position, frozen
  conditional gold-token loss supplies offline utility. All actions within
  0.01 nat of the best are acceptable, including NoRead when appropriate. Equal
  candidate values receive equal utility. Natural recurrence pointers do not
  become query-address labels. Natural, later-answer and first-decision exposure
  are reported separately.
- Both arms receive eight total actual-answer head updates, eight forced-source
  and NoRead auxiliary updates where applicable, and eight terminal Stop updates.
  The target token remains an offline label, and the same greedy decoder runs
  after export/reload.
- The coarse context-address bank stays fixed at A3. A4 also freezes the prior
  fine-code/legacy-energy and old gate credit, since they no longer govern its
  action choice. Its new selector and shared State/output operators learn. The
  A3 continuation control retains its original fine/energy/gate credit. This is
  an architectural continuation comparison, not an isolated one-factor ablation
  or end-to-end fine-encoder learning result.
- Legacy energy is still calculated for diagnosis in A4; those reads remain in
  access accounting. No efficiency claim may omit that work.

**Post-execution scope correction, September 24:** the paragraph above described
the disabled *direct* fine-code training as a freeze of fine codes. That is too
strong: State updates shared encoder rows used by fine Key/Query formation, and
their causal inputs also change. Only the dedicated context-token coefficients
and legacy energy are retained unchanged. The A4 result records this correction;
the original source-bound reports are preserved. Neither suffix interference nor
representation drift is isolated by these runs.

## Matched executions and decision endpoint

Run C120 and binary-icosahedral (2I) A4 treatments and their own matched A3
continuation controls: four fits total, eight epochs and 1,124,776 new token
updates each. Parents, tokenizer, source snapshot and data are hash-pinned by the
Rust driver. Each child retains its parent's 1,124,776 historical updates and
reports both new and cumulative work. Use the existing fit/development data;
there is no fresh final draw or held-out qualification in this continuation.

The endpoint is actual loaded generation on the existing 28-row panel per arm:
changed-source pairs, read-disabled controls, complete correction answers and
natural prose/code continuations. Check source admission/rank/selection as
diagnostics, together with conditional language loss, generated tokens, Stop,
logical parameter access and exact per-case changes from the parent/control.
Export and reload before the cheap smoke generation, then run the full panel and
independent process replay. Preserve and seal all attempts.

Success means a learned read causes an appropriate uncopied consequence and a
complete answer under source-only edits, with the read-disabled control and prior
cases reported. Retrieval counts or lower loss alone do not satisfy this endpoint.
If selection operates while output remains degenerate, the next mechanism must
address evidence-to-output learning and bounded decoding in this same model.
Do not respond with another retrieval-only benchmark or a larger geometry.

The exposed A3 panel has a material limitation: across its 19 admitted first
positives (17 new and two inherited) in each arm, each positive token differs
from all competing admitted tokens. A token preference can therefore help.
Report parameter-bank changes and score components; this panel cannot establish
geometric advantage, entity disambiguation or general language even if it passes.

## Resources and delivery

The prospective [budget receipt](../evidence/integrated-attention-a4-budget-2026-09-24.json)
charges the complete cycle to the shared ledger: 120 minutes, four fits, at most
4.5 million new training tokens, 900 seconds per fit, one Cargo process, two build
threads, an 8 GiB RSS target and bounded new storage with the existing physical
reserve plus 128 MiB stop margin. All models/research/worktrees are preserved.
Any necessary extension must be recorded before use under standing authorization;
no external paid compute is authorized.

Extend protected PR #1387 with implementation, executed results and the canonical
current-state/roadmap update. The owner's latest instruction explicitly authorizes
the protected merge when this work is complete. Verify actual merged main and
reviewed-tree equality. CI compatibility acknowledgements are not model tests.
