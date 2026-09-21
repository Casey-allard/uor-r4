# DeepSeek research brief: shared learned transitions and complete result continuation

September 21, 2026. Successor to PR #1338 and its [principal review](derived-state-decoder-review-2026-09-21.md). Read this brief and the review completely, including the latest executed replay/check receipt. This assignment owns a complete constructive lifecycle, not only implementation helpers or another isolated cell-score experiment.

## Objective and research autonomy

Build a **persistent learned read → compute → emit/stop trajectory**, with a shared action applied to actual selected content and reused across observed instruction sequences. Retain the useful lexical decoder; connect it to owned computed-result state and learned continuation. When the one-read primitive is useful, carry it into a dependent read whose query and final answer change with the first selected evidence. This advances the native geometric model toward conversation/memory and coding/reasoning.

Codex remains principal mathematician/ML/systems architect and returned-run reviewer. You are a capable research contributor: choose and revise learning methods, finite representations, transition/lexical objectives, diagnostics and appropriate follow-ons. Explain disagreements using source, mathematics or measured behavior. Necessary local time/storage extensions are already authorized when projected and recorded before use; no arbitrary short time window or fixed retry quota should distort the mechanism. Preserve physical limits, unique research and artifact evidence; no paid/external compute is authorized.

Do not require a new issue for this work: #973 owns integration, with #820 programme, #1139 binding, #962 memory, #963 cost and #964 scoped invariants. Do not close broad scopes after a component result.

## Recover the correct context

Refresh origin/main and verify PR #1338's actual merged contents, not only its original branch. If it is still queued, locate the reviewed head and avoid duplicate delivery. Preserve the owner's original checkout and use an isolated full worktree.

Read README → canonical plan → current-state → model-direction → PROJECT_MAP, then AGENTS, execution policy and DECISIONS. Read the principal review, independent audit and executed checks/replay. Read the original derived-state-decoder result with its corrections, the previous geometric-computation review and the cross-mechanism/Hopf structural-memory synthesis. Search the UOR knowledge store for these source-bound records, and verify current facts against source/artifacts. Historical records and expired restrictions are context, not overriding instructions. Reuse the architecture audit and follow concrete dependencies into older source rather than repeating a broad census.

Inspect `native_geometric/learner/result_decoder.rs`, the shared `dsd_step` and new runner lifecycle. Inspect the actual donors you use: `addressed_attention/objects.rs`, `operation_transition.rs`, `shared_operator_refinement.rs`, `dependent_language/scheduling.rs`, and optionally `hamming_policy/policy.rs` or `shared_core.rs`. Their owned-result, typed-action and continuation interfaces are useful; their authored byte grammar and old artifacts are not a learned BPE solution.

Rust is required for model preparation, learning, artifacts and inference. Offline floating point/matmul/gradients are allowed. Serving targets geometric routing, finite state/actions and bounded integer/table lookup; D0-b permits low-bit additive maps within its declared numerical contract. Keep the separate frozen runtime contract intact. No hidden transformer/provider, authored answer interpreter or Python model dependency.

## What to retain, and what was corrected

The original same-run value-only decoder matches the selected-value dictionary at 161/180 development, 108/120 tune, 104/120 final with eight states. The previous residual 33/120 used different seeds, so do not call 33→104 a paired improvement. The original winning value-only model was neither exported nor served through the shared predictor; direct model-hit counts also omitted actual read presence. The principal source repairs and corrected exposed replay address the actual lifecycle. Read their executed outcomes rather than copying historical counts into the new artifact.

Original composition H4 scores 103/192 development and 8/64 held-out; C120 132/192 and 20/64. The development probe operations were absent from the initialized operation vocabulary and fell back to identity, while their labels guided every coordinate choice. This is supervised outer development fitting through a defective domain, not independent generalization evidence. The corrected model represents all observable supervised-development domains; decoder calibration and supervised outer map objectives remain explicitly separate. Final data must never guide either.

The old operation intervention changed source and query roles together; the identity control checked target class 0 instead of the actual group identity; the absence parity boolean was not a real comparison with the local predictor. The new source makes these distinctions testable. New artifact semantics are versioned; old saved artifacts retain their legacy meaning. Actual NoRead, unknown operation/value and missing decoder state have different causes even when all select local fallback. A valid computed identity must remain emittable.

The claim that missing cells are independent free parameters is withdrawn. On source-correct fixture cells, absorb the fixed per-operation binding relation into `A[o]=T_bind[r(o)]*U[o]`; wrong-source rows retain a separate relation input. For the resulting latent products `P[o,v]=A[o]*B[v]`, three rectangle corners constrain the fourth by `P[o,v0]*inverse(P[o0,v0])*P[o0,v]`. The observed graph has shared anchors and is connected. The unknown/many-to-one decoder means labels do not necessarily identify those latent products, but the defective optimizer does not establish nonidentifiability. Do not conduct a broad proof or solver campaign unless a concrete discriminator changes the next implementation choice.

