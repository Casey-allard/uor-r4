#!/usr/bin/env python3
"""Static instruction audit of a compiled WebAssembly module for the D11 classes.

For each audited symbol the tool decodes every instruction of the function body
and counts:
- Class I: integer multiply (`i32.mul`, `i64.mul`);
- Class II: integer divide and remainder (`i32.div_*`, `i32.rem_*`, `i64.div_*`,
  `i64.rem_*`);
- Class III: floating point (`f32.*`, `f64.*`, float loads/stores/constants,
  conversions and reinterpretations, and the 0xFC 0..7 saturating
  float-to-integer truncations);
- unclassified: every other 0xFC-prefixed instruction (bulk memory and table
  operations), every 0xFD-prefixed (SIMD) and every 0xFE-prefixed (atomic)
  instruction. These are reported as findings and fail the symbol; they are
  never silently skipped.

Scope. By default only the bodies of the matched root functions are decoded:
direct calls are NOT followed, so a clean root says nothing about its callees.
`--call-graph` follows direct calls (`call`, `return_call`) transitively. As in
`audit_zero_matmul_serving.py`, callees matching the allocation, formatting,
panic, I/O, tokenizer and loading allowlist are not audited. Indirect call
targets (`call_indirect`, `call_ref`) cannot be resolved statically; their
sites are counted and reported, not followed.

An unknown opcode, a truncated immediate or a body that does not end with `end`
is a decode error, and the symbol fails. The build commit is an input
(`--build-commit`); the tool never reads the current working tree, because the
checked-out HEAD is not evidence of the commit that built the artifact.

A passing result is a scoped static measurement of the named symbols in one
artifact. It is not a serving-runtime qualification.

Usage:
    python3 scripts/audit_zero_matmul_wasm.py WASM [--build-commit SHA] [--tap]
        [--call-graph] [--json]
    python3 scripts/audit_zero_matmul_wasm.py --self-test
"""

import argparse
import hashlib
import json
import os
import re
import sys
from collections import Counter

UNRECORDED = "UNRECORDED"

# ---------------------------------------------------------------------------
# Instruction classes
# ---------------------------------------------------------------------------

CLASS_I_OPS = {0x6C, 0x7E}  # i32.mul, i64.mul
CLASS_II_OPS = {
    0x6D,  # i32.div_s
    0x6E,  # i32.div_u
    0x6F,  # i32.rem_s
    0x70,  # i32.rem_u
    0x7F,  # i64.div_s
    0x80,  # i64.div_u
    0x81,  # i64.rem_s
    0x82,  # i64.rem_u
}
CLASS_III_OPS = (
    {0x2A, 0x2B, 0x38, 0x39, 0x43, 0x44}  # f32/f64 load, store, const
    | set(range(0x5B, 0x67))  # f32/f64 comparisons
    | set(range(0x8B, 0xA7))  # f32/f64 arithmetic
    | set(range(0xA8, 0xAC))  # i32.trunc_f32/f64
    | set(range(0xAE, 0xC0))  # i64.trunc, convert, demote/promote, reinterpret
)
# 0xFC 0..7: i32/i64.trunc_sat_f32/f64_s/u (float to integer conversions).
FC_FLOAT_SUBOPS = set(range(0, 8))

# Opcodes whose instruction has no immediate operand.
NO_IMMEDIATE_OPS = {0x00, 0x01, 0x05, 0x0B, 0x0F, 0x1A, 0x1B, 0xD1} | set(
    range(0x45, 0xC5)
)
VALTYPE_BYTES = {0x7F, 0x7E, 0x7D, 0x7C, 0x7B, 0x70, 0x6F}


class DecodeError(Exception):
    """The byte stream is not a WebAssembly encoding this auditor can decode."""


def read_u(data, pos, end):
    """Unsigned LEB128."""
    result = 0
    shift = 0
    while True:
        if pos >= end:
            raise DecodeError(f"truncated unsigned LEB128 at byte {pos}")
        byte = data[pos]
        pos += 1
        result |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return result, pos
        shift += 7
        if shift > 63:
            raise DecodeError(f"unsigned LEB128 longer than 64 bits at byte {pos}")


