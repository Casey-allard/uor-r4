# One Bound Geometric Decision Graph

Status: selected mathematical/learning design, NOT an implemented joint trainer.
The owner holds training. A design can specify coherent credit; only executed
qualification can show that the implementation has it, and only model results
can show useful learning. No current PASS or mechanism advantage is inferred.

## 1. Shared State, Distinct Types

For an episode x, let theta = (E,C,K,Q,R,O,P,G,U) contain the shared Context
encoder, compiler heads, source-key/query-route heads, bounded read coefficients,
observation/Copy factors, Prefix/bridge factors, Generate prototypes/factors and
continuation field. They are one revision and optimizer inventory, not separately
fitted heads whose encoders can drift. Fixed tables are outside theta.

| Object | Authority and role | Learns? |
|---|---|---|
| Original bytes/BPE IDs | Bound tokenizer, original half-open span, actual reply IDs | No |
| Exact record handle | Store lineage/scope/entity/relation/record/commit/offset/payload digest | No |
| Ordered H4 state | Existing120-state native recurrence; retain all lanes and signed order | E and relevant typed factors |
| Compiler act/relation/span/view | Existing additive native heads, extended with typed view head | C, through E |
| E8 root | Existing240-root codebook, separate ID/basis/scale | K/Q assign content to roots/families |
| Cl(0,6) action/frame | Existing64-action SpiralCore group, labels/order/inverse | Fixed change of coordinates |
| Exact store write/read | Existing StackStore/scoped-memory transaction | Its compiler policies, not payload bytes |
| Copy/Generate/EOS | Existing finite factors/common native pool | R/O/P/G/U, through shared E |

Scope and entity are trusted explicit caller inputs in this first design.
Learning entity extraction, unrestricted prose, coding and frontier capability
are not claimed. Hashes identify/check content; no hash/IP distance is semantic.
Deduplicated immutable bytes retain distinct occurrences, versions and aliases.

Reuse points (paths are relative to repository root):

- `crates/uor-r4-training/src/geometric_turn_compiler.rs`: original word spans,
  ordered/local H4 feature policies, packed act/relation/span heads and decoder.
  Its current frozen-feature `fit_examples` is NOT the joint trainer.
- `crates/uor-r4-training/src/native_grounded_session.rs` and `stack_store.rs`:
  atomic source/store/emission, trusted reload, version views, original values.
- `crates/uor-r4-training/src/geometric_bank_generate.rs`,
  `geometric_source_realizer.rs`, `geometric_generate_learning.rs` and
  `geometric_continuation_learning.rs`: live bank, common aliases, native Q24
  anchor, prototypes, finite120 utilities and source-bound export.
- `crates/uor-r4-core/src/spiralcore_operator.rs` and training `e8_memory.rs`:
  labelled64-action group,240 roots,15 orbits of16, exact transport and existing
  structural decoder. Episodic content is NOT an E8 learned value-table slot.
- Claude `geometric_stack.rs::starling_select/starling_read`: bounded candidate
  buffers, shared kept set, separate role-specific query/key maps, competitive
  content ranking. Its dense content helper/KL is an offline optional comparator,
  not a served teacher or a prerequisite for this native design.
- Existing historical H4-to-canonical witness/Hamming metric and paired-icosian
  basis/glue/inverse witness. No second geometry library or unchecked ID cast.
- DeepSeek's delivered #2228 `stack_address.rs` is an adjacent fixed token/age
  address primitive, not this learned record/profile route. Its leading-bit
  agreement and scan-based selector are not adopted as a semantic metric or
  assumed constant-time candidate lookup. Reuse/coordinated index hooks can be
  assessed after its caller is delivered; do not duplicate its peer-owned work.

## 2. Hard Forward From Raw Turns

1. Decode raw source with the bound ByteBPE. Compute compiler feature traces and
   Context traces using the SAME E revision. Independent word encodings retain
   their reset/tokenization semantics; turn/prefix traces can share a DAG only
   where their actual input prefix is identical. No cached frozen features.
