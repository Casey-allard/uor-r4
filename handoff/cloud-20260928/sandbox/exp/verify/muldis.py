#!/usr/bin/env python3
"""Scratch x86-64 per-symbol multiply/divide/float census for the uor-r4-integer binary.

Reads `objdump -d -C --no-show-raw-insn <bin>` output, splits by function symbol, and
reports, for symbols matching the given substrings, counts of:
  - integer multiply (imul, mul, mulx) and SIMD integer multiplies (pmul*, pmadd*)
  - integer divide (div, idiv)
  - floating arithmetic/conversion (x87, SSE/AVX scalar/packed float ops, cvt*)
and prints each hit with its operand text so it can be classified (address vs value).
Static presence is reported, not execution counts.
"""
import re
import subprocess
import sys

binary = sys.argv[1]
patterns = sys.argv[2:] or ["uor_r4_integer::"]

MUL = re.compile(r"^(imul|mul|mulx|pmul\w*|vpmul\w*|pmadd\w*|vpmadd\w*)$")
DIV = re.compile(r"^(div|idiv)$")
FLT = re.compile(
    r"^(v?(add|sub|mul|div|sqrt|min|max|fmadd\w*|fmsub\w*|fnmadd\w*|fnmsub\w*)(ss|sd|ps|pd)|v?cvt\w+|f\w+)$"
)

out = subprocess.run(
    ["objdump", "-d", "-C", "--no-show-raw-insn", binary],
    capture_output=True, text=True, check=True,
).stdout

sym_re = re.compile(r"^([0-9a-f]+) <(.*)>:$")
ins_re = re.compile(r"^\s*([0-9a-f]+):\s+(\S+)\s*(.*)$")
current = None
data = {}
for line in out.splitlines():
    m = sym_re.match(line)
    if m:
        current = m.group(2)
        data.setdefault(current, [])
        continue
    if current is None:
        continue
    m = ins_re.match(line)
    if m:
        data[current].append((m.group(1), m.group(2), m.group(3)))

selected = {k: v for k, v in data.items() if any(p in k for p in patterns)}
tot = {"ins": 0, "mul": 0, "div": 0, "flt": 0}
for name in sorted(selected):
    ins = selected[name]
    muls = [i for i in ins if MUL.match(i[1])]
    divs = [i for i in ins if DIV.match(i[1])]
    flts = [i for i in ins if FLT.match(i[1]) and not i[1].startswith(("fs", "fence"))]
    tot["ins"] += len(ins)
    tot["mul"] += len(muls)
    tot["div"] += len(divs)
    tot["flt"] += len(flts)
    if muls or divs or flts:
        print(f"== {name[:200]}  ({len(ins)} instructions)")
        for a, mn, ops in muls + divs + flts:
            print(f"   {a}: {mn} {ops}")
print(f"\nSYMBOLS={len(selected)} INSTRUCTIONS={tot['ins']} MUL={tot['mul']} DIV={tot['div']} FLOAT={tot['flt']}")
