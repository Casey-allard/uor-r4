# Attention, learnability, and prose under the UOR-R4 serving boundary

Research date: 2026-09-13. This is architecture research, not model qualification. No training, inference, build, paid compute, or source changes were performed. The companion attention-sources.json records 25 source groups, including targeted paper sections and source implementation reads. This is a bounded primary-source investigation; it is not a claim to have read every paper, repository, theorem proof, or available resource.

## Recommendation

Pursue **learned relational geometric state with exact addressed reads and shared nonlinear typed transitions**. Treat Hamming distance as a useful geometric comparison and candidate selection mechanism inside that model. Preserve the directed relationships from which scalar distances are derived. Do not assume that scalar Hamming rankings, more zeta channels, or exact canonical identity alone constitute a language model.

The next design should be selected on whether it can learn a simple contextual dependency and then compose dependencies, rather than whether an untrained mechanism matches ten days of accumulated repair. The existing initialized LUT4 path supplies a useful deterministic execution scaffold, but its current topology has a concrete representational bottleneck and lacks a demonstrated learning algorithm. Keeping that topology by default would not be evidence-supported.

The strongest three hypotheses are H1 below (structured geometric transition model), H2 (learned traversal over a canonical relational graph), and H3 (a JEPA-like predictive state hierarchy plus a native lexical decoder). H1 is the best first learning experiment. H2 supplies a natural extension for exact composition. H3 is a worthwhile auxiliary objective once H1 actually learns; it should not postpone lexical prediction or substitute a pretrained text decoder.

## What attention must accomplish

A transformer separates learned query/key compatibility, competition over eligible context, transport of values, and a nonlinear update of state. Stacked layers let a later query depend on earlier retrieved information; causal masking excludes future content. Geometry can replace each operation, but replacing only the similarity function leaves most of the learning problem untouched. Attention weights are not independently taught grammar rules; they are trained through the model's prediction objective. [Attention Is All You Need, §§3.1–3.3](https://papers.neurips.cc/paper/2017/file/3f5ee243547dee91fbd053c1c4a845aa-Paper.pdf).

At a geometric level, the useful abstraction is:

1. Encode the present causal situation into a state, and committed occurrences into contextual keys plus values.
2. Compare query and candidates in an explicit common representation or after transport between frames.
3. Let candidates compete; retain uncertainty or alternatives where the architecture supports them.
4. Bring relevant value information into the working state while preserving the distinctions needed later.
5. Apply a learned nonlinear transition, then query again or emit a token.
6. Train the complete composition against prediction, including earlier reads that cause later decisions.

This six-part decomposition is our synthesis, not a theorem that a particular geometry must satisfy these exact interfaces. A hard selected read can implement contextual access without softmax. It changes the optimization problem: losing soft weights removes an easy dense gradient path through competing candidates.