2. Predict act and relation. On writes choose an original contiguous word span;
   on queries choose one of the four existing HistoryViews. Unknown relation,
   no positive span or unsupported act is Unresolved, not a fabricated fact.
3. Clone/stage the exact store. Assert/correct writes exact original value IDs,
   plus original assertion/byte-span witness and version links. Query changes
   no bytes. Never substitute gold compiler labels or answer records here.
4. Look up the PREDICTED `(scope,entity,relation,view)` address. Also use the
   geometric index below to collect bounded additional candidate records.
   The direct result is a candidate, not an authored response or mandatory
   winning donor. A wrong compiler key can miss or return a wrong value.
5. Carry status/view/conflict as typed numerical input, not synthetic Source
   text. Reuse StoreSignal's16 status/view codes and a separate conflict bit;
   proposed learned additive/lookup signal transition is part of E/O. Excluded
   or unfollowed routes are not proof of Absent or Evicted. Eviction requires
   the store's physical witness, not geometric-buffer truncation.
6. Admit original assertion Context and exact value Source for the union of
   eligible candidates, preserving all physical offsets/aliases of those
   records. Append bound recent conversation Context, raw query and actual
   emitted prefix. Compute native Copy/Cue/Prefix, selected donor/bridge,
   Generate and shared continuation energies in their existing order.
7. One common final clip and token mass aggregation over every admitted physical
   Copy alias and every Generate token. Emit by the existing native token rule,
   including normal EOS. No answer template, hidden provider or forced target.
8. Commit staged writes, index changes and actual reply IDs together ONLY after
   successful bounded generation. Failed proposals roll back the staged store.

The new route/signal/view contracts need a new explicit artifact policy/version.
#2225's all-address packet and current-query-only native compiler remain valid
compatibility controls. Do not silently claim they already implement this path.
Training labels are allowed in loss only; native forward never reads the oracle.

### Compiler Support And Span Law

Use current four act classes and ordered relation labels plus NONE. Add a
four-class view head, used only on queries; its Current default is compatibility
mode, not learned historical understanding. A write's span alternatives are all
original contiguous word intervals of length <= saved max_value_words plus
no-span. If per-word inside/outside logits are b_j, define

    s([i,k)) = sum_{j=i}^{k-1}(b_j[inside] - b_j[outside]); s(no-span)=0.

Greedy positive maximum and earliest interval tie reproduce `selected_span`.
Offline sampling over these same alternatives is a new explicit learning mode;
it is not independent inside-bit sampling or a differentiable text slice.
Mask only structural validity/length, never the labelled value/relation.
At64 words and limit8 there are <=512 intervals, no unbounded span search.

## 3. Bounded Routes With A Collective Profile

Use E8's EXISTING standard Euclidean root basis for the route, H4's ordered
native basis for content operators. Assign each record a root through a
role-specific key head K on shared H4 features. Query head Q remains distinct:
Claude's source explains why equating query/key maps under its j-shift can make
the two orthogonal. Shared representation does not mean identical role maps.

K scores240 fixed root alternatives by bounded additive tables over H4 lanes;
greedy export is a compact typed lookup/scoring head, not a per-token dense MLP
or a dimension-cast H4->E8 isometry. Learned semantic root assignment is not
required to be a group homomorphism and cannot inherit such a guarantee.

Maintain one depth-M geometric ring at each root for eligible record handles.
M is frozen in the future manifest. Physical store retention/capacity is
separate: a ring removal is candidate truncation, NOT payload eviction.
Maintain, for each orbit o, the resident count n_o and signed-coordinate sum

    mu_o = sum_{resident r in orbit o} v(r),

where v is the existing root's exact scaled integer coordinate key. Writes,
version changes and ring replacement update these summaries by add/subtract.
Copies of the same occurrence are not independently inserted as new records.
At most16M handles contribute to an orbit; counts and accumulators have checked
bounds. The index is scoped/entity-separated; it cannot leak another scope.