def read_s(data, pos, end):
    """Signed LEB128."""
    result = 0
    shift = 0
    while True:
        if pos >= end:
            raise DecodeError(f"truncated signed LEB128 at byte {pos}")
        byte = data[pos]
        pos += 1
        result |= (byte & 0x7F) << shift
        shift += 7
        if not byte & 0x80:
            if byte & 0x40:
                result -= 1 << shift
            return result, pos
        if shift > 70:
            raise DecodeError(f"signed LEB128 longer than 64 bits at byte {pos}")


def skip(pos, count, end):
    pos += count
    if pos > end:
        raise DecodeError(f"immediate runs past the function body end at byte {end}")
    return pos


def read_blocktype(data, pos, end):
    if pos >= end:
        raise DecodeError(f"truncated block type at byte {pos}")
    if data[pos] == 0x40 or data[pos] in VALTYPE_BYTES:
        return pos + 1
    _, pos = read_s(data, pos, end)  # s33 type index (multi-value block)
    return pos


def read_memarg(data, pos, end):
    align, pos = read_u(data, pos, end)
    if align & 0x40:  # multi-memory: an explicit memory index follows
        _, pos = read_u(data, pos, end)
    _, pos = read_u(data, pos, end)  # offset
    return pos


class FunctionScan:
    def __init__(self):
        self.instructions = 0
        self.class_i = 0
        self.class_ii = 0
        self.class_iii = 0
        self.unclassified = Counter()
        self.direct_calls = set()
        self.direct_call_sites = 0
        self.indirect_call_sites = 0


