# Homological Analysis

The {@ns homology} and {@ns cohomology} namespaces add an algebraic topology layer
to UOR. When constraints interact in complex ways, topological invariants diagnose
whether {@concept resolution} will converge or stall.

## Simplicial Complexes

The {@class https://uor.foundation/homology/Simplex} class represents a single
simplex -- a generalized triangle. A collection of simplices that is closed under
taking faces forms a {@class https://uor.foundation/homology/SimplicialComplex}.

In UOR, the Cech nerve is the key simplicial complex. Each constraint in the
resolution process is a vertex. A set of constraints that can be simultaneously
satisfied forms a simplex. The resulting complex encodes the combinatorial structure
of constraint compatibility.

## Chain Homology and Betti Numbers

Chain homology measures the "holes" in a simplicial complex:

- **Betti number beta_0** counts connected components. If beta_0 = 1, the constraint
  space is connected -- all constraints can reach each other.
- **Betti number beta_1** counts independent loops. Nonzero beta_1 means the
  constraints contain cyclic dependencies that may cause resolution to stall.
- **Higher Betti numbers** detect higher-dimensional voids in the constraint
  structure, signaling more complex obstructions.

When all Betti numbers are trivial (zero except beta_0 = 1), the constraint space is
contractible -- resolution converges without obstruction. This is the ideal case.

## Sheaf Cohomology

The {@class https://uor.foundation/cohomology/Sheaf} class models data that varies
smoothly over the constraint space. Sheaf cohomology detects when local constraint
satisfaction fails to globalize.

A {@class https://uor.foundation/cohomology/GluingObstruction} in the first
cohomology group H^1 pinpoints exactly where local solutions cannot be assembled
into a global resolution. This is the algebraic formalization of "works locally,
fails globally."

## Spectral Sequences

The {@class https://uor.foundation/observable/SpectralSequencePage} class models
the pages of a spectral sequence -- a tool that refines the homological analysis
level by level. At each {@concept witt-levels} scale, the spectral sequence
provides increasingly precise information about obstructions.

The spectral sequence connects the homological analysis to the Witt level
tower: obstructions visible at W8 may dissolve at W16, or new obstructions may
appear at higher levels.

## Euler Characteristics

The {@prop https://uor.foundation/resolver/nerveEulerCharacteristic} property
records the Euler characteristic of the Cech nerve -- a single integer that
summarizes the topological complexity. A positive Euler characteristic suggests
contractibility (good for convergence); a negative one signals complexity.

## Connection to the PRISM Pipeline

Homological analysis operates within the Resolve stage. When {@concept resolution}
stalls, the Cech nerve's topology explains why. The {@concept site} bundle
structure provides the geometric substrate that homology and cohomology analyze.
The {@concept observables} namespace measures the topological invariants
(holonomy groups, monodromy classes) that feed into this analysis.
