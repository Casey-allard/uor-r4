# R4 Prime Router — Evolving Hypersphere Brain World Model

An advanced, interactive world model and visualization dashboard leveraging **Hopf $S^3$ geometry** and **GCD prime-seeded coordinates** for zero-weight geometric sequence generation, stateful context drift tracking, and semantic projection mapping.

---

<img width="1432" height="680" alt="image1" src="https://github.com/user-attachments/assets/a4805d7e-6a16-42eb-96ac-5b8f004f2f15" />

## Features

* **Dual-Engine Text Generation**:
  1. **Zero-Weight World Model (Pure Geometric)**: Driven entirely by grammatical transition matrices and unit-sphere geodesic evolution, bypassing traditional transformer neural weights.
  2. **Transformer Voice (Ollama Gemma)**: Integrates with local Ollama service to run advanced reasoning LLMs (`gemma4:e2b`) grounded on geometric manifolds.
* **Hopf $S^3$ Geodesic Evolution**: Evolve state vectors along unit-sphere geodesics on user prompts:
  $$H_{new} = \gamma \cdot H_{state} + (1 - \gamma) \cdot S_{query}$$
  $$H_{state} = rac{H_{new}}{\|H_{new}\|}$$
* **Holographic Metrics**: Real-time evaluation of curvature ($	heta_d$), quantum stratum, carry cascade length, winding numbers ($\omega$), commutator curvature, and monodromy.
* **Butter-Smooth Visualization**:
  * **Euclidean Projection**: Flat 2D map of corpus points with glowing resonances and active query trajectories.
  * **Riemannian Hopf Canvas**: 3D stereographic representation of the 4D hypersphere orbital zeta rings ($\gamma_1..\gamma_{16}$), drag to rotate.
* **Auto-Bootstrap System**: Automatic dependency installations, macOS Ollama app startup, and model preloading directly inside the main script.
* **UOR Attestation Layer**:
  * Accepts UOR addresses or plain text identities for routing/QIMC mapping.
  * Emits deterministic UOR addresses with witness metadata (label, fingerprint, verification result) and multihash address variants.
  * Includes `/api/uor/verify` for independent address re-verification against canonical payloads.
  * Includes `/api/uor/capabilities` and `/api/uor/attest` to expose supported hash algorithms and generate attestations for arbitrary payloads.
* **Identity-Scoped Runtime Core (UOR-enabled)**:
  * Maintains per-identity hypersphere brain states so sessions do not overwrite each other.
  * Stores and retrieves identity-scoped conversation corpus shards while preserving shared baseline knowledge.
  * Exposes scoped map points (`scope`) and system stats (`identity_scopes`, `session_states`) for operations visibility.

---

## Directory Structure

```
├── server.py                 # Core HTTP server, metrics API, and model router
├── index.html                # Premium visualization dashboard & chat panel
├── prime_router_package.py   # Riemann zeta zero seeds, projections & geometry functions
├── glove_loader.py           # Blends GloVe word vector spaces with geometry coordinates
├── glove_cache.npz           # Pre-loaded GloVe embeddings cache
├── manifold_cache.json       # Precompiled database of indexed sentences and word coordinates
├── wiki_corpus.txt           # Base corpus for manifold indexing
├── cli.py                    # Terminal-based query tracer client
├── zeta_data/
│   └── zeta_zeros_100k.txt   # Mathematical constants (Riemann zeta zeros)
└── extra_reading/            # Supplementary document files indexed at startup
```

---

## Getting Started

### Prerequisites
* **Python 3.10+**
* **Ollama app** (optional, required for the Transformer engine)

### Running the Server
You do not need to install dependencies or run boot scripts manually. Simply run the server file:
```bash
python3 server.py
```
On startup, `server.py` will automatically:
1. Verify and auto-install Python dependencies (`numpy`, `psutil`, `opentelemetry`, `uor-addr`).
2. Verify if the Ollama service is running (and launch it automatically on macOS).
3. Verify if the model `gemma4:e2b` is ready (and pull it from the Ollama library if missing).
4. Load the manifold cache. If the cache is missing or corrupt, it automatically regenerates it from `wiki_corpus.txt` within 20 seconds.

Open your browser and navigate to:
```
http://localhost:8000
```

### Running the CLI Client
To trace queries and print step-by-step state trajectory coordinate values directly in your terminal:
```bash
python3 cli.py
```

## Strict V&V

Run the strict verification and validation suite:

```bash
python3 -m unittest -v tests/test_vv_strict.py
```

The suite validates:

* Conceptual model invariants (identity required, deterministic identity mapping, UOR-first attestation, UOR control-plane routing effects).
* Function-level contracts for identity profiling, QIMC seeding, routing structure, state isolation, and attestation determinism.
* HTTP API contracts for `/api/chat`, `/api/uor/capabilities`, `/api/uor/attest`, and `/api/uor/verify`.
* Multi-algorithm UOR test vectors across `sha256`, `sha3-256`, `blake3`, `keccak256`, and `sha512`.
* Real-world routing test vectors listed in `tests/vv_test_vectors.json`.

If any invariant fails, the suite exits non-zero and reports the violated contract.

### Integration + End-to-End Coverage

Run the full local test stack (V&V + integration + end-to-end):

```bash
python3 -m unittest -v \
  tests/test_vv_strict.py \
  tests/test_integration_system.py \
  tests/test_e2e_api_journey.py
```

### CI Workflow

The repository includes a GitHub Actions workflow at `.github/workflows/ci.yml` that:

* Validates the CI workflow structure with `tests/validate_ci_config.py`.
* Runs compile checks for core modules and tests.
* Executes strict V&V, integration, and end-to-end suites.

To run the CI-equivalent checks locally:

```bash
python3 tests/validate_ci_config.py && \
python3 -m py_compile server.py cli.py prime_router_package.py \
  tests/test_vv_strict.py tests/test_integration_system.py tests/test_e2e_api_journey.py && \
python3 -m unittest -v \
  tests/test_vv_strict.py tests/test_integration_system.py tests/test_e2e_api_journey.py
```

<img width="1432" height="680" alt="1779508477472" src="https://github.com/user-attachments/assets/83ea0d72-df3a-43bd-a8e1-dfbe569fb941" />
<img width="1432" height="680" alt="1779509321012" src="https://github.com/user-attachments/assets/86428ca7-3312-4b53-ae21-ef3f7b8d0b43" />
