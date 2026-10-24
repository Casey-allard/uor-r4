---
id: owner-ruling-spin-ternary-arithmetic
title: Rule on spin-native weights and ternary/XOR, or build a geometric alphabet
milestone: research
area: mechanism
status: done
open_to: lab
needs_gpu: false
depends_on: []
assignee: Casey-allard
---

## Why it matters

Both are PARKED replacements for the matrix-arithmetic slot.

## What is missing

Recorded on 24 Oct 2026: D74 in [DECISIONS](../../integration/DECISIONS.md) (the owner's ruling of 2026-10-24T03:41:41Z).

Before the ruling, spin-native weights had reached 1.5148 BPB, level with dense, but only through matmul. Ternary/XOR weights lost on export (7.416 versus 7.171 bits), and a dense ternary sum is still dense (D25). The owner ruled both READY (D74); those caveats stand.

## Done when

The owner records a ruling for each in DECISIONS.md, or a geometric-alphabet proposal exists with a pre-registration.

## How to start

Open these first:

- `docs/mechanisms/spin-weights.md`
- `docs/mechanisms/ternary-xor.md`
- `docs/integration/DECISIONS.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Owner decision** (§5.1): only the owner decides this. The issue lays out the options and their consequences; labs do not act on their own reading of it.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:63`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:118`](../../../docs/mechanisms/README.md)
- [`docs/mechanisms/README.md:119`](../../../docs/mechanisms/README.md)

