# Query-relative geometric memory: exact reduction and limits

Date: September 28, 2026. Status: mathematical construction and independently enumerated finite checks; no trained language result or claim of global novelty. UOR source context: `15d2ce3fac9f66ddcc945a8097e5c6216352c453`.

## 1. Objects and contract

Let G be a finite group, V a real vector space or an exact module, and rho: G -> GL(V) a representation. For a query code a in G, past events have key b_i in G and value v_i in V. Fix a nonnegative kernel kappa: G -> R with support S. The current query may select a kernel from a bounded, precompiled family; all statements below then apply to that selected kernel. Selection of that family has its own cost and learning obligation.

Define, over causally eligible events:

N(a) = sum_i kappa(a^-1 b_i) rho(a^-1 b_i) v_i,
Z(a) = sum_i kappa(a^-1 b_i).

For Z(a)>0 the corresponding normalized read is N(a)/Z(a). For zero mass the API returns an explicit NoRead state. Define bucket statistics:

B(b) = sum_{i:b_i=b} v_i,
C(b) = number of eligible events with key b.

### Exact addressed reduction

N(a) = sum_{d in S} kappa(d) rho(d) B(a d),
Z(a) = sum_{d in S} kappa(d) C(a d).

Proof: partition the event sum by b_i. For fixed a, b=a d is a bijection from G to G, with inverse d=a^-1 b. Every event within that bucket has the same coefficient and value action, and rho(d) is linear, so it can act on the bucket sum. Terms outside S vanish. This is a direct finite-sum identity, not a statistical approximation.

The result generalizes standard quantized-key aggregation with a group-relative kernel and value action. It is not a new general theorem about attention; VQ caching and group convolution are established ideas. Its proposed role here is to give the native reader a computational contract before fitting it.

### Restrictions

The grouping is exact only when bucket members have the same eligibility and scalar weighting for this query. Different role, gain, age, version, confidence or permission conditions require corresponding sub-buckets, algebraically separable summaries, or explicit records. The size of those partitions must be charged. Arbitrary query-dependent masking is not free.

The theorem does not preserve the original unquantized dot/softmax model. It defines a distinct operator. A sparse support is part of that definition, not a certificate that omitted dense-model attention is harmless.

Exact numerator and denominator do not automatically give an exact finite-width normalized output. Bounds, rounding, saturation, lookup/shift division and zero handling require a separate native numerical contract. A nonnegative coefficient code in {0,1,2,4} permits multiplication by a shift. This does not certify compiler output or all other instructions.

## 2. What is geometric, and what is not

The relative code is invariant under a common left action: (ha)^-1(hb)=a^-1 b. This is a statement about already-discrete codes. It is not independent local-gauge invariance, not proof that text possesses this symmetry, and not proof that a learned encoder respects it.

A regular permutation representation is exact without coordinate multiplications: rho(d)v(x)=v(d^-1 x). It uses |G| value coordinates. The prototype uses 120, not a compact four-dimensional quaternion value action. A quaternion action might use fewer coordinates but brings exact-ring coefficient growth or quantized table-action costs. Those alternatives need equal-information and actual-cost comparisons.

For unit quaternions, Re(conj(q) k)=q dot k. More generally a fixed linear functional of conj(q) k is bilinear and can often be absorbed into unconstrained query/key projections. It is therefore not, alone, an expressive advantage.

For any representation, sum_i alpha_i rho(a^-1 b_i)v_i = rho(a)^-1 sum_i alpha_i rho(b_i)v_i. Endpoint transport is a useful factorization, but it is not evidence for path-dependent holonomy or additional reasoning. Edge-dependent nonlinear actions would be a different hypothesis and must be tested separately.

## 3. Three obstructions

### 3.1 Norm removal changes the task

q=(1,0), k_A=(4,3), k_B=(6,8). Dot scores are 4 and 6; unit-direction scores are 0.8 and 0.6. Ranking reverses. Hence a unit-coded geometric reader is not merely a new implementation of an old dot reader. Separate gain, normalization and directional coding when attributing failure. This counterexample does not establish that gain loss caused the repository's observed negative result.

