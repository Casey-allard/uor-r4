
# Geometric Routing for LLMs — Architecture Summary for Future Work

Purpose: Provide a concise architectural interpretation of the current hyperbolic routing research and clarify how learning and geometry should interact. This document is intended for Codex ingestion to guide future experimentation.

---

# Core Idea

The routing system should **combine fixed geometric structure with learned embeddings**.

Instead of allowing neural networks to discover routing from scratch, we provide a **mathematical routing manifold** and allow the model to learn **how information maps into that space**.

This creates a hybrid system:

- Geometry defines **routing rules**
- Learning defines **semantic positioning within the space**

---

# Architectural Separation

The current research indicates three distinct components must be separated:

1. **Routing Manifold**
2. **Transport / State Field**
3. **Retrieval / Key Field**

Each should be treated as an independent mathematical object.

---

# Routing Manifold

The routing space is hypothesized to be a hyperbolic manifold.

Candidate structure:

    H⁴

With polar decomposition:

    H⁴ ≈ radial coordinate × S³

Angular structure is likely Hopf‑based with coordinates:

    (χ, θ1, θ2)

Where:

- χ controls latitude on S³
- θ1 and θ2 represent phase rotations

Routing address:

    Route(x) = (shell, sector)

---

# Hyperbolic Embedding (Learned)

The model must learn an embedding function:

    f(x) → H⁴

This mapping places semantic representations into the routing manifold.

Important:

The manifold structure is fixed, but the embedding is learned.

This allows semantic relationships to organize naturally within the hyperbolic geometry.

---

# Shell and Sector Structure

Routing partitions the manifold into discrete regions.

Example:

Shells determined by radial coordinate:

    r

Sectors determined by angular coordinates:

    (χ, θ1, θ2)

Partitioning should follow hyperbolic measure growth:

    volume ∝ sinh³(r)

This ensures routing capacity grows consistently with space volume.

---

# What Geometry Should Control

The following properties should remain fixed:

- manifold curvature
- shell growth law
- angular topology
- sector adjacency
- routing partition measure

These are structural invariants.

---

# What the LLM Should Learn

The model must still learn:

1. Embedding into the routing manifold

       f(x) → H⁴

2. Routing preference among candidate sectors

3. Expert/module specialization

4. State dynamics of the coupled field

Learning occurs **within the geometric constraints**.

---

# Second Hyperbolic Factor

Current evidence suggests a second hyperbolic manifold may act as a **coupled field** rather than additional routing coordinates.

Structure:

    (R, F) ∈ H⁴ × H⁴

Where:

R = routing manifold  
F = addressing or state field

Possible roles of F:

- retrieval key generator
- memory pressure signal
- route correction field
- transport dynamics

---

# Routing vs Retrieval

Routing determines where computation occurs.

Retrieval determines how information is accessed.

Combined addressing structure:

    K(x) = K_base(x) ⊕ K_field(x)

Where:

K_base = geometric routing address  
K_field = field‑derived key

---

# Expected Benefits

If successful this architecture could provide:

- deterministic routing
- sparse expert activation
- improved hardware locality
- lower compute requirements
- stable routing capacity scaling

Potential compute reduction:

    10x – 100x (depending on sparsity)

---

# Key Research Goals

Future work should focus on:

1. Implementing **learned hyperbolic embeddings**
2. Deriving **measure‑consistent shell laws**
3. Implementing **Hopf‑structured angular partitions**
4. Testing **H⁴ × H⁴ field coupling**
5. Measuring routing invariants

Important diagnostics:

- shell occupancy vs theoretical measure
- sector distribution
- routing entropy
- neighbor preservation
- retrieval efficiency

---

# Guiding Principle

Geometry defines the **structure of the world**.

Learning defines **how representations live inside that world**.

If geometry is allowed to change arbitrarily, routing degenerates into learned gating.

If learning is restricted too heavily, routing becomes rigid and ineffective.

The architecture must balance these forces.

