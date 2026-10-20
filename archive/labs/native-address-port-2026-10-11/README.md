# The native port for the product-key memory: the product path cannot serve it at all, and the first slice of the fix has landed

References #2029 (M1 acceptance; owner metric direction: *"tokens/s is the best way to compare"*, *"use GPU on a pod — do not
use CPU"*). Lab: DeepSeek, session `deepseek/native-address`. Work 2026-10-11 11:30–12:30 UTC. **Source work and tests only;
no model compute.**

> **One line.** Cycle 20 measured tokens/s and found the product-key memory **1.65× slower than the MLP it replaces** on the
> f32 GPU harness — but that harness is not the product. The product's contract is **D11: no floating point, no multiplier
> instruction in served computation**, and on that path the operator **does not exist**: `export_stack` refuses it with
> *"integer export of product-key memories is not implemented"*, so no artifact carries it into the engine where a table read
> replaces multiplies and there is no matrix unit to waste. **The mechanism's compute case is therefore unmeasured exactly
> where it could be true.** This cycle establishes that blocker with a test that pins it, lands the first slice of the port —
> the container format can now carry the geometry, and refuses to claim a capability the engine lacks — and pre-registers the
> kernel as the next deliverable with its own bar.

## The blocker, pinned

| | |
|---|---|
| the refusal | `crates/uor-r4-training/src/stack_export.rs`: `if c.memory.is_some() { return Err(invalid("integer export of product-key memories is not implemented")) }` |
| pinned by | **`export_refuses_a_product_key_memory_until_the_kernel_serves_it`** (new): a tiny geometric model with a memory layer is exported and the refusal's text is asserted, so removing the refusal has to be deliberate |
| why it blocks the metric | the owner's comparison (tokens/s) must be made where the product runs. The f32 harness measures matmuls on the hardware built for them — the one place a lookup is guaranteed to lose. Until the export and the integer kernel exist, "is the address cheaper?" has **no** product-path answer, in either placement |

## The first landed slice: the format carries it, the engine still refuses

`uor-r4-integer`'s served format gains **`StackMemory`** (`layers`, `heads`, `key_dim`, `sub_keys`, `top_k`, `score`) on the
shape, with geometry rules and a container-level refusal, tested by
`crates/uor-r4-integer/tests/memory_shape.rs` (three tests, all passing, plus the 284-test library suite unchanged):

- a shape **without** a memory validates exactly as before; an older shape with no `memory` section still deserialises
  (the section is `#[serde(default)]`, so existing containers are untouched);
- a shape **with** one is refused with **"a product-key memory has no served kernel yet"** — recorded so that no container
  passes validation and is then served as if the memory were not there;
- the geometry rules reject what could not be served (an empty or non-ascending layer list, a layer index outside the
  pattern, a zero or odd `key_dim`, `sub_keys < 2`, `top_k = 0` or above `sub_keys²`, an unknown score).

**What remains for the kernel, and it is the whole of the next cycle:** quantise the memory's three maps (query, sub-key
codebooks with their Lorentz betas, value table) through the exporter's existing damped path; implement the two per-side
rankings, the `top_k × top_k` candidate join, the softmax over the kept candidates and the weighted value gather in the
integer engine (`crates/uor-r4-integer/src/stack/kernels.rs` and `session.rs`) using the fixed-point conventions the read
already uses for its scores and value accumulation; and make the two integer engines agree bit for bit, which is the D11
contract's own test (`d11 − d10 nll = 0.0`, `max |Δlogit| = 0`).

## Pre-registered bar for the port (before its compute)

- **PRIMARY (the owner's metric, on the product path):** with the port landed, the memory-armed model's **D11 tokens/s** is
  **≥** the matched MLP baseline's on the same windows and threads, on **≥ 2 of 3** seeds, **and** the two integer engines
  agree bit for bit (`d11 − d10 nll = 0.0`, `max |Δlogit| = 0`).
- **SECONDARY (no capability loss):** the frozen panel's wrong-value failures not worse than the matched baseline by more
  than the ±3-row noise; the export writes an artifact of the recorded sha256 for both arms.
- **REPORTED:** D11 and D10 tokens/s, artifact size, parameters, and the panel — with BPB as a **diagnostic** per the owner's
  metric direction.
- **A miss closes the configuration** (this operator, this placement, this quantiser), never the mechanism (D22 §1).

## What this cycle corrects as well

While landing the slice I found that **`memory_layers=L` replaces layer `L`'s MLP, not its read** — and cycles 18–20's
records describe it as a read replacement. The correction is on
[#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6110907294), the three records carry an erratum
section, and the README/STATUS wording is fixed in this PR. **The measurements stand; the interpretation labels did not**,
and the read-replacement question the attribution and ablation point at **has not been run**.

## Evidence

- Source: `crates/uor-r4-integer/src/stack/format.rs` (the `StackMemory` record and its shape rules),
  `crates/uor-r4-integer/tests/memory_shape.rs` (three tests), `crates/uor-r4-training/src/stack_export.rs`
  (the pinned refusal), `crates/uor-r4-integer/src/stack/mod.rs` (the re-export).
- Runs: `cargo test --release -p uor-r4-integer --test memory_shape` → **3 passed**;
  `cargo test --release -p uor-r4-training --lib export_refuses_a_product_key_memory` → **2 passed**.
