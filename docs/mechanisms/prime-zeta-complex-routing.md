# Prime-rank indexing and complex-plane zeta-phase routing

**Milestones:** M1 #2029 and M4 #2032. **Authority:** D32 October 14 addendum (#2398).

## Idea and implemented role

Replace arbitrary pseudo-random and hash-based token codebook initialization with deterministic, arithmetic prime-rank indexing and complex-plane Riemann zeta-zero phase routing.

Every token $v$ with vocabulary ID $\text{id}(v) \in \{0, \dots, V-1\}$ is mapped bijectively to a unique prime atom:
$$r(v) = p_{\text{id}(v)+1}$$
where $p_1 = 2, p_2 = 3, p_3 = 5, \dots$

For each prime atom $p$, its phase along the $k$-th non-trivial Riemann zeta zero $\rho_k = \frac{1}{2} + i\gamma_k$ is governed by the prime log-phase:
$$\theta_k(p) = \gamma_k \ln p \pmod{2\pi}$$
where $\gamma_k$ are the deterministic imaginary parts from Odlyzko's computed zeros (`crates/uor-r4-core/src/zeta_zeros.rs`).

In the complex plane, each channel defines a unit-modulus phasor:
$$z_k(p) = e^{i\theta_k(p)} = \cos(\gamma_k \ln p) + i \sin(\gamma_k \ln p) \in \mathbb{C}$$

These phasors assemble into complex phase vectors $\mathbf{z}^{(4)}(p) \in \mathbb{C}^4$ and $\mathbf{z}^{(8)}(p) \in \mathbb{C}^8$. The canonical realification maps:
$$\mathbb{C}^4 \to \mathbb{R}^4 \quad \text{and} \quad \mathbb{C}^8 \to \mathbb{R}^8$$
bridge the complex spectral domain to 4D quaternion/spin space and 8D octonionic/$E_8$ space, connecting Euclidean token indexing to Riemannian manifold routing.

### Exact Integer Quantization Boundary (D32 Compliant)