Q scores15 orbit summaries using shared H4 query state and learned <=4-bit
additive coefficients on `(n_o,mu_o)`; shifts/additions suffice for coefficient
products at serving. Take B orbit families, a fixed manifest bound, collect
their <=16MB handles and union the predicted exact-address candidate. Rank only
these using shared typed H4 query/record features and content score factors;
keep K records shared across emitting heads, preserving stable ordinal ties.
All their physical Source offsets enter the common emitter pool.

These families select exact MEMORY RECORDS, not expert neural networks or
separately learned value-table responses. There is no MoE/expert serving gate.

This chooses the E8 orbit-index route for the first implementation, rather than
combining multiple unrelated routers. Claude's600-cell neighbour reader remains
an independently owned alternative/comparator; its bounded/shared-candidate and
competitive-credit design is reused without pretending the graph is identical.
The existing Hamming distance is available for typed H4 tie/ranking features,
not as a distance on E8 IDs, digests or relation numbers.

The collective `(n_o,mu_o)` profile is a finite-geometric ANALOGUE inspired by
the supplied elliptic-curve murmurations paper, not elliptic-curve arithmetic.
The profile is lossy (opposite roots can cancel). Exact handles and individual
candidate features therefore remain necessary. Ring/family truncation can
still miss a fact; the residual does not magically recover an unadmitted record.
Literal Frobenius/conductor/prime trace adoption remains a separate scientific
mechanism needing its own map; no online point counting or audio synthesis.
Fixed prime/zeta/ordered-n-let metadata stay bound where actually implemented,
not relabelled as trained semantic structure merely by entering a manifest.

## 4. Frame Algebra And Content Binding

Store canonical root r plus explicit action frame a. A displayed/transported
root is A(a)r; the canonical key is recovered by A(a)^-1. Same-frame transport
of all roots changes mu_o to A(a)mu_o and permutes within-orbit root buffers.
Normalization before scoring restores the same canonical query/candidates.
Never rotate roots but leave the index/profile or query/read coefficients in
an old frame. Retain the action label/order: equal endpoints can have distinct
operator histories and stabilizers.

For H4 relative content key q^-1 p, common left action g preserves it:

    (gq)^-1(gp) = q^-1 p.

Common right action instead gives g^-1(q^-1 p)g; corresponding factor indices
must be conjugated, not left unchanged. Pure frame reparameterization is not a
semantic update and should receive zero task advantage. E learns physical
ordered transitions; the gauge convention is fixed.

No direct E8 action is applied to paired-icosian coefficient coordinates.
Any future such bridge must supply a concrete T, induced metric G=T^T T,
A=T^-1 S T, verify A^T G A=G and admitted lattice closure, and retain its inverse
witness. The golden/Galois companion is constrained, not an independent latent.
Hopf/S2/audio/IP outputs are observations/codecs, not full-state replacements;
fiber, chirality, order and exact payload residual remain retained.

SpiralCore's route/Bell alphabets belong at explicit transport boundaries. A
route collision is allowed; `(lineage,record,commit,offset,digest)` stays distinct.
Hashes verify retained side information, not reconstruct it. A128-bit IP string
cannot alone reversibly carry every256-bit digest. No tone round trip enters
the numerical training/serving loop.

## 5. One Objective, Credit Across Time

Memory itself has no trainable replacement values: its write/query POLICY is
learned. A statement's value must influence credit when a later query uses or
fails to use it. Training episodes therefore contain writes, distractors,
corrections/reassertions and delayed queries, not independent compiler rows.

Let z be all sampled hard choices for one episode: retained H4 right actions,
observed roots/categories,
compiler acts/relations/spans/views, record roots, orbit admissions, bounded
read/donor choices, Generate prototype codes and, in a free rollout, token IDs.
Each conditional distribution uses the relevant current native integer logits
converted to offline probabilities. Chosen branches execute real exact store
and table operators, never a fractional record or a weighted text span.
Sampling is OFF at serving; serving uses each declared greedy native decoder.
It is an explicit stochastic training distribution, not bit-identical greedy
training or a derivative of argmax. New forced-choice replay APIs are required.

