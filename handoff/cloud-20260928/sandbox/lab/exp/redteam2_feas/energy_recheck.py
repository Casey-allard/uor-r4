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
# --- redteam2 additions: 4-bit LUT weights (what the roadmap serves first) and Q4_0 under the same QoS as ours.
def run4(name, core, P_body, P_head, kv_bytes=0.0, attn_macs=0.0, n_read=0, read_bytes=0.0):
    pw, rt, r4, rq, rf, bw, rd = CORES[core]
    wb = 0.5*(P_body+P_head) + (P_body+P_head)/32*1/8   # 4-bit codes + one 8-bit exponent per 32 weights (assumption)
    tc = (P_body+P_head)/r4
    t = max(wb/bw, tc) + (max(kv_bytes/bw, attn_macs/rf) if kv_bytes else 0.0) + n_read*rd + MISC
    dram = wb + kv_bytes + read_bytes
    e = pw*t + dram*DRAM
    print(f"{name:50s} {core:5s} {1e3*t:6.2f} ms {1/t:6.0f} tok/s {1e3*e:6.1f} mJ  (core {1e3*pw*t:5.1f} | DRAM {1e3*dram*DRAM:5.1f}; {dram/MB:6.1f} MB/token)")
d, L, H, KVH, I = 576, 30, 9, 3, 1536
hd = 64
Pb = L*(d*H*hd*2 + d*KVH*hd*2) + L*3*d*I
Ph = 49152*d
kv16 = L*KVH*hd*2*2*2048; macs = L*H*hd*2*2048; nq = L*H
rb = L*KVH*2048*4.5 + nq*32*128
print(f"\n== 135M at 2K: body {Pb/1e6:.1f}M head {Ph/1e6:.1f}M")
for core in ['P', 'E4', 'E4lo']:
    run("(b) Q4_0 MAD + fp16 KV", core, Pb, Ph, 'q4', kv16, macs)
    run4("(d) 4-bit LUT + fp16 KV", core, Pb, Ph, kv16, macs)
    run4("(d') 4-bit LUT + Lorentz-36 read", core, Pb, Ph, 0, 0, nq, rb)
    run("(c') ternary LUT + Lorentz-36 read", core, Pb, Ph, 'tern', 0, 0, nq, rb)
# --- matched KV treatment: our read with int8 64-dim keys (what arch2 F5 measured as near-lossless) + top-32 value lines;
#     baseline Q4_0 with llama.cpp q8_0 KV cache (-ctk q8_0 -ctv q8_0: 8.5 bits/elem).
rb_int8 = L*KVH*2048*64 + nq*32*128
kv_q8 = kv16*8.5/16
print(f"\n== matched-KV variant: our read bytes {rb_int8/MB:.1f} MB (int8 keys) vs 36-bit {rb/MB:.2f} MB; Q4_0 KV q8_0 {kv_q8/MB:.1f} MB")
for core in ['P', 'E4', 'E4lo']:
    run("(b) Q4_0 MAD + q8_0 KV", core, Pb, Ph, 'q4', kv_q8, macs)
    run4("(d'') 4-bit LUT + Lorentz read, int8 keys", core, Pb, Ph, 0, 0, nq, rb_int8)
# --- context sweep: Q4_0 + q8_0 KV (reads all keys/values) vs 4-bit LUT + admitted read (256 cells/KV head, 3% of int8 keys scored, top-32 value lines).
print("\n== context sweep (E4lo and E4), 135M")
for C in (2048, 8192):
    kvq8 = L*KVH*hd*2*C*8.5/8; macsC = L*H*hd*2*C
    rbC = L*KVH*(256*64 + 0.03*C*64) + nq*32*128
    for core in ['E4', 'E4lo']:
        run(f"(b) Q4_0 + q8_0 KV, ctx {C}", core, Pb, Ph, 'q4', kvq8, macsC)
        run4(f"(d) 4-bit LUT + admitted read, ctx {C}", core, Pb, Ph, 0, 0, nq, rbC)
