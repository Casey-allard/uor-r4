# Stuck-point review: independent assessment and implementation correction

September 24, 2026. The owner supplied **UOR-R4 Stuck-Point Review.html** and
asked us to assess it across the project, apply warranted changes, and explain
disagreement. The attachment is evidence to assess; its proposed policies are
not automatically instructions. Its SHA-256 is
`906a8638efec147a73320ceaf70009cfbcb635f39bbd70b9a853c4bada5dae20`.

## Conclusion

**Agree with the central diagnosis and change the implementation sequence.**
The programme has advanced discrete serving mechanisms while failing to deliver
the language-to-context credit path specified in its own design. Another local
selector adjustment is not the best-supported next investment. Restore a working
language reference, implement an explicit differentiable training-to-serving
bridge, and run a persistent, sufficiently trained native model against fixed
comparators. The [canonical plan](project-track.md) and D8 now own that sequence.

This corrects how we learn and evaluate the model. The goal remains a native
transformerless geometric model with exact memory, integer/table serving and
eventual useful language, coding and reasoning. A transformer reference is an
offline instrument, not a serving replacement or hidden response source.

## Evidence that changes the decision

Three independent specialist reviews checked current source/history, mathematical
claims and primary literature, and Rust training/toolchain feasibility. The lead
also read the attachment, live policy/source, historical result documents and
the actual A4 outputs. Internal knowledge retrieval was checked against current
source rather than accepted as live authority.

1. **A4 operates but does not retain its fit decisions.** Four matched runs now
   finish with zero complete answers. Correct first-source selection is 2/24 per
   A4 arm versus zero for both controls. Every eligible provenance example is
   correct immediately after its local update (1,336/1,336), but final loaded fit
   selection is only 28/167 C120 and 24/167 2I among admitted sources. This is a
   training-objective/representation problem before held-out generalization.
   [Executed A4 result](integrated-attention-a4-result-2026-09-24.md).
2. **Representations drift despite a frozen dedicated address bank.** Shared
   State updates also affect fine Key/Query rows. Against A3, 21/24 query codes
   change in both A4 arms. The earlier phrase “retaining fine codes” was wrong;
   the sealed reports are preserved with an explicit correction. Suffix/provenance
   imbalance alone does not identify the cause.