For a physical action i (Generate or admitted Copy alias), let c_i be the final
clipped native Q24 energy, w_i its existing integer Q31 lookup weight and
token(i) its actual token. Native pooled token probability is

    P_native(y|h,z) = sum_{i:token(i)=y} w_i / sum_i w_i.

The existing training adapter anchors probability/log-mass values to this
integer trace while using a clipped-score softmax differential. Preserve that
declared approximation: an exp softmax is not the native Q31 mass calculation.
The free-rollout training TOKEN SAMPLER uses an explicit offline smooth law
`pi_token(y)=sum_{i:token(i)=y} softmax(c_i / temperature)` (with the exploration
mixture below), so its recorded log probability is the law actually sampled.
Sample pooled tokens, not a separate prototype/choice for each equivalent alias.
This differs from both integer-mass sampling and served greedy mass selection;
report that gap. No distribution normalization enters served computation.

Use existing native pooled-ranking complete-answer/EOS loss at canonical
teacher prefixes, plus a declared closed-loop loss on own sampled prefixes:

    J(theta) = E_z[L_rank(x,y,z) + eta L_compiler + xi L_typed_memory]
               + lambda E_rollout[L_complete].

`L_rank` is the existing pooled winner/rival, equal-episode bottleneck objective;
do not replace it by first-token CE. `L_complete` uses complete answer membership
and store/view correctness under typed training intents, normal EOS and bounded
reply length. Its numeric reward weights are preregistration fields, not hidden
grader changes. A wrong own prefix has no invented aligned gold next-token label;
teacher-prefix and terminal closed-loop losses remain distinct trajectories.
Compiler auxiliary CE and typed write/view losses regularize identifiable acts;
they do not supply the inference record or substitute for reply success.

For a hard stochastic decision graph, the logit-level estimator is

    grad J = E[partial_theta L +
               sum_d (L_suffix(d)-b_d) grad log pi_theta(z_d | history_d)].

Here the suffix includes later memory use, route admissions and replies; b_d is
independent of that decision's sampled action conditional on its pre-decision
history. Terminal episode loss can be used without reward-to-go subtraction
when no valid decomposition is available. Teacher tokens/oracle labels never
enter history_d. Costs before a decision may be subtracted as a causal baseline.
Use a detached previous-update/prefix baseline, NOT a baseline secretly dependent
on the current action or current outcome. Log each baseline/support policy.

For each structurally valid discrete head, use a numerically stable categorical
law on its native score units, with a preregistered positive exploration mixture
`pi=(1-epsilon) softmax(score/temperature)+epsilon Uniform(valid)`. Its log
derivative is the derivative of this MIXTURE, not of softmax alone. Never mask
missing answers into existence. Orbit/record selection samples sequentially
without replacement with the actual masked conditional law at each rank;
greedy mode repeats highest-score removal with the bound tie policy. Context
samples120 RIGHT ACTIONS from the actual token/OLD-own/OLD-neighbor transition
scores, then executes `h_next=h_old*action` by the native table. All lane actions
are computed from the same OLD state vector and committed synchronously.
Retained destination probabilities are its group-product pushforward, not raw
action logits mislabelled as destination logits. Separate token/NEW-own/
NEW-neighbor heads sample observed root120 and category33; category0 emits
Absent observation without erasing retained state or exact store content.
These root/category policies have their own log terms and downstream credit;
they cannot be trained by an energy coefficient partial through a frozen
observation. Both sampling and greedy modes obey this same causal phase order.

