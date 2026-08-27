# Normative CPU-Only, Multiplication-Free, Zero-Allocation Inference Contract

> **Preserved runtime reference:** this contract governs the existing TLA/R4G1
> transformerless graph hot path named below. It is not the execution contract
> for the unfinished route-native geometric-intelligence decoder and does not
> establish source-free chat. Current architecture and sequencing are in the
> [Geometric Intelligence Programme](geometric_intelligence_programme.md).

**Version:** 1.0.0  
**Date:** 2026-07-24  
**Source Baseline:** `docs/hologram_formal_analysis_direction.md` §§1, 9, 13; `docs/formal_vocabulary.md`; GitHub Issue #157.

---

## 1. Scope, Boundaries, and Exclusions

This document defines the normative execution contract for the production R⁴/R4G1 transformerless graph inference engine.

### Explicit Inclusions
- Production hot-path graph execution kernel (`uor-r4-core::transformerless::runtime`, `uor-r4-graph-runtime`).
- State resolution, vector symbol operations, patch evaluation, and token prediction.
- Memory allocation lifecycle during steady-state repeated inference steps.

### Explicit Exclusions
- Teacher model execution, training, offline graph compilation, quantization, behavioral probing, and certificate generation tooling. These off-line stages may freely use floating-point arithmetic, multiplication, tensors, GPUs, and heap allocation.

---

## 2. Inference Boundary Definition

The production inference boundary encompasses:
- `uor-r4-core::transformerless::runtime::predict`
- `uor-r4-graph-runtime::engine::R4G1Runtime::predict_step`
- `uor-r4-graph-runtime::routing::evaluate_program`

Every call crossing into the inference boundary must strictly conform to the permitted operation set, CPU-only execution target, and zero-allocation steady-state lifecycle.

---

## 3. Permitted Operation Set

The deployed inference hot path shall ONLY execute operations belonging to the following permitted classes:

1. **Bitwise Operations:** `XOR`, `AND`, `OR`, `NOT`, `NAND`, `NOR`, `XNOR`.
2. **Shift and Rotate Operations:** Bitwise logical/arithmetic left shift, right shift, bitwise rotation (`rotate_left`, `rotate_right`).
3. **Population Count & Bit Counting:** `popcount` (count ones), trailing zeros count (`cttz`), leading zeros count (`ctlz`).
4. **Integer Addition and Subtraction:** Signed and unsigned fixed-width integer addition and subtraction with saturating or wrapping arithmetic (`saturating_add`, `saturating_sub`, `wrapping_add`, `wrapping_sub`).
5. **Comparisons:** Equal (`==`), not equal (`!=`), less than (`<`), less than or equal (`<=`), greater than (`>`), greater than or equal (`>=`).
6. **Table Reads & Array Lookups:** Fixed-offset indexing into immutable contiguous arrays.

---

## 4. Forbidden Operation Classes & Legal Exception

### Forbidden Operations
- **No Floating-Point Arithmetic:** `f32` and `f64` types, operations, and instructions are forbidden in the production hot path.
- **No Multiplication or Division:** `*`, `/`, `%` arithmetic operators are forbidden in the production hot path.
- **No Heap Allocation:** `alloc::alloc`, `malloc`, `Box::new`, `Vec` reallocation, or `String` allocations are forbidden during steady-state execution steps.

### Legal Exception
- Pointer offset calculations for array indexing and memory addressing (`ptr::add`, slice indexing `&buf[idx]`) utilizing integer addition/shift are permitted exclusively for memory address generation.

---

## 5. Zero-Allocation Lifecycle

The execution lifecycle consists of three distinct phases:

1. **Phase 1: Instantiation & Loading (Warmup)**  
   Graph containers (`GraphView`, table stores) are mapped or allocated once during initialization. Heap allocation is permitted only during this phase.

2. **Phase 2: Steady-State Execution (Hot Path)**  
   Repeated execution steps (`infer_step`, `predict_step`) operate strictly over pre-allocated fixed-capacity memory buffers (`RuntimeState`, `StepOutput`). **Heap allocation is 0 bytes.**

3. **Phase 3: Teardown & Disposal**  
   Resources are freed upon context destruction.

---

## 6. CPU-Only Target Platform Specification

- The production inference runtime targets standard CPU architectures (x86_64, AArch64, and portable scalar fallback).
- **GPU/Accelerator Exclusions:** No GPU, NPU, TPU, or hardware accelerator driver calls or dependencies are permitted or required for the normative inference path.

---

## 7. Typed Execution Signatures & Integer-Only Types

All hot-path interfaces operate strictly over integer types (`u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `usize`, `isize`).

```rust
pub fn infer_step(
    graph: &GraphView<'_>,
    state: &mut RuntimeState,
    token: u32,
    output: &mut StepOutput,
) -> Result<(), RuntimeError>;
```

---

## 8. Machine-Readable Contract Representation & Ownership

The contract is encoded programmatically in `crates/uor-r4-graph-format/src/inference_contract.rs` (`#![no_std]`, `alloc`-free) and exposed across the workspace for machine-checked audit and verification.