def scan_function(data, start, end):
    """Decode one code-section function body `data[start:end]`."""
    pos = start
    groups, pos = read_u(data, pos, end)
    for _ in range(groups):
        _, pos = read_u(data, pos, end)  # local count
        pos = skip(pos, 1, end)  # value type
    scan = FunctionScan()
    last_op = None
    while pos < end:
        at = pos
        op = data[pos]
        pos += 1
        scan.instructions += 1
        last_op = op
        sub = None
        if op in NO_IMMEDIATE_OPS:
            pass
        elif op in (0x02, 0x03, 0x04):  # block, loop, if
            pos = read_blocktype(data, pos, end)
        elif op in (0x0C, 0x0D):  # br, br_if
            _, pos = read_u(data, pos, end)
        elif op == 0x0E:  # br_table
            count, pos = read_u(data, pos, end)
            for _ in range(count + 1):
                _, pos = read_u(data, pos, end)
        elif op in (0x10, 0x12):  # call, return_call
            callee, pos = read_u(data, pos, end)
            scan.direct_calls.add(callee)
            scan.direct_call_sites += 1
        elif op in (0x11, 0x13):  # call_indirect, return_call_indirect
            _, pos = read_u(data, pos, end)  # type index
            _, pos = read_u(data, pos, end)  # table index
            scan.indirect_call_sites += 1
        elif op in (0x14, 0x15):  # call_ref, return_call_ref
            _, pos = read_u(data, pos, end)
            scan.indirect_call_sites += 1
        elif op == 0x1C:  # select with explicit value types
            count, pos = read_u(data, pos, end)
            pos = skip(pos, count, end)
        elif 0x20 <= op <= 0x26:  # local/global get/set/tee, table.get/set
            _, pos = read_u(data, pos, end)
        elif 0x28 <= op <= 0x3E:  # loads and stores
            pos = read_memarg(data, pos, end)
        elif op in (0x3F, 0x40):  # memory.size, memory.grow
            _, pos = read_u(data, pos, end)
        elif op == 0x41:  # i32.const
            _, pos = read_s(data, pos, end)
        elif op == 0x42:  # i64.const
            _, pos = read_s(data, pos, end)
        elif op == 0x43:  # f32.const
            pos = skip(pos, 4, end)
        elif op == 0x44:  # f64.const
            pos = skip(pos, 8, end)
        elif op == 0xD0:  # ref.null heap type
            _, pos = read_s(data, pos, end)
        elif op == 0xD2:  # ref.func (address taken; not a call)
            _, pos = read_u(data, pos, end)
        elif op == 0xFC:
            sub, pos = read_u(data, pos, end)
            if sub <= 7:  # trunc_sat
                pass
            elif sub == 8:  # memory.init data, memory
                _, pos = read_u(data, pos, end)
                _, pos = read_u(data, pos, end)
            elif sub in (9, 13):  # data.drop, elem.drop
                _, pos = read_u(data, pos, end)
            elif sub in (10, 12, 14):  # memory.copy, table.init, table.copy
                _, pos = read_u(data, pos, end)
                _, pos = read_u(data, pos, end)
            elif sub in (11, 15, 16, 17):  # memory.fill (one memory index), table.*
                _, pos = read_u(data, pos, end)
            else:
                raise DecodeError(f"unknown 0xFC sub-opcode {sub} at byte {at}")
        elif op == 0xFD:
            sub, pos = read_u(data, pos, end)
            if sub <= 11 or sub in (92, 93):  # v128 loads/stores, load_zero
                pos = read_memarg(data, pos, end)
            elif sub in (12, 13):  # v128.const, i8x16.shuffle
                pos = skip(pos, 16, end)
            elif 21 <= sub <= 34:  # extract_lane / replace_lane
                pos = skip(pos, 1, end)
            elif 84 <= sub <= 91:  # load/store lane
                pos = read_memarg(data, pos, end)
                pos = skip(pos, 1, end)
            # every other SIMD instruction has no immediate
        elif op == 0xFE:
            sub, pos = read_u(data, pos, end)
            if sub == 0x03:  # atomic.fence
                pos = skip(pos, 1, end)
            elif sub <= 0x02 or 0x10 <= sub <= 0x4E:
                pos = read_memarg(data, pos, end)
            else:
                raise DecodeError(f"unknown 0xFE sub-opcode {sub} at byte {at}")
        else:
            raise DecodeError(f"unsupported opcode 0x{op:02X} at byte {at}")

        if op in CLASS_I_OPS:
            scan.class_i += 1
        elif op in CLASS_II_OPS:
            scan.class_ii += 1
        elif op in CLASS_III_OPS:
            scan.class_iii += 1
        elif op == 0xFC and sub in FC_FLOAT_SUBOPS:
            scan.class_iii += 1
        elif op in (0xFC, 0xFD, 0xFE):
            scan.unclassified[f"0x{op:02X} {sub}"] += 1
    if pos != end:
        raise DecodeError(f"instruction stream overran the function body end at byte {end}")
    if last_op != 0x0B:
        raise DecodeError(f"function body ending at byte {end} does not end with `end`")
    return scan


# ---------------------------------------------------------------------------
# Module structure
# ---------------------------------------------------------------------------


