# Threat Model — R⁴ Holographic Graph Compiler

Phase 0 deliverable (issue #32; plan §9.22). Sources: PDF §14 (safety, bias, adversarial
collisions) and §30 (risks). Scope: the deployed runtime, its artifacts, and the patch lifecycle.
The compiler and certifier are offline and trusted; the runtime consumes artifacts that may come
from untrusted storage or networks.

## 1. Security posture

- Semantic route codes are **never** an authorization, authentication, or security identity.
  Locality is intentional, so collisions exist by design; any security decision keyed on a route
  code is out of scope and a misuse we explicitly reject.
- Content CIDs (κ) provide identity and integrity of bytes, not semantics. Verification of the
  artifact CID is a precondition to constructing a `GraphView`.
- The runtime trusts only validated bytes: two-stage validation (R4G1.md §6) precedes any read.

## 2. Adversary capabilities considered

1. Crafts **input token streams** to a deployed runtime (most exposed surface).
2. Supplies or tampers with **artifact bytes** (downloaded graphs, patch layers).
3. Submits **poisoned evidence** to patch/epoch pipelines (overlap poisoning).
4. Knows the graph layout and calibration (no security by obscurity).

## 3. Threats and required defenses

| Threat | Mechanism | Defense (where enforced) |
|---|---|---|
| Crafted region activation | inputs steered to hot/privileged regions to skew predictions | calibrated radii + support/margin signals; explicit `ResolutionStatus`; no auth use of routes (runtime, Phase 5) |
| Overlap poisoning | malicious patch adds overlap nodes/interactions to smuggle behavior | patch certificates, provenance roots, held-out gain requirement for overlap nodes, bounded patch layers (Phase 9) |
| Frontier explosion | inputs engineered to maximize active regions / candidates per step | fixed-capacity frontier and candidate buffers; manifest-declared bounds A/C/K/D; deterministic replacement/rejection (Theorem 9) |
| Fallback denial-of-service | forcing constant widening / exhaustive fallback / EXCT hits | fallback is itself bounded by manifest constants; per-status policy deterministic; rate reported in certificate (Gate I) |
| Integer saturation / overflow | crafted scores or counts hitting i32/u16 extremes | declared wrapping/checked/saturating semantics (A4); fuzzed integer boundaries; checked arithmetic in validator |
| Malformed artifact bytes | bad offsets, overlapping ranges, invalid bytecode, version tricks | two-stage validation, checked offset arithmetic, bytecode validation, unknown-mandatory-section rejection, fuzz targets (Phase 1) |
| Collision with privileged concepts | engineered text routed to a region whose emissions are unsafe | per-slice certification, undesirable-behavior amplification metrics, emission audit (Phase 3/9); product-level filtering stays a separate layer |
| Bias amplification / rare-group erasure | compression strengthens source-model failures or drops rare behavior | amplification/erasure metrics, worst-group slices in certificates, provenance at membership-edge granularity enabling deletion (PDF §14) |
| Patch-chain abuse | forks, unbounded layers, replayed old layers | immutable ordered layers, newest-valid precedence, bounded lookup, chain validation, canonical compaction (Theorem 11) |
| Witness forgery | fabricated prediction witnesses | independent replay verifier recomputes every step from validated bytes; CID inclusion checks (Theorem 6) |
| Release-asset tampering (supply chain) | swapped or modified bundle assets on the distribution channel; archives smuggling unattested files or symlinks | `r4 install-release` (#741, since v0.1) verifies **every** component's blake3 digest against the release's attested `release-bundle.json` before anything is installed; archives containing any unattested entry or any symlink are refused outright; existing installs are never overwritten; the fetch is explicit-only — no serving path or first request ever downloads (`docs/RELEASE_PIPELINE.md`). The manifest binds the bundle to the release tag, which binds the code SHA and contract version |

## 4. Explicit non-defenses (documented limits)

- We do not defend the **teacher's** training data or behavior; inherited bias is measured
  (amplification/erasure metrics), not eliminated.
- We do not make membership private: prototypes/masks are shipped in the clear; the artifact is
  not a confidential representation.
- Denial of service at the API/serving layer (r4 server) is out of scope; the runtime kernel only
  guarantees bounded work per token (Theorem 4).

## 5. Test obligations (suites land in Phases 5/6/9, Gate I)

- Adversarial collision suites: crafted inputs targeting hot regions; assert bounds hold and
  status escalates deterministically.
- Frontier/candidate exhaustion fuzzing: random + adversarial token streams at manifest limits.
- Fallback-rate stress: measure and certify fallback rates under attack vs. benign traffic.
- Integer boundary fuzzing: scores near saturation, counts near width limits.
- Patch-layer abuse: forked chains, oversized layer counts, tombstone replay.
