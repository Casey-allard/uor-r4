import os
import shutil

src_dir = "/Users/adminamn/gemini-dev"
dest_dir = os.path.join(src_dir, "prime-router")

print(f"[*] Creating target directory: {dest_dir}")
os.makedirs(dest_dir, exist_ok=True)

files_to_copy = [
    "server.py",
    "index.html",
    "prime_router_package.py",
    "glove_loader.py",
    "glove_cache.npz",
    "manifold_cache.json",
    "wiki_corpus.txt",
    "cli.py",
]

dirs_to_copy = [
    "zeta_data",
    "extra_reading"
]

# 1. Copy files
for filename in files_to_copy:
    src_path = os.path.join(src_dir, filename)
    dest_path = os.path.join(dest_dir, filename)
    if os.path.exists(src_path):
        print(f"[*] Copying {filename}...")
        shutil.copy2(src_path, dest_path)
    else:
        print(f"[-] Warning: {filename} not found!")

# 2. Copy directories
for dirname in dirs_to_copy:
    src_path = os.path.join(src_dir, dirname)
    dest_path = os.path.join(dest_dir, dirname)
    if os.path.exists(src_path):
        print(f"[*] Copying directory {dirname}...")
        if os.path.exists(dest_path):
            shutil.rmtree(dest_path)
        shutil.copytree(src_path, dest_path)
    else:
        print(f"[-] Warning: Directory {dirname} not found!")

# 3. Create README.md
readme_content = """# R4 Prime Router — Evolving Hypersphere Brain World Model

An advanced, interactive world model and visualization dashboard leveraging **Hopf $S^3$ geometry** and **GCD prime-seeded coordinates** for zero-weight geometric sequence generation, stateful context drift tracking, and semantic projection mapping.

---

## Features

* **Dual-Engine Text Generation**:
  1. **Zero-Weight World Model (Pure Geometric)**: Driven entirely by grammatical transition matrices and unit-sphere geodesic evolution, bypassing traditional transformer neural weights.
  2. **Transformer Voice (Ollama Gemma)**: Integrates with local Ollama service to run advanced reasoning LLMs (`gemma4:e2b`) grounded on geometric manifolds.
* **Hopf $S^3$ Geodesic Evolution**: Evolve state vectors along unit-sphere geodesics on user prompts:
  $$H_{new} = \gamma \cdot H_{state} + (1 - \gamma) \cdot S_{query}$$
  $$H_{state} = \frac{H_{new}}{\|H_{new}\|}$$
* **Holographic Metrics**: Real-time evaluation of curvature ($\theta_d$), quantum stratum, carry cascade length, winding numbers ($\omega$), commutator curvature, and monodromy.
* **Butter-Smooth Visualization**:
  * **Euclidean Projection**: Flat 2D map of corpus points with glowing resonances and active query trajectories.
  * **Riemannian Hopf Canvas**: 3D stereographic representation of the 4D hypersphere orbital zeta rings ($\gamma_1..\gamma_{16}$), drag to rotate.
* **Auto-Bootstrap System**: Automatic dependency installations, macOS Ollama app startup, and model preloading directly inside the main script.

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
1. Verify and auto-install Python dependencies (`numpy`, `psutil`, `opentelemetry`).
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
"""

readme_path = os.path.join(dest_dir, "README.md")
with open(readme_path, "w", encoding="utf-8") as f:
    f.write(readme_content)

print(f"[+] README.md created at {readme_path}")
print("[+] Project isolation completed successfully.")
