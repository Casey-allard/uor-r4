import Lake
open Lake DSL

package «wasm-gemm-gnaf» where
  leanOptions := #[
    ⟨`autoImplicit, false⟩,
    ⟨`relaxedAutoImplicit, false⟩
  ]

@[default_target]
lean_lib «WasmGemmGnaf» where
  -- `.andSubmodules` builds the root module too. `.submodules` alone skips it,
  -- which silently hid a broken root import for an entire build cycle.
  globs := #[.andSubmodules `WasmGemmGnaf]