## Start from the corrected result, not the original diagnosis

The principal replay `derived-state-decoder-principal-1` completed with all four independently loaded RLDSv2 artifacts used downstream and 1,352 full prediction parity comparisons. The value-only winner retains 161/108/104 emitted answers and three correct first-token association rollouts. It is now a real exported/served component. Full-factor association remains 166/80/91.

Corrected composition is H4 92/192 development and 3/64 held-out, C120 106/192 and 4/64. Every held-out H4 position reads, but 60/64 outcomes are MissingDecoderState; C120 misses decoder support at 58/64. H4 has 45 distinct development result states, 20 held-out states and eight overlapping, with 16 grounded decoder states. Among 58 correct-source H4 positions, 56 have no grounded decoder output and only two emit correctly. Treat state/outcome consistency and sharing as a learning problem. You may revise capacity if a measured need warrants it, but increasing arbitrary state memorization is not demonstrated compositional transfer. Do not rerun another unchanged optimizer sweep or blame the entire group architecture.

The fixed-selected-operand operation pair is correct. The actual operation-only prefix intervention loses source selection, so the end-to-end pair fails. Preserve evidence identity independently from requested action, using an actual selected source rather than an oracle. Association continuation loses its result after the first output; composition rollouts miss their first answers. The NoRead/read-disabled/update-disabled controls now really compare full local logits and pass. Use these concrete observations to design shared transition/outcome learning and persistent source/result/response state. Preserve the corrected replay as exposed regression; fresh qualification follows design selection.

## Chosen architectural milestone

Separate exact selected evidence, geometric result, instruction position and response phase. A suitable starting state machine is:

- Read obtains an exact owned payload/reference and initializes a learned value state.
- Compute applies a shared action inferred from observed input or learned control state.
- Emit lexicalizes the retained result with the learned decoder.
- Stop terminates when the learned policy chooses it; a subsequent Read may instead use a result-conditioned query.

An illustrative core is `s0=E[selected payload]`, `s[j+1]=A[observed primitive j]*s[j]`, `token=D(s, phase)`. You may choose another representation with a stronger causal justification. The important property is reuse of the same primitive across contexts and sequences, rather than an independent entry for each compound request. The actual predictor must retain its state between emitted tokens. It must not reinterpret the most recent generated answer tokens as a new fixed-format query at every step.

The current query-role token serves both source selection and operation decoding. Separate the persistent source address/owned operand from the observed operation instruction; measure any read change separately from computation. The current decoder forces its chosen token above the local argmax, so learn appropriate dispatch/continuation and retain false-read/unknown controls before claiming natural-text integration.

Learn the action/value/result grounding from development data. A useful auxiliary objective can tie observed transitions: `E(next outcome)≈A[action]*E(current outcome)`, with learned lexical grounding `D(E(outcome))≈token`. Declare what intermediate supervision is provided, compare against the relevant weaker supervision, and measure actual transfer. Do not fill model codes from the fixture's true latent states or set `U[operation]=h^(hidden operation index)` and call it learned. If a compound operation is supplied as an observed primitive sequence, learn its token/action grounding and consume that sequence at serving. An arbitrary atomic operation name has no assumed numeric meaning without training evidence.

Geometric transport may be reversible; the whole engine need not be. Replacement, copying/commit, emission, branching, stopping and missing-evidence outcomes can be partial or noninvertible typed operators. Preserve exact occurrence/version/lease ownership separately from compact geometry. Retain orientation and meaningful frame/fiber information. Avoid forcing every language operation into a single group product.

## Execute the constructive research lifecycle

First use the principal corrected replay as regression context. Exercise the actual loaded winning decoder and strict operand/fallback interface. If the replay exposes a new real defect, correct it and record the scope; do not repeat an unchanged long fit merely to recreate known association counts. Then implement the shared-transition/continuation milestone.

Before final outcome selection, describe the task, observable inputs, auxiliary labels, primitive coverage, novel combinations/trajectories, comparators and useful-effect/preservation criteria. Use development openly and adapt when it teaches you something. A set used to choose every map is supervised training, whatever its name. Repeatedly viewed final data are exposed regression. Keep one genuine independent final evaluation after selecting the design, with enough independent problems rather than counting repeated renderings as unrelated observations.

Use a task where primitive meanings and output lexicalizations are grounded in development, but some ordered action combinations, lengths or complete trajectories are new. Demonstrate that relevant older content is necessary with identical-query/different-evidence pairs. Hold the evidence fixed when changing the requested operation. If order is the claimed property, include a deliberate order reversal and a noncommuting witness; an additive cyclic task cannot establish unique H4 advantage. An authored finite circuit is still only a circuit result, but it can validate a reusable operator efficiently.

