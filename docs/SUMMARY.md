# Summary

[UOR-R4 documentation](README.md)

# Start here

- [What UOR-R4 is](start/what-is-uor-r4.md)
- [Transformer parts and their geometric replacements](start/transformer-vs-geometry.md)
- [Where UOR-R4 is today](start/where-we-are.md)
- [Where UOR-R4 is going](start/where-were-going.md)
- [Frequently asked questions](start/faq.md)
- [Glossary](start/glossary.md)
- [How the repository is laid out](start/repository-layout.md)

# Products

- [The three products](products/README.md)
  - [The language model](products/language-model.md)
  - [The web client](products/web-client.md)
  - [The desktop client (specification)](products/desktop-client.md)
  - [Releases and artifacts](products/releases-and-artifacts.md)

# The model

- [The model, end to end](model/architecture.md)
  - [How the model learns](model/training.md)
  - [Data and tokenizer](model/data-and-tokenizer.md)
  - [Serving and export](model/serving-and-export.md)
  - [GPUs and CUDA](model/gpu-and-cuda.md)

# Geometry

- [The geometry of UOR-R4](geometry/README.md)
  - [Quaternions and rotation](geometry/quaternions-and-rotation.md)
  - [E8, H4 and the 600-cell](geometry/e8-h4-and-the-600-cell.md)
  - [Golden integers](geometry/golden-integers.md)
  - [The Hopf map and the kept fibre](geometry/hopf-observation.md)
  - [Primes and zeta phases](geometry/primes-and-zeta.md)
  - [VSA and Hamming distance](geometry/vsa-and-hamming.md)
  - [SpiralCore](geometry/spiralcore.md)

# Mechanisms

- [The mechanism board](mechanisms/README.md)
  - [Routed read](mechanisms/route-read.md)
  - [Flock read](mechanisms/flock-read.md)
  - [Relation read](mechanisms/relation-read.md)
  - [Admission](mechanisms/admission.md)
  - [Prime-zeta codebook](mechanisms/prime-zeta-codebook.md)
  - [Address-structured cells](mechanisms/address-cells.md)
  - [Vector-symbolic binding](mechanisms/vsa.md)
  - [Hopf observation with a kept fibre](mechanisms/hopf-observation.md)
  - [Tokenizer](mechanisms/tokenizer.md)
  - [Discrete geometric learning](mechanisms/discrete-learning.md)
  - [Rank-margin hinge objective](mechanisms/rank-margin.md)
  - [Protected legal construction](mechanisms/legal-construction.md)
  - [Live geometric carrier credit](mechanisms/carrier-credit.md)
  - [Exact addressed memory](mechanisms/exact-memory.md)
  - [Exact byte object store](mechanisms/object-store.md)
  - [Read binding](mechanisms/read-binding.md)
  - [Pointer copy with an exact identity key](mechanisms/pointer-key.md)
  - [E8 orbit memory](mechanisms/e8-orbit-memory.md)
  - [Ternary and XOR/Hamming arithmetic](mechanisms/ternary-xor.md)
  - [Spin-native weights](mechanisms/spin-weights.md)
  - [Hamilton rotation gate](mechanisms/hamilton-gate.md)
  - [Geometric recurrence gates](mechanisms/recurrence-gates.md)
  - [SpiralCore transport with ternary tree binders](mechanisms/spiralcore-fbs.md)
  - [Geometric stack](mechanisms/geometric-stack.md)
  - [R4G1 compiled lookup graph](mechanisms/r4g1-graph.md)
  - [D11 integer table serving](mechanisms/lut-serving.md)
  - [Native geometric engine](mechanisms/native-geometric.md)

# Research

- [How research is done now](research/README.md)
  - [What the project knows](research/findings.md)
  - [What is still open](research/open-questions.md)
  - [How the project got here](research/history.md)
  - [The research lines](research/lines/README.md)
    - [Discrete attention for the language base (M1)](research/lines/m1-discrete-attention.md)
    - [The prime-zeta codebook (M1)](research/lines/m1-prime-zeta.md)
    - [The float route to rung 0 (M1)](research/lines/m1-float-rung0.md)
    - [Grounded replies from exact memory (M2)](research/lines/m2-exact-memory.md)
    - [Durable memory and the object store (M3)](research/lines/m3-durable-memory.md)
    - [Integer serving (M4)](research/lines/m4-serving.md)
    - [Laptop energy and cost (M5)](research/lines/m5-energy.md)
    - [API, browser runtime and Studio (M7)](research/lines/m7-distribution.md)
    - [The geometric stack](research/lines/geometric-stack-pretraining.md)
    - [The September native geometric model](research/lines/native-geometric-sept.md)
    - [The August geometric intelligence programme](research/lines/geometric-intelligence-aug.md)
    - [Compiling a teacher into tables (R4G1)](research/lines/r4g1-transformerless.md)
    - [SpiralCore wire transport](research/lines/spiralcore-wire.md)
    - [Data and tokenizer](research/lines/data-tokenizer.md)
    - [Evaluation panels and graders](research/lines/eval-panels.md)
    - [GPU and pod operations](research/lines/compute-ops.md)
    - [Research tooling and rules](research/lines/harness-governance.md)
  - [Research notebooks](research/notebook/README.md)
    - [Discrete attention for the language base (M1)](research/notebook/m1-discrete-attention.md)
    - [Prime and zeta starting codebook (M1)](research/notebook/m1-prime-zeta.md)
    - [Durable memory (M3)](research/notebook/m3-durable-memory.md)
    - [Integer serving (M4)](research/notebook/m4-serving.md)
    - [Data and tokenizer](research/notebook/data-tokenizer.md)
    - [Research tooling and governance](research/notebook/harness-governance.md)

# Rules

- [The rules in plain language](rules/README.md)
  - [Where each old decision went](rules/decision-map.md)
  - [Change log for the rules](rules/changes.md)
- [The charter](../CHARTER.md)

# Reference

- [Reference documents](reference/README.md)
- [Storage indexes](reference/storage.md)
- [Formal vocabulary and claim classes](formal_vocabulary.md)
- [Configuration reference](CONFIGURATION.md)
- [Canonical artifact byte equality](reproducibility.md)
- [Minimal terminal client and local API](minimal_client.md)
- [Release pipeline](RELEASE_PIPELINE.md)

# Side projects

- [Side projects](../projects/README.md)
  - [AI-Research: the original root research workspace](../projects/ai-research/README.md)
  - [MSA: Modular Structural Arithmetic](../projects/msa/README.md)
  - [Prime Analysis: the Prime Phase and Chebyshev psi spectral program](../projects/prime-analysis/README.md)
  - [Research archives: the Gemini-era sandbox](../projects/research-archives/README.md)
  - [Riemann-Lean: a Lean 4 scaffold for a prime-number proof chain](../projects/riemann-lean/README.md)
  - [SpiralCore: networking protocols and compression](../projects/spiralcore/README.md)
  - [WASM-GEMM-GNAF: a Lean proof project about a matrix-multiply kernel](../projects/wasm-gemm-gnaf/README.md)
