import UorAddr.HexEncoding
import UorAddr.Onnx.Value

/-!
# ONNX κ-derivation — the address-from-commitment map.
-/
namespace UorAddr.Onnx

open UorAddr.HexEncoding

/-- The κ-label of a commitment under hash axis `sha`. -/
def kappaOf (sha : Commitment → (Fin 32 → UInt8)) (c : Commitment) : Fin 71 → UInt8 :=
  kappaLabel (sha c)

end UorAddr.Onnx
