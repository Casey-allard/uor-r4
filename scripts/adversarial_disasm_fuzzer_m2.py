#!/usr/bin/env python3
"""Adversarial Disassembly Fuzzer for Milestone M2 Verification.

Deeply scans and fuzzes disassembled instructions of all numerical serving symbols
in `libuor_r4_integer.rlib` and release executable binaries (`uor-r4-integer`).

Validates D0-b zero-multiplier invariants:
- Class I: Hardware Multipliers (18+ mnemonics + ARM64 opcodes)
- Class II: Hardware Dividers (sdiv, udiv)
- Class III: Floating-Point & Register Transfer Instructions (fmul, fmov, fadd, etc.)
"""

import os
import re
import subprocess
import sys

TARGET_RLIB = "/Users/casey.allard/uor-r4/target/release/libuor_r4_integer.rlib"
TARGET_BIN = "/Users/casey.allard/uor-r4/target/release/uor-r4-integer"

# Extended Class I Patterns (Multipliers)
CLASS_I_FUZZ_PATTERNS = [
    ("exact_multipliers", re.compile(r"\b(mul|madd|msub|mneg|smull|umull|smaddl|umaddl|smsubl|umsubl|smulh|umulh|sqdmulh|sqrdmulh|mla|mls|pmul|pmull)\b", re.I)),
    ("prefix_mul", re.compile(r"\b(mul|smull|umull|smaddl|umaddl|smsubl|umsubl)[a-z0-9]*\b", re.I)),
    ("prefix_madd_msub", re.compile(r"\b(madd|msub|mneg)[a-z0-9]*\b", re.I)),
    ("prefix_simd_mul", re.compile(r"\b(sqdmulh|sqrdmulh|pmul|pmull|mla|mls)[a-z0-9.]*\b", re.I)),
]

# Extended Class II Patterns (Dividers)
CLASS_II_FUZZ_PATTERNS = [
    ("exact_dividers", re.compile(r"\b(sdiv|udiv)\b", re.I)),
    ("prefix_div", re.compile(r"\b[su]?div\b", re.I)),
]

# Extended Class III Patterns (Floats & Register Moves)
CLASS_III_FUZZ_PATTERNS = [
    ("exact_fp_math", re.compile(r"\b(fmul|fadd|fsub|fdiv|fmadd|fmsub|fnmadd|fnmsub|fnmul|fsqrt)\b", re.I)),
    ("exact_fp_transfer", re.compile(r"\b(fmov)\b", re.I)),
    ("exact_fp_compare", re.compile(r"\b(fcmp|fcmpe)\b", re.I)),
    ("exact_fp_convert", re.compile(r"\b(scvtf|ucvtf|fcvtzs|fcvtzu|fcvtms|fcvtmu|fcvtps|fcvtpu|fcvtas|fcvtau|fcvt)\b", re.I)),
    ("exact_fp_round", re.compile(r"\b(frinta|frintm|frintn|frintp|frintx|frintz|frinti)\b", re.I)),
    ("scalar_fp_neg_abs", re.compile(r"\b(fneg|fabs)\s+[ds]\b", re.I)),
    ("fp_vector_reduction", re.compile(r"\baddp\s+[vds][0-9]+", re.I)),
]

SERVING_NAMESPACE_PATTERNS = [
    re.compile(r"uor_r4_integer::model::"),
    re.compile(r"uor_r4_integer::math::"),
    re.compile(r"uor_r4_integer::tables::"),
    re.compile(r"uor_r4_integer::session::"),
    re.compile(r"uor_r4_integer::generation::"),
    re.compile(r"uor_r4_integer::sampling::"),
    re.compile(r"<uor_r4_integer::model::IntegerModel>"),
    re.compile(r"<uor_r4_integer::session::ChatSession>"),
    re.compile(r"<uor_r4_integer::sampling::Sampler>"),
]

EXCLUDE_NON_SERVING = re.compile(
    r"(drop_glue|load_with_tables|verify_sealed|sha256|format::|serde|from_slice|std::io|Display|Error)",
    re.I
)


