import tables
FAMS = ["SC-pow2", "SCchan", "TQ-qr", "TQ-qr-c", "P2-rht", "PQ-rht", "GS4-rht-vec", "GS4-rht-vec-c", "GS4-none-vec",
        "GS4-rht-abs", "GS4-rht-abs-c", "VQ4km-rht", "VQ4km-rht-c", "D4-rht", "D4-rht-c", "E8-rht", "E8-rht-c", "E8-none"]
DS = ["gauss", "t3", "mvt3", "outlier", "aniso"]
rs = {d: tables.load(d) for d in DS}
print("| family | " + " | ".join(DS) + " |")
print("|---|" + "---|" * len(DS))
for f in FAMS:
    cells = []
    for d in DS:
        s2 = rs[d]["stage2"].get(f, {})
        a, b = s2.get("2.0"), s2.get("3.0")
        if a is None:
            cells.append("–")
            continue
        cells.append(f"{a['relmse']:.3f} / {b['relmse']:.4f} / {a['recall10']:.2f}")
    print(f"| {f} | " + " | ".join(cells) + " |")
