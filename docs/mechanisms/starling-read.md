# Starling read: topological neighbours on the 600-cell

**Milestone:** M4 #2032 (no softmax at runtime; flat cost at long context). **Lab:** claude. **Owner idea:** a starling tracks about seven neighbours, and the flock's coherence emerges from that local rule.

## The idea

Real starlings respond to about seven neighbours chosen topologically, not by distance ([Princeton 2013](https://www.princeton.edu/news/2013/02/07/birds-feather-track-seven-neighbors-flock-together)). The read applies the same rule:
- Each token sits at an icosian vertex of the 600-cell.
- It reads only the ~K nearest recent tokens found in its own cell and the 12 adjacent cells.
- It never scans the whole context and never normalises with a softmax.
- Coherence across the sequence comes from layers and the quaternion state passing information one hop further each time.

SpiralCore's Kuramoto witness shows the principle: local coupling over 60 geometric edges reaches the same synchrony as 435 all-to-all edges.

## In code

`read_route=starling:M:K`: `crates/uor-r4-training/src/geometric_stack.rs` (`StarlingRoute`, `starling_select`, `starling_read`). Also in `mqar-bench`.

## Tried

- **Rung 1: recall at chance when trained from scratch** ([record](../labs/starling-read-2026-10-11/README.md)). Hard selection gives no gradient to keys outside the kept set. The cost saving is confirmed: 7× fewer positions scored.

## A fair test needs

- A training signal for the addresses: soft-to-hard annealing, or an address-agreement loss.
- Recall measured by distance bucket (so eviction is distinguishable).
- More than one read layer before any multi-hop claim.

## Open questions

- Soft-to-hard schedule against an auxiliary loss.
- Whether to share the address with the quaternion state (h_t on S3) rather than learn it separately.
- K and M at long context.