3. **A stronger bounded language reference really exists.** Independent fresh
   hashing matches both #1014/#1017 checkpoints and all four exports each to their
   manifests (10/10); all 15 #1017 trainer/dependency source entries match its
   recorded Git revision. Its train/dev stores and tokenizer also match. #1014
   measured a 2.677393-nat attention-off penalty; #1017 reached 1.572752 nats/token
   after 149,995,520 cumulative tokens. These are retained historical results,
   not new quality runs. [#1014](../r4_softmax_end_to_end_attention_1014.md),
   [#1017](../r4_softmax_quality_capacity_continuation_1017.md).
4. **The missing graph is an engineering task we can perform in Rust.** There was
   no established autodiff dependency in this workspace. Rust already permits
   float/matmul training, and Candle provides autodiff and Metal. Autodiff alone
   does not differentiate argmax: the relaxation, estimator, hardening schedule
   and actual serving mask must be implemented explicitly. Inference-only
   `softmax_last_dim` uses a no-backward custom operation; blindly importing an
   inference model would recreate the failure. Use differentiable primitives and
   check actual Q/K/V gradients. [Candle source](https://github.com/huggingface/candle/blob/0.9.2/candle-nn/src/ops.rs).

## Where the supplied review needs correction

| Review claim | Reconciled position |
|---|---|
| Native addressing has no negatives and cannot learn | This describes A1. A2 introduced contextual negatives; A3 changes actual integer admission from 1/24 to 17/24; A4 learns candidate-or-NoRead. Their remaining local/truncated language credit is still inadequate. |
| Admission forces all candidates to identical relative codes | Prefix admission forces equality only in the coarse lane. A1's all-lane code collapse was measured, not an unavoidable property of all later admission. |
| The policy rules out the continuous training route | The review's diagnosis passage overstates this, while its closing toolchain section correctly offers Candle/Burn. Current policy already permits Rust float/matmul training; we adopt that Rust autodiff option. |
| D7 replaced D4–D6 | D7 explicitly preserves those constraints and changes implementation sequencing. D8 keeps their substantive goals too. |
| #1017 was simply parked and never used | Later grounding fine-tunes, pointer/relation heads and attention-LoRA work exist. Their failures must inform the renewed reference/student path. The larger #1019 capacity attempt had the hardware stop; #1017 completed. |
| A fixed minimum token count guarantees a meaningful test | Data, objective, task complexity, optimizer and learning curves matter. Tiny fixtures cannot establish language, but can reject an arithmetic bug. Token floors are planning targets, not universal laws. |
| Reuse the sealed #1017 split throughout the ladder | It was revealed in August. It is now a fixed regression benchmark. Development selection and a new final holdout remain separate. |
| New models will run at #1014's approximately 7.1k tokens/s | That rate belongs to one PyTorch/MPS architecture and batch schedule. Measure this implementation before projecting multi-hour runs. |

#1017 and A4 also use different tokenizers and corpora. Their reported
likelihoods are not a matched numerical comparison. The historical transformer
missed its frozen 1.50 quality gate; using it as a reference does not rewrite that
failure or turn its five scene-retention cases into general chat competence.

## What the literature supports

[LEMA v1](https://arxiv.org/html/2609.25802v1), published September 22, is authentic.
Its hard-from-start ablation reports 5.497 versus 3.542 validation cross-entropy
for a 77M model; its successful method uses straight-through binarization and
annealed soft selection. That supports a tested soft-to-hard route. It does not
prove that every discrete learner is impossible, and its transformer architecture
does not satisfy our serving target.

[Transformer-VQ](https://proceedings.iclr.cc/paper_files/paper/2024/file/18eb80b9faaed5d003b31574bd2a3e9d-Paper-Conference.pdf)
trains quantized keys with a straight-through estimator, language loss and
commitment/codebook learning. A separately pretrained teacher is optional. Its
quantization equivalence assumptions do not make isotropic codebook distortion
equivalent to language quality on learned anisotropic activations.

[PaTH](https://arxiv.org/html/2505.16381v2) supports studying products of
input-dependent transport. Its Householder-like factors can contract and become
singular; pure SU(2)/2I isometries do not automatically inherit its result. An
ordinary recurrent/Householder comparator belongs alongside geometric transport.

## Mathematical corrections that matter to implementation

- **Labels versus structure:** unrestricted tables on equally sized label sets
  are equivalent under reindexing. Fixed noncommutative 2I composition is not
  equivalent to fixed abelian C120 composition; that would require an isomorphism.
  An ordinary finite-state implementation can still emulate either, at a cost
  that must be measured. A failed label-set comparison is not a general theorem
  against geometric computation.
- **600-cell optimality:** universal optimality for specified pair potentials and
  optimal minimum-distance packing do not prove minimum mean quantization error
  on language activations. The attachment's sandbox distortion/word-problem runs
  were not reproduced here and remain unverified external observations.
  [Cohn–Kumar](https://arxiv.org/pdf/math/0607446).
- **Nine scores:** the review is correct that class functions have nine degrees
  of freedom. This does not imply rank nine or rule out useful inductive bias;
  their 120-by-120 kernels can have rank 120: `f(g)=1[g=e]` gives the identity matrix. Parameter count is not
  kernel rank. A dot-product class score is a particular restricted operator.
- **Complexity:** Barrington's nonsolvable-group result and TC0 limitations apply
  under specified depth, precision and architectural assumptions. They do not
  establish learnability, language utility or superiority over all recurrent or
  transformer variants. [Barrington](https://www.cctbio.ece.umn.edu/wiki/images/3/3f/Barrington_Bounded-Width_Polynomial-Size_Branching_Programs_Recognize_Exactly_Those_Languages_in_NC1.pdf),
  [The Illusion of State](https://arxiv.org/html/2404.08819v2).
- **Transport:** Cayley closure eliminates re-quantization of a key after each
  *chosen discrete* transport. It does not remove the error in quantizing the
  transport itself. Composition order, frames, orientation and a non-canceling
  observable must be explicit. A fair comparator may keep a cumulative frame;
  charge its cost rather than forcing unnecessary repeated quantization.
- **Exact arithmetic:** a finite binary shift-add chain cannot multiply an
  ordinary fixed-point scalar by irrational `phi/2` exactly. Algebraic pairs give
  `phi(a+b phi)=b+(a+b)phi`; halves require an explicit dyadic scale and bounds on
  denominator/coefficient growth. Pure finite-group table transport is exact;
  mutable algebraic-valued recurrence needs its own overflow/rounding contract.
- **Memory and energy:** a finite group product is not an injective record of all
  prior input. Exact bounded events remain separately stored. A learned empirical
  score is not automatically a physical Hamiltonian or an energy-saving proof.

## Adopted implementation changes

D8 parks A1–A4 local-credit fitting as the main learning strategy. Their code,
parents, negative outputs and exact-memory/serving components remain available.
The active ladder now starts with a pinned #1017 reference and evaluator,
followed by a jointly trained native recurrent-memory student, explicit
discretization, admission under the serving mask, then complete integer export.
Geometry earns distinct roles through codebook, transport and arithmetic controls.

The first concrete deliverable is an offline-only `uor-r4-training` crate using
pinned Candle 0.9.2 (an intentional established-version pin, not a claim of latest).
It imports the actual checkpoint and checks differentiable next-token computation
against the existing Rust reference before larger training is projected. Its
device, parity, gradients and execution outcome belong in the current-state
entry; writing code or compiling it is not a training result.

### Executed correction: reference and autodiff foundation

The new crate was compiled from committed source `e4f7e9d9` with pinned Candle
0.9.2, `metal` and `reference-accelerate`. Both actual device runs pass. The
existing oracle reports Apple Accelerate CPU arithmetic. Independent review
confirmed input hashes, token window, unchanged tolerances and architecture.

| Integrity check | CPU | Metal |
|---|---:|---:|
| Compared logits | 131,072 | 131,072 |
| Maximum absolute logit difference | 0.000015259 | 0.000016212 |
| Matching top-one prediction positions | 32/32 | 32/32 |
| Finite nonzero parameter-gradient tensors | 56/56 | 56/56 |
| Finite nonzero Q/K/V-gradient tensors | 18/18 | 18/18 |
| Selected finite-difference checks | 3/3 | 3/3 |
| Optimizer steps | 0 | 0 |

The [compact execution evidence](../evidence/reference-autodiff-integrity-2026-09-24.json)
binds both full local reports, executable, source and
[evaluator manifest](reference-evaluator-v1.json). The CLI enforces the weights
and architecture; the full evaluator manifest was checked independently, not
automatically enforced by the executable. Build wall time was 113.75 seconds;
integrity process times were 1.39 seconds CPU and 12.88 seconds Metal. Those
include loading, oracle comparison, backward work and finite differences, with
cold Metal setup; they are not training-throughput or GPU-active measurements.

This is an executed prerequisite for the new learning route. No new language
quality, native student, hard-selection gradient, corpus fit or complete baseline
table is claimed. The baseline table and real generation replay remain rung 0's
next work; the canonical plan now specifies the subsequent recurrent computation
graph so another local selector patch cannot substitute for it.

The live state is shortened; its complete prior contents are preserved in a
linked archive in the same directory. The existing canonical plan owns rung
status, avoiding a second competing ladder document. The team protocol now
continues the authorized rung through its declared decision, rather than inventing
a separate milestone after each micro-result. Evaluator hashes and named gates
must survive implementation changes; revisions need an explicit explanation and
independent review. A local file/agent policy cannot make the evaluator technically
uneditable, and no such protection is claimed.

We do not adopt an arbitrary six-week freeze, unmeasured throughput promises,
paid/external compute, universal token-count laws, or a new Python dependency.
The next natural-text campaign must declare meaningful exposure and seed/control
budgets from measured hardware throughput, and report model compute separately
from orchestration time. The owner retains strategic authority. General language,
geometric advantage, D5 full-path sparsity and physical energy remain separate
unmet acceptance conditions.
