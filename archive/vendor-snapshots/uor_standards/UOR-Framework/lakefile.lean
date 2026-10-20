import Lake
open Lake DSL

package uor where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]
  preferReleaseBuild := true

@[default_target]
lean_lib UOR where
  srcDir := "lean4"
