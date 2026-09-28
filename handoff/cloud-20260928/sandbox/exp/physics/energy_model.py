#!/usr/bin/env python3
"""First-principles energy / throughput model for batch-1 autoregressive decode on Apple M1-class SoCs.

Physics review for UOR-R4 (scratch; not part of the repo). Run:  python3 energy_model.py
Every constant carries a provenance tag:
  [LIT]  retrieved this session (URL in the report)
  [DER]  derived here from [LIT] numbers (formula shown)
  [HYP]  modelling assumption (sensitivity shown via low/mid/high scenarios)

What the model is and is not:
  * A *physics floor* (accelerator-like) estimate: energy of moving bits (DRAM, SRAM) and of the
    arithmetic itself.  It deliberately omits instruction/control overhead, which on a CPU/GPU is
    usually the largest term; that is handled separately by the calibrated "platform" column
    (energy = platform power x time), anchored to measured Apple-silicon numbers.
  * Batch 1 decode only (one token at a time), which is the local-chat regime.
"""
import math

MiB = 2 ** 20
GB = 1e9

# ---------------------------------------------------------------- hardware  [LIT]
HW = {
    # M1: 128-bit LPDDR4X-4266 = 68.25 GB/s spec (AnandTech); 61.4 GB/s measured achievable (ziraph
    # calibrate, IOReport); P-cluster L2 12 MiB, E-cluster L2 4 MiB, SLC 8 MiB (IEEE Micro / Wikipedia).
    "M1": dict(bw_spec=68.25e9, bw_eff=61.4e9, l2=12 * MiB, slc=8 * MiB),
    # M1 Pro: 200 GB/s spec (Apple). Effective 171.8 GB/s = llama.cpp F16 TG 12.75 tok/s x 13.475 GB
    # (12.55 GiB Llama-2-7B F16) on M1 Pro 16-core GPU (llama.cpp discussion #4167)            [DER]
    "M1Pro": dict(bw_spec=200e9, bw_eff=12.75 * 12.55 * 2 ** 30, l2=2 * 12 * MiB, slc=24 * MiB),
}

# ---------------------------------------------------------------- energies (picojoules)
# Arithmetic, 7 nm proxy for TSMC N5 (M1).  BitNet Table 2 [Hor14, ZZL22]:                 [LIT]
E7 = dict(add_fp32=0.38, add_fp16=0.16, add_int8=0.007, mul_fp32=1.31, mul_fp16=0.34, mul_int8=0.07)
# 32-bit integer add at 7 nm: Horowitz 45 nm 0.1 pJ x (0.007/0.03) int8-add scaling = 0.023; use 0.03 [DER]
E7["add_int32"] = 0.03
# 45 nm reference (Horowitz ISSCC 2014 slides/paper)                                      [LIT]
E45 = dict(add_int8=0.03, add_int32=0.1, mul_int8=0.2, mul_int32=3.1, add_fp16=0.4, add_fp32=0.9,
           mul_fp16=1.1, mul_fp32=3.7, sram8k_64b=10.0, sram32k_64b=20.0, sram1m_64b=100.0,
           dram_64b_lo=1300.0, dram_64b_hi=2600.0, instr_overhead=70.0)

# SRAM per bit at ~5-7 nm: Horowitz 45 nm 64-bit reads scaled by 0.4 (the fp16-add 45->7 nm ratio,
# 0.16/0.4); wire-dominated arrays scale worse than logic, so this is optimistic-to-fair.     [DER/HYP]
S = 0.4
SRAM_PJ_PER_BIT = dict(
    L1=S * (E45["sram8k_64b"] + E45["sram32k_64b"]) / 2 / 64,  # 128 KiB L1D ~ between 8K and 32K: ~0.09
    L2=S * 2.0 * E45["sram1m_64b"] / 64,                        # 12 MiB L2 ~2x a 1 MiB array: ~1.25
    SLC=S * 3.0 * E45["sram1m_64b"] / 64,                       # 8 MiB SLC across fabric: ~1.9
)
# DRAM (device + I/O + controller/PHY) per bit.  Bracketed by HBM2 3.9-3.97 pJ/bit (O'Connor MICRO'17),
# GDDR5 14 pJ/bit (same), Horowitz "~10 pJ/bit even with improved I/O; DDR I/O alone >20 pJ/bit".
# M1 calibration: AnandTech single-thread active power 6.3 W (compute) vs 10.5 W (DRAM-heavy) ->
# 4.2 W / (58 GB/s x 8) >= 9 pJ/bit incremental; ziraph Gemma-4-12B ~1.0 J/token at 7.35 GB/token
# -> ~17 pJ/bit all-in (GPU+CPU domains).                                                  [LIT/DER]
DRAM_PJ_PER_BIT = dict(low=5.0, mid=10.0, high=20.0)