class Module:
    def __init__(self, data):
        self.data = data
        self.imported_functions = 0
        self.bodies = []  # (start, end) of each defined function body
        self.names = {}  # function index -> [export and name-section names]
        self._scans = {}
        self._parse()

    def _parse(self):
        data = self.data
        if len(data) < 8 or data[:4] != b"\x00asm":
            raise DecodeError("missing WebAssembly magic header")
        pos = 8
        code = None
        while pos < len(data):
            section = data[pos]
            pos += 1
            length, pos = read_u(data, pos, len(data))
            end = pos + length
            if end > len(data):
                raise DecodeError(f"section {section} runs past the end of the file")
            if section == 2:
                self._parse_imports(pos, end)
            elif section == 7:
                self._parse_exports(pos, end)
            elif section == 10:
                code = (pos, end)
            elif section == 0:
                self._parse_custom(pos, end)
            pos = end
        if code is None:
            raise DecodeError("no code section")
        pos, end = code
        count, pos = read_u(data, pos, end)
        for _ in range(count):
            size, pos = read_u(data, pos, end)
            if pos + size > end:
                raise DecodeError("function body runs past the code section")
            self.bodies.append((pos, pos + size))
            pos += size

    def _parse_limits(self, pos, end):
        flags, pos = read_u(self.data, pos, end)
        _, pos = read_u(self.data, pos, end)
        if flags & 1:
            _, pos = read_u(self.data, pos, end)
        return pos

    def _parse_imports(self, pos, end):
        data = self.data
        count, pos = read_u(data, pos, end)
        for _ in range(count):
            for _ in range(2):  # module name, field name
                length, pos = read_u(data, pos, end)
                pos = skip(pos, length, end)
            kind = data[pos]
            pos += 1
            if kind == 0:  # function
                _, pos = read_u(data, pos, end)
                self.imported_functions += 1
            elif kind == 1:  # table
                pos = skip(pos, 1, end)
                pos = self._parse_limits(pos, end)
            elif kind == 2:  # memory
                pos = self._parse_limits(pos, end)
            elif kind == 3:  # global
                pos = skip(pos, 2, end)
            elif kind == 4:  # tag
                pos = skip(pos, 1, end)
                _, pos = read_u(data, pos, end)
            else:
                raise DecodeError(f"unknown import kind {kind}")

    def _parse_exports(self, pos, end):
        data = self.data
        count, pos = read_u(data, pos, end)
        for _ in range(count):
            length, pos = read_u(data, pos, end)
            name = data[pos : pos + length].decode("utf-8", errors="replace")
            pos = skip(pos, length, end)
            kind = data[pos]
            pos += 1
            index, pos = read_u(data, pos, end)
            if kind == 0:
                self.names.setdefault(index, []).append(name)

    def _parse_custom(self, pos, end):
        data = self.data
        length, pos = read_u(data, pos, end)
        if data[pos : pos + length] != b"name":
            return
        pos += length
        while pos < end:
            subsection = data[pos]
            pos += 1
            size, pos = read_u(data, pos, end)
            sub_end = pos + size
            if subsection == 1:  # function names
                count, q = read_u(data, pos, sub_end)
                for _ in range(count):
                    index, q = read_u(data, q, sub_end)
                    name_length, q = read_u(data, q, sub_end)
                    name = data[q : q + name_length].decode("utf-8", errors="replace")
                    q = skip(q, name_length, sub_end)
                    names = self.names.setdefault(index, [])
                    if name not in names:
                        names.append(name)
            pos = sub_end

    def is_defined(self, index):
        return self.imported_functions <= index < self.imported_functions + len(self.bodies)

    def scan(self, index):
        """Decoded scan of a defined function, or a DecodeError (cached)."""
        if index not in self._scans:
            start, end = self.bodies[index - self.imported_functions]
            try:
                self._scans[index] = scan_function(self.data, start, end)
            except DecodeError as error:
                self._scans[index] = error
        return self._scans[index]

    def raw_name(self, index):
        names = self.names.get(index)
        return names[0] if names else f"<function {index}>"


def display_name(raw):
    """Readable tail (last three path components) of a legacy or v0 mangled name."""
    if raw.startswith("_R"):
        # Drop v0 crate and impl disambiguators (`Cs<base62>_`, `Ms1_`, `Xs_`).
        raw = re.sub(r"(?<=[CMXY])s[0-9A-Za-z]*_", "", raw)
    tokens = []
    i, n = 0, len(raw)
    at_boundary = True  # a length prefix may start here
    while i < n:
        if raw[i].isdigit() and (at_boundary or not raw[i - 1].isdigit()):
            j = i
            while j < n and raw[j].isdigit():
                j += 1
            length = int(raw[i:j])
            k = j + 1 if j < n and raw[j] == "_" else j
            ident = raw[k : k + length]
            if (
                length
                and len(ident) == length
                and (ident[0].isalpha() or ident[0] == "_")
                and all(c.isalnum() or c == "_" for c in ident)
            ):
                tokens.append(ident)
                i = k + length
                at_boundary = True
                continue
            i = j
        else:
            i += 1
        at_boundary = False
    return "::".join(tokens[-3:]) if tokens else raw


