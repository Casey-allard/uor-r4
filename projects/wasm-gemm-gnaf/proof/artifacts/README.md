# artifacts/

SPEC §5 requires `wasm-gemm-gnaf.wasm`, `atlas-seal.bin`,
`generated-proof-input.json`, `pre-final-environment.json`, and
`proof-manifest.json` here.

**Empty by design.** Emitting an artifact requires the mechanized Core 3.0
semantics (`WS-001`) and the verified emitter. Writing a `.wasm` by any other
route would be a byte sequence with no proved relationship to the Lean term,
which SPEC §11.4 forbids: the committed bytes must equal
`Release.committedArtifactBytes`, checked byte for byte.

Release gate step 6 fails while this directory is empty. That is correct.
