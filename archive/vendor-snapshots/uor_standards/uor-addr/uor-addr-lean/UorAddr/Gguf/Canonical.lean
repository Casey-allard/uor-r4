import UorAddr.HexEncoding
import UorAddr.Gguf.Value

/-!
# GGUF κ-derivation — the address-from-commitment map.

`kappaOf sha c = "sha256:" ‖ hex(sha c)`, the ψ_9 projection over the
canonical commitment `c`. Parametric over the σ-axis `sha` (the concrete
axis is `prism::crypto::Sha256Hasher`).
-/
namespace UorAddr.Gguf

open UorAddr.HexEncoding

/-- The κ-label of a commitment under hash axis `sha`. -/
def kappaOf (sha : Commitment → (Fin 32 → UInt8)) (c : Commitment) : Fin 71 → UInt8 :=
  kappaLabel (sha c)

end UorAddr.Gguf