# Mirrors the call-graph allowlist of scripts/audit_zero_matmul_serving.py,
# written to match legacy/v0 mangled names (`4core3fmt`) as well as demangled
# ones (`core::fmt`). Matching callees are not audited.
CALL_GRAPH_ALLOW_PATTERNS = [
    r"alloc",
    r"free",
    r"realloc",
    r"core(?:::|\d+)fmt",
    r"panic",
    r"fmt(?:::|\d+)Display",
    r"fmt(?:::|\d+)Debug",
    r"unwind",
    r"rust_begin_unwind",
    r"std(?:::|\d+)io",
    r"std(?:::|\d+)panicking",
    r"Bundle",
    r"[Tt]okenizer",
    r"uor_r4_tokenizer",
    r"core(?:::|\d+)str",
    r"load",
    r"from_file",
    r"from_serialized",
]


def is_allowlisted(names):
    return any(re.search(p, name) for p in CALL_GRAPH_ALLOW_PATTERNS for name in names)


# ---------------------------------------------------------------------------
# Audited symbols
# ---------------------------------------------------------------------------

# Roots are matched by substring against export and name-section names. The
# thirteen `uor_wasm_*` helpers are the functions `wasm.rs` defines; the module
# exports no load, session or step entry point. The model step is audited as
# its own root because no helper reaches it.
MANDATORY_WASM_SYMBOLS = [
    ("uor_wasm_version", "uor_wasm_version"),
    ("uor_wasm_state_dim", "uor_wasm_state_dim"),
    ("uor_wasm_vocab_size", "uor_wasm_vocab_size"),
    ("uor_wasm_context_capacity", "uor_wasm_context_capacity"),
    ("uor_wasm_exact_min_p_threshold", "uor_wasm_exact_min_p_threshold"),
    ("uor_wasm_hopf_project_x", "uor_wasm_hopf_project_x"),
    ("uor_wasm_hopf_project_y", "uor_wasm_hopf_project_y"),
    ("uor_wasm_hopf_project_z", "uor_wasm_hopf_project_z"),
    ("uor_wasm_step_zeta_phase_scalar", "uor_wasm_step_zeta_phase_scalar"),
    ("uor_wasm_galois_lfsr_step", "uor_wasm_galois_lfsr_step"),
    ("uor_wasm_compute_turn_prime_signature", "uor_wasm_compute_turn_prime_signature"),
    ("uor_wasm_atan2_q30", "uor_wasm_atan2_q30"),
    ("uor_wasm_score_token_salience_default_key", "uor_wasm_score_token_salience_default_key"),
    ("IntegerModel::project_vocab", "IntegerModel13project_vocab"),
    ("IntegerModel::project_vocab_with_products", "IntegerModel27project_vocab_with_products"),
    ("low_bit_dot_8_pair_tables", "low_bit_dot_8_pair_tables"),
    ("low_bit_dot_4_contiguous_wide_pair_tables", "low_bit_dot_4_contiguous_wide_pair_t"),
    ("packed_rows::low_bit_dot", "packed_rows11low_bit_dot"),
    ("sampling::exact_min_p_threshold", "sampling21exact_min_p_threshold"),
    ("math::atan2_q30", "math9atan2_q30"),
    ("IntegerModel::step_conversational_into", "IntegerModel24step_conversational_into"),
]


