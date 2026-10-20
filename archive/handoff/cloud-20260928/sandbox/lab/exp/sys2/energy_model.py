"""sys2 M1 per-token time/energy model (Derived; batch 1, context 2048, decode only).
Constants (label):
 DRAM 72 pJ/B   Derived from AnandTech M1 mini: DRAM-heavy single thread +4.2 W active at <=58 GB/s (wall power)
 L2 3 pJ/B      Hypothesis (Jouppi ISCA'21 7nm: 1 MB SRAM 14 pJ per 64-bit = 1.75 pJ/B; x~2 for 12 MB + wires)
 P-core 3.7 W   Literature: Notebookcheck, M1 Cinebench R23 single-core package power 3.68-3.75 W
 4 E-cores 1.2 W at 2064 MHz  Literature: T. Kaiser (powermetrics) < 1.5 W fully loaded
 4 E-cores 0.17 W at ~1 GHz   Literature: Eclectic Light (powermetrics) 160-170 mW, background QoS
 BW 58 GB/s one P-core (AnandTech); E-cluster 30 GB/s (Hypothesis)
 Kernel rates: dougallj (Firestorm 4 SIMD/cycle incl. TBL, Icestorm 2) x geo-bench instruction counts x 0.7.
"""
DRAM, L2 = 72e-12, 3e-12
P_TERN, P_4B, P_Q4, P_F16MAC = 0.7*64/3.5*3.2e9, 0.7*32/3.5*3.2e9, 0.7*32/2*3.2e9, 0.7*32*3.2e9
CORES = {  # name: (power W, ternary w/s, 4-bit w/s, Q4 w/s, fp16 MAC/s, DRAM bytes/s, read s per query head)
 'P':    (3.7,  P_TERN,           P_4B,           P_Q4,          P_F16MAC,          58e9, 1.2e-6),
 'E4':   (1.2,  4*0.32*P_TERN*2.064/3.2*3.2/2.064, 4*0.32*P_4B, 4*0.32*P_Q4, 4*0.32*P_F16MAC, 30e9, 1.2e-6/0.32/4),
 'E4lo': (0.17, 4*0.32*P_TERN*1.0/2.064, 4*0.32*P_4B*1.0/2.064, 4*0.32*P_Q4/2.064, 4*0.32*P_F16MAC/2.064, 30e9, 1.2e-6/0.32/4*2.064),
}
MISC = 0.08e-3
MB = 1e6

def run(name, core, P_body, P_head, wfmt, kv_bytes=0.0, attn_macs=0.0, n_read=0, read_bytes=0.0, M=1.0, k1=1.0, resident=False):
    pw, rt, r4, rq, rf, bw, rd = CORES[core]
    if wfmt == 'fp16':
        wb = 2*(P_body+P_head); tc = 0.0
    elif wfmt == 'q4':
        wb = 4.5/8*(P_body+P_head); tc = (P_body+P_head)/rq
    else:
        wb = 0.25*P_body + 0.5*P_head; tc = P_body/rt + P_head/r4
    f = k1/M*0.65 if M > 1 else 1.0          # verification compute per accepted token
    t_w = max(0 if resident else wb/M/bw, tc*f)
    t_a = max(kv_bytes/bw, attn_macs/rf) if kv_bytes else 0.0
    t_r = n_read*rd*f
    t = t_w + t_a + t_r + MISC
    dram = (0 if resident else wb/M) + kv_bytes + read_bytes
    l2 = (wb if resident else 0.0)
    e_core, e_mem = pw*t, dram*DRAM + l2*L2
    print(f"{name:50s} {core:5s} {1e3*t:6.2f} ms {1/t:6.0f} tok/s {1e3*(e_core+e_mem):6.1f} mJ  (core {1e3*e_core:5.1f} | DRAM {1e3*dram*DRAM:5.1f}; {dram/MB:6.1f} MB/token)")

