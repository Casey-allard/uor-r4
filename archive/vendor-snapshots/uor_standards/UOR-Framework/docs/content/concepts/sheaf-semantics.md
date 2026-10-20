# Sheaf Semantics

## Definition

**Sheaf semantics** interprets the resolution pipeline through the lens of
sheaf cohomology. The constraint topology — where open sets correspond to
compatible subsets of constraints — carries a natural
{@class https://uor.foundation/cohomology/Sheaf} of resolution data. This
viewpoint unifies local constraint satisfaction with global resolution.

## Local vs Global Consistency

The distinction between local and global consistency is captured by the
sheaf structure:

- **Stalks**: A {@class https://uor.foundation/cohomology/Stalk} at a single
  constraint holds the local resolution data — the sites pinned by that
  constraint alone. Local consistency means each stalk is individually
  satisfiable.
- **Global sections**: A {@class https://uor.foundation/cohomology/Section}
  over the entire constraint space represents a globally consistent
  resolution — an assignment that simultaneously satisfies all constraints.

Local consistency does not imply global consistency. The gap between the two
is precisely what cohomology measures.

## Gluing Obstructions

When local sections over overlapping open sets cannot be assembled into a
global section, a {@class https://uor.foundation/cohomology/GluingObstruction}
arises. These obstructions live in the first cohomology group H^1:

- **H^0** (global sections): captures the space of fully resolved states.
  A nonzero H^0 means at least one global resolution exists.
- **H^1** (gluing obstructions): classifies the ways local resolutions fail
  to glue. A nontrivial H^1 signals that the constraint set has intrinsic
  conflicts visible only at the global level.

## Connection to the Resolution Pipeline

The sheaf-cohomological perspective connects to the resolution pipeline
through stages psi_5 and psi_6 of the structural reasoning pipeline:

1. **psi_5** dualizes the chain complex into a cochain complex, lifting
   boundary data to coboundary data.
2. **psi_6** computes cohomology from the cochain complex, producing
   obstruction classes.

The iterative resolution loop (from `resolver/`) can then be understood as
an attempt to kill cohomology classes: each refinement step reduces H^1
until all obstructions vanish and a global section (complete resolution)
exists.

## Practical Interpretation

| Cohomology group | Resolution meaning |
|------------------|--------------------|
| H^0 = 0 | No global resolution exists |
| H^0 nontrivial | At least one global resolution exists |
| H^1 = 0 | Local solutions always glue to global solutions |
| H^1 nontrivial | Gluing obstructions present; iterative refinement needed |

## Gluing Obstruction Example

Consider three constraints C_1, C_2, C_3 on R_4 where:

- C_1 and C_2 overlap on site 0 and agree locally (both pin site 0 to the
  same value).
- C_2 and C_3 overlap on site 1 and agree locally.
- C_1 and C_3 do not overlap directly, but the transitive path
  C_1 → C_2 → C_3 induces a consistency requirement.

If the value forced at site 0 by C_1 propagates through C_2 to site 1 in a
way that contradicts C_3's requirement, a
{@class https://uor.foundation/cohomology/GluingObstruction} arises. The
obstruction class lives in H^1 of the constraint
{@class https://uor.foundation/cohomology/Sheaf} and measures exactly this
cyclic inconsistency.

When H^1 = 0, every compatible family of local
{@class https://uor.foundation/cohomology/Section} objects glues into a global
section — meaning local constraint satisfaction guarantees global resolution.
When H^1 ≠ 0, the resolver must apply iterative refinement to kill the
obstruction classes before a global solution can be assembled.

See [Cohomology](cohomology.html) for the cochain complex construction and
[Analytical Completeness](analytical-completeness.html) for the dual homological
perspective.
