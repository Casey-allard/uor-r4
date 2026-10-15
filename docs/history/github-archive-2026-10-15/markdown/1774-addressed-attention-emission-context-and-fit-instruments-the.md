# #1774 — addressed_attention: emission-context and fit instruments — the context is a 1024→9 hash that erases the digit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T01:15:45Z
- **Updated:** 2026-10-09T13:18:56Z
- **Closed:** 2026-10-06T01:16:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1774
- **Merged:** 2026-10-06T01:16:54Z
- **Merge commit:** 932f4f3fa0cd4d77e3433a90d78135ccdeea24c9
- **Base / Head:** main / UOR-Foundation:deepseek/aa-instruments-2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Three measurement bins from the emission investigation, all measurement-only, production code unchanged. They complete the arc that began with `selected_operations = 0`.

## The finding

The arithmetic result **is** wired byte-wise into the emission input: `engine.rs:228-238` maps the active lease to `Active{byte: bytes[cursor], length}`, `objects.rs:132-134` supplies the cursor byte, `inputs.rs:120` puts it at **bits 792..800**. The context is then a **1024→9 LUT cascade** (`circuit.rs:5` `WIDTHS=[256,64,16,9]`; `circuit.rs:143-147`) addressing the 257-way emission row (`policy.rs:99,124`).

**No wiring is missing. The hash is lossy at init.**

Measured (frozen `Parameters::seeded(7341)`, forced read + forced operator): sweeping the result digit's byte field over **0..255** with the other 1023 input bits held fixed yields only **2–4 distinct 9-bit contexts, never 256** — and is **CONSTANT (zero bits of the digit) in 3 of 9 examples**, including `1+2`. A causal swap (`AddAB` vs `SubAB`, 33 pairs) moves the packed input in **32/33** and the context in **7/33**.

The digit is eight bits; the context is nine. **Capacity is sufficient — the LUTs simply don't route it.**

## Why the fit got a null

78 updates, gradient live (gate L2 2.004, head 0.379), ΔL2 0.9405 — but only **331 of 49,030 compiled bytes changed**, copy rate **0/80 before and after**, emission still statistically uniform. The obstruction is that the circuit LUTs are **discrete truth tables**, which is the project's own recorded blocker: *"learning through changing discrete choices remains unresolved"* (`native_geometric_attention_reassessment_973.md:34`).

An independent check on a **different model** agreed on the verdict but corrected the framing: this is a **trainable bottleneck, not a wire cut** — neither "merely an untrained head" nor "structurally silent".

## Separate unresolved defect

**EOS is never a packed byte (0/32)**, so termination is not representable — independent of the routing problem.

## Instruments

| bin | modes |
|---|---|
| `addressed-attention-arith-fit.rs` | `criteria` / `run` — bounded fit with sealed acceptance criteria frozen before any engine call |
| `addressed-attention-emission-context.rs` | `trace` / `causal` / `verdict` — the emission-phase trace and the causal swap |
| `addressed-attention-emission-reach.rs` | digit→context sweep over 0..255 |

Hashes verified against the authors' reports (`arith-fit` sha256 `70d0e343…`). Sealed roots cited in each bin's output. References #820, #973.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

