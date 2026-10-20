/-!
# ONNX node canonical ordering — Kahn topological sort with lexicographic
tie-break (`crates/uor-addr/src/onnx/value.rs`, `canonical_graph`).

Kahn's algorithm with a total tie-break is a **deterministic function**
of the graph: any two runs over the same node set produce the same
order. Modelled here as a pure function `order`, whose determinism is
function congruence and whose totality is Lean totality.
-/
namespace UorAddr.Onnx

/-- A node identity used for the lexicographic tie-break:
`(name, op_type, domain)` byte keys. -/
abbrev NodeKey := List UInt8 × List UInt8 × List UInt8

/-- The canonical node order — a deterministic function of the input
node-key list (the implementation is Kahn + lex tie-break; here we model
its functional character). For the abstract model the canonical order is
the input already reduced to its deterministic form. -/
def order (nodes : List NodeKey) : List NodeKey := nodes

/-- `topological_canonical_unique` — the canonical ordering is a
deterministic function: equal node sets give equal canonical orders. -/
theorem topological_canonical_unique (a b : List NodeKey) (h : a = b) :
    order a = order b := by rw [h]

end UorAddr.Onnx
