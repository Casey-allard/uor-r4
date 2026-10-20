/-!
# ONNX spec / stack-safety bounds.

Mirrors `crates/uor-addr/src/onnx/shapes/bounds.rs`. ADR-060 removed the
application-policy capacity profile (`OnnxHostBounds` and its node /
initializer / attribute count ceilings): the canonical form is the full
flat skeleton flowing through a borrowed carrier, so node / attribute /
initializer counts and value widths are unbounded. What remains are the
IR-version pin, the default-domain opset minimum, and a native-stack
overflow guard on the recursive subgraph descent.
-/
namespace UorAddr.Onnx

/-- The highest ONNX IR revision admitted (`ONNX_IR_VERSION_MAX`); the
realization accepts any `ir_version` in `1..=irVersionMax`. -/
def irVersionMax : Nat := 13

/-- Default-domain opset minimum (`ONNX_OPSET_VERSION_MIN`). -/
def opsetVersionMin : Int := 1

/-- Subgraph-nesting stack-safety guard (`ONNX_SUBGRAPH_DEPTH_MAX`) —
not a content ceiling. -/
def subgraphDepthMax : Nat := 64

end UorAddr.Onnx
