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

- **Historical:** `read_route=starling:M:K`: `crates/uor-r4-training/src/geometric_stack.rs` (`StarlingRoute`, `starling_select`, `starling_read`). Also in `mqar-bench`. It selects on the host and gates with a sigmoid or tanh, so under D23–D25 it is inactive evidence.
- **Native, rung 0b:** `native_read=starling_native:M:K:E` on the native trunk. Source: `crates/uor-r4-training/src/starling_native.{rs,cu}`; wiring: `StackModel::set_native_starling_read`.
  - Kernel K1 selects neighbours on the GPU. It is built ahead of time by nvcc into its own module (`build.rs`; no NVRTC).
  - The read is a hard rank read over the kept slots, with a learned null score.
  - Its declared credit, injected in backward (`CreditCapture`): a pairwise hinge over utility-ordered kept, exploration and null candidates, and an address margin toward an off-home helpful candidate.

## Tried

- **Rung 1: recall at chance when trained from scratch** ([record](../labs/starling-read-2026-10-11/README.md)). Hard selection gives no gradient to keys outside the kept set. The cost saving is confirmed: 7× fewer positions scored.
- **Rung 2: the addresses learn, the readout does not** ([record](../labs/starling-anneal-2026-10-11/README.md)). A training-only dense helper is annealed to zero. With it, the true source enters the kept 7 neighbours 73% of the time at peak, against 10% at initialisation. On the pure hard read it stays at 43–56%. Recall is still at chance: the rank table and gate do not pick the right value among the 7.
- **Rung 3: a content gate does not help** ([record](../labs/starling-content-2026-10-11/README.md)). Gating the kept 7 by the read's q·k leaves recall at chance (0.010), with the source kept 47–56% of the time. The gate was not the bottleneck; the value path, or per-head availability of the source, is suspect. Note: content logits need a tanh-form sigmoid (the exp form overflowed).
- **Pivot diagnostics** ([record](../labs/starling-pivot-2026-10-11/README.md)):
  - Velocity routing adds nothing beyond token identity.
  - With the source at a fixed slot, recall is 1.000, so the value path works.
  - A shared kept set raises per-head availability to 61–72%, but recall stays at chance.
  - Content ranking does not learn to rank the source first (0.03–0.29). The likely cause is the missing competitive gradient.
  - Recommended: a training-only softmax surrogate (owner decision).
- **Softmax surrogate (training only)** ([record](../labs/starling-surrogate-2026-10-12/README.md)):
  - It teaches content selection: the source is ranked first 0.55–0.78 of the time when kept, against 0.03–0.12 without it.
  - With a shared kept set, recall lifts off chance unevenly (0.044, 0.288, 0.091).
  - Below the pre-registered reading-2 bar, so the line goes to the owner.

Rungs 1–3, the pivot and the surrogate were measured beside softmax helpers, surrogates or teachers. Under D23 they are inactive evidence: they stay recorded and are not to be rerun.

- **Rung 0b, the lawful native read: design and source.** Card [#2032 6146209361](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6146209361), amendment [A1](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6147797508), §4 finite controls [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6148141029); [record](../labs/starling-rung0b-source-2026-10-13/README.md).
  - It replaces the softmax surrogate's competitive gradient with a rank-consistent hinge whose order comes from each slot's stopped-gradient utility `u = -<g, v>`.
  - The null is a utility-0 candidate, so abstention is learned through truncation.
  - Exploration slots (zero forward weight) give near-miss keys credit, fixing rung 1's "keys outside the kept set get no gradient".
  - The decisive §5 run (in-context recall top-1 on copyable dev targets against rung 0) needs its own sheet after rung 0 reports.
- **D21 pivot card, 13 October** ([#2032 6150703214](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6150703214)). The line is at 3/3, and the card continues it with one decisive run instead of archiving it: §5, plus three amendments.
  - P1: the native `native_read=starling_native` read.
  - P2: the key-address gap closed first. The recommended fix is a symmetric key-address margin, with its own finite controls and FIT.
  - P3: no RoPE arm and no transformer comparison.

  The run is gated on rung 0, P2 and the owner's hold release.

## A fair test needs

- A training signal for the addresses: soft-to-hard annealing works (rung 2).
- A readout that singles out the right kept source: a supervised or agreement term on the rank table and gate, or a sharper gate.
- Recall measured by distance bucket (so eviction is distinguishable).
- More than one read layer before any multi-hop claim.

## Open questions

- Soft-to-hard schedule against an auxiliary loss.
- Whether to share the address with the quaternion state (h_t on S3) rather than learn it separately.
- K and M at long context.
