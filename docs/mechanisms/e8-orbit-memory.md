# E8 orbit memory: an addressed memory in place of the MLP

**Milestone:** M4 #2032. **Lab:** claude. **Design:** the spin-native model ([#2032 comment](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6104974064)).

## The idea

The MLPs are 44% of a token's multiply-adds. An addressed memory reads a few rows of a large table instead. The E8 lattice supplies the addresses exactly:
- 240 roots, with an O(8) nearest-root decode;
- SpiralCore's Cl(0,6) group splits them into 15 orbits of 16;
- two query halves give 240² slots, of which a head keeps k.

It uses no softmax: a plain rank table and a gate.

## In code

`crates/uor-r4-training/src/e8_memory.rs`: `e8_tables`, `E8Tables::nearest`, `e8_select`, `e8_memory_read`, and usage logging. `StackModel::set_e8_memory` and the CLI `train e8_memory=LAYERS:HEADS:K`. `evaluate` writes `e8_usage`. Export refuses: the memory has no served form yet.

## Tried

- **Rung 1, row 3, keep and iterate** ([record](../labs/e8-orbit-memory-2026-10-11/README.md)):
  - Replacing half the MLPs costs +0.033 BPB at width 256, at about 10× fewer multiply-adds per replaced layer.
  - A learned product-key memory of equal capacity is 0.023 better, so the geometry is not credited yet.
  - Routing is healthy: 3.85 of 3.91 bits of entropy, and 85% of slots read.
  - Replacing all MLPs costs +0.116.

## A fair test needs

- A capacity-matched learned-key control (P).
- Routing health (entropy and slot share), to tell collapse from capacity.
- Matched training steps and the same panel.
- GPU training: a CPU run of E8a took about 6 h.

## Open questions

- Is the gap to P addressing (fixed roots) or value capacity? Learned per-orbit offsets on the E8 frame would separate the two.
- k, and which layers.
- A served form: integer value tables, read as table lookups.