# ---------------------------------------------------------------- arithmetic per weight-term (pJ) [DER]
# One weight-term = one weight x one activation, accumulated.
ARITH = {
    # fp16 weights, fp16 activations: fp16 mul + fp16 add
    "fp16":    dict(mult=E7["mul_fp16"] + E7["add_fp16"], nomult=None),
    # int8 weights x int8 activations: int8 mul + int32 accumulate
    "int8":    dict(mult=E7["mul_int8"] + E7["add_int32"], nomult=3 * E7["add_int32"]),
    # signed 4-bit x 8/16-bit act.: small multiplier ~ int8 mul, or shift-add (<=2 adds) + accumulate
    "int4":    dict(mult=E7["mul_int8"] + E7["add_int32"], nomult=3 * E7["add_int32"]),
    "int2":    dict(mult=E7["mul_int8"] + E7["add_int32"], nomult=2 * E7["add_int32"]),
    # ternary: add/sub/skip, ~2/3 of weights non-zero (bitnet reports ~35% zeros in MatMul-free LM)
    "ternary": dict(mult=E7["mul_int8"] + E7["add_int32"], nomult=0.67 * E7["add_int32"]),
}
BITS = {"fp16": 16.0, "int8": 8.0, "int4": 4.0, "int2": 2.0, "ternary": 1.58}
# practical packing: Q4_0 = 4.5 b/w; I2_S = 2 b/w; TL2 = 5 b per 3 weights = 1.67 b/w       [LIT: bitnet.cpp]
BITS_PRACTICAL = {"fp16": 16.0, "int8": 8.5, "int4": 4.5, "int2": 2.5, "ternary": 1.67}

MODELS = [  # (label, params, embedding/tied-head fraction note)
    ("1.7M (UOR-R4 joint)", 1.678338e6),
    ("30M", 30e6),
    ("125M", 125e6),
    ("1B", 1.0e9),
    ("3B", 3.0e9),
    ("7B", 7.0e9),
]


def uor_param_count():
    """Exact count from crates/uor-r4-integer/src/config.rs shapes (d=256, r=64, V=4096, ctx=256)."""
    d, r, V, ctx = 256, 64, 4096, 256
    shapes = {
        "embedding.weight": V * d, "recurrent.input.weight": 3 * d * d, "recurrent.state.weight": 3 * d * d,
        "recurrent.bias": 3 * d, "read.query.weight": r * d, "read.query.bias": r, "read.key.weight": r * d,
        "read.key.bias": r, "read.value.weight": d * d, "read.value.bias": d, "read.age": ctx - 1,
        "read.no_read.weight": d, "read.no_read.bias": 1, "update.weight": d * 2 * d, "update.bias": d,
        "update.gate.weight": 2 * d, "update.gate.bias": 1, "copy.gate.weight": 2 * d, "copy.gate.bias": 1,
        "output.norm.weight": d, "output.bias": V,
    }
    total = sum(shapes.values())
    matrix_terms = (3 * d * d) * 2 + r * d * 2 + d * d + d * 2 * d + 2 * d * 2 + d + V * d  # per-token weight-terms
    return total, shapes["embedding.weight"], matrix_terms


def resident(model_bytes, hw):
    if model_bytes <= 0.75 * hw["l2"]:
        return "L2"
    if model_bytes <= 0.75 * (hw["l2"] + hw["slc"]):
        return "L2+SLC"
    return "DRAM"