Prototype code choices are global shared parameters, not context-dependent
experts. Sample one complete token/lane prototype table per batch and reuse it
for every consumer/episode in that batch; its log-probability term receives the
batch objective ONCE. Export uses the greedy table. Do not sample a different
prototype for each Copy alias, or count the shared prototype draw once per
occurrence. This requires explicit forced-choice table construction and is not
the current prepared Generate adapter. Other decisions are sampled at their
actual chronological nodes, with revision/trace/decision IDs for ownership.

The identity is for the declared stochastic graph at differentiable logits.
Quarter-Q4 packing/argmax export still needs a declared STE/finite-update policy;
that approximation is biased relative to packed greedy optimization. Do not
claim an unbiased gradient of the served model, guaranteed descent or general
posterior. Greedy native outcomes and actual reload are the selection authority.

### Credit Ownership And No Double Counting

| Parameter owner | Credit and boundary |
|---|---|
| E transition tables | Right-action log-probabilities receive compiler, later memory, route and complete-reply suffix credit; synchronous old-state order and shared traces accumulate once |
| C heads | Native act/relation/span/view logits get auxiliary plus downstream score-function credit; exact slices/store writes have no fake derivative |
| K source-root heads | Later route/reply suffix credit reaches the earlier record-root assignment; fixed group actions receive none |
| Q family/admission heads | Actual candidate-inclusion outcomes receive downstream loss; no gold-candidate admission or softmax-over-kept surrogate for missing candidates |
| R donor/bridge policies | Their actual consumed branch gets suffix credit; clipped token-score gradient is a separate coefficient partial |
| O observation heads | Root120/category33 log-probabilities get downstream credit, including presence; Absent observation is not payload eviction |
| O Copy/Cue/P factors | Native-anchored coefficient partial with categorical inputs detached; do not freeze factors trained on an old E representation |
| G prototype choices | Prototype policy/finite alternative utility affects all Copy/Generate/U consumers that use it; fixed codebook is not optimized |
| G/U energy coefficients | Native-anchored coefficient partial through shared final clip/pool, no second per-branch normalization |

One chosen discrete node uses ONE primary estimator. Existing finite120 utilities
can replace its sampled estimator by conditional expectation only for a closed
subgraph with every downstream alternative correctly replayed/marginalized.
#2225's observation utility, which holds bank/other states factual, is a local
conditional tool, not that full episode marginal. Otherwise use it only as a
valid detached baseline/control variate with the required residual correction,
or as a separately declared biased comparator. Do not add its existing STE
pullback on top of the score-function estimator for the same state/prototype.
Never credit the Hamming route with an unrelated Copy-softmax derivative.

Categorical logits depend on shared upstream choices; fixed factual inputs are
detached for coefficient partials. Those upstream choices have their own
log-probability terms. Sum all losses once, run one backward/optimizer step per
batch, then export/rebind. There is no per-head optimizer using cached old E.
Signal transitions learn typed numerical observations, not authored abstentions.

## 6. One Revision Transaction And Replay

After an update:

1. Quantize/compile E, C, K/Q/R, O/P/G/U from the SAME fractional master revision.
   Preserve optimizer moments and seeds; no silently fresh moments per head.
2. Rebuild every dependency edge: tokenizer/labels -> encoder/algebra -> compiler;
   E/prototypes -> potential/bridge/Cue/Prefix/Generate -> continuation -> native
   consumer -> index/profile -> session. Bind codebook IDs/scales, composition,
   metric/frame, read policy, bounds and exact source data hashes in one manifest.
3. Instantiate a NEW compiler/session against the new revision. Do not mutate
   a compiler's encoder digest to bypass #2225 checks. Old prepared packets,
   feature/index caches and snapshots cannot be consumed by the new revision.
4. Training replays each raw episode from its declared initial store with this
   revision. Clear revision-stale latents, key assignments and family summaries.
   Persistent serving content can be reindexed from retained original witnesses
   under an explicit migration; preserve bytes/version IDs and recompute keys,
   never silently reinterpret a snapshot from an older model.
