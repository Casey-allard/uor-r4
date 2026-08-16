/-
  The unified machine fault of SPEC §7.1.

  `Wasm.SpecMachine` exposes a single `Fault` type used by both
  `decode : ByteArray → Except Fault Module` and
  `initial : Module → Invocation → Except Fault Config`.

  The concrete model keeps the two failure domains as separate inductives —
  `DecodeFault` in `Wasm/Binary.lean` (malformed bytes) and
  `InstantiationFault` in `Wasm/Config.lean` (allocation and export failures) —
  because they are genuinely different sets of outcomes and SPEC §7.1's ownership
  rule assigns them to different modules.  This module joins them into the single
  `Fault` the machine interface requires, without either module depending on the
  other.

  Both injections are proved injective and their images provably disjoint, so a
  decoding failure can never be silently reported as an instantiation failure.
-/
import WasmGemmGnaf.Wasm.Binary
import WasmGemmGnaf.Wasm.Config

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

/-- The machine fault of SPEC §7.1: either the bytes failed to decode, or a
decoded module failed to instantiate. -/
inductive Fault
  /-- The byte sequence did not decode under the pinned binary grammar. -/
  | decoding (fault : DecodeFault)
  /-- The decoded module did not instantiate under the release profile. -/
  | instantiation (fault : InstantiationFault)
  deriving DecidableEq, Repr, Inhabited

namespace Fault

theorem decoding_injective {a b : DecodeFault}
    (h : Fault.decoding a = Fault.decoding b) : a = b := by
  cases h; rfl

theorem instantiation_injective {a b : InstantiationFault}
    (h : Fault.instantiation a = Fault.instantiation b) : a = b := by
  cases h; rfl

/-- The two failure domains are disjoint: no fault is both. -/
theorem decoding_ne_instantiation (a : DecodeFault) (b : InstantiationFault) :
    Fault.decoding a ≠ Fault.instantiation b := by
  intro h; cases h

/-- Which phase failed. -/
def isDecoding : Fault → Bool
  | .decoding _ => true
  | .instantiation _ => false

@[simp] theorem isDecoding_decoding (a : DecodeFault) :
    (Fault.decoding a).isDecoding = true := rfl

@[simp] theorem isDecoding_instantiation (b : InstantiationFault) :
    (Fault.instantiation b).isDecoding = false := rfl

end Fault

end WasmGemmGnaf.Wasm
