#!/usr/bin/env python3
"""Per-function multiply certification for the serving path (D0-a).

WHY A ZERO-CHECK AND NOT A COUNT
--------------------------------
Four counting instruments for "how many multiplies does serving execute" failed in this
project: a raw grep of `*` (counts comments, derefs, type syntax), the `forbidden_arith`
scanner (counts operators, not circuits, and misses method forms), a token-level classifier
(cannot see compiler decisions and counts float scalings as variable), and a static binary
disassembly count (counts instructions PRESENT, not EXECUTED -- a multiply in a branch serving
never takes is counted as if it ran). A hardware multiply counter is not available on Apple
Silicon either: `xctrace` is absent and ARMv8 defines no architectural multiply event.

A per-function ZERO-CHECK is sound where a count is not. If the disassembly of a function's
own instruction range contains no multiply mnemonic, that function cannot execute a multiply.
That is a sufficient condition. It is indifferent to dead code, to inlining and to compiler
strength reduction. It cannot certify "no multiply anywhere in the process", so it makes no
such claim: it certifies a declared list of serving symbols, and it reports the ones it does
not cover.

WHAT IT DOES NOT COVER
----------------------
Release builds inline aggressively, so several serving functions have no symbol of their own
(for example `context_hopf_fiber_q30`, `score_readout` and `score_vsa_candidate` are inlined
into their callers in this tree). A symbol that does not exist cannot be certified here. The
script prints how many declared names were not found so that gap stays visible.

USAGE
-----
    cargo build --release -p uor-r4-api --bin r4-native-chat
    python3 scripts/serving_multiplier_check.py [path/to/binary]

Exit status is non-zero if any certified symbol contains a multiply, or if any watched
symbol exceeds its recorded ceiling. Ceilings may only be LOWERED as multiplies are removed.
"""

import re
import subprocess
import sys

DEFAULT_BIN = "target/release/r4-native-chat"

MUL = re.compile(
    r"\b(mul|madd|msub|mneg|smull|umull|smaddl|umaddl|smsubl|umsubl|"
    r"smulh|umulh|sqdmulh|sqrdmulh|imul|mla|mls)\b"
)

# Symbols that MUST contain zero multiply instructions. Substring-matched against the
# demangled-ish symbol name.
CERTIFIED = [
    ("context_predicted_fiber_from_ring", "JEPA forward prediction on the serving path"),
    ("context_fiber_from_ring", "ring-buffer geometric state"),
    ("compose_context_roots", "exact 2I composition (table lookup)"),
    ("compose_ring_roots", "exact 2I composition (table lookup)"),
    ("binary_model", "zero-copy mmap serving model"),
    ("score_readout", "S2 readout"),
    ("score_vsa_candidate", "VSA candidate scoring"),
    ("hamming_distance", "hypervector distance"),
    ("bipolar_correlation_q15", "VSA correlation"),
    ("similarity_q15", "VSA similarity"),
    ("score_token", "hierarchical lattice scoring"),
    ("cluster_of", "fine-cluster lookup"),
]

# Symbols allowed a non-zero count while the fix is outstanding, with a ratcheting ceiling.
# `attend`'s multiplies are SplitMix64 on-demand hypervector generation
# (`Hypervector::from_seed`), reached through `Codebook::get`'s fallback. The fix is a
# multiplier-free xorshift/rotate generator so the on-demand path cannot emit a multiply at all;
# until then the ceiling may only be lowered.
WATCH = [
    ("MultiHeadVsaAttention", 400, "SplitMix64 on-demand generation; target 0 via xorshift mixer"),
    ("from_seed", 400, "SplitMix64; target 0 via xorshift mixer"),
    ("get_atom", 100, "SplitMix64 fallback for atoms; target 0"),
]


def disassemble(binary):
    out = subprocess.run(["otool", "-tV", binary], capture_output=True, text=True).stdout
    if not out:
        sys.exit(f"error: no disassembly from `otool -tV {binary}`; is the binary built?")
    per_symbol = {}
    sym = None
    for line in out.splitlines():
        s = line.strip()
        if s.endswith(":") and not line.startswith("\t") and re.match(r"^[_\w]", s):
            sym = s[:-1]
            per_symbol.setdefault(sym, [])
            continue
        if sym is not None and re.match(r"^[0-9a-fA-F]{6,}\s", s):
            per_symbol[sym].append(s)
    return per_symbol


def count_mul(instrs):
    return sum(1 for i in instrs if MUL.search(i))


def main():
    binary = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_BIN
    per_symbol = disassemble(binary)
    total_instr = sum(len(v) for v in per_symbol.values())
    total_mul = sum(count_mul(v) for v in per_symbol.values())
    print(f"binary   : {binary}")
    print(f"symbols  : {len(per_symbol)}   instructions: {total_instr}   multiplies: {total_mul}")
    print("(the whole-binary multiply total is INFORMATIONAL: it counts instructions present,")
    print(" not instructions executed, and a multiply in a branch serving never takes counts too.)")
    print()

    failures = []
    print("CERTIFIED (must be zero):")
    missing = []
    for name, why in CERTIFIED:
        matched = [k for k in per_symbol if name in k]
        if not matched:
            missing.append(name)
            print(f"  [ -- ] {name:<42} NO SYMBOL (inlined in release; NOT certified)")
            continue
        n = sum(count_mul(per_symbol[k]) for k in matched)
        status = "OK  " if n == 0 else "FAIL"
        if n:
            failures.append((name, n))
        print(f"  [{status}] {name:<42} mul={n:<5} in {len(matched)} symbol(s)  - {why}")
    print()

    print("WATCH (ratcheting ceiling; target zero):")
    for name, ceiling, why in WATCH:
        matched = [k for k in per_symbol if name in k]
        if not matched:
            print(f"  [ -- ] {name:<42} NO SYMBOL")
            continue
        n = sum(count_mul(per_symbol[k]) for k in matched)
        status = "OK  " if n <= ceiling else "FAIL"
        if n > ceiling:
            failures.append((name, n))
        print(f"  [{status}] {name:<42} mul={n:<5} ceiling={ceiling:<5} - {why}")
    print()

    if missing:
        print(f"NOT CERTIFIED (no release symbol, inlined): {', '.join(missing)}")
        print("These names are part of the serving path but cannot be checked at this level.")
        print()

    if failures:
        print("RESULT: FAIL")
        for name, n in failures:
            print(f"  {name}: {n} multiply instruction(s)")
        print()
        print("A ceiling may only be LOWERED as multiplies are removed. Raising one weakens D0-a")
        print("and requires an owner decision.")
        return 1
    print("RESULT: PASS (every certified symbol is multiply-free; every watch ceiling respected)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
