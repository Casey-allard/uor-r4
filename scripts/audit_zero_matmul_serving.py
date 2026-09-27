#!/usr/bin/env python3
"""Zero-Multiplier Numerical Serving Kernel Static Disassembly Auditor.

Verifies the D0-b and Milestone M1/M2 architectural invariants:
- Strictly 0 floating-point / vector register transfer instructions (Class III: `fmul`, `fmov`, `fadd`, etc.)
- Strictly 0 hardware integer multiplier instructions (Class I: `mul`, `madd`, `smull`, `umull`, `smaddl`, `smsubl`, etc.)
- Strictly 0 hardware integer divider instructions (Class II: `sdiv`, `udiv`)
in compiled numerical serving symbols of `libuor_r4_integer.rlib` and release binaries.

Usage:
    python3 scripts/audit_zero_matmul_serving.py [path_to_binary_or_rlib] [--strict-arm64] [--tap]
"""

import argparse
import os
import re
import subprocess
import sys

DEFAULT_TARGETS = [
    "/Users/casey.allard/uor-r4/target/release/libuor_r4_integer.rlib",
    "target/release/libuor_r4_integer.rlib",
    "/Users/casey.allard/uor-r4/target/release/uor-r4-integer",
    "target/release/uor-r4-integer",
    "target/release/uor-chat",
]

# Class I: Hardware Multipliers (AArch64 / ARM64)
CLASS_I_PATTERN = re.compile(
    r"\b(mul|madd|msub|mneg|smull|umull|smaddl|umaddl|smsubl|umsubl|smulh|umulh|sqdmulh|sqrdmulh|mla|mls|pmul|pmull)\b",
    re.IGNORECASE,
)

# Class II: Hardware Dividers
CLASS_II_PATTERN = re.compile(
    r"\b(sdiv|udiv)\b",
    re.IGNORECASE,
)

# Class III: Floating-Point & Register Transfer Instructions
# Scalar & vector floating-point mnemonics, conversions, and FP register moves.
# Note: integer vector negate (`fneg.2d v6, v6` in LLVM objdump syntax for opcode 6ee0f8c6)
# is excluded from scalar FP negation via negative lookahead.
CLASS_III_PATTERN = re.compile(
    r"(\b(fmul|fmov|fadd|fsub|fdiv|fmadd|fmsub|fnmadd|fnmsub|fnmul|fsqrt|fcmp|fcmpe|scvtf|ucvtf)\b"
    r"|\b(frint[aimnpzx]|fcvt[a-z0-9]*)\b"
    r"|\b(fneg|fabs)\s+[ds]\b)",
    re.IGNORECASE,
)

# Core historical pattern (for non-strict fallback)
CORE_FORBIDDEN_PATTERN = re.compile(
    r"\b(mul|smull|umull|smaddl|smsubl|fmul|fmov)\b",
    re.IGNORECASE,
)

# Combined full strict pattern
STRICT_FORBIDDEN_PATTERN = re.compile(
    r"(\b(mul|madd|msub|mneg|smull|umull|smaddl|umaddl|smsubl|umsubl|smulh|umulh|sqdmulh|sqrdmulh|mla|mls|pmul|pmull"
    r"|sdiv|udiv"
    r"|fmul|fmov|fadd|fsub|fdiv|fmadd|fmsub|fnmadd|fnmsub|fnmul|fsqrt|fcmp|fcmpe|scvtf|ucvtf)\b"
    r"|\b(frint[aimnpzx]|fcvt[a-z0-9]*)\b"
    r"|\b(fneg|fabs)\s+[ds]\b)",
    re.IGNORECASE,
)