### 3.2 A sum forgets bindings

History A: Alice->blue, Bob->green. History B: Alice->green, Bob->blue. If both owners occupy one key bucket, both histories produce the same count and vector sum, but the requested Alice answer differs. No deterministic decoder of only that identical summary can answer both correctly. The remedy is to retain joint owner/relation/version structure or exact records, not to increase the sophistication of a downstream scalar score.

This obstruction is conditional on the colliding representation. Distinct learned owner keys can avoid this particular collision; finite capacity and multi-key conjunction remain empirical concerns. Exact storage also cannot correct a wrong learned owner/write decision.

### 3.3 More products are not finer coordinates

Every product of elements of a finite group remains in that same group. A single 2I product still has at most 120 possible values, however many factors were used. Refinement requires retained tuples, residual coordinates, a distinct larger codebook or additional state. Calling a longer multiplication chain a higher-resolution code is incorrect.

Purely invertible group updates cannot implement a many-to-one erase map on the same state space. General memory requires an irreversible write/replace/forget mechanism, or retained auxiliary history. Adding that mechanism is compatible with geometric addressing but its correctness does not follow from the group law.

## 4. Decision preservation is the relevant approximation target

Write q=q_hat+e_q and k_i=k_hat_i+e_i. Cauchy-Schwarz gives:

|q dot k_i - q_hat dot k_hat_i| <= ||e_q|| ||k_hat_i|| + ||q_hat|| ||e_i|| + ||e_q|| ||e_i|| = epsilon_i.

If a candidate's approximate score minus its bound exceeds every competitor's approximate score plus its bound, that candidate is the exact winner of this specified dot scorer. This is not a semantic correctness certificate.

A sparse implementation must obtain bounds for unvisited regions without scoring every item. The bound alone supplies no such index. Loose bounds may certify no decisions, and norm/residual metadata and refinement have a cost.

Top-1 preservation also does not preserve an attention average. If omitted attention mass is m<1 and all value norms are at most V_max, renormalizing the kept mass changes the read by at most 2 m V_max: the full read is (1-m)y_keep + m y_drop and both component averages have norm at most V_max. Thus pruning must report captured mass and output changes, not just winner recall. Learned value actions must have an appropriate norm bound for this statement to apply.

## 5. Collision versus occupancy tradeoff

Under an explicitly artificial uniform independent-key model, querying s distinct cells in a group of size M among N stored keys has expected selected events N*s/M and probability of no event (1-s/M)^N. At M=120, s=8, N=256 the expected candidate count is about17.1. At M=120^2 it is about0.142, and most queries are empty. This is not a prediction for task-trained codes; it shows why increasing product-code capacity cannot be treated as a free fix. Query and write learning, multiscale indexing and explicit bounded collision handling are central.

## 6. Executed finite checks

`group_attention_probe.py` independently enumerates SL(2,F_5), 120 elements, using exact modular arithmetic. It does not use or claim compatibility with the UOR group label order. The group-independent reduction applies to this group directly; transfer to the repository requires its actual bound table and representation.

The run checks all 1,728,000 associativity triples, all corresponding regular-representation and common-left-frame identities, 1,200 direct-versus-grouped numerator/denominator comparisons at 0/1/32/256/1024 events over two fixed seeds, and 256 insert/evict steps of a 32-event window. It includes zero-mass queries, a corrupted-identity negative control, a signed noncommutative subgroup witness and the owner-binding counterexample. Exact results, versions, seed, runtime and source hash are in the JSON receipt.

The 120-coordinate prototype's one-head int32 aggregate storage would be 57,600 bytes plus480 count bytes; an unpadded u8 group table is14,400 bytes. These are arithmetic storage counts, not measured memory traffic. Encoders, exact records, postings, role/gain partitions, output projection and runtime overhead are excluded.

Reproduce:

```sh
OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1 python group_attention_probe.py
```

Python is used only for independent mathematical analysis outside the repository. No Python model or product dependency was added, no repository checkpoint was trained/evaluated, and no active local lab job was changed.