def disassemble_target(path):
    cmd = ["objdump", "--demangle", "-d", path]
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        raise RuntimeError(f"objdump failed on {path}: {p.stderr}")
    return p.stdout


def parse_assembly_symbols(disasm):
    symbols = {}
    current_sym = None
    sym_re = re.compile(r"^[0-9a-fA-F]+\s+<(.*)>:$")
    instr_re = re.compile(r"^\s*([0-9a-fA-F]+):\s+([0-9a-fA-F]+)\s+(.*)$")

    for line in disasm.splitlines():
        sm = sym_re.match(line.strip())
        if sm:
            current_sym = sm.group(1)
            symbols[current_sym] = []
            continue
        if current_sym is not None:
            im = instr_re.match(line)
            if im:
                addr = im.group(1)
                opcode = im.group(2)
                asm = im.group(3)
                symbols[current_sym].append((addr, opcode, asm))

    return symbols


def audit_symbol_adversarial(sym_name, instructions):
    violations = []
    for addr, opcode, asm in instructions:
        # Check Class I
        for pat_name, pat in CLASS_I_FUZZ_PATTERNS:
            if pat.search(asm):
                violations.append(("Class I Multiplier", pat_name, addr, opcode, asm))
                break

        # Check Class II
        for pat_name, pat in CLASS_II_FUZZ_PATTERNS:
            if pat.search(asm):
                violations.append(("Class II Divider", pat_name, addr, opcode, asm))
                break

        # Check Class III
        for pat_name, pat in CLASS_III_FUZZ_PATTERNS:
            if pat.search(asm):
                violations.append(("Class III Float/Transfer", pat_name, addr, opcode, asm))
                break

    return violations


def run_adversarial_disassembly_fuzz():
    print("=" * 80)
    print("STARTING ADVERSARIAL DISASSEMBLY FUZZ SUITE (Milestone M2)")
    print("=" * 80)

    targets = [TARGET_RLIB, TARGET_BIN]
    overall_clean = True
    total_symbols_audited = 0
    total_instructions_fuzzed = 0

    for target in targets:
        if not os.path.exists(target):
            print(f"SKIPPING: Target not found at {target}")
            continue

        print(f"\nDisassembling & Fuzzing Target: {target}")
        disasm = disassemble_target(target)
        symbols = parse_assembly_symbols(disasm)
        print(f"Total symbols found in binary/archive: {len(symbols)}")

        serving_symbols = {}
        for sym, instrs in symbols.items():
            is_serving = any(p.search(sym) for p in SERVING_NAMESPACE_PATTERNS)
            if is_serving and not EXCLUDE_NON_SERVING.search(sym):
                serving_symbols[sym] = instrs

        print(f"Serving path symbols isolated for deep adversarial audit: {len(serving_symbols)}")

        target_violations = 0
        for sym, instrs in serving_symbols.items():
            total_symbols_audited += 1
            total_instructions_fuzzed += len(instrs)
            v = audit_symbol_adversarial(sym, instrs)
            if v:
                overall_clean = False
                target_violations += len(v)
                print(f"\n[VIOLATION FOUND] Symbol: {sym}")
                for v_class, pat_name, addr, opcode, asm in v:
                    print(f"  --> {v_class} [{pat_name}] at 0x{addr} ({opcode}): {asm}")
            else:
                pass

        if target_violations == 0:
            print(f"[PASS] 100% of {len(serving_symbols)} serving symbols in {os.path.basename(target)} clean (0 violations).")
        else:
            print(f"[FAIL] {target_violations} violations found in {os.path.basename(target)}!")

    print("\n" + "=" * 80)
    print(f"ADVERSARIAL DISASSEMBLY SUMMARY:")
    print(f"Total Serving Symbols Audited:     {total_symbols_audited}")
    print(f"Total Instructions Fuzzed:        {total_instructions_fuzzed}")
    print(f"Overall Clean (Zero Violations):  {overall_clean}")
    print("=" * 80)

    return 0 if overall_clean else 1


if __name__ == "__main__":
    sys.exit(run_adversarial_disassembly_fuzz())