def audit_symbol(module, name, pattern, follow_calls):
    roots = [
        index
        for index, names in sorted(module.names.items())
        if module.is_defined(index) and any(pattern in n for n in names)
    ]
    result = {
        "name": name,
        "pattern": pattern,
        "root_functions": roots,
        "audited_functions": 0,
        "instructions": 0,
        "mul": 0,
        "div": 0,
        "float": 0,
        "unclassified": 0,
        "unclassified_opcodes": {},
        "direct_call_sites_not_followed": 0,
        "allowlisted_callees_not_audited": 0,
        "imported_callees_not_audited": 0,
        "indirect_call_sites_not_followed": 0,
        "decode_errors": [],
        "findings": [],
    }
    if not roots:
        result["status"] = "NOT_FOUND"
        return result

    unclassified = Counter()
    visited = set(roots)
    queue = list(roots)
    allowlisted = set()
    imported = set()
    while queue:
        index = queue.pop(0)
        scan = module.scan(index)
        label = f"fn[{index}] {display_name(module.raw_name(index))}"
        if isinstance(scan, DecodeError):
            result["decode_errors"].append(f"{label}: {scan}")
            continue
        result["audited_functions"] += 1
        result["instructions"] += scan.instructions
        result["mul"] += scan.class_i
        result["div"] += scan.class_ii
        result["float"] += scan.class_iii
        unclassified.update(scan.unclassified)
        result["indirect_call_sites_not_followed"] += scan.indirect_call_sites
        if scan.class_i or scan.class_ii or scan.class_iii or scan.unclassified:
            result["findings"].append(
                {
                    "function": label,
                    "mul": scan.class_i,
                    "div": scan.class_ii,
                    "float": scan.class_iii,
                    "unclassified": sum(scan.unclassified.values()),
                }
            )
        if not follow_calls:
            result["direct_call_sites_not_followed"] += scan.direct_call_sites
            continue
        for callee in sorted(scan.direct_calls):
            if callee in visited:
                continue
            visited.add(callee)
            if not module.is_defined(callee):
                imported.add(callee)
            elif is_allowlisted(module.names.get(callee, [])):
                allowlisted.add(callee)
            else:
                queue.append(callee)
    result["allowlisted_callees_not_audited"] = len(allowlisted)
    result["imported_callees_not_audited"] = len(imported)
    result["unclassified"] = sum(unclassified.values())
    result["unclassified_opcodes"] = dict(sorted(unclassified.items()))
    if result["decode_errors"]:
        result["status"] = "DECODE_ERROR"
    elif result["mul"] or result["div"] or result["float"]:
        result["status"] = "FAIL"
    elif result["unclassified"]:
        result["status"] = "UNCLASSIFIED"
    else:
        result["status"] = "PASS"
    return result


def scope_text(follow_calls):
    if follow_calls:
        return (
            "direct calls followed transitively; allowlisted callees, imported "
            "callees and indirect call targets are not audited"
        )
    return "root function bodies only; direct calls are NOT followed"


def summarize(result, follow_calls):
    counts = (
        f"mul={result['mul']}, div={result['div']}, float={result['float']}, "
        f"unclassified={result['unclassified']}"
    )
    extent = f"{result['instructions']} instructions in {result['audited_functions']} fn(s)"
    if follow_calls:
        extent += (
            f"; {result['allowlisted_callees_not_audited']} allowlisted and "
            f"{result['imported_callees_not_audited']} imported callee(s) not audited"
        )
    else:
        extent += f"; {result['direct_call_sites_not_followed']} direct call site(s) not followed"
    extent += f"; {result['indirect_call_sites_not_followed']} indirect call site(s)"
    return counts, extent


def print_tap(meta, results, follow_calls):
    print("TAP version 13")
    print(f"1..{len(results)}")
    print(f"# Artifact: {meta['artifact']}")
    print(f"# Artifact SHA-256: {meta['sha256']}")
    print(f"# Artifact bytes: {meta['bytes']}")
    print(f"# Build commit: {meta['build_commit']} (supplied by --build-commit)")
    print(f"# Scope: {scope_text(follow_calls)}")
    for number, result in enumerate(results, 1):
        counts, extent = summarize(result, follow_calls)
        if result["status"] == "PASS":
            print(f"ok {number} - {result['name']}: {counts} ({extent})")
            continue
        if result["status"] == "NOT_FOUND":
            print(f"not ok {number} - {result['name']}: NOT_FOUND (pattern '{result['pattern']}')")
            continue
        print(f"not ok {number} - {result['name']}: {result['status']} ({counts}; {extent})")
        print("  ---")
        if result["unclassified_opcodes"]:
            print(f"  unclassified_opcodes: {json.dumps(result['unclassified_opcodes'])}")
        for error in result["decode_errors"][:5]:
            print(f"  decode_error: {error}")
        for finding in result["findings"][:10]:
            print(
                f"  - {finding['function']}: mul={finding['mul']} div={finding['div']} "
                f"float={finding['float']} unclassified={finding['unclassified']}"
            )
        if len(result["findings"]) > 10:
            print(f"  # {len(result['findings']) - 10} more function(s) with findings")
        print("  ...")


