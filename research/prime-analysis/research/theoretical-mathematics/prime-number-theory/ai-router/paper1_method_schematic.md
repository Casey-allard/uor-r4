# Paper 1 Method Schematic

This file describes the intended method schematic for the Paper 1 manuscript and the draft asset.

## Purpose

The schematic should make the routing mechanism legible in one glance before the reader hits the main scaling figure.

It should explain:
- normalized embedding input,
- fixed 4D projection,
- Hopf-style coordinate extraction,
- sectorization under budget `K`,
- concentrated routing output.

## Visual structure

Recommended left-to-right flow:

1. **Input block**
- Label: `L2-normalized embedding x in R^D`
- Visual: a high-dimensional vector box

2. **Fixed projection block**
- Label: `Fixed 4D projection`
- Include canonical coordinate tuple `(3, 65, 2, 21)`
- Output shown as `(a, b, c, d)`

3. **Hopf coordinate block**
- Label: `Hopf-style coordinates`
- Show:
  - `chi_u`
  - `delta`
  - `alpha`
- Optional small note:
  `transported alpha = alpha + 0.5 * lambda * cos(2 chi) * delta`

4. **Sectorization block**
- Label: `Discretize under routing budget K`
- Show product bins:
  `k_chi x k_delta x k_alpha <= K`

5. **Output block**
- Label: `Fixed routing sector`
- Visual: one highlighted sector among many possible sectors

6. **Concentration note**
- Small side note or footer:
  `Structured embeddings occupy a non-uniform subset of sectors`

## Style guidance

- Keep it diagrammatic, not decorative
- Use neutral ML-paper styling
- Avoid speculative architecture labels
- Do not mention `H^4 x H^4`
- Do not mention internal increments

## Canonical support

The schematic is derived from:
- `router-research/papers/main.tex`
- `paper1_method_notes.md`

It is a method-illustration asset, not a result-bearing figure.

## Draft asset

Draft schematic asset:
- `paper1_assets/figures/paper1_method_schematic.svg`

If the manuscript rewrite needs a PDF version, the SVG can be converted later without changing the structure.