def row(label, params, fmt, access, hw_name="M1", dram="mid", practical=True):
    hw = HW[hw_name]
    bits = (BITS_PRACTICAL if practical else BITS)[fmt]
    model_bytes = params * bits / 8
    active = params * access
    bytes_tok = active * bits / 8
    where = resident(model_bytes, hw)
    # DRAM energy: only if weights do not stay on chip between tokens
    e_dram = bytes_tok * 8 * DRAM_PJ_PER_BIT[dram] if where == "DRAM" else 0.0
    # SRAM: every weight bit passes L2 (or SLC+L2) and L1 once per token
    sram_levels = SRAM_PJ_PER_BIT["L1"] + SRAM_PJ_PER_BIT["L2"] + (SRAM_PJ_PER_BIT["SLC"] if where != "L2" else 0)
    e_sram = bytes_tok * 8 * sram_levels
    e_mult = active * ARITH[fmt]["mult"]
    e_nomult = active * ARITH[fmt]["nomult"] if ARITH[fmt]["nomult"] is not None else float("nan")
    ceil_bw = hw["bw_eff"] / bytes_tok if where == "DRAM" else float("inf")
    total_mult = e_dram + e_sram + e_mult
    saving_nomult = (e_mult - e_nomult) / total_mult if not math.isnan(e_nomult) else float("nan")
    return dict(label=label, fmt=fmt, access=access, bytes_tok=bytes_tok, where=where, ceil=ceil_bw,
                e_dram=e_dram * 1e-9, e_sram=e_sram * 1e-9, e_mult=e_mult * 1e-9, e_nomult=e_nomult * 1e-9,
                total=total_mult * 1e-9, saving_nomult=saving_nomult)


def fmt_bytes(b):
    for unit, k in (("GB", 1e9), ("MB", 1e6), ("KB", 1e3)):
        if b >= k:
            return f"{b / k:.3g} {unit}"
    return f"{b:.0f} B"


