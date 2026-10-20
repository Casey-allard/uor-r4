week=7*24*3600
rates={"current sequential learner (measured, whole machine)":28.0e9,
       "parallel design, M1 8-core GPU @0.3 TFLOP/s (=#1014 MPS measured MFU)":0.3e12,
       "parallel design, M1 8-core GPU @0.6 TFLOP/s (optimistic)":0.6e12,
       "parallel design, M1 Max 32-core @1.2 TFLOP/s (assumed ~12% of 10.4)":1.2e12}
sizes={"10M":10e6,"30M":30e6,"100M":100e6,"300M":300e6}
print("tokens trainable in one week (6*N*D FLOPs, +10% overhead); Chinchilla 20N shown for reference")
print(f"{'':72s}"+"".join(f"{k:>10s}" for k in sizes))
for lab,r in rates.items():
    print(f"{lab:72s}"+"".join(f"{r*week/(6.6*n)/1e9:9.2f}B" for n in sizes.values()))
print(f"{'Chinchilla-optimal tokens (20 N)':72s}"+"".join(f"{20*n/1e9:9.2f}B" for n in sizes.values()))
# proposed configs
def cfg(name,d,L,dk,glu,V):
    per=4*d*d + 3*d*glu + (d//4)*3*d//64  # q,k,v,o + GLU + small quaternion/decay heads (approx)
    emb=V*d; N=per*L+emb; heads=d//dk; state=L*heads*dk*dk
    print(f"{name}: d={d} L={L} heads={heads}x{dk} glu={glu} V={V}: params {N/1e6:.1f}M (emb {emb/1e6:.1f}M); "
          f"ternary weights {N*2/8/1e6:.1f} MB + int4 emb; per-token serving ~{(N)/1e6:.0f}M adds; "
          f"recurrent state {state/1e6:.2f}M entries ({state*2/1e6:.1f} MB int16)")
    return N
for args in [("S",384,12,64,1024,4096),("M",768,16,64,2048,16384),("L",1024,24,64,2816,32768)]:
    N=cfg(*args)
    for lab,r in list(rates.items())[1:]:
        print(f"    1 week {lab[:60]:60s}: {r*week/(6.6*N)/1e9:6.2f}B tokens = {r*week/(6.6*N)/N:6.1f} tok/param")
