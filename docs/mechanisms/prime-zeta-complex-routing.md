# Prime-rank indexing and complex-plane zeta-phase routing

**Milestones:** M1 #2029 and M4 #2032. **Authority:** D32 October 14 addendum (#2398), pre-registration sheet [`docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md`](../harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md) (`TEST FITNESS: FIT`, #2433), owner finite-qualification release ([#2029 comment 6170312990](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6170312990)), delivered standalone constructor and CUDA construction qualification ([#2473](https://github.com/UOR-Foundation/uor-r4/pull/2473), [#2477](https://github.com/UOR-Foundation/uor-r4/pull/2477), [`docs/labs/prime-zeta-construction-2026-10-14/README.md`](../labs/prime-zeta-construction-2026-10-14/README.md)).

## Idea and implemented role

Provide a standalone, host-side, one-time, offline token codebook constructor based on deterministic prime-rank indexing and complex-plane Riemann zeta-zero phase routing that populates `Snapshot.table` (`Device::from_snapshot`) without modifying the frozen D5 runtime initializer (`discrete_init` in `crates/uor-r4-discrete/src/discrete.cu:257-267`, which retains its xorshift PRNG initialization).

Every token $v$ with vocabulary ID $\text{id}(v) \in \{0, \dots, V-1\}$ is mapped bijectively to a unique prime atom:
$$r(v) = p_{\text{id}(v)+1}$$
where $p_1 = 2, p_2 = 3, p_3 = 5, \dots, p_{4096} = 38873$.

For each prime atom $p$, its phase along the $k$-th non-trivial Riemann zeta zero $\rho_k = \frac{1}{2} + i\gamma_k$ is governed by the prime log-phase:
$$\theta_k(p) = \gamma_k \ln p \pmod{2\pi}$$
where $\gamma_k$ are the deterministic imaginary parts from Odlyzko's computed zeros (`crates/uor-r4-core/src/zeta_zeros.rs:4-517`).

In the complex plane, each channel defines a unit-modulus phasor on the critical line ($\sigma = \frac{1}{2}$):
$$z_k(p) = e^{i\theta_k(p)} = \cos(\gamma_k \ln p) + i \sin(\gamma_k \ln p) \in \mathbb{C}$$

For each 8D block $b \in \{0, \dots, B-1\}$, four consecutive zeta channels $k = 4b + c$ ($c \in \{0,1,2,3\}$) assemble into $\mathbf{z}_b(p) \in \mathbb{C}^4$, split into two $\mathbb{C}^2 \to \mathbb{R}^4$ spinor pairs and embedded into the four orthogonal 2D root planes of the 16-root $E_8$ orbit (`geometry::ROOTS: [[i8; 8]; 16]`).

### Exact Integer Quantization Boundary (D32 Compliant)