print(f"# per-core rates (G weights/s): P ternary {P_TERN/1e9:.0f}, 4-bit {P_4B/1e9:.0f}, Q4-MAD {P_Q4/1e9:.0f}; 4xE ternary {CORES['E4'][1]/1e9:.0f}")
print(f"# energy per ternary weight: P-core {3.7/P_TERN*1e12:.0f} pJ, 4xE@2GHz {1.2/CORES['E4'][1]*1e12:.0f} pJ, 4xE@1GHz {0.17/CORES['E4lo'][1]*1e12:.0f} pJ; DRAM fetch {0.25*72:.0f} pJ (fp16 weight: {2*72:.0f} pJ)")

print("\n== Generic sizes (vocab 8192, MHA, context 2048)")
for label, d, L, R in [("15M", 384, 7, 2), ("60M", 640, 11, 3), ("150M", 896, 15, 4)]:
    Pb, Ph = 12*d*d*L, 8192*d
    kv, macs = 2*L*d*2*2048, 2*L*d*2048
    wb_t = 0.25*Pb + 0.5*Ph
    rb = R*(2048*4.5 + 32*33)
    print(f"-- {label}: body {Pb/1e6:.1f}M, head {Ph/1e6:.1f}M; ternary weights {wb_t/MB:.1f} MB; fp16 KV {kv/MB:.1f} MB; geometric read {rb/1e3:.0f} KB")
    run("(a) fp16 transformer", 'P', Pb, Ph, 'fp16', kv, macs)
    run("(b) ternary LUT transformer, fp16 KV", 'P', Pb, Ph, 'tern', kv, macs)
    run("(c) geometric (ternary mixing + 36-bit read)", 'P', Pb, Ph, 'tern', 0, 0, R, rb, resident=wb_t <= 8*MB)
    run("(c) geometric", 'E4', Pb, Ph, 'tern', 0, 0, R, rb, resident=wb_t <= 3*MB)
    run("(c) geometric", 'E4lo', Pb, Ph, 'tern', 0, 0, R, rb, resident=wb_t <= 3*MB)

print("\n== Converted SmolLM2, context 2048 (GQA; 36-bit Lorentz keys + 4-bit values, top-32)")
for label, d, L, H, KVH, I in [("135M", 576, 30, 9, 3, 1536), ("360M", 960, 32, 15, 5, 2560)]:
    hd = 64
    Pb = L*(d*H*hd*2 + d*KVH*hd*2) + L*3*d*I
    Ph = 49152*d
    kv16 = L*KVH*hd*2*2*2048
    macs = L*H*hd*2*2048
    nq = L*H
    rb = L*KVH*2048*4.5 + nq*32*128   # key codes once per KV head + top-32 value lines (128-B lines)
    print(f"-- {label}: body {Pb/1e6:.1f}M, head {Ph/1e6:.1f}M; fp16 KV {kv16/MB:.1f} MB; read {rb/MB:.2f} MB; {nq} query heads")
    for core in ['P', 'E4']:
        run("(a) fp16 (llama.cpp-class)", core, Pb, Ph, 'fp16', kv16, macs)
        run("(b) Q4_0 MAD + fp16 KV", core, Pb, Ph, 'q4', kv16, macs)
        run("(b) ternary LUT + fp16 KV", core, Pb, Ph, 'tern', kv16, macs)
        run("(c') ternary LUT + Lorentz-36 read", core, Pb, Ph, 'tern', 0, 0, nq, rb)
    run("(c') ternary LUT + Lorentz-36 read", 'E4lo', Pb, Ph, 'tern', 0, 0, nq, rb)
    run("(c') + exact-memory drafts (M=2, k+1=3)", 'E4', Pb, Ph, 'tern', 0, 0, nq, rb, M=2.0, k1=3.0)
    run("(c') + exact-memory drafts (M=2, k+1=3)", 'E4lo', Pb, Ph, 'tern', 0, 0, nq, rb, M=2.0, k1=3.0)
    run("(c') + drafts, agentic/code-edit (M=5, k+1=8)", 'E4lo', Pb, Ph, 'tern', 0, 0, nq, rb, M=5.0, k1=8.0)