5. Greedy native qualification uses exported/reloaded bytes, no soft helper or
   training sampler. Write source/model/config/optimizer/consumer hashes and
   claimed/sealed report inventories before any scientific interpretation.

The new joint compile API must permit adapting all declared factor families.
`compile_context_transition_rebound` correctly forbids observation drift for its
old frozen-observation policy; do not weaken it or use it as the new joint API.
Likewise current frozen compiler feature generation must be replaced in the
joint caller with live shared traces, while preserving the old comparator.

## 7. Data And Test Fitness Before Compute

Training inputs are new RAW episodes from existing Rust world/dialogue tooling,
with typed source intents authored once for losses using the existing oracle.
They must cover all four views, same-value reassertion, distinct correction,
conflict, absent/no-history/physical eviction, distractors, variable values and
delayed queries. The existing native compiler needs the declared view extension;
no unsupported generated task is scored as a learned failure.
Bind actual tokenizer coverage, source spans, answer byte round trips and EOS.
Split on episode/bank/value/entity combinations, not merely row IDs. Prove no
held-out tuple/target leakage from train materials before executing predictions.
No training data, label fit or fresh panel is created in this design delivery.

**Route-pressure admission:** the current job/home schema supplies only two
eligible current relations per scope/entity. Exact predicted-address lookup
may already solve that task, and bounded geometric routing may never be used.
Before a route/profile comparison, statically inventory the training/evaluation
schema, eligible record counts, capacities and nontrivial query types. It must
actually include more eligible records than the declared route budget and
queries requiring the geometric fallback, under supported compiler labels and
tokenizer inputs. Otherwise FIT rejects a claim about profile/routing quality
or savings: use exact lookup, do not rent a pod to test an inactive mechanism.
Adding supported training relation labels requires a newly bound compiler
artifact/schema and prospective data provenance; it is not an acceptance-panel
change, and the existing512 cannot stand in for this missing instrument.

The unchanged512 component panel still uses supplied records: retain its exact
model-parent baseline and grader and report it independently. Its two records
cannot exercise large-store routing or compiler writes. A separate prospectively
fixed RAW-transcript instrument is required for whole-path causal diagnosis,
not a replacement acceptance panel. Freeze that instrument and its oracle before
its predictions; no retrospective answer rewriting or post-score endpoint choice.
The opened128 remains diagnostic and follows the registered seals; a genuinely
fresh later qualification remains necessary for M2 acceptance.

Required implementation checks, no training implied:

- Zero-update compatibility reproduces saved437 and its exact parent consumer
  in old packet mode; new route mode is explicitly separate, not presumed equal.
- Tiny complete RAW episodes use the actual learned compiler adapter and saved
  native heads (no scripted compiler qualification); compare exact predicted
  spans, versions, admitted candidates, consumed occurrence, reply IDs and EOS.
- Analytic tiny categorical graph enumeration checks logit score-function credit
  incl delayed writes/root keys against finite differences; missing probability
  support, baseline bias and duplicated pullbacks are blockers to fix.
- Field coefficient partial and sampled-state credit are separately audited;
  current clipping/saturation and physical aliases stay exact. A no-op gauge
  transport yields invariant output and zero advantage, not artificial learning.
- Every family intended to train has attributable derivative/optimizer ownership;
  zero unused/factual-cell gradients are not fabricated failures. Rebinding and
  model revision invalidation are exercised before and after one synthetic step.
- Full all-root transport/frame/profile checks reuse existing fixtures; exact
  handles survive route collisions, spin/inverse, record reassertions and reload.
- CUDA producer uses bounded device work, no host full-model per-alternative
  replay or hidden CPU tensor fallback. Check data/forward/credit parity and
  complete work inventory before any necessary paid model run.