A useful comparison set includes the retained local/NoRead path, value-only lexical decoder, the current flat factorized model, a competent finite transition/dictionary comparator with explicit unseen-input fallback, and an appropriate matched additive or ordinary-state alternative. Match the information, served output interface, model bytes and fitting opportunity relevant to the claim. Do not require H4 to win a cyclic task as an entry gate. A practical near miss may deserve retention and a prospectively adjusted successor; preserve the original decision and explain the tradeoff.

Use actual loaded artifacts in teacher forcing, generation, interventions and timing. Export all claimed winners, including factor-ablation arms. Save which loaded object is passed to the predictor. Preserve enough actual per-step events to reconstruct conclusions: exact prefix/observation, read occurrence/version, selected payload, primitive/control action, pre/post state and validity, emitted token/stop, expected answer and relevant scores/loss. Derive aggregates from these events, rather than hand-maintained boolean labels.

Test complete behavior:

1. Relevant payload changes at the same query must change the answer appropriately; irrelevant distractors should preserve it.
2. An observed operation or order change at fixed evidence must have the expected effect. If it changes selection, record that separately from a controlled computation-only comparison using the same selected operand.
3. Missing/evicted evidence, unknown operands and ungrounded decoder states must be distinguished from a legitimate identity result. Compare disabled/NoRead output with the actual local path, not a boolean implication.
4. A retained result must survive intervening emission steps, and the response must stop appropriately. Use variable complete responses/trajectories where needed to avoid an always-one-token shortcut.
5. For a dependent read, intervene on the first source and verify the second query, selected occurrence and final output change together. A supplied gold hop count or oracle continuation is not a learned scheduler.

Once shared computation and complete response control work, use them on one language-facing memory/relation task where the first returned entity becomes the next query. Include differently rendered held-out phrasing or source separation appropriate to the mechanism. Preserve prior absence/conflict and exact-version behaviors. Broader prose, conversation and executed Rust remain the next programme responsibilities; the tiny composition fixture should not become the permanent research objective.

## Geometry and wider research context

Read [Sequential Group Composition v2, May 2026 / ICML 2026](https://arxiv.org/html/2602.03655v2): recurrence can reuse associative binary actions, but its constructive existence results do not prove our learning dynamics. [Intermediate-state supervision for robust generalization](https://proceedings.mlr.press/v162/dan22a.html) motivates a falsifiable auxiliary transition objective; [Compositional Interfaces](https://proceedings.mlr.press/v274/luketina25a.html) motivates separating observation, shared control and output. [Structural Identification, August 2026](https://arxiv.org/abs/2608.26465) distinguishes data admissibility from learned performance and trains no predictive model. Do not import its supplied syntactic structures as learned serving ability. Search newer primary research if it changes an actual mechanism decision.

The owner's broader ideas remain in the toolbox with specific roles: relative H4/Spin and retained Hopf fiber for transport/context; scalar compatibility for selected access; structural role/scope banks for lifetime; fixed zeta/prime/ordered-n-let and UOR identities in their declared roles; finite spectral tables for structured sharing when useful; paired-H4/E8/S7 or coefficient fields for a demonstrated extra distinction/capacity. A single point is not an arbitrary harmonic field, and physical spin/supersymmetry metaphors are not runtime mechanisms. Reuse what helps the measured problem without requiring all geometry in every token.

## Resources, verification and delivery

Refresh the absolute shared ledger `/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json`, current free space and active worktrees. The principal review records its own debit; do not repeat a historical charge or update a relative worktree copy. Project preparation/build/fitting/controls/generation/retries/checkpoints, workers, peak RAM and new/retained storage. Necessary local extensions are already authorized: record reason, increment and updated limit before use, atomically verify the actual JSON, and retain all prior charges. Preserve the 36766079385-byte physical reserve plus 128MiB stop margin unless an explicit later owner decision changes it. Do not delete unique research, artifacts, source freezes, personal material or active caches; no paid/external compute.

Compile and exercise the changed Rust path. Use focused tests for actual domain coverage, typed validity, action order, continuation, serialization, source ownership and independent recount risks. Run real loaded generation. Debug execution can validate function but is not optimized serving performance. If claiming cost, measure the complete useful path and memory traffic; energy remains UNAVAILABLE without measurement. No broad new proof/ledger/testing framework is needed.

Deliver a protected PR with named staged paths, actual checks and exact evidence scopes. Verify merge and source/tree equality; preserve the original checkout. Update current-state, canonical plan, relevant README/front-door claims, evidence and owning issues. Store source-bound knowledge and verify retrieval. Return your chosen mechanism, actual learned/exported/generated outcomes, baselines/interventions, limitations, resource/disk receipts, disagreements and one coherent successor. Do not stop after compiling helpers; do not let a weak toy score become a blanket architectural diagnosis.