Per Decision D32 October 14 Addendum (#2398), floating-point arithmetic (`f64` `ln`, `sin`, `cos`, `sqrt`, `powf`) is strictly confined to **one-time, host-side, offline codebook generation** prior to the integer quantization boundary. Immediately at the output of the realification map, the representation is quantized into discrete integer types:
1. **`PhaseQ29` (`prime_zeta.rs:48-66`)**: $\text{raw}_{Q29} = \text{round}(\text{wrap}_{[-\pi,\pi)}(\theta_k) \cdot 2^{29}) \in [-2^{29}\pi, 2^{29}\pi) \subset \mathbb{Z}$, providing 29 fractional bits of angular precision in `i32`.
2. **`UnitS3Q30` (`prime_zeta.rs:68-99`)**: Normalized 4D points $\mathbf{x} / \|\mathbf{x}\|$ scaled by $2^{30}$ into `[i32; 4]` on the 3-sphere $S^3 \subset \mathbb{R}^4$.
3. **$E_8$ Orbit Alignment & Non-Aliasing Assignment (`prime_zeta.rs:252-470`)**: Pairs of `UnitS3Q30` spinors embed into the four orthogonal 2D root planes `(0,1), (2,4), (3,7), (5,6)` (`embed_s3_pair_into_e8_planes`), score against the 16 orbit roots `geometry::ROOTS: [[i8; 8]; 16]` by exact `i64` inner product (`exact_root_alignment`), balance writer product cells `(root_0, root_{B-1})` under explicit quota `cell_quota = vocab.div_ceil(256)`, and enforce multi-block vector uniqueness (`used_vectors: BTreeSet<Vec<u8>>`) when $16^B \ge V$.
4. **`Snapshot.table` Codebook (`prime_zeta.rs:464-470`, `prime-zeta-qualify.rs:306-320`)**: Quantized root coordinates store as `Vec<i32>` of shape `[vocabulary * blocks * 8]` and load into the integer CUDA module via `Snapshot.table` and `Device::from_snapshot`.

Downstream of this offline host boundary, **100% of model compute and learning remains discrete on CUDA**:
- Bivector actions in $\mathrm{Cl}(0,6)$ and BFS transport on the 64-state signed axis group (`discrete.cu:18-77`).
- Ordered ternary FBS binder hierarchies (`BinderPlan` in `metadata.rs:14-178`).
- Murmuration / Flock reads based on frame L1, coordinate Hamming distance, recency, and node identity (`discrete.cu:97-201`).
- Ternary Flock level gates $\{-1, 0, +1\}$ (`discrete.cu:88-94, 159-195`).
- Shared integer vector writer ($8\cdot \text{token\_hinge} + \text{width}\cdot \text{address\_hinge}$, `discrete.cu:229-253`).
- Counterfactual proposal selection without floating-point forward passes, gradients, or optimizers (`discrete.cu:281-345`).

## Exact source scope and architectural boundary (D22)

- `crates/uor-r4-core/src/zeta_zeros.rs:4-517`: `ZETA_ZEROS: [f64; 512]` provides the exact Odlyzko ordinates $\gamma_k$ (included at `crates/uor-r4-discrete/src/prime_zeta.rs:28-32` without adding `uor-r4-core` as a runtime dependency of `uor-r4-discrete`).
- `crates/uor-r4-core/src/prime_route_attention.rs:338-377`, `485-570`, `909-919`: core reference definitions of `PhaseQ29`, `UnitS3Q30`, and `zeta_phase_delta`.
- `crates/uor-r4-discrete/src/prime_zeta.rs:48-130`: standalone host-side `PhaseQ29`, `UnitS3Q30`, and `first_n_primes` ($V \le 8192$).
- `crates/uor-r4-discrete/src/prime_zeta.rs:135-153` & `226-244`: `CodebookMode` (`PrimeZeta`, `TangentNull { delta }`, `NormalNull { seed }`) and deterministic xorshift64* prime permutation `permute_primes_for_normal_null`.
- `crates/uor-r4-discrete/src/prime_zeta.rs:252-508`: `embed_s3_pair_into_e8_planes`, `exact_root_alignment`, and `generate_codebook` implementing the one-time host `f64` phase/spinor evaluation, exact `i64` root alignment, writer-cell quota balancing, and multi-block non-aliasing assignment.
- `crates/uor-r4-discrete/src/prime_zeta.rs:532-666`: `compare_codebooks` (structural Hamming, L1, and writer-cell separation) and `verify_zero_float_ptx`.
- `crates/uor-r4-discrete/src/bin/prime-zeta-qualify.rs:63-284` & `306-690`: standalone `prime-zeta-qualify` binary (`host-verify` and CUDA `qualify` subcommands, `build_snapshot`, and Prime-Zeta controls `C18..C23`).
- `crates/uor-r4-discrete/src/discrete.cu:257-267` & `crates/uor-r4-discrete/src/metadata.rs:1-211`: `discrete_init` in `discrete.cu:257-267` **remains the untouched xorshift PRNG default initializer** (`state = seed ^ block ^ 0x9e3779b9u`), and `metadata.rs:1-211` contains only `BinderPlan` forest metadata. All 10 files hashed by `device::source_sha256()` and `training::driver_source_sha256()` remain untouched (`prime_zeta.rs:809-828`).
- `crates/uor-r4-discrete/build.rs:25-83`: `admit_ptx` enforces zero floating-point or tensor-core instructions in the compiled integer CUDA module.

## What has been tried, honestly

1. **Float word vector baseline (`get_word_vector` in `crates/uor-r4-core/src/lib.rs:386-402`)**: Evaluated continuous $\sin(\gamma_k \ln p)$ vectors normalized to float length 0.1. This operated in continuous floating-point space, which D32 superseded and retired.
2. **Default discrete xorshift PRNG codebook initialization (`discrete.cu:257-267`)**: Populates initial discrete token embeddings using `state = seed ^ block ^ 0x9e3779b9u` over seeds and blocks. Because `block = i / 8` depends only on the global block index Modulo 16 via `state & 15` across periodic block offsets, it yields only 16 distinct full-vector patterns across 4,096 tokens at epoch 0 (`docs/labs/a2-pilot-diagnosis-2026-10-14/README.md`). It remains the active initializer for the frozen D5 completion lineage.
3. **Standalone Prime-Zeta construction qualification (#2473 / #2477, [`docs/labs/prime-zeta-construction-2026-10-14/README.md`](../labs/prime-zeta-construction-2026-10-14/README.md))**:
   - **Host codebook & cross-architecture determinism (`PASS`)**: `host-codebook-qualification.json` (`sha256: 41e828872cb9ef7b3b9cc921481cd916b9258153e9d381210258eb2ec0d3a52c`) and `v4096-witnesses.json` (`sha256: e7f4c63667e14873b6d03d370d231c353724618bb5d17d5a61c444fc789f1d0d`) are 100% byte-identical across `aarch64-apple-darwin` and `x86_64-unknown-linux-gnu`. At `V4096 B4`, `PrimeZeta` produces `4096/4096` unique `Snapshot.table` vectors and `256/256` occupied writer cells (`table_sha256: e9e03bcd95b78632a7f18d1b27c07d37f0ba82c52eee692c37da011eadac8804`), separating from `TangentNull` ($\delta = 1.0$, `4090/4096` differing tokens) and `NormalNull` (`seeds 0, 1, 2`, `4096/4096` differing tokens).
   - **Bounded CUDA construction controls (`PASS`)**: Executed on `vast-55945260` (`1×RTX 5090`, executable `953f8f08…` from `912a8a069`, `prime-zeta-qualification.json` `sha256: 082ed664c60000aa842d479f579f4a0a431eda821970cfd243a2df99965e5951`). Passed `admit_ptx` (`11` entries, `0` float instructions), `17/17` base controls (`11` Route + `6` Flock), and `6/6` Prime-Zeta controls (`C18..C23`) with `applied_updates: 0`, `corpus_rows_consumed: 0`, and `training: "HELD"`.
   - **Honest scope disclosures**:
     - Zero full-vector collisions (`4096/4096` at `V4096 B4`) and uniform 256-cell occupancy (quota 16) are enforced by the explicit post-scoring cell-quota and uniqueness pass (`prime_zeta.rs:361-470`), which is shared with `TangentNull` and `NormalNull`; collision freedom is therefore guaranteed by the quota/uniqueness assignment rather than established as an unconstrained effect of prime phases alone.
     - In `CodebookMode::PrimeZeta`, the `seed` parameter (`0, 1, 2`) does not alter the generated codebook (`b4_seed0.table == b4_seed1.table == b4_seed2.table`), verifying deterministic repeatability of the primary map rather than independent seeded initializations (whereas `CodebookMode::NormalNull { seed }` varies the prime permutation per seed).
     - The remote job wrapper's trailing `sha256sum *` shell command included two subdirectories (`flock-controls`, `route-controls`) and exited `1` after `prime-zeta-qualify qualify` itself exited `0` and wrote all control JSONs; `receipts/SHA256SUMS` records the six verified file digests.
     - **What this test could and could not judge**: This finite construction qualification judged offline quantization, PTX integer purity, snapshot loading, non-aliasing, and counterfactual non-vacuity at epoch 0. Because `applied_updates = 0` and `corpus_rows_consumed = 0`, it **cannot** judge whether `PrimeZeta` improves trained language quality, writer admission, or rank margins over `TangentNull`, `NormalNull`, or xorshift initialization.

4. **Wired as an opt-in initializer for new lineages (D39 item 3; #2692, #2694, 2026-10-18):**
   - **Wiring.** `DriverConfig.codebook_initialization: pinned_table_v1` loads an exact integer table. `prime-zeta-codebook` exports this constructor's table, or a matched null, as `uor-r4.codebook-table/1`. The pinned table is:
     - bound into the lineage by its file SHA;
     - checked on the host for schema, spec, length, word digest and the declared distinct-vector count;
     - admitted as roots on the GPU by `Device::from_snapshot`;
     - enforced at checkpoint 0 (G-Init).

     The seeded default and the frozen lineage are byte-unchanged.
   - **Coverage census** (`coverage-census`, #2678, host grouping only), on the 14,287-row DEV panel. The xorshift checkpoint 0 has **16** distinct vectors, a DEV alias ceiling of 1,986 rows, and 4 of 674 targets that can win a tie. PrimeZeta V4096 B4 has **4,096** vectors, a ceiling of 14,287, all 674 targets, and zero tie mass.
   - **The first downstream sheet is a draft:** `docs/harness/prep/m1-attention-injective-init-DRAFT-2026-10-18.md`. It asks the attention question (Relation vs FrozenZero) on this initialization. It is **not** the PrimeZeta-vs-null attribution below, which stays a separate sheet.
   - **Caveat on what injectivity proves.** Because the quota/uniqueness pass also runs on both nulls (see the disclosures above), injectivity is a property of the construction, not of the prime phases. An attention result on this initialization is therefore evidence about distinguishable keys, not about prime-zeta geometry.

## What success and failure look like

- **Finite construction stage (`PASS`, #2473/#2477)**:
  - Host-side offline codebook generation is bit-exact across architectures, emits zero float instructions in compiled PTX (`admit_ptx`), preserves frozen D5 runtime hashes untouched, and passes all `17 + 6` finite CUDA construction controls with `0` applied updates.
- **Downstream learned attribution stage (`NOT_RUN`, held)**:
  - **Success**: Under a separately pre-registered, `TEST FITNESS: FIT`-approved, owner-released trained-in comparison at matched coordinate budget and writer lineage, `PrimeZeta` outperforms `NormalNull` and separates from `TangentNull` on the milestone's registered evaluation metrics and fresh transfer.
  - **Failure**: Trained-in `PrimeZeta` fails to separate from `NormalNull` or `TangentNull` beyond panel noise, indicating that prime-log zeta phases provide no learned advantage beyond quota-balanced multi-block non-aliasing.

## What a fair test needs (Downstream learned attribution bar)

- **Pre-registration & Release**: Requires its own downstream pre-registration sheet, `TEST FITNESS: FIT` review on #2029 / #2032, and explicit owner training release (the finite construction qualification release [#2029 comment 6170312990](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6170312990) authorized zero training and zero corpus dose).
- **Matched Attribution Controls (D22 §3)**:
  1. **Primary (`CodebookMode::PrimeZeta`)**: $r(v) = p_{v+1}$ with critical-line Odlyzko ordinates $\gamma_k$.
  2. **Tangent null (`CodebookMode::TangentNull`)**: $\gamma_k \to \gamma_k + \delta$ ($\delta = 1.0$) with identical prime ranks and identical quota/uniqueness assignment.
  3. **Normal null (`CodebookMode::NormalNull`)**: Seeded prime-rank permutation $\pi_s(p)$ + off-critical-line perturbation with identical quota/uniqueness assignment.
  4. **Quota-only / PRNG control**: Separates the effect of 4,096-unique quota-balanced initialization from prime-zeta phase geometry.
- **Power & Observables**: $\ge 2$ seeds (via distinct corpus-order or read-init seeds where applicable) or $\ge 200$ fresh evaluation rows, trained in through the shared discrete vector writer, reporting target-cell admission (`B3`), conditional top-1/top-8 ranks, separated token/address hinge loss, and fresh transfer.

## Open design questions

1. **Separating Quota Balance from Raw Phase Geometry**: Because `generate_codebook` applies `cell_quota = vocab.div_ceil(256)` and `used_vectors` deduplication to both `PrimeZeta` and the nulls, how much of any initial admission change comes from uniform 256-cell coverage versus the specific prime-zeta inner-product ranking `exact_root_alignment`?
2. **Seed Diversity for Primary `PrimeZeta` Runs**: Since `CodebookMode::PrimeZeta` is a deterministic function of token ID, multi-seed training comparisons must introduce seed variation through a declared orthogonal axis (e.g., cyclic channel offset, signed-axis frame gauge, or batch order) if initializer-seed variance is required.
3. **Special Token Handling**: Should structural tokens (`<|bos|>`, `<|eos|>`, `<|pad|>`) use designated small primes ($p=2, 3$) or reserved geometric fixed points?

