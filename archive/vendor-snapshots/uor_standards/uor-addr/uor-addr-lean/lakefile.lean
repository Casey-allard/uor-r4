import Lake
open Lake DSL

package UorAddrLean where

-- Pin against UOR-Framework's Lean library (mirrors the Rust crate's
-- `uor-foundation = "0.4.5"` dependency). No mathlib — load-bearing
-- identities are provable in pure Lean 4 by `decide` / `omega` / `rfl`.
require uor from git
  "https://github.com/UOR-Foundation/UOR-Framework"
  @ "main"

lean_lib «UorAddr» where
  roots := #[`UorAddr]
