#!/usr/bin/env python3
"""Forbidden-construct scan over Lean CODE, ignoring comments and strings.

SPEC section 19 bans sorry / admit / native_decide / project axiom / unsafe /
partial / noncomputable on the proof path. A plain text search cannot express
that: `sorry` appears legitimately in doc comments that explain the ban, and a
gate that fires on its own documentation gets switched off, which is the same
failure as a gate that never fires.

This strips Lean block comments (nested `/- -/`, including `/-- -/`), line
comments (`--`), and string literals, then searches what remains.

The decisive audit is still `#print axioms` over the compiled environment
(Tools/axioms.py); this is defence in depth.
"""
import os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(ROOT)

def strip(src: str) -> str:
    """Blank out comments and string bodies, preserving line structure."""
    out, i, n, depth = [], 0, len(src), 0
    while i < n:
        two = src[i:i+2]
        if depth == 0 and two == '/-':
            depth, i = 1, i + 2
            out.append('  ')
            continue
        if depth > 0:
            if two == '/-':
                depth += 1; i += 2; out.append('  '); continue
            if two == '-/':
                depth -= 1; i += 2; out.append('  '); continue
            out.append('\n' if src[i] == '\n' else ' '); i += 1; continue
        if two == '--':
            while i < n and src[i] != '\n':
                out.append(' '); i += 1
            continue
        if src[i] == '"':
            out.append(' '); i += 1
            while i < n and src[i] != '"':
                if src[i] == '\\' and i + 1 < n:
                    out.append('  '); i += 2; continue
                out.append('\n' if src[i] == '\n' else ' '); i += 1
            if i < n:
                out.append(' '); i += 1
            continue
        out.append(src[i]); i += 1
    return ''.join(out)

BANNED = [
    (re.compile(r'(^|[^A-Za-z_.])(sorry|admit|native_decide)([^A-Za-z_]|$)'), 'placeholder'),
    (re.compile(r'^\s*axiom\s'), 'project-declared axiom'),
    (re.compile(r'^\s*(unsafe|partial)\s+(def|abbrev|instance|theorem)'), 'unsafe/partial'),
]

targets = sys.argv[1:] or ['WasmGemmGnaf']
hits = []
scanned = 0
for t in targets:
    walk = ([(os.path.dirname(t), None, [os.path.basename(t)])] if os.path.isfile(t)
            else os.walk(t))
    for r, _, fs in walk:
        for f in sorted(fs):
            if not f.endswith('.lean'):
                continue
            p = os.path.join(r, f)
            scanned += 1
            code = strip(open(p, errors='replace').read())
            for ln, line in enumerate(code.splitlines(), 1):
                for rx, why in BANNED:
                    if rx.search(line):
                        hits.append(f'{p}:{ln}: {why}: {line.strip()[:80]}')

if hits:
    print('FORBIDDEN CONSTRUCT ON THE PROOF PATH (SPEC 19):')
    for h in hits:
        print('  ' + h)
    sys.exit(1)
print(f'forbidden-construct scan clean: {scanned} modules (comments and strings excluded)')