def print_text(meta, results, follow_calls):
    print("=" * 80)
    print("UOR-R4 WebAssembly instruction audit (D11 classes)")
    print(f"Artifact:     {meta['artifact']}")
    print(f"SHA-256:      {meta['sha256']}")
    print(f"Bytes:        {meta['bytes']}")
    print(f"Build commit: {meta['build_commit']}")
    print(f"Scope:        {scope_text(follow_calls)}")
    print("=" * 80)
    for result in results:
        counts, extent = summarize(result, follow_calls)
        print(f"[{result['status']}] {result['name']}: {counts} ({extent})")
    passed = sum(1 for r in results if r["status"] == "PASS")
    print("-" * 80)
    print(f"{passed}/{len(results)} symbols clean within the stated scope")


# ---------------------------------------------------------------------------
# Decoder sentinels
# ---------------------------------------------------------------------------


def _uleb(value):
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            out.append(byte | 0x80)
        else:
            out.append(byte)
            return bytes(out)


def _section(section_id, payload):
    return bytes([section_id]) + _uleb(len(payload)) + payload


def _name(text):
    raw = text.encode()
    return _uleb(len(raw)) + raw


def run_self_test():
    """Decoder sentinels; returns a list of failure messages (empty when clean)."""
    failures = []

    def check(label, body, expect):
        data = bytes(body)
        try:
            scan = scan_function(data, 0, len(data))
        except DecodeError as error:
            if expect != "decode_error":
                failures.append(f"{label}: unexpected decode error: {error}")
            return
        if expect == "decode_error":
            failures.append(f"{label}: expected a decode error")
            return
        got = {
            "mul": scan.class_i,
            "div": scan.class_ii,
            "float": scan.class_iii,
            "unclassified": sum(scan.unclassified.values()),
            "calls": sorted(scan.direct_calls),
        }
        for key, value in expect.items():
            if got[key] != value:
                failures.append(f"{label}: {key}={got[key]}, expected {value}")

    none = {"mul": 0, "div": 0, "float": 0, "unclassified": 0}
    # memory.fill has one memory-index immediate; the following i32.mul must be seen.
    check("memory.fill then i32.mul", [0, 0xFC, 11, 0, 0x6C, 0x0B], {**none, "mul": 1, "unclassified": 1})
    check("memory.copy then i64.mul", [0, 0xFC, 10, 0, 0, 0x7E, 0x0B], {**none, "mul": 1, "unclassified": 1})
    # A SIMD splat has no immediate; the following i64.mul must be seen.
    check("i32x4.splat then i64.mul", [0, 0xFD, 17, 0x7E, 0x0B], {**none, "mul": 1, "unclassified": 1})
    # extract_lane has one lane byte and no memarg.
    check("i32x4.extract_lane then i32.div_s", [0, 0xFD, 27, 0, 0x6D, 0x0B], {**none, "div": 1, "unclassified": 1})
    # A multi-byte SIMD sub-opcode (i32x4.mul = 181) is an unclassified finding.
    check("i32x4.mul", [0, 0xFD, 0xB5, 0x01, 0x0B], {**none, "unclassified": 1})
    # v128.const immediates contain mul/div bytes that are not instructions.
    check("v128.const payload", [0, 0xFD, 12] + [0x6C] * 16 + [0x1A, 0x0B], {**none, "unclassified": 1})
    check("i32.trunc_sat_f32_s", [0, 0xFC, 0, 0x0B], {**none, "float": 1})
    check("f64.const payload", [0, 0x44] + [0x7E] * 8 + [0x1A, 0x0B], {**none, "float": 1})
    check("i64.const LEB payload", [0, 0x42, 0xFE, 0x00, 0x1A, 0x0B], none)
    check("i64.rem_u", [0, 0x82, 0x0B], {**none, "div": 1})
    check("i64.extend_i32_s is integer", [0, 0xAC, 0x0B], none)
    # A block with a type-index block type encoded as a two-byte s33.
    check("type-index block", [0, 0x02, 0x80, 0x01, 0x6C, 0x0B, 0x0B], {**none, "mul": 1})
    check("direct call", [0, 0x10, 0x85, 0x01, 0x0B], {**none, "calls": [133]})
    check("unknown opcode fails closed", [0, 0x06, 0x40, 0x0B], "decode_error")
    check("missing end fails closed", [0, 0x6C], "decode_error")
    check("truncated immediate fails closed", [0, 0x41, 0x80], "decode_error")

    # Module level: one imported function shifts the defined-function index space.
    types = _section(1, _uleb(1) + bytes([0x60, 0, 0]))
    imports = _section(2, _uleb(1) + _name("env") + _name("host") + bytes([0]) + _uleb(0))
    functions = _section(3, _uleb(2) + _uleb(0) + _uleb(0))
    exports = _section(7, _uleb(1) + _name("clean_root") + bytes([0]) + _uleb(2))
    body_mul = bytes([0, 0x6C, 0x0B])
    body_call = bytes([0, 0x10]) + _uleb(1) + bytes([0x0B])
    code = _section(
        10, _uleb(2) + _uleb(len(body_mul)) + body_mul + _uleb(len(body_call)) + body_call
    )
    module_bytes = b"\x00asm\x01\x00\x00\x00" + types + imports + functions + exports + code
    try:
        module = Module(module_bytes)
        root = audit_symbol(module, "clean_root", "clean_root", follow_calls=False)
        if root["root_functions"] != [2] or root["status"] != "PASS":
            failures.append(f"import offset: root-only result {root['status']} {root['root_functions']}")
        graph = audit_symbol(module, "clean_root", "clean_root", follow_calls=True)
        if graph["status"] != "FAIL" or graph["mul"] != 1 or graph["audited_functions"] != 2:
            failures.append(f"call graph: {graph['status']} mul={graph['mul']}")
    except DecodeError as error:
        failures.append(f"module sentinel: {error}")
    return failures