def main():
    total, emb, terms = uor_param_count()
    print("## A. Current UOR-R4 joint model, parameter count from integer config shapes [SOURCE-derived]")
    print(f"total parameters = {total:,}; tied embedding = {emb:,} ({emb / total:.1%}); "
          f"non-embedding = {total - emb:,}; weight-terms per token (all affine maps incl. tied head) = {terms:,}")
    print(f"packed 4-bit size = {total * 0.5 / MiB:.2f} MiB; as served today (i16 codes) = {total * 2 / MiB:.2f} MiB; "
          f"F32 = {total * 4 / MiB:.2f} MiB  -> all fit in the 12 MiB P-cluster L2")
    print()

    print("## B. Per-token weight traffic, residency, bandwidth ceiling and physics-floor energy (M1, DRAM mid = 10 pJ/bit)")
    print("Energies in millijoules per token. 'mult' = arithmetic with multipliers; 'no-mult' = shift/add or LUT.")
    print("| model | fmt (b/w) | access | bytes/token | resident | M1 ceiling tok/s | M1 Pro ceiling tok/s | E_DRAM | E_SRAM | E_arith mult | E_arith no-mult | no-mult saving of total |")
    print("|---|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|")
    for label, params in MODELS:
        for fmt in ("fp16", "int8", "int4", "int2", "ternary"):
            for access in (1.0, 0.1):
                r = row(label, params, fmt, access)
                r_pro = row(label, params, fmt, access, hw_name="M1Pro")
                ceil = "cache-resident" if r["where"] != "DRAM" else f"{r['ceil']:.0f}"
                ceil_pro = "cache-resident" if r_pro["where"] != "DRAM" else f"{r_pro['ceil']:.0f}"
                sav = "n/a" if math.isnan(r["saving_nomult"]) else f"{100 * r['saving_nomult']:.2f}%"
                nm = "n/a" if math.isnan(r["e_nomult"]) else f"{r['e_nomult']:.3g}"
                print(f"| {label} | {fmt} ({BITS_PRACTICAL[fmt]:g}) | {int(access * 100)}% | {fmt_bytes(r['bytes_tok'])} | "
                      f"{r['where']} | {ceil} | {ceil_pro} | {r['e_dram']:.3g} | {r['e_sram']:.3g} | "
                      f"{r['e_mult']:.3g} | {nm} | {sav} |")
    print()

    print("## C. Sensitivity: DRAM energy scenario (low 5 / mid 10 / high 20 pJ/bit), dense access, M1")
    print("| model | fmt | E_total low | mid | high (mJ) | arithmetic share (mid) |")
    print("|---|---|---:|---:|---:|---:|")
    for label, params in MODELS[2:]:
        for fmt in ("fp16", "int4", "ternary"):
            vals = [row(label, params, fmt, 1.0, dram=s) for s in ("low", "mid", "high")]
            share = vals[1]["e_mult"] / vals[1]["total"]
            print(f"| {label} | {fmt} | {vals[0]['total']:.3g} | {vals[1]['total']:.3g} | {vals[2]['total']:.3g} | {100 * share:.2f}% |")
    print()

    print("## D. KV/event-memory read traffic per generated token (2 bytes/element)")
    # Illustrative MHA configs from bitnet.cpp Appendix A (retrieved) and GQA variants (kv_dim = d/4).
    CFG = [
        ("UOR-R4 joint (1 layer; K64+V256)", 1, None, 64 + 256),
        ("125M (12 x 768, MHA)", 12, 768, None),
        ("1B (24 x 2048, MHA)", 24, 2048, None),
        ("1B (24 x 2048, GQA kv=512)", 24, 512, None),
        ("7B (32 x 4096, MHA)", 32, 4096, None),
        ("7B (32 x 4096, GQA kv=1024)", 32, 1024, None),
    ]
    print("| config | KV bytes/position | ctx 256: KV bytes/token | ctx 4096: KV bytes/token | ctx 4096: E_DRAM mJ (10 pJ/b) | ctx 4096 KV / 4-bit weights |")
    print("|---|---:|---:|---:|---:|---:|")
    wparams = {"UOR": 1.678338e6, "125M": 125e6, "1B": 1e9, "7B": 7e9}
    for name, layers, kv_dim, custom in CFG:
        per_pos = (custom * 2) if custom else (2 * layers * kv_dim * 2)
        kv256, kv4096 = per_pos * 256, per_pos * 4096
        key = "UOR" if name.startswith("UOR") else name.split(" ")[0]
        wbytes = wparams[key] * 4.5 / 8
        e = kv4096 * 8 * 10 * 1e-9  # mJ
        print(f"| {name} | {fmt_bytes(per_pos)} | {fmt_bytes(kv256)} | {fmt_bytes(kv4096)} | {e:.3g} | {kv4096 / wbytes:.2f}x |")
    print()

    print("## E. Calibrated platform view (energy = measured-class power x time), batch-1 decode on M1")
    # Anchors [LIT]: llama.cpp Llama-2-7B Q4_0 on M1 GPU 14.19 tok/s; ziraph M1 GPU peak ~6.5 W on that run;
    # ziraph Gemma-4-12B ~4.9 bpw ~7.2 tok/s at ~1.0 J/token (GPU+CPU domains).
    p_gpu = 6.5
    for name, params, bpw, tps in (("Llama-2-7B Q4_0 (llama.cpp, M1 GPU)", 6.74e9, 4.5, 14.19),):
        print(f"{name}: {tps} tok/s x {params * bpw / 8 / 1e9:.2f} GB = {tps * params * bpw / 8 / 1e9:.1f} GB/s "
              f"({tps * params * bpw / 8 / HW['M1']['bw_eff']:.0%} of 61.4 GB/s); ~{p_gpu / tps:.2f} J/token at ~{p_gpu} W "
              f"-> {p_gpu / tps / (params * bpw) * 1e12:.1f} pJ per streamed weight-bit (all-in, GPU domain)")
    # Current UOR integer serving (measured by the repo): 3.695 ms per full-window model call, one P-core.
    for p_core in (3.0, 5.0, 6.3):
        e = p_core * 3.695e-3
        print(f"UOR integer session, 3.695 ms/token at an assumed {p_core} W single P-core (+uncore) -> {1e3 * e:.1f} mJ/token "
              f"= {e / terms * 1e12:.0f} pJ per weight-term (physics floor for its arithmetic: ~{ARITH['int4']['nomult']:.2f} pJ)")
    print()

    print("## F. What 'no multiplier' can save vs bit-width and access sparsity (1B dense, M1, mid DRAM)")
    base = row("1B", 1e9, "fp16", 1.0)
    for fmt, acc in (("fp16", 1.0), ("int8", 1.0), ("int4", 1.0), ("ternary", 1.0), ("int4", 0.1), ("ternary", 0.1)):
        r = row("1B", 1e9, fmt, acc)
        nm = r["e_nomult"] if not math.isnan(r["e_nomult"]) else r["e_mult"]
        tot_nm = r["e_dram"] + r["e_sram"] + nm
        print(f"{fmt:>7} access {int(acc * 100):>3}%: total(mult) {r['total']:.3g} mJ = {r['total'] / base['total']:.1%} of fp16-dense; "
              f"removing multipliers changes it by {100 * (tot_nm - r['total']) / r['total']:+.2f}%")


if __name__ == "__main__":
    main()
