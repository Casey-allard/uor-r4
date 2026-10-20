# Does "radius + direction-on-S3 (600-cell)" quantization of keys preserve attention?  Compare codebooks.
import numpy as np, itertools
rng=np.random.default_rng(1)
V600=np.load('/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/exp/arch/two_I.npy')
def unit(a): return a/np.linalg.norm(a,axis=-1,keepdims=True)
# 24-cell (D4 roots normalized, 24 pts) and a random 120-pt code
D4=np.array([v for v in itertools.product((-1,0,1),repeat=4) if sum(abs(x) for x in v)==2],float); D4=unit(D4)
R120=unit(rng.normal(size=(120,4)))
# E8 roots (240) in 8D
E8=[]
for i,j in itertools.combinations(range(8),2):
    for s1,s2 in itertools.product((1,-1),repeat=2):
        v=np.zeros(8); v[i]=s1; v[j]=s2; E8.append(v)
for s in itertools.product((.5,-.5),repeat=8):
    if sum(1 for x in s if x<0)%2==0: E8.append(np.array(s))
E8=unit(np.array(E8)); assert len(E8)==240
# --- covering / distortion on S3 for uniform directions
X=unit(rng.normal(size=(200000,4)))
for name,C in [("600-cell (120)",V600),("random (120)",R120),("24-cell (24)",D4)]:
    cos=(X@C.T).max(1); ang=np.degrees(np.arccos(np.clip(cos,-1,1)))
    print(f"{name:15s} bits {np.log2(len(C)):.2f}  mean angle err {ang.mean():5.2f} deg  max(covering) {ang.max():5.2f} deg  E|x-c|^2 {np.mean(2-2*cos):.4f}")
# --- blockwise polar quantizer: radius log-quantized (rb bits, relative to vector scale) x direction codebook
def polar_quant(K,C,rbits=4):
    d=C.shape[1]; B=K.reshape(K.shape[0],-1,d); r=np.linalg.norm(B,axis=-1,keepdims=True)+1e-12
    u=B/r; idx=(u@C.T).argmax(-1); uq=C[idx]
    # log-radius grid: 2^(e/2) steps (sqrt2 spacing) over 2^rbits levels, per-vector max anchor
    lr=np.log2(r); top=lr.max(axis=1,keepdims=True)
    e=np.clip(np.round((lr-top)*2),-(2**rbits-1),0); rq=2**(top+e/2)
    rq=np.where(e<=-(2**rbits-1),0.0,rq)           # lowest code -> zero
    # scale the direction by the mean cosine so reconstruction is unbiased in expectation
    return (rq*uq).reshape(K.shape)
def scalar_quant(K,bits):
    L=2**(bits-1)-1; s=np.abs(K).max(1,keepdims=True)/L; return np.round(K/s)*s
def ternary(K):
    s=np.abs(K).mean(1,keepdims=True); return np.clip(np.round(K/s),-1,1)*s
# --- retrieval test: T keys, query aimed at one target key; measure top-1 and attention mass/KL
dk,T,N=64,256,400
def trial(quant):
    top1=0; kl=0; mass=0; mass_ex=0
    for n in range(N):
        K=rng.normal(size=(T,dk)); tgt=rng.integers(T)
        q=1.0*K[tgt]+rng.normal(size=dk)*1.0
        s=K@q/np.sqrt(dk); p=np.exp(s-s.max()); p/=p.sum()
        Kq=quant(K); s2=Kq@q/np.sqrt(dk); p2=np.exp(s2-s2.max()); p2/=p2.sum()
        top1+=(s2.argmax()==tgt); kl+=np.sum(p*(np.log(p+1e-30)-np.log(p2+1e-30))); mass+=p2[tgt]; mass_ex+=p[tgt]
    return top1/N, kl/N, mass/N, mass_ex/N
cfgs=[("exact fp",lambda K:K,32),
      ("int4 per-coord (absmax)",lambda K:scalar_quant(K,4),4),
      ("int3 per-coord (absmax)",lambda K:scalar_quant(K,3),3),
      ("ternary (absmean)",ternary,1.58),
      ("600-cell dir + 4b log-radius",lambda K:polar_quant(K,V600,4),(np.log2(120)+4)/4),
      ("random-120 dir + 4b radius",lambda K:polar_quant(K,R120,4),(np.log2(120)+4)/4),
      ("24-cell dir + 4b radius",lambda K:polar_quant(K,D4,4),(np.log2(24)+4)/4),
      ("E8-root(240) dir + 4b radius",lambda K:polar_quant(K,E8,4),(np.log2(240)+4)/8)]
print(f"\nretrieval: dk={dk}, T={T} keys, {N} trials; query = target key + N(0,1) noise")
for name,f,bpd in cfgs:
    t1,kl,m,mex=trial(f)
    print(f"{name:30s} bits/dim {bpd:5.2f}  top1 {t1:.3f}  KL(p||p_q) {kl:.4f}  target mass {m:.3f} (exact {mex:.3f})")
