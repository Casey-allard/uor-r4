# Principal review: grounded transitions and an owned dependent session

September 21, 2026. Review of PR #1339 submitted head `832c1c33e25a9534d8fb4c73fbba5d2247ecd2c1`, based on PR #1338 `86623138`. Read with the [saved-data audit](../evidence/shared-transition-principal-review-2026-09-21.json), [executed checks](../evidence/shared-transition-principal-checks-2026-09-21.json), and [complete next brief](deepseek-grounded-dependent-session-step-2026-09-21.md).

## Principal decision

Retain the learned shared left-action recurrence and its independently loaded artifacts. It is a useful finite-program component. Its apparent advantage over transition lookup and its learned-stopping claim are not established. Advance a **grounded, resumable session that owns selected evidence and uses a computed result for a later read**. Repair transition grounding within that milestone using the observed development transition graph, rather than spending another campaign on unchanged random restarts or requiring a new geometry dimension.

The objective remains the native UOR-R4 Geometric Language Model for conversation/memory and coding/reasoning on consumer hardware. The new result is a step toward reusable computation, not general language. Codex retains principal architecture and evidence interpretation. DeepSeek owns substantial implementation and diagnostic choices, complete lifecycle delivery and prospectively accounted necessary local extensions.

## What the original saved result actually supports

H4's saved events independently reconstruct **186/288** development responses, **48/128** length-four responses and **10/64** reversal responses. All 288 development and 64 reversal examples select the correct source. Length four selects the correct source on 124/128; four reads select an unknown query-role token instead. Thus the reversal failure is computational/learning failure in this population, not explained by the claimed reader confounding. The shared interface can couple instruction and selection, but a plausible mechanism is not an observed cause when every source is correct.

The length-four population is 32 programs times four values. Enumeration with stride 128 in base eight fixes the first two primitive tokens for every program. The four showcased generations use four values of one program, with three complete successes. Preserve these useful results with that limited breadth; do not call them four independent novel programs or broad length extrapolation. Repeatedly inspected populations are exposed regression.

The fit/probe split is by flat item-index parity, after nesting all four values inside each program. Consequently the calibration half uses value indices 0/2 and the outer half uses 1/3; **every program occurs in both halves**. Both halves guide optimization and the final decoder is refitted on all development data. This is supervised value-disjoint fitting with shared programs, not sequence-disjoint validation. It does not itself expose final labels. Coordinate selection and restart selection use different lexicographic objectives; report their actual order.

The H4 recurrence keeps a local result variable across instructions within one `serve` call and does not feed emitted labels back into its geometric updates. Exported models are independently loaded and actually used, a genuine improvement over earlier lifecycle defects. There is no externally resumable source/result frame, durable occurrence lease or dependent read yet. The initial values and primitive codes are learned; the data generator's true Q8 codes are not directly installed as model parameters.

## A fair transition comparator changes the interpretation

The submitted `transition_dictionary` caches `(initial payload, entire program) -> whole answer`. For an unseen program its fallback emits one token, making complete length-four success impossible by construction. It is a whole-program memorization baseline, not an ordinary shared-transition learner.

The same development targets already supply a much stronger ordinary representation:

- 32 distinct `(initial payload, first primitive) -> first outcome` transitions;
- 64 distinct `(previous outcome, next primitive) -> next outcome` transitions;
- one consistent target for every such key.

Every held-out length-four and reversal step has this development support. By induction, a finite recurrent table fitted to those observations determines every computation-only held-out trajectory. With the saved actual reader and strict unknown fallback, its supported outcome is 124/128 length-four and 64/64 reversal responses. This was a saved-data coverage/consistency deduction; the corrected Rust replay now independently executes the predicted 124/128 length-four and 64/64 reversal result, with 288/288 development. It uses no hidden group indices or held-out target fitting.

The appropriate conclusion is **useful learned recurrence below a fully supported ordinary finite-state comparator**. A table win does not invalidate geometric transport. It reveals that the current learner fails to recover structure already present in its supervision. Preserve the table as an equal-supervision control and possible offline source of transition constraints; evaluate any compact geometric factorization on correctness, generalization, parameter sharing and actual cost. Do not require a geometric model to beat a deliberately incapable cache.