Scientific controls belong to one preregistered decisive comparison, not many
discovery jobs: same raw episodes/full trainable inventory/budget with route
enabled vs exact-address plus admitted unpruned control; source/write/route/
memory-off interventions only in declared diagnostic evaluation. A topology-null
control must actually destroy adjacency/orbit structure while matching occupancy
and capacity. A coherent relabelling of roots PLUS all tables is isomorphic and
is an invariance check, not evidence against geometry. Report whether direct
address alone solves the task; do not credit the geometric branch for that win.
Literal arithmetic murmurations require separate arithmetic controls later.

Also declare two DIFFERENT within-family operations: a common labelled group
action with all frames normalized is a behavior-preserving gauge check; changing
individual root representatives within their existing orbit while leaving the
semantic content/key model fixed is a profile/representative intervention. The
latter can change mu and capacity collisions without changing the orbit label,
so report occupancy/candidate support and route-consumption changes separately.
This respects the owner's tangent-control direction without treating a common
coordinate relabelling as a destructive null or assuming a finite orbit is a
continuous manifold. DeepSeek's #2229 cell/representative controls are
peer work, not automatically a validated E8 control in this design.

No training numbers, steps/rates/weights or bar are inherited from the interrupted
recipe. Preregistration must fix parent artifacts, data hashes/splits, full
inventory, sampler/temperatures/entropy and clipping policy, auxiliaries/reward,
checkpoint endpoint, duration sufficient for joint adaptation, matched controls,
>=2 seeds, noise at the128 bank clusters rather than512 independent Bernoulli
rows, mechanism and complete-reply/transfer bars, stop/failure handling and cost.
Until these fields, exact-source checks, TEST FITNESS and the owner's training
hold release are present, compute is BLOCKED, not an owner decision to stop the
mechanism. Tooling failures are not model negatives; no fresh acceptance waiver.

Delivery constraint: #2225 and this design consume the two preparation PRs for
the named run. Its implementation/qualification belongs in the single next
integrated result delivery, not a third preparatory PR or another reading loop.
If the owner keeps training held, preserve implementation as unfinished source
without declaring a model result; a further source-only delivery needs explicit
owner scope rather than silently restarting the preparation allowance.

## 8. Work Bounds, Not An Efficiency Claim

For store records N, maximum value length V, L state lanes and fixed M/B/K:

- Exact address lookup uses the existing index (cost measured for its real
  implementation, not assumed O(1)); original payload storage is O(NV).
- Root rings have at most240M handles per scope/entity,15 bounded summaries;
  query scans <=16MB +1 record headers, then <=K*V physical Copy positions.
- A write computes the key once, updates bounded ring/profile entries; exact
  store/version mutation is separate. Index rebuild at model migration costs
  O(N) record re-encodings and must be charged, not hidden behind token speed.
- Context work depends on actual admitted Context/Source tokens and L. The
  existing Generate vocabulary Vocab remains complete (saved parent4096), so
  output cost still scales with its finite factors. Routing is not a promise
  of constant total-model cost or elimination of all parameter access.
- Stochastic discrete credit needs sampled hard paths, not120^L or a Cartesian
  product of span/route/prototype alternatives. Training probability heads
  still cost their full bounded class counts; report them. Scalar log-prob and
  causal baseline records replace giant all-episode counterfactual tensors.
- Device batching must preserve chronological store dependence. Immutable
  source/token/record arrays may be staged once; per-update model/index views
  are rebuilt on device. Rust host admission/checkpoint I/O is not a CPU model
  replay. No long CPU learner, speculative pod or overnight idle lease.

The initial product remains bounded by explicit session/store/value/context
limits, not unlimited durable recall. Long-history serving needs retained source
witnesses outside the recent context, index capacity policy and snapshot/version
tests; the present128-turn cap is not silently removed. Measure actual native
tokens/s, build/index/reload costs, bytes/parameter reads and device/host work at
two admitted context/store sizes before claiming whole-path savings. Statistical
quality/transfer and energy are separate results. Final D11 still requires all
numerical compiler/route/read/emitter kernels to use integer/add/shift/table
operations; the current float stack/E8 decoder is not automatically that kernel.