def main():
    parser = argparse.ArgumentParser(
        description="Scoped static audit of a WebAssembly module for D11 instruction classes."
    )
    parser.add_argument("wasm_path", nargs="?", help="Path to the .wasm artifact")
    parser.add_argument(
        "--build-commit",
        default=UNRECORDED,
        help=f"Commit that built the artifact (default: {UNRECORDED}); never read from HEAD",
    )
    parser.add_argument("--tap", action="store_true", help="Emit TAP version 13")
    parser.add_argument("--json", action="store_true", help="Emit machine-readable JSON")
    parser.add_argument(
        "--call-graph",
        action="store_true",
        help="Follow direct calls transitively from each audited symbol",
    )
    parser.add_argument("--self-test", action="store_true", help="Run decoder sentinels only")
    args = parser.parse_args()

    failures = run_self_test()
    if failures:
        print("Decoder sentinel failures:", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        sys.exit(2)
    if args.self_test:
        print("WASM auditor decoder sentinels: all passed")
        return
    if not args.wasm_path:
        parser.error("a .wasm artifact path is required")

    wasm_path = os.path.abspath(args.wasm_path)
    try:
        with open(wasm_path, "rb") as handle:
            data = handle.read()
    except OSError as error:
        print(f"Error: cannot read {wasm_path}: {error}", file=sys.stderr)
        sys.exit(2)
    try:
        module = Module(data)
    except DecodeError as error:
        print(f"Error: {wasm_path}: {error}", file=sys.stderr)
        sys.exit(2)

    meta = {
        "artifact": wasm_path,
        "sha256": hashlib.sha256(data).hexdigest(),
        "bytes": len(data),
        "build_commit": args.build_commit,
        "scope": scope_text(args.call_graph),
    }
    results = [
        audit_symbol(module, name, pattern, args.call_graph)
        for name, pattern in MANDATORY_WASM_SYMBOLS
    ]
    if args.json:
        print(json.dumps({"schema": "uor-r4.wasm-instruction-audit/1", **meta, "results": results}, indent=2))
    elif args.tap:
        print_tap(meta, results, args.call_graph)
    else:
        print_text(meta, results, args.call_graph)
    sys.exit(0 if all(r["status"] == "PASS" for r in results) else 1)


if __name__ == "__main__":
    main()