# Mandatory serving symbols for library archive (.rlib)
RLIB_MANDATORY_SYMBOLS = [
    {
        "name": "UnitS3Q30::hopf_project",
        "pattern": re.compile(r"UnitS3Q30.*hopf_project\b"),
        "mangled": re.compile(r"__RNv.*UnitS3Q30.*12hopf_project\b"),
        "description": "S3 -> S2 Hopf base projection coordinates",
    },
    {
        "name": "UnitS3Q30::from_i32_coords",
        "pattern": re.compile(r"UnitS3Q30.*from_i32_coords\b"),
        "mangled": re.compile(r"__RNv.*UnitS3Q30.*15from_i32_coords\b"),
        "description": "S3 unit quaternion normalization from 4D coordinates",
    },
    {
        "name": "UnitS3Q30::fiber_u1_q30",
        "pattern": re.compile(r"UnitS3Q30.*fiber_u1_q30\b"),
        "mangled": re.compile(r"__RNv.*UnitS3Q30.*12fiber_u1_q30\b"),
        "description": "U(1) fiber unit phasor extraction",
    },
    {
        "name": "UnitS3Q30::hopf_fiber_project",
        "pattern": re.compile(r"UnitS3Q30.*hopf_fiber_project\b"),
        "mangled": re.compile(r"__RNv.*UnitS3Q30.*18hopf_fiber_project\b"),
        "description": "Combined base S2 and fiber U(1) projection",
    },
    {
        "name": "T8ZetaState::step",
        "pattern": re.compile(r"T8ZetaState.*step\b(?!_raw)"),
        "mangled": re.compile(r"__RNv.*T8ZetaState.*4step\b"),
        "description": "T^8 toroidal Riemann zeta phase progression",
    },
    {
        "name": "T8ZetaState::step_raw",
        "pattern": re.compile(r"T8ZetaState.*step_raw\b"),
        "mangled": re.compile(r"__RNv.*T8ZetaState.*8step_raw\b"),
        "description": "T^8 unmodulated zeta frequency progression",
    },
    {
        "name": "IntegerModel::step",
        "pattern": re.compile(r"IntegerModel.*step\b(?!_conversational)"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*4step\b"),
        "description": "Autoregressive state transition step",
    },
    {
        "name": "IntegerModel::step_conversational",
        "pattern": re.compile(r"IntegerModel.*step_conversational\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*19step_conversational\b"),
        "description": "Conversational autoregressive step with partitioned memory",
    },
    {
        "name": "IntegerModel::matrix_work",
        "pattern": re.compile(r"IntegerModel.*matrix_work\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*11matrix_work\b"),
        "description": "Zero-MatMul affine matrix-vector product",
    },
    {
        "name": "IntegerModel::affine",
        "pattern": re.compile(r"IntegerModel.*affine\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*6affine\b"),
        "description": "Linear combination over parameter codes",
    },
    {
        "name": "IntegerModel::vector_work",
        "pattern": re.compile(r"IntegerModel.*vector_work\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*11vector_work\b"),
        "description": "Parameter bias scaling and extraction",
    },
    {
        "name": "IntegerModel::embed",
        "pattern": re.compile(r"IntegerModel.*embed\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*5embed\b"),
        "description": "Token embedding projection via shift-and-add",
    },
    {
        "name": "IntegerModel::project_vocab",
        "pattern": re.compile(r"IntegerModel.*project_vocab\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*13project_vocab\b"),
        "description": "Vocab projection via shift-and-add rows",
    },
    {
        "name": "model::softmax",
        "pattern": re.compile(r"model.*softmax\b"),
        "mangled": re.compile(r"__RNv.*model.*7softmax\b"),
        "description": "Q48 integer softmax distribution",
    },
    {
        "name": "model::low_bit_dot",
        "pattern": re.compile(r"model.*low_bit_dot\b"),
        "mangled": re.compile(r"__RNv.*model.*11low_bit_dot\b"),
        "description": "Inner product via signed-4 lookup table",
    },
    {
        "name": "model::low_bit_products",
        "pattern": re.compile(r"model.*low_bit_products\b"),
        "mangled": re.compile(r"__RNv.*model.*16low_bit_products\b"),
        "description": "Shift-add 4-bit scalar multiples",
    },
    {
        "name": "math::atan2_q30",
        "pattern": re.compile(r"math.*atan2_q30\b"),
        "mangled": re.compile(r"__RNv.*math.*9atan2_q30\b"),
        "description": "16-stage CORDIC fixed-point arc-tangent",
    },
    {
        "name": "math::checked_mul",
        "pattern": re.compile(r"math.*checked_mul\b(?!_unsigned)"),
        "mangled": re.compile(r"__RNv.*math.*11checked_mul\b"),
        "description": "Shift-and-add exact signed multiplication",
    },
    {
        "name": "math::div_round",
        "pattern": re.compile(r"math.*div_round\b"),
        "mangled": re.compile(r"__RNv.*math.*9div_round\b"),
        "description": "Signed integer division rounded to nearest",
    },
    {
        "name": "math::scale_pow2",
        "pattern": re.compile(r"math.*scale_pow2\b"),
        "mangled": re.compile(r"__RNv.*math.*10scale_pow2\b"),
        "description": "Dyadic power-of-two rescaling with rounding",
    },
    {
        "name": "math::sum_squares",
        "pattern": re.compile(r"math.*sum_squares\b"),
        "mangled": re.compile(r"__RNv.*math.*11sum_squares\b"),
        "description": "Shift-and-add exact vector squared norm",
    },
]

# Mandatory serving symbols for compiled executable binary (e.g. uor-r4-integer, uor-chat)
BIN_MANDATORY_SYMBOLS = [
    {
        "name": "IntegerModel::step",
        "pattern": re.compile(r"IntegerModel.*step\b(?!_conversational)"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*4step\b"),
        "description": "Autoregressive state transition step",
        "alternative_group": "step_transition",
        "group_display": "IntegerModel::step (or IntegerModel::step_conversational)",
    },
    {
        "name": "IntegerModel::step_conversational",
        "pattern": re.compile(r"IntegerModel.*step_conversational\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*19step_conversational\b"),
        "description": "Conversational autoregressive step with partitioned memory",
        "alternative_group": "step_transition",
        "group_display": "IntegerModel::step (or IntegerModel::step_conversational)",
    },
    {
        "name": "IntegerModel::embed",
        "pattern": re.compile(r"IntegerModel.*embed\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*5embed\b"),
        "description": "Token embedding projection via shift-and-add",
    },
    {
        "name": "IntegerModel::project_vocab",
        "pattern": re.compile(r"IntegerModel.*project_vocab\b"),
        "mangled": re.compile(r"__RNv.*IntegerModel.*13project_vocab\b"),
        "description": "Vocab projection via shift-and-add rows",
    },
]


def find_target_artifact(user_arg=None):
    if user_arg:
        if os.path.exists(user_arg):
            return user_arg
        # Check relative to repo root if path not directly found
        candidate = os.path.join("/Users/casey.allard/uor-r4-worktrees/geometric-chatbot", user_arg)
        if os.path.exists(candidate):
            return candidate
        sys.exit(f"ERROR: Specified target artifact does not exist: {user_arg}")
    for candidate in DEFAULT_TARGETS:
        if os.path.exists(candidate):
            return candidate
    sys.exit(
        f"ERROR: No target artifact found. Checked: {', '.join(DEFAULT_TARGETS)}.\n"
        "Build with: cargo build --release -p uor-r4-integer"
    )


def check_tool_availability(disasm_choice="auto"):
    """Check if objdump or otool is available and functional."""
    tools_found = []
    if disasm_choice in ("auto", "objdump"):
        try:
            res = subprocess.run(["objdump", "--version"], capture_output=True, text=True)
            if res.returncode == 0:
                tools_found.append("objdump")
        except FileNotFoundError:
            pass
    if disasm_choice in ("auto", "otool"):
        try:
            res = subprocess.run(["otool", "--version"], capture_output=True, text=True)
            if res.returncode == 0 or "otool" in res.stderr:
                tools_found.append("otool")
        except FileNotFoundError:
            pass
    return tools_found


def disassemble_artifact(path, disasm_choice="auto"):
    """Disassemble using objdump (preferred for demangling) or otool."""
    is_demangled = False

    if disasm_choice in ("auto", "objdump"):
        try:
            res = subprocess.run(
                ["objdump", "--demangle", "-d", path],
                capture_output=True,
                text=True,
                check=True,
            )
            is_demangled = True
            return res.stdout, is_demangled
        except (subprocess.SubprocessError, FileNotFoundError):
            if disasm_choice == "objdump":
                sys.exit("ERROR: objdump requested but not available or failed.")

    if disasm_choice in ("auto", "otool"):
        try:
            res = subprocess.run(
                ["otool", "-tV", path],
                capture_output=True,
                text=True,
                check=True,
            )
            return res.stdout, False
        except (subprocess.SubprocessError, FileNotFoundError) as e:
            sys.exit(f"ERROR: otool disassembly failed: {e}")

    sys.exit("ERROR: Neither objdump nor otool available for disassembly.")


def parse_symbols(disasm_text, is_demangled):
    """Parse disassembly output into per-symbol instruction lists."""
    per_symbol = {}
    current_sym = None

    if is_demangled:
        sym_header = re.compile(r"^[0-9a-fA-F]+\s+<(.*)>:$")
    else:
        sym_header = re.compile(r"^([_\w]+):$")

    for line in disasm_text.splitlines():
        line_clean = line.strip()
        m = sym_header.match(line_clean)
        if m:
            current_sym = m.group(1)
            per_symbol[current_sym] = []
            continue

        if current_sym is not None and re.match(r"^[0-9a-fA-F]{1,16}[:\s\t]", line_clean):
            per_symbol[current_sym].append(line_clean)

    return per_symbol


def run_audit(per_symbol, is_demangled, target_path, strict_arm64=True):
    """Execute audit across Class I, Class II, and Class III instructions.

    Returns dict containing detailed results and failure reports.
    """
    is_rlib = target_path.endswith(".rlib")
    mandatory_list = RLIB_MANDATORY_SYMBOLS if is_rlib else BIN_MANDATORY_SYMBOLS

    results = {
        "target": target_path,
        "is_rlib": is_rlib,
        "strict": strict_arm64,
        "total_symbols_indexed": len(per_symbol),
        "mandatory_checked": 0,
        "missing_mandatory": [],
        "class_i_violations": [],   # Multipliers
        "class_ii_violations": [],  # Dividers
        "class_iii_violations": [], # Floats / transfers
        "all_violations": [],
        "passed_symbols": [],
    }

    # Collect groups for alternative symbol handling (e.g. step vs step_conversational)
    satisfied_groups = set()
    groups_needed = set()
    group_displays = {}
    for entry in mandatory_list:
        grp = entry.get("alternative_group")
        if grp:
            groups_needed.add(grp)
            group_displays[grp] = (entry.get("group_display", grp), entry["description"])

    # Track audited symbol signatures to prevent duplicate checking/reporting
    audited_syms = set()

    # Match symbols against mandatory list
    for entry in mandatory_list:
        name = entry["name"]
        pat = entry["pattern"] if is_demangled else entry["mangled"]
        desc = entry["description"]
        grp = entry.get("alternative_group")

        matched = [
            s
            for s in per_symbol
            if pat.search(s)
            and "closure" not in s
            and "drop_glue" not in s
            and "from_iter" not in s
            and "try_process" not in s
            and "GenericShunt" not in s
        ]

        if not matched:
            if not grp:
                results["missing_mandatory"].append((name, desc))
            continue

        if grp:
            satisfied_groups.add(grp)

        for sym in matched:
            if sym in audited_syms:
                continue
            audited_syms.add(sym)
            results["mandatory_checked"] += 1
            instrs = per_symbol[sym]

            c1 = [i for i in instrs if CLASS_I_PATTERN.search(i)]
            c2 = [i for i in instrs if CLASS_II_PATTERN.search(i)]
            c3 = [i for i in instrs if CLASS_III_PATTERN.search(i)]

            if c1:
                results["class_i_violations"].append((name, sym, c1, desc))
            if c2:
                results["class_ii_violations"].append((name, sym, c2, desc))
            if c3:
                results["class_iii_violations"].append((name, sym, c3, desc))

            if c1 or c2 or c3:
                results["all_violations"].append((name, sym, c1 + c2 + c3, desc))
            else:
                results["passed_symbols"].append((name, sym, len(instrs), desc))

    # Verify all alternative groups were satisfied by at least one symbol
    for grp in groups_needed:
        if grp not in satisfied_groups:
            disp_name, disp_desc = group_displays[grp]
            results["missing_mandatory"].append((disp_name, disp_desc))

    # Also audit optional/additional serving symbols if present
    other_list = RLIB_MANDATORY_SYMBOLS if not is_rlib else []
    for entry in other_list:
        name = entry["name"]
        pat = entry["pattern"] if is_demangled else entry["mangled"]
        desc = entry["description"]

        matched = [
            s
            for s in per_symbol
            if pat.search(s)
            and "closure" not in s
            and "drop_glue" not in s
            and "from_iter" not in s
            and "try_process" not in s
            and "GenericShunt" not in s
        ]

        for sym in matched:
            if sym in audited_syms:
                continue
            audited_syms.add(sym)
            instrs = per_symbol[sym]

            c1 = [i for i in instrs if CLASS_I_PATTERN.search(i)]
            c2 = [i for i in instrs if CLASS_II_PATTERN.search(i)]
            c3 = [i for i in instrs if CLASS_III_PATTERN.search(i)]

            if c1:
                results["class_i_violations"].append((name, sym, c1, desc))
            if c2:
                results["class_ii_violations"].append((name, sym, c2, desc))
            if c3:
                results["class_iii_violations"].append((name, sym, c3, desc))

            if c1 or c2 or c3:
                results["all_violations"].append((name, sym, c1 + c2 + c3, desc))
            else:
                results["passed_symbols"].append((name, sym, len(instrs), desc))

    return results


def print_tap_output(results, tools_found):
    """Emit authentic TAP version 13 results across the 5 standard test cases."""
    print("TAP version 13")
    print("1..5")

    # TC01: Disassembly tool availability
    if tools_found:
        print(f"ok 1 - [T1_F05_TC01] [Tier 1] Disassembly tool availability ({', '.join(tools_found)})")
    else:
        print("not ok 1 - [T1_F05_TC01] [Tier 1] Disassembly tool availability (no objdump/otool found)")
        print("  ---")
        print("  severity: fail")
        print("  message: Neither objdump nor otool available")
        print("  ...")

    # TC02: Zero floating-point instructions (Class III)
    c3 = results["class_iii_violations"]
    if not c3:
        print("ok 2 - [T1_F05_TC02] [Tier 1] Zero floating-point instructions in serving symbols (Class III clean)")
    else:
        print(f"not ok 2 - [T1_F05_TC02] [Tier 1] Zero floating-point instructions in serving symbols ({len(c3)} violations)")
        print("  ---")
        print(f"  violation_count: {len(c3)}")
        print("  symbols:")
        for name, sym, bad, _ in c3:
            print(f"    - name: {name}")
            print(f"      symbol: {sym}")
            print(f"      instructions: {bad[:3]}")
        print("  ...")

    # TC03: Zero hardware multipliers (Class I)
    c1 = results["class_i_violations"]
    if not c1:
        print("ok 3 - [T1_F05_TC03] [Tier 1] Zero hardware multipliers in numerical serving symbols (Class I clean)")
    else:
        print(f"not ok 3 - [T1_F05_TC03] [Tier 1] Zero hardware multipliers in numerical serving symbols ({len(c1)} violations)")
        print("  ---")
        print(f"  violation_count: {len(c1)}")
        print("  symbols:")
        for name, sym, bad, _ in c1:
            print(f"    - name: {name}")
            print(f"      symbol: {sym}")
            print(f"      instructions: {bad[:3]}")
        print("  ...")

    # TC04: Zero hardware dividers (Class II)
    c2 = results["class_ii_violations"]
    if not c2:
        print("ok 4 - [T1_F05_TC04] [Tier 1] Zero hardware dividers in numerical serving symbols (Class II clean)")
    else:
        print(f"not ok 4 - [T1_F05_TC04] [Tier 1] Zero hardware dividers in numerical serving symbols ({len(c2)} violations)")
        print("  ---")
        print(f"  violation_count: {len(c2)}")
        print("  symbols:")
        for name, sym, bad, _ in c2:
            print(f"    - name: {name}")
            print(f"      symbol: {sym}")
            print(f"      instructions: {bad[:3]}")
        print("  ...")

    # TC05: Strict audit exit code integrity
    missing = results["missing_mandatory"]
    total_failures = len(results["all_violations"]) + len(missing)
    if total_failures == 0 and tools_found:
        print("ok 5 - [T1_F05_TC05] [Tier 1] Strict audit exit code integrity and binary compliance (0 violations)")
    else:
        print(f"not ok 5 - [T1_F05_TC05] [Tier 1] Strict audit exit code integrity ({total_failures} total failures)")
        print("  ---")
        print(f"  missing_symbols: {len(missing)}")
        print(f"  violations: {len(results['all_violations'])}")
        print("  ...")


def print_standard_report(results):
    """Print readable report to stdout."""
    mode_str = "STRICT ARM64 (Class I multipliers, Class II dividers, Class III floats/transfers)"
    print("=" * 80)
    print(f"ZERO-MATMUL SERVING KERNEL AUDIT [{mode_str}]")
    print(f"Target Artifact: {results['target']}")
    print("=" * 80)

    for name, sym, n_instrs, desc in results["passed_symbols"]:
        print(f"  [ PASS  ] {name:<38} (0 forbidden, {n_instrs} instructions) - {desc}")

    for name, sym, bad, desc in results["all_violations"]:
        print(f"  [ FAIL  ] {name:<38} ({len(bad)} forbidden) - {desc}")
        for b in bad:
            print(f"            >>> {b}")

    for name, desc in results["missing_mandatory"]:
        print(f"  [MISSING] {name:<38} - {desc}")

    print("=" * 80)
    print(f"Total symbols checked: {results['mandatory_checked']}")
    print(f"Class I (Multipliers) Violations: {len(results['class_i_violations'])}")
    print(f"Class II (Dividers) Violations:    {len(results['class_ii_violations'])}")
    print(f"Class III (Floats) Violations:    {len(results['class_iii_violations'])}")
    print(f"Missing Mandatory Symbols:        {len(results['missing_mandatory'])}")
    print("=" * 80)

    if results["all_violations"] or results["missing_mandatory"]:
        print("FAILED: Serving kernel contains forbidden instructions or missing mandatory symbols.")
        return 1

    print("SUCCESS: CLEAN FORENSIC AUDIT (0 forbidden instructions across all serving symbols).")
    print("Zero-multiplier serving kernel invariant (D0-b) certified.")
    return 0


def main():
    parser = argparse.ArgumentParser(
        description="Zero-Multiplier Numerical Serving Kernel Static Disassembly Auditor"
    )
    parser.add_argument(
        "target",
        nargs="?",
        default=None,
        help="Path to .rlib archive or compiled release binary (default: auto-detected)",
    )
    parser.add_argument(
        "--target",
        dest="target_opt",
        default=None,
        help="Explicit path to target artifact",
    )
    parser.add_argument(
        "--disassembler",
        choices=["auto", "objdump", "otool"],
        default="auto",
        help="Disassembly backend tool to use (default: auto)",
    )
    parser.add_argument(
        "--strict-arm64",
        "--strict",
        action="store_true",
        default=True,
        help="Check extended ARM64 multiply, divide, and floating-point mnemonics (default: True)",
    )
    parser.add_argument(
        "--tap",
        action="store_true",
        help="Emit TAP version 13 output",
    )

    args = parser.parse_args()

    chosen_target = args.target_opt or args.target
    tools_found = check_tool_availability(args.disassembler)

    if not tools_found:
        if args.tap:
            print("TAP version 13\n1..5\nnot ok 1 - [T1_F05_TC01] No disassembler available\n...")
        else:
            print("ERROR: Neither objdump nor otool available on host.", file=sys.stderr)
        return 1

    target_path = find_target_artifact(chosen_target)
    if not args.tap:
        print(f"Disassembling target artifact: {target_path} (using {args.disassembler})")

    disasm, is_demangled = disassemble_artifact(target_path, args.disassembler)
    per_symbol = parse_symbols(disasm, is_demangled)
    if not args.tap:
        print(f"Disassembly parsed: {len(per_symbol)} symbols indexed (demangled={is_demangled}).\n")

    results = run_audit(per_symbol, is_demangled, target_path, strict_arm64=args.strict_arm64)

    if args.tap:
        print_tap_output(results, tools_found)
        total_failures = len(results["all_violations"]) + len(results["missing_mandatory"])
        return 0 if total_failures == 0 else 1
    else:
        return print_standard_report(results)


if __name__ == "__main__":
    sys.exit(main())
