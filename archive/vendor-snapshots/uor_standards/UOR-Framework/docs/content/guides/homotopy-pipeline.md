# Homotopy Pipeline

The **homotopy pipeline** extends the [psi pipeline](psi-pipeline.html) with three
additional stages (psi_7 through psi_9) that compute the full homotopy type of the
constraint nerve.

## Extended Pipeline Stages

The {@class https://uor.foundation/resolver/HomotopyResolver} runs stages psi_7
through psi_9 on a
{@class https://uor.foundation/resolver/CechNerve}:

| Stage | Input | Output | Description |
|-------|-------|--------|-------------|
| psi_7 | KanComplex | PostnikovTower | Compute {@class https://uor.foundation/homology/PostnikovTruncation} for k = 0, ..., dim(N(C)) |
| psi_8 | PostnikovTower | HomotopyGroups | Extract {@class https://uor.foundation/observable/HomotopyGroup} from each truncation stage |
| psi_9 | HomotopyGroups | KInvariants | Compute the {@class https://uor.foundation/homology/KInvariant} classifying each extension |

The resolver connects via {@prop https://uor.foundation/resolver/homotopyTarget}
(the input CechNerve) and {@prop https://uor.foundation/resolver/homotopyResult}
(the output HomotopyGroup observables).

## Prerequisite: Kan Promotion

Before entering psi_7, the CechNerve (already a SimplicialComplex from psi_1)
must be promoted to a {@class https://uor.foundation/homology/KanComplex}. Identity
{@ind https://uor.foundation/op/HT_1} certifies that the finite constraint nerve
always satisfies the Kan extension condition. The
{@class https://uor.foundation/homology/HornFiller} witnesses provide constructive
proof.

## Worked Example

Continuing the 3-constraint example from the psi pipeline guide (constraints C_1,
C_2, C_3 on R_4 producing a path nerve C_1 — C_2 — C_3):

**psi_7 — Postnikov tower.** The nerve has dim = 1 (edges only), so we compute
tau_{<=0} and tau_{<=1}:
- tau_{<=0}: the set of path components (one component, pi_0 = Z)
- tau_{<=1}: the full 1-type (pi_1 = 0 since the nerve is a tree)

**psi_8 — Homotopy groups.** Extract:
- pi_0 = Z (one connected component) — confirms beta_0 = 1 from psi_4
- pi_1 = 0 (no loops) — confirms the nerve is simply connected
- pi_k = 0 for k >= 2 (dim = 1 bound, identity {@ind https://uor.foundation/op/HT_4})

**psi_9 — K-invariants.** The single k-invariant kappa_1 classifying the extension
from tau_{<=0} to tau_{<=1} is trivial (since pi_1 = 0). The homotopy type is a
product — confirming the nerve has no higher structure beyond connected components.

## Pipeline Composition

Identity {@ind https://uor.foundation/op/HP_1} states that psi_7 composed with psi_1
equals Kan promotion of the nerve. Identity {@ind https://uor.foundation/op/HP_2}
ensures that psi_8 restricted to the k-truncation agrees with psi_3 restricted to
the k-skeleton. Identity {@ind https://uor.foundation/op/HP_3} connects psi_9 to
the QLS_4 spectral sequence convergence.

The overall complexity of the HomotopyResolver is O(n^{d+1}) where n is the number
of constraints and d is the nerve dimension ({@ind https://uor.foundation/op/HP_4}).
