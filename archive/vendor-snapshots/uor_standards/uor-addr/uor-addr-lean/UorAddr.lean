-- Root module: imports all UorAddr sub-modules.
-- Run: cd uor-addr-lean && lake build

import UorAddr.HexEncoding
import UorAddr.HashAxes
import UorAddr.AddressShape
import UorAddr.KappaDerivation
import UorAddr.AlgebraicClosure
import UorAddr.NfcIdempotence
import UorAddr.VerbDiscipline
import UorAddr.TypedInput
import UorAddr.CompositionLaws

-- GGUF realization theorems
import UorAddr.Gguf.Bounds
import UorAddr.Gguf.Value
import UorAddr.Gguf.Canonical
import UorAddr.Gguf.Recursion
import UorAddr.Gguf.Theorems

-- ONNX realization theorems
import UorAddr.Onnx.Bounds
import UorAddr.Onnx.Value
import UorAddr.Onnx.Canonical
import UorAddr.Onnx.TopologicalSort
import UorAddr.Onnx.Recursion
import UorAddr.Onnx.Theorems
