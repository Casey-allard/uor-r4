#!/usr/bin/env python3
"""Generate canonical::nfc UCD tables from the vendored UCD 15.1.0 data.

Reads ../data/ucd/15.1.0/{UnicodeData.txt,CompositionExclusions.txt,
DerivedNormalizationProps.txt} and emits
../crates/uor-addr/src/canonical/nfc/tables.rs.

The generated file is committed; this script regenerates it when the
UCD version pin bumps. The crate build does not depend on this script;
no Python is required to build or test uor-addr.

UCD parsing rules:
- UnicodeData.txt:
    field 0 = code point (hex)
    field 3 = canonical combining class (decimal, 0..254)
    field 5 = decomposition mapping; "<type> ..." prefix flags
              compatibility (we discard those — NFC uses canonical only)
- CompositionExclusions.txt: one code point per line, explicit
  composition exclusions (subset of Full_Composition_Exclusion).
- DerivedNormalizationProps.txt:
    Full_Composition_Exclusion (the union of script-specific +
        singleton + non-starter decomposition exclusions — the runtime
        composition-pair table excludes all of these).
    NFC_QC (Yes/No/Maybe property) — drives the quick_check fast path.

NFC compose pair construction (UAX #15 §3):
- For each code point X whose canonical decomposition is [A, B]
  (two code points exactly), if X is NOT in Full_Composition_Exclusion,
  emit pair (A, B) → X.
- Hangul composition is algorithmic (UAX #15 §3.12) — not stored.
"""

from __future__ import annotations

import sys
from pathlib import Path

UCD_VERSION = "15.1.0"
UCD_DIR = Path(__file__).resolve().parent.parent / "data" / "ucd" / UCD_VERSION
OUT_FILE = (
    Path(__file__).resolve().parent.parent
    / "crates"
    / "uor-addr"
    / "src"
    / "canonical"
    / "nfc"
    / "tables.rs"
)

# Hangul algorithmic range (UAX #15 §3.12).
HANGUL_S_BASE = 0xAC00
HANGUL_S_COUNT = 11172
HANGUL_S_LAST = HANGUL_S_BASE + HANGUL_S_COUNT - 1


def parse_unicode_data() -> tuple[dict[int, int], dict[int, list[int]]]:
    ccc: dict[int, int] = {}
    decomp: dict[int, list[int]] = {}
    with (UCD_DIR / "UnicodeData.txt").open() as fh:
        for line in fh:
            line = line.rstrip("\n")
            if not line or line.startswith("#"):
                continue
            fields = line.split(";")
            cp = int(fields[0], 16)
            ccc_val = int(fields[3])
            if ccc_val != 0:
                ccc[cp] = ccc_val
            decomp_field = fields[5].strip()
            if decomp_field and not decomp_field.startswith("<"):
                # Canonical decomposition only — compat decomps (with
                # <type> prefix) are excluded.
                decomp[cp] = [int(p, 16) for p in decomp_field.split()]
    return ccc, decomp


def parse_full_composition_exclusion() -> set[int]:
    excluded: set[int] = set()
    with (UCD_DIR / "DerivedNormalizationProps.txt").open() as fh:
        for line in fh:
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            line_no_comment = line.split("#", 1)[0].strip()
            if not line_no_comment:
                continue
            parts = [p.strip() for p in line_no_comment.split(";")]
            if len(parts) < 2 or parts[1] != "Full_Composition_Exclusion":
                continue
            range_field = parts[0]
            if ".." in range_field:
                start_s, end_s = range_field.split("..")
                start, end = int(start_s, 16), int(end_s, 16)
            else:
                start = end = int(range_field, 16)
            for cp in range(start, end + 1):
                excluded.add(cp)
    return excluded