## Corrected execution and its remaining limits

The corrected exposed replay takes 49.754 seconds in debug configuration and retains every original H4/C120 complete-response count. The independently reloaded finite table reaches 288/288 development, 124/128 length four and 64/64 reversal. Its 1,668-byte JSON artifact has 32 initial and 64 recurrent transitions; this is equal intermediate supervision, not equal serialized size or total cost. All 2,880 arm/item records and 8,316 step/terminal events come from the actual metric passes.

The actual changed-source and order interventions now produce complete expected answers with the same selected occurrence. Actual source removal and ReadDisabled separately produce NoRead. These controls do not establish ring eviction, general absence behavior or local-output parity. Permuting loaded decoder labels changes the output while leaving the geometric states unchanged, testing the claimed lack of output feedback. The noncommuting witness has one correct complete order and one incorrect reversed final answer: latent order sensitivity is real, but that witness does not establish correct learned ordered computation in both directions. Four showcased generations still use one program, three complete correctly. See the [independent corrected audit](../evidence/shared-transition-corrected-replay-audit-2026-09-21.json).

## Input exhaustion is not learned reasoning termination

The stop targets are constructed as `remaining == 0`, the input supplies the complete primitive list, and the old runner forces `Stop` at list exhaustion even if the learned table would continue. This is explicit-program completion. It is useful and legitimate; it does not establish learned answer length, a learned hop count or a data-dependent continuation policy.

The principal repair distinguishes explicit `Exhausted` from a model stop decision while preserving legacy artifact behavior. Even an explicit learned table for observed input exhaustion remains a narrow task. A later controller must decide whether to compute, retrieve, emit or stop from causal result/query/memory observations; it cannot be credited with learning those choices when the harness supplies every step.

## Causal and evidence boundaries repaired

The submitted payload intervention runs the model on the fixture's intended operand, bypassing actual selection. The absence test disables the reader without removing the source. The state-independence test repeats an identical function call; source inspection establishes lack of emitted-token feedback, but that is not an executed perturbation. The order witness finds different learned final states without requiring both expected answers. Additive final-state invariance does not imply identical intermediate output sequences.

The original helper receives both the prefix and a separate primitive list from a supervised item. Even when they agree in ordinary evaluation, an intervention can make them disagree. The repaired boundary consumes or validates observed instructions against the actual prefix, avoids gold-payload fallback, records real selection and response events for every arm, and separates end-to-end interventions from fixed-selected-operand computation probes. A typed instruction span is declared supplied structure, not a learned parser.

Actual source references and output-state traces must follow the model used for the metric. The old rows independently support H4's aggregate but do not reconstruct the other arms and omit explicit stop events. The correction saves actual events from scoring, including ending status, rather than rerunning one arm for a separate report. Complete evaluation, generation, intervention and reload claims need the same causal boundary and artifact semantics.

## Constructive learning: identify useful state, then factor the transition graph

The current fitter uses a learned input-value map and a separately refitted output decoder. It can assign different states to the same typed computational outcome in ways that predict training labels but fail under subsequent actions. For a typed outcome alphabet that is sufficient to determine the next result, a candidate repair is a learned injective embedding `Z[y]` and shared learned actions satisfying

`Z[next_y] = A[observed_primitive] * Z[current_y]`.

The initial step connects `E[selected payload]` to the same outcome states. Learn these assignments from development transitions; do not install the fixture's hidden group/state codes, subgroup indices or operation powers. A common right change of frame permits fixing one reference `Z[y0]` to the group identity without supplying semantic information. Closed paths, inverse consistency and repeated transitions can constrain a finite search much more directly than arbitrary restarts against a changing decoder. An exact or approximate Rust constraint fit is an implementation choice, not a required broad proof campaign.

