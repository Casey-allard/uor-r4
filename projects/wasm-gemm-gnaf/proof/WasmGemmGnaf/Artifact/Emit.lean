import WasmGemmGnaf.Wasm.Binary

set_option autoImplicit false

/-!
# Artifact: the emitter (SPEC §11.4)

SPEC §11.4 asks for

```lean
def Artifact.emit : Wasm.Module → ByteArray
theorem decode_emit : Wasm.decode (Artifact.emit m) = .ok m
```

`Wasm/Binary.lean` already owns the verified codec and proves
`Wasm.encode_decode_roundtrip`.  The emitter is therefore *defined* to be that
encoder — it is not a second, independently written serializer that would have
to be re-verified — and `Artifact.decode_emit` is the round trip transported
along that definition.

SPEC §11.4 further insists that "the emitter constructs byte candidates
produced from GNAF plans.  Universal selection, however, ranges over raw byte
strings and SHALL retain the exact winning bytes.  A permissively valid winner
is not silently decoded and re-encoded."  Nothing in this file decodes and
re-encodes anything: `emit` is a one-way map from a module to its canonical
bytes, and `emit_injective` shows distinct modules never share bytes.  The
retention of *selected* bytes is the selection layer's obligation, not the
emitter's, and no theorem here licenses replacing selected bytes by
`emit (decode bytes)`.

Every declaration in this file is proved.  Nothing is assumed.
-/

namespace WasmGemmGnaf.Artifact

/-- **SPEC §11.4.**  The artifact emitter: the canonical binary encoding of a
module. -/
def emit (m : Wasm.Module) : ByteArray := Wasm.encode m

/-- **SPEC §11.4, `decode_emit`.**  Decoding an emitted artifact returns
exactly the module it was emitted from. -/
theorem decode_emit (m : Wasm.Module) : Wasm.decode (emit m) = .ok m :=
  Wasm.encode_decode_roundtrip m

/-- Distinct modules emit distinct artifacts. -/
theorem emit_injective : Function.Injective emit :=
  Wasm.encode_injective

/-- The emitted bytes are exactly the canonical encoding's bytes. -/
theorem emit_toList (m : Wasm.Module) : (emit m).toList = Wasm.encodeList m :=
  Wasm.toList_encode m

/-- The emitted artifact is not empty: it always carries the pinned preamble. -/
theorem emit_prefix (m : Wasm.Module) :
    ∃ t : List UInt8, (emit m).toList = Wasm.magicBytes ++ t := by
  rw [emit_toList]; exact Wasm.encodeList_prefix m

/-- Emitting is the *only* way to obtain bytes that decode: any byte string
that decodes to `m` is exactly `emit m`.  Together with `decode_emit` this makes
`emit` and `Wasm.decode` mutually inverse on decodable byte strings. -/
theorem eq_emit_of_decode {b : ByteArray} {m : Wasm.Module}
    (h : Wasm.decode b = .ok m) : b = emit m :=
  Wasm.decode_is_encode h

/-- Two artifacts that decode to the same module are byte-identical. -/
theorem emit_bytes_unique {b c : ByteArray} {m : Wasm.Module}
    (hb : Wasm.decode b = .ok m) (hc : Wasm.decode c = .ok m) : b = c :=
  Wasm.decode_injective hb hc

end WasmGemmGnaf.Artifact