def parse_nfc_qc() -> tuple[set[int], set[int]]:
    """Return (NFC_QC=No code points, NFC_QC=Maybe code points)."""
    qc_no: set[int] = set()
    qc_maybe: set[int] = set()
    with (UCD_DIR / "DerivedNormalizationProps.txt").open() as fh:
        for line in fh:
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            line_no_comment = line.split("#", 1)[0].strip()
            if not line_no_comment:
                continue
            parts = [p.strip() for p in line_no_comment.split(";")]
            if len(parts) < 3 or parts[1] != "NFC_QC":
                continue
            value = parts[2]
            target = (
                qc_no if value == "N" else qc_maybe if value == "M" else None
            )
            if target is None:
                continue
            range_field = parts[0]
            if ".." in range_field:
                start_s, end_s = range_field.split("..")
                start, end = int(start_s, 16), int(end_s, 16)
            else:
                start = end = int(range_field, 16)
            for cp in range(start, end + 1):
                target.add(cp)
    return qc_no, qc_maybe


def fully_decompose(
    cp: int, decomp: dict[int, list[int]], cache: dict[int, list[int]]
) -> list[int]:
    """Recursively expand cp's canonical decomposition to its NFD form."""
    if cp in cache:
        return cache[cp]
    if cp not in decomp:
        cache[cp] = [cp]
        return cache[cp]
    out: list[int] = []
    for child in decomp[cp]:
        out.extend(fully_decompose(child, decomp, cache))
    cache[cp] = out
    return out


def build_composition_pairs(
    decomp: dict[int, list[int]], excluded: set[int]
) -> list[tuple[int, int, int]]:
    """Build the NFC composition table: (starter, mark, composed) triples."""
    pairs: list[tuple[int, int, int]] = []
    for cp, mapping in decomp.items():
        # NFC composes only pair decompositions; longer decompositions
        # are composed pairwise during the runtime compose stage by
        # consulting this same table iteratively.
        if len(mapping) != 2:
            continue
        if cp in excluded:
            continue
        pairs.append((mapping[0], mapping[1], cp))
    pairs.sort()
    return pairs