The modern Hopfield interpretation relates softmax retrieval to an energy-based associative update. Its convergence and capacity statements concern its specified energy, update, separation, and dimension assumptions; they do not transfer automatically to arbitrary repeated H4 transitions. [Hopfield Networks Is All You Need, §2 and Appendix A.4](https://arxiv.org/html/2008.02217).

Sparse Distributed Memory is particularly relevant: overlaps of address neighborhoods produce a similarity-dependent associative read, and under the analyzed conditions approximate the exponential compatibility used in attention. That supports investigating Hamming neighborhoods as attention. It does not prove that the present finite H4 signatures, fixed two-lane sum, or typed hard-read policy inherit the same behavior. [Attention Approximates Sparse Distributed Memory, §§2–3 and Appendices B.3–B.5](https://arxiv.org/html/2111.05498).

The HDC/VSA literature supplies binding, permutation, cleanup, and compositional memory operations; it also explicitly identifies capacity/interference and flow control as unresolved practical limits. Which operations obey this project's runtime contract depends on the VSA family. Neural Turing Machines separately demonstrate learning content addressing and sequential location traversal, but through a differentiable weighted memory/controller. These are useful design precedents rather than complete prose solutions. [VSA framework, §§III–IV and VI-A](https://arxiv.org/html/2106.05268), [Neural Turing Machines, §3](https://arxiv.org/pdf/1410.5401).

## Hamming, full relations, and the owner's triangulation proposal

For bipolar vectors b,c in {-1,+1}^D, the identity b·c = D−2 d_H(b,c) is exact. For random hyperplane sign hashes of arbitrary nonzero vectors, the disagreement probability is angle/π; finite hashes estimate that angle with sampling error. These are different statements. A deterministic H4 reference bank is neither independent random hyperplanes nor arbitrary semantic hashing. [Charikar, §3](https://www.cs.princeton.edu/courses/archive/spring04/cos598B/bib/CharikarEstim.pdf).

The current implementation's single-root finite census is evidence about that codebook. It does not establish equivalence between summed two-lane Hamming distance and every cross-lane angular relation. Full signatures, scalar Hamming distances, exact root IDs, vector positions, and learned semantic relationships are distinct objects.

**Promising extension, derived here:** where the implemented finite roots form an actual unit-quaternion group G, keep the directed relative element

r_ij = g_i^(-1) g_j.

Then r_ij r_jk = r_ik, and a common left action h cancels: (h g_i)^(-1)(h g_j)=r_ij. This records a relative transformation rather than only its angle. The identity is conditional on an associative group and the stated action; it should not be copied unchanged to nonassociative octonions. For four-root states, the sixteen cross-lane relative elements between two states are a richer candidate descriptor than two independently summed scalar distances. Whether all sixteen are useful is a learning question, not an obligation to compute them for every pair in every round.

Within a fixed common frame, all pairwise relative elements are derivable from node states. Storing every pair may accelerate access, but does not add independent information. For N occurrences there are N(N−1)/2 unordered pairs; a score and directed group relation for each pair increase storage quadratically. At the current N=256 bound there are 32,640 pairs, requiring only 255 KiB at eight bytes per pair before metadata. An all-pairs small-window reference is therefore quite feasible and should not be rejected on asymptotics alone. At N=4096 there are 8,386,560 unordered pairs; eight bytes per pair is about 64 MiB before identity, version, type, or proof metadata. Retaining sixteen cross-lane relations and more phase channels multiplies this expense.

A sensible comparison is:
- offline all-pairs relations for a small reference window, to understand what a bounded reader discards;
- online exact node states and canonical typed links, with local and retrieved candidate relations computed on demand;
- deterministic incremental updates only for new or changed occurrences;
- bounded multi-round traversal so earlier results change the next query.

This is not a dismissal of triangulation. It gives the proposal an exact algebra, an information accounting, and a feasible experiment. Scalar pairwise distances alone can lose orientation and distinguish configurations only up to applicable isometries; canonical frame/chirality/fiber data must survive independently.

For additional zeta zeros, a valid hypothesis is a richer fixed phase basis with learned use of phase differences and group-relative descriptors. Required measurements are aliasing over the actual context window, predictive contribution, distinct channel utilization, phase ablation, and total per-token cost. More constants alone are not more learned meaning. The companion geometry investigation owns which zeta-to-state map is actually available and justified.

Prime addresses can identify typed operators and nodes. A semiprime may identify an unordered pair of factors, but pq=qp cannot encode ordered composition by itself. Preserve ordered n-lets, types, parent receipts, and the actual composed operator. A prime-labeled module becomes a learned expert only when there are separately learned parameters and a trained routing decision. No such adoption follows from this research.

## What geometric papers contribute

SE(3)-Transformer distinguishes invariant attention scores from equivariant value messages, using spherical-harmonic/tensor-field structure. This is directly useful conceptual guidance: invariant scalar scores should not replace orientation-bearing values. Its experiments concern geometric data, and its actual implementation contains tensor contractions and softmax. It is not a matrix-free prose model. [Paper, §§3.1–3.2](https://proceedings.neurips.cc/paper_files/paper/2020/file/15231a7ce4ba789d13b722cc5c955834-Paper.pdf), [official modules](https://github.com/FabianFuchsML/se3-transformer-public/blob/master/equivariant_attention/modules.py).

GATr gives objects and transformations typed multivector representations, with geometric products and equivariant layers. Its attention source explicitly uses geometric/scalar compatibility and weighted value contractions; its efficient path calls scaled dot-product attention. Borrow the typed representation/transport discipline, not a claim that geometric algebra eliminates contractions. [Paper, §§2–3](https://arxiv.org/html/2305.18415), [official attention source](https://github.com/Qualcomm-AI-research/geometric-algebra-transformer/blob/main/gatr/primitives/attention.py).

nGPT keeps states and weight vectors on hyperspheres and interprets updates as displacements. The official block still contains learned dense projections, FlashAttention, and an MLP. HELM constructs Lorentz-space transformations, hyperbolic distance attention, positional rotations, and normalization; its linear map explicitly includes W^T x. These papers support treating geometry and optimization together, not replacing the present runtime with either architecture. [nGPT paper](https://openreview.net/pdf?id=se4vjm7h4E), [nGPT source](https://github.com/NVIDIA/ngpt/blob/main/model.py), [HELM §§3–4](https://arxiv.org/html/2505.24722v2).

No geometric result read here establishes that H4, zeta phases, hyperbolic geometry, or Cayley–Dickson multiplication automatically supplies the semantic compatibility learned by an attention model.

## What nontransformer language models do and do not solve

| Family | Transferable lesson | Why its published runtime is not a drop-in under this project's rule |
|---|---|---|
| Mamba | Input-dependent retention/write/read dynamics matter; static oscillations alone are inadequate for selective sequence tasks. | Dense input/output and parameter projections, contractions, floating arithmetic. |
| RWKV | Recurrent token mixing can generate language; context-dependent recurrence and state capacity matter. | Matrix-vector projections remain; later versions use matrix-valued memory. |
| RetNet | Parallel training and recurrent execution can represent the same mathematical process. | Key/value outer products and query-state contraction remain, even when written as multiply/sum. |
| GLA | Learned forgetting improves compressed associative state. | It is a matrix-state linear-attention model with projections. |
| Gated DeltaNet | Selective correction of an association differs from globally decaying memory. | Matrix-state regression and key/value contractions remain. |
| “MatMul-free” LM | Low-precision recurrent models can reduce implementation cost. | Its ternary signed-accumulation dense layer is still a mathematical matrix product, explicitly excluded here. |
| Reformer | Approximate geometric candidate search can avoid evaluating all sequence pairs. | It still performs attention inside selected buckets and retains a transformer architecture. |

Sources: [Mamba method and selection analysis](https://arxiv.org/html/2312.00752), [single-token source](https://github.com/state-spaces/mamba/blob/main/mamba_ssm/modules/mamba_simple.py); [RWKV-5/6 §§3, Appendix A](https://arxiv.org/html/2404.05892), [RWKV-4 inference source](https://github.com/BlinkDL/ChatRWKV/blob/main/RWKV_in_150_lines.py); [RetNet §2](https://arxiv.org/html/2307.08621), [official recurrent source](https://github.com/microsoft/torchscale/blob/main/torchscale/component/multiscale_retention.py); [GLA §§2–3](https://arxiv.org/html/2312.06635); [Gated DeltaNet §3](https://arxiv.org/html/2412.06464v3); [Scalable MatMul-free LM §3.1](https://arxiv.org/html/2406.02528v3); [Reformer §2](https://arxiv.org/html/2001.04451).

A published label such as attention-free, linear-time, or matmul-free is therefore insufficient for admission. A bitwise implementation of a dense linear contraction remains a contraction. Conversely, a typed finite group action selected and composed by table lookup is not automatically a disguised transformer; inspect its actual mathematical function.

## Why the present LUT4 policy needs reconsideration

Inspected source: shared-geometric-core/crates/uor-r4-core/src/native_geometric/hamming_policy/policy.rs, constants and Topology::seeded/validate.

Its 4096→1024→256→1796 topology has three layers of four-input gates. Therefore:

- One output bit has at most 4^3=64 distinct input bits in its per-call dependency cone.
- One four-bit scalar score has at most 256 input bits in the union of its cones.
- Every output is a function of the same 256-bit intermediate representation. Expanding the last layer does not restore distinctions lost at that point.
- Last-layer wiring repeats every 64 gates. Different truth tables still allow different functions, but they operate on only 64 repeating input-cone families.
- “All inputs reach some output” is not the same as “every relevant decision can combine the required inputs.” Phase, payload, and orientation fields may lack a joint path to a particular candidate score.

Caveat: the final argmax compares many scores and can depend on their joint union; recurrence can also expand effective dependencies over time. Thus the 64-bit bound is a per-output, per-call bound, not a proof that the complete model depends on only 64 inputs or cannot learn anything.

This is a structural concern worth resolving before an expensive fit. The proposed alternative is not simply another larger random funnel. Use structured shared local logic blocks over ordered bytes/geometry fields; explicit lossless carry channels for identity and payload; cross-field blocks for relations; residual/pass-through initialization; and enough composition depth to combine the declared causal dependencies. Capacity and channel counts should follow the task and laptop-cost projection.

The original differentiable logic networks train continuous mixtures and discretize gate choices. Their official implementation makes the training/evaluation difference explicit. The later logic-tree work identifies random connectivity, limited depth, and washed-out activations as problems, and investigates structure and residual initialization. Both chiefly establish classification results; neither establishes a scalable prose model. [Original paper](https://arxiv.org/html/2210.08277), [official LogicLayer source](https://github.com/Felix-Petersen/difflogic/blob/main/difflogic/difflogic.py), [logic-tree §§2–3 and Appendix A.4](https://arxiv.org/html/2411.04732).

## Three concrete hypotheses

### H1 — Structured geometric recurrent transducer with relational reads

State: several explicitly typed finite geometric channels, exact occurrence identity, retained orientation/fiber data, and a bounded lexical state. Key/query encoders are learned shared nonlinear circuits. Each read builds a relation-preserving descriptor, uses Hamming neighborhoods for bounded candidate search, and applies a learned shared comparator. Selected values enter state through typed finite-group actions and nonlinear logic transitions. Read again after updating; emit through a learned byte/EOS decision circuit.

Training: offline Rust reverse-mode through a declared relaxation or hard-forward surrogate, with actual deterministic whole-sequence forward execution. Begin with a short-context lexical/read task and dense enough training evidence, not random truth-table bit search. Use codebook occupancy and pair margins diagnostically; no target-authored query, operator, or output is admitted at inference.

Advantages: direct continuation of reusable exact memory and geometry; native no-contraction inference is plausible; all parts can be tied to token prediction. Risks: sparse hard-read credit, topology undercapacity, quantization collapse, and costly training. The new task must show that the read representation actually becomes predictive.

### H2 — Canonical relational graph with learned bounded walks

State: prime-addressed occurrence/result nodes, ordered typed edges, full relative group transformations, and a learned current query state. A shared policy reads a small neighborhood plus geometric candidates, chooses or rejects relations, transports/composes the selected result, and repeats within a declared bound. Newly inferred results receive canonical identities and lineage, but are distinguishable from observations.

Training: learn edge selection, operator composition, and lexical emission together from sequence data. Offline all-pairs construction can be a small-window diagnostic comparator. Avoid supplying the correct semantic graph as an undeclared parser/oracle; if relation supervision is used during training, declare it.

Advantages: directly expresses recursive dependence and object lineage; semiprime/ordered-factor ideas can supply compositional naming without requiring many separate experts. Risks: learning the graph from prose is itself hard; exact storage does not choose correct facts; irrelevant branching and fixed walk depth limit coverage. Prefer shared operators before parameter-specialized experts.

### H3 — JEPA-like predictive geometric hierarchy plus native surface model

State: a slow geometric representation predicts future or missing contextual states, while a fast native decoder predicts exact bytes. The auxiliary target encoder is offline, geometrically bound, and protected against collapse. A retained lexical path carries distinctions intentionally discarded by semantic abstraction.

Training: add latent prediction to an already learning native autoregressive model and compare with the same model without that objective. Prediction horizon, causal availability, target stop-gradient/EMA rules, and lexical preservation must be explicit. Never provide future target representations at inference.

Advantages: potentially learns reusable longer-range relationships and improves state organization. Risks: representation collapse, invariant targets erasing wording/version distinctions, a latent predictor that does not improve prose, and an expensive decoder becoming the actual model. This is an auxiliary research route, not a decoder replacement.

Ranking is a research judgment: H1 first; design its memory interface to permit H2; evaluate H3 after basic H1 learning. These are hypotheses, not claims of demonstrated language capability.

## Learning, credit, and a fair progression

There is no exact ordinary derivative through deterministic argmin or a truth-table choice. Straight-through estimators and continuous relaxations are approximations, not mathematical repairs of that discontinuity. VQ-VAE demonstrates a nearest-code forward path with copied backward gradients and a codebook/commitment objective. Gumbel-Softmax includes an explicitly biased straight-through variant. Their success elsewhere licenses an experiment, not an unbiased-gradient claim. [VQ-VAE §3.2](https://arxiv.org/pdf/1711.00937), [Gumbel-Softmax §§2.2–3](https://arxiv.org/pdf/1611.01144).

For UOR-R4, any parameter update can change an early address, later observed value, subsequent state, and later query. Recompute the affected sequence. A frozen pointer trace measures a conditional local effect, not full-path learning. Whole-trajectory finite contrasts can check an estimator direction on a tiny task; enumerating truth-table proposals is not thereby a scalable optimizer.

JEPA has something real to offer. VL-JEPA predicts text embeddings with anti-collapse training and invokes a text decoder separately; its implementation description uses a vision transformer, Llama-based predictor, and pretrained text embedding model. That architecture is excluded, but latent prediction remains a transferable objective. Data2vec independently supplies text evidence for predicting contextualized latent targets; its NLP evaluation is language understanding, not proof that such an objective alone generates prose. [VL-JEPA §§2–3](https://arxiv.org/html/2512.10942v2), [data2vec method](https://proceedings.mlr.press/v162/baevski22a/baevski22a.pdf), [official text training source](https://github.com/facebookresearch/fairseq/blob/main/examples/data2vec/models/data2vec_text.py).

Proposed progression:
1. **Learning gate:** one contextual relation and an actual generated byte plusEOS; learn across changed content and positions. Compare initialization, trained model, changed-source, and read-disabled. Record training/dev curves and complete outputs. A failed first fit diagnoses the learner; it is not broad architectural rejection.
2. **Composition gate:** two dependent reads where the second address depends on the first value, then reuse a newly computed result. Hold out compositions and relocations; inspect exact emitted sequences.
3. **Small prose gate:** train on licensed, document-separated natural text; measure held-out byte loss/bits-per-byte and free-running grammatical/coherent continuation. Compare equal-resource native baselines and context ablations. Retrieval success alone cannot pass this stage.
4. **Capability growth:** conversation/memory and coding evaluation at increasing context, evidence on actual generated behavior, scaling curves, and full runtime cost.
5. **Replacement gate:** only now apply the accumulated repair controls to decide promotion. Preserve the old model throughout.

Do not turn this sequence into twenty process steps with no learning. Each stage should combine the smallest complete training experiment, generation, diagnosis, and a finite decision under one recorded resource budget.

## Evidence boundaries and unresolved work

- The papers establish several viable ingredients, not the owner's full constrained model.
- None read here demonstrates frontier general prose on an M1-class laptop using this combination and no mathematical matrix products.
- We have not measured the learning/cost of H1–H3.
- We have not proved a semantic role for additional zeta zeros or prime factorization.
- The current initialized mechanism should remain a scaffold/reference, not a presumed final architecture.
- Source URLs on branch heads are access-date observations, not immutable source receipts. Any adoption requires a pinned commit and an implementation-level license/contract check.
- Geometry-specific theorem scrutiny, local UOR/PRISM/N3mesis inventory, Cayley–Dickson and number-theory synthesis are assigned to companion research records; this note makes no claim to have completed those.

## Owner routing analogy: OSPF — September 13 follow-up

The owner suggested OSPF while the order-sensitive state experiment was being built. OSPF separates a link-state database from a shortest-path calculation over its directed graph; route cost is the sum of constituent link costs, and the calculation preserves next-hop information. [RFC 2328 §16.1](https://www.rfc-editor.org/rfc/rfc2328.html#section-16.1).

**Architectural hypothesis for H1/H2:** keep canonical object/occurrence identity, typed connectivity and observed lineage as the available route map; let a shared learned contextual state determine useful relations and their compatibility/cost; use bounded native routing to access and compose them. Exact addresses must remain separate from geometric relevance. Source-content changes must propagate into later state and route decisions. A contextual score that changes with the walk does not automatically satisfy the assumptions of one static shortest-path calculation; expanded state or a learned bounded policy may be needed.

This analogy motivates an interface, not an OSPF implementation or a proof of language ability. A known topology and a shortest route cannot supply an unknown linguistic destination or semantic edge. Learning those bindings and useful costs from output remains the next substantive responsibility. The current ordered-prefix experiment uses strict structural compatibility against supplied sequence keys; it neither runs Dijkstra nor establishes learned semantic costs. Avoid a separate protocol/control-plane project before demonstrating a language-learning benefit.