Per Decision D32 October 14 Addendum (#2398), floating-point arithmetic is permitted **only** on the complex plane up to the integer quantization boundary. Immediately at the output of the realification map, the representation is strictly quantized into discrete types:
1. **`PhaseQ29`**: $\text{raw}_{Q29} = \text{round}(\theta_k \cdot 2^{29}) \in [-2^{29}\pi, 2^{29}\pi) \subset \mathbb{Z}$, providing 29 fractional bits of angular precision in $i32$.
2. **`UnitS3Q30`**: Normalized 4D points $\mathbf{x} / \|\mathbf{x}\|$ scaled by $2^{30}$ into $[i32; 4]$ on the 3-sphere $S^3 \subset \mathbb{R}^4$.
3. **$E_8 = H_4 \oplus \varphi H_4$ Snap**: Continuous 8D vectors snap to the nearest root in the discrete 16-root anchor orbit (`ROOTS: [[i8; 8]; 16]`) of $E_8$.
4. **`Snapshot.table` Codebook**: Quantized root coordinates store directly into the discrete integer device codebook `int table[token * width + offset]`.

Downstream of this boundary, **100% of model compute and learning remains discrete**:
- Bivector actions in $\mathrm{Cl}(0,6)$ and BFS transport on the 64-state signed axis group.
- Ordered ternary FBS binder hierarchies (`BinderPlan`).
- Murmuration / Flock reads based on coordinate Hamming distance, frame L1, recency, and stable node identity.
- $G_2$ ternary recurrence gates $\{-1, 0, 1\}$.
- Shared integer vector writer ($8\cdot \text{token\_hinge} + \text{width}\cdot \text{address\_hinge}$).
- Counterfactual proposal selection without floating-point forward passes, gradients, or optimizers.

## Exact source scope and architectural boundary (D22)

- `crates/uor-r4-core/src/zeta_zeros.rs:4-517`: `ZETA_ZEROS: [f64; 512]` provides the exact ordinates $\gamma_k$.
- `crates/uor-r4-core/src/prime_route_attention.rs:338-377`: `PhaseQ29` defines the 29-bit phase quantization and wrapping.
- `crates/uor-r4-core/src/prime_route_attention.rs:485-570`: `UnitS3Q30` defines the 30-bit $S^3$ quaternion state and Hopf fibrations.
- `crates/uor-r4-core/src/prime_route_attention.rs:909-919`: `zeta_phase_delta` computes exact logarithmic phase differences.
- `crates/uor-r4-discrete/src/metadata.rs:344-367` & `crates/uor-r4-discrete/src/discrete.cu:257-267`: Replaces the multiplicative hash initialization (`token_id.wrapping_mul(1315423911)`) and xorshift PRNG initialization (`state = seed ^ block ^ 0x9e3779b9u`) with deterministic prime-zeta codebook population.
- `crates/uor-r4-discrete/build.rs:25-83`: `admit_ptx` strictly enforces zero floating-point instructions in the compiled integer CUDA module.

## What has been tried, honestly

1. **Float word vector baseline (`get_word_vector` in `uor-r4-core/src/lib.rs:386`)**: Evaluated continuous $\sin(\gamma_k \ln p)$ vectors normalized to float length 0.1. This showed strong harmonic properties across primes but operated in continuous floating-point space, which D32 superseded and retired.
2. **Discrete PRNG codebook initialization (`discrete.cu:261`)**: Populated initial discrete token embeddings using an xorshift pseudo-random state over seeds and blocks. While passing the 11/17 CUDA construction controls, it provided zero arithmetic duality or prime-route structure.
3. **Prior pointer and MLP addressed memory attempts**: Earlier attempts evaluated soft attention pointers (#2127, #2161) or addressed memory replacing MLPs. None evaluated the unified prime-rank indexing with complex-plane zeta-phase quantization into the shared discrete vector writer.

## What success and failure look like

- **Success**:
  - Deterministic generation of the discrete token codebook from prime-rank zeta phases matches analytic witnesses and passes 100% of finite CUDA construction controls.
  - Emitted PTX passes `admit_ptx` with zero floating-point instructions.
  - Prime-zeta codebook exhibits distinct, structured cross-token Hamming distances and interference patterns that outperform arbitrary PRNG initialization on downstream discrete move selection.
- **Failure**:
  - Quantization into $E_8$ anchor roots suffers severe aliasing (too many primes collapsing onto identical anchor roots), destroying token distinction.
  - Or the tangent null control (shifting phases along the critical line) matches or outperforms the prime-zeta assignment, indicating the arithmetic structure is decorative rather than load-bearing.

## What a fair test needs (Pre-registration bar)

- **Pre-registration**: Pre-registered under D22 before any model compute; requires `TEST FITNESS: FIT` review on milestone #2029 (M1) or #2032 (M4).
- **Finite Qualification Bar**:
  1. 100% bit-exact reproduction of quantized token codebooks across multiple seeds.
  2. Zero float instructions in emitted PTX verified via `admit_ptx`.
  3. Strict non-aliasing: all vocabulary tokens retain distinct vector representations across multi-block allocations.
  4. Passing all 17 discrete construction and flock controls with the new codebook.
- **Attribution Controls (D22 §3)**:
  1. **Tangent null**: Phase shifted along the manifold ($\gamma_k \to \gamma_k + \delta$).
  2. **Normal null**: Primes randomly permuted or phases moved off the critical line.
  Attribution requires the prime-zeta routing to outperform the normal null and separate cleanly from the tangent null.

## Open design questions

1. **Multi-Block Channel Allocation**: For models with $B$ blocks ($8B$ dimensions), should block $b$ sample consecutive zeta zeros $8b \dots 8b+7$, or apply distinct group rotations to the primary 8 zeros?
2. **Anchor Orbit Capacity**: The current anchor orbit has 16 roots. For large vocabularies ($V \ge 4096$), how to allocate multiple 8D blocks to prevent token representation collision while maintaining discrete group symmetry.
3. **Special Token Handling**: Should structural tokens (`<|bos|>`, `<|eos|>`, `<|pad|>`) use designated small primes ($p=2, 3$) or reserved non-prime geometric fixed points?