def format_rust(
    ccc: dict[int, int],
    decomp: dict[int, list[int]],
    composition: list[tuple[int, int, int]],
    qc_no: set[int],
    qc_maybe: set[int],
) -> str:
    cache: dict[int, list[int]] = {}
    fully = {cp: fully_decompose(cp, decomp, cache) for cp in decomp}
    # Strip identity decompositions (cp -> [cp]); store only multi-cp.
    fully = {cp: seq for cp, seq in fully.items() if seq != [cp]}

    ccc_entries = sorted(ccc.items())
    decomp_entries = sorted(fully.items())
    decomp_data: list[int] = []
    decomp_table: list[tuple[int, int, int]] = []
    for cp, seq in decomp_entries:
        decomp_table.append((cp, len(decomp_data), len(seq)))
        decomp_data.extend(seq)

    qc_no_sorted = sorted(qc_no)
    qc_maybe_sorted = sorted(qc_maybe)

    def fmt_u32(v: int) -> str:
        return f"0x{v:04X}"

    lines: list[str] = []
    lines.append(
        "//! Generated by `tools/gen_nfc_tables.py` from UCD "
        f"{UCD_VERSION}. Do not edit by hand."
    )
    lines.append("//!")
    lines.append("//! UCD source files vendored at `data/ucd/" + UCD_VERSION + "/`.")
    lines.append("//! Regenerate by running `python3 tools/gen_nfc_tables.py`.")
    lines.append("")
    lines.append("#![allow(clippy::unreadable_literal)]")
    lines.append("")
    lines.append(f'pub const UCD_VERSION: &str = "{UCD_VERSION}";')
    lines.append("")
    lines.append(
        f"pub const HANGUL_S_BASE: u32 = 0x{HANGUL_S_BASE:04X};"
    )
    lines.append(
        f"pub const HANGUL_S_COUNT: u32 = {HANGUL_S_COUNT};"
    )
    lines.append(
        f"pub const HANGUL_S_LAST: u32 = 0x{HANGUL_S_LAST:04X};"
    )
    lines.append("pub const HANGUL_L_BASE: u32 = 0x1100;")
    lines.append("pub const HANGUL_V_BASE: u32 = 0x1161;")
    lines.append("pub const HANGUL_T_BASE: u32 = 0x11A7;")
    lines.append("pub const HANGUL_L_COUNT: u32 = 19;")
    lines.append("pub const HANGUL_V_COUNT: u32 = 21;")
    lines.append("pub const HANGUL_T_COUNT: u32 = 28;")
    lines.append("pub const HANGUL_N_COUNT: u32 = 588;  // V_COUNT * T_COUNT")
    lines.append("")
    lines.append("/// Canonical combining class entries (code_point, ccc).")
    lines.append("/// Sorted ascending by code_point. Code points absent")
    lines.append("/// from this table have ccc = 0 (starters).")
    lines.append(
        f"pub const CCC_TABLE: &[(u32, u8)] = &[  // {len(ccc_entries)} entries"
    )
    for cp, c in ccc_entries:
        lines.append(f"    ({fmt_u32(cp)}, {c}),")
    lines.append("];")
    lines.append("")
    lines.append(
        "/// Full (recursive) canonical decomposition mappings."
    )
    lines.append("/// Each entry: (code_point, data_offset, data_length).")
    lines.append(
        "/// The decomposition code points live in `DECOMP_DATA[offset..offset+length]`."
    )
    lines.append("/// Sorted ascending by code_point.")
    lines.append(
        f"pub const DECOMP_TABLE: &[(u32, u16, u8)] = &[  // {len(decomp_table)} entries"
    )
    for cp, off, length in decomp_table:
        lines.append(f"    ({fmt_u32(cp)}, {off}, {length}),")
    lines.append("];")
    lines.append("")
    lines.append(
        f"pub const DECOMP_DATA: &[u32] = &[  // {len(decomp_data)} code points"
    )
    for chunk_start in range(0, len(decomp_data), 8):
        chunk = decomp_data[chunk_start : chunk_start + 8]
        lines.append("    " + " ".join(f"{fmt_u32(v)}," for v in chunk))
    lines.append("];")
    lines.append("")
    lines.append("/// Canonical composition pairs (starter, mark, composed).")
    lines.append("/// Sorted ascending by (starter, mark). Excludes")
    lines.append(
        "/// Full_Composition_Exclusion code points per UAX #15."
    )
    lines.append(
        f"pub const COMP_TABLE: &[(u32, u32, u32)] = &[  // {len(composition)} entries"
    )
    for a, b, c in composition:
        lines.append(f"    ({fmt_u32(a)}, {fmt_u32(b)}, {fmt_u32(c)}),")
    lines.append("];")
    lines.append("")
    lines.append("/// NFC_Quick_Check = No code points (definite non-NFC).")
    lines.append("/// Sorted ascending.")
    lines.append(
        f"pub const NFC_QC_NO: &[u32] = &[  // {len(qc_no_sorted)} entries"
    )
    for chunk_start in range(0, len(qc_no_sorted), 8):
        chunk = qc_no_sorted[chunk_start : chunk_start + 8]
        lines.append("    " + " ".join(f"{fmt_u32(v)}," for v in chunk))
    lines.append("];")
    lines.append("")
    lines.append(
        "/// NFC_Quick_Check = Maybe code points (require full check)."
    )
    lines.append("/// Sorted ascending.")
    lines.append(
        f"pub const NFC_QC_MAYBE: &[u32] = &[  // {len(qc_maybe_sorted)} entries"
    )
    for chunk_start in range(0, len(qc_maybe_sorted), 8):
        chunk = qc_maybe_sorted[chunk_start : chunk_start + 8]
        lines.append("    " + " ".join(f"{fmt_u32(v)}," for v in chunk))
    lines.append("];")
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    ccc, decomp = parse_unicode_data()
    excluded = parse_full_composition_exclusion()
    qc_no, qc_maybe = parse_nfc_qc()
    composition = build_composition_pairs(decomp, excluded)
    rust = format_rust(ccc, decomp, composition, qc_no, qc_maybe)
    OUT_FILE.parent.mkdir(parents=True, exist_ok=True)
    OUT_FILE.write_text(rust)
    print(
        f"Wrote {OUT_FILE.relative_to(Path.cwd())} "
        f"(ccc={len(ccc)}, decomp={len(decomp)}, comp={len(composition)}, "
        f"qc_no={len(qc_no)}, qc_maybe={len(qc_maybe)})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