The [development-only action audit](../evidence/shared-transition-observed-action-audit-2026-09-21.json) now verifies a constructive opportunity: all eight observed primitive permutations are distinct, bijective and closed under composition; their action on the eight typed outcomes is regular. Their order spectrum is one identity, one order-two element and six order-four elements, identifying Q8. The initial embeddings derived from `inverse(action) * first_outcome_state` agree across every observed primitive for each payload. This uses only saved development outcomes, without reading generator group codes, learned parameters or held-out labels. It is a mathematical identification result, not a trained new model. Construct an isomorphism from the observed permutation group into a verified quaternion subgroup of the served table, then bind the learned initial and output maps. This is preferable here to unconstrained random restarts.

This is conditional on the state semantics. An arbitrary finite transition system need not embed into the free left action of a subgroup of H4. Noninvertible updates, nonfree actions or context-dependent outcomes may require a quotient representation, retained context or a separate typed nonlinear operator. Equal emitted BPE tokens do not generally imply equal language state. Apply outcome tying to a declared sufficient typed result; retain source identity, scope, phase and meaningful context separately.

The ordinary transition table itself is not the final language architecture. It supplies a competent baseline and observed consistency constraints. When geometry compresses/shares those transitions, measure the advantage; when it does not, keep the useful component and report the limit. Do not silently adopt a teacher at serving, a transformer backbone or expert gates.

## The owner's spin and Hopf ideas connect directly here

For the quaternion subgroup, write `q(s,x,z)=(-1)^s i^x j^z` with binary coordinates. Multiplication yields the next coordinates

`(s xor t xor (z&u) xor (x&u) xor (z&v), x xor u, z xor v)`.

The central sign records part of the order dependence. Quotienting by `{+1,-1}` produces the commuting Klein four-group and loses that distinction. This gives a concrete role for retained spinor sign/phase without a physical quantum claim. The usual Hopf observation also identifies `q` and `-q`; preserve the relevant fiber/sign if an output distinguishes them. This is a mathematical connection, not permission to seed the learner from hidden fixture codes or a measured advantage of a new implementation.

Relative H4/Spin, retained Hopf fiber, role/scope memory and finite spectral features remain primary tools with specific jobs. A scalar policy can choose useful actions; a harmonic coefficient bank can summarize protected channels if its capacity and update costs help. Paired-H4/E8/S7 is available when a demonstrated extra distinction warrants it. None is needed merely to repair a wrong comparator or supplied stop schedule.

## Research and roadmap

[Sequential Group Composition, ICML 2026](https://arxiv.org/html/2602.03655v2) motivates reusing associative actions; its constructive and restricted learning results do not guarantee our discrete fitter. [On the Induction Bias in Sequence Models, ICML 2026](https://arxiv.org/html/2602.18333v2) reports useful parameter sharing across lengths for recurrent models on state-tracking tasks; it is not evidence about this artifact or general language superiority.

[Understanding and Improving Length Generalization in Recurrent Models](https://arxiv.org/html/2507.02782v2) studies unexplored recurrent-state distributions and training interventions such as state passing in Mamba/GLA/RWKV-family settings. Here it motivates measuring and training on the actual reachable result states, not importing Gaussian states or its dense kernels into exact H4. [Intermediate-state supervision](https://proceedings.mlr.press/v162/dan22a.html) supports testing observed transition/outcome constraints as an auxiliary; it does not justify hidden target-derived state at serving.

The next milestone combines grounded shared computation, a resumable exact-source/result frame and the first dependent language-facing relation/memory answer: changing the first source must change a later read and the complete derived output. Broader continuation and source transfer, followed by source-separated prose and executed Rust on the same native path, remain subsequent programme responsibilities. Structural lifetime and representation changes follow observed failures. The programme remains open and pre-alpha; complete-path energy and broad language capability are unqualified.

## Delivery

The checks receipt owns the actual corrected replay, source/artifact identities, resource debit and free-space observation. The principal audit preserves the original reports, negative candidates and explicit source/control limits. Owning issues remain #973, #820, #1139, #962, #963 and #964; no new issue or closure is warranted by this component result.
