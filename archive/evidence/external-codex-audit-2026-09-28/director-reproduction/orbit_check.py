"""Director re-derivation of three Codex packet claims on actual 2I roots (float64)."""
import itertools, json, numpy as np
phi=(1+5**0.5)/2
def qmul(a,b):
    a0,a1,a2,a3=a[...,0],a[...,1],a[...,2],a[...,3]; b0,b1,b2,b3=b[...,0],b[...,1],b[...,2],b[...,3]
    return np.stack([a0*b0-a1*b1-a2*b2-a3*b3, a0*b1+a1*b0+a2*b3-a3*b2,
                     a0*b2-a1*b3+a2*b0+a3*b1, a0*b3+a1*b2-a2*b1+a3*b0],-1)
def conj(a): return a*np.array([1,-1,-1,-1])
R=set()
for i in range(4):
    for s in (1,-1):
        v=[0,0,0,0]; v[i]=s; R.add(tuple(v))
for s in itertools.product((0.5,-0.5),repeat=4): R.add(s)
even=[p for p in itertools.permutations(range(4)) if sum(p[i]>p[j] for i in range(4) for j in range(i+1,4))%2==0]
base=(0.0,0.5,phi/2,1/(2*phi))
for p in even:
    for s in itertools.product((1,-1),repeat=3):
        v=[0,0,0,0]; vals=[base[0],s[0]*base[1],s[1]*base[2],s[2]*base[3]]
        for k in range(4): v[p[k]]=vals[k]
        R.add(tuple(round(x,12) for x in v))
G=np.array(sorted(R)); assert len(G)==120, len(G)
P=qmul(G[:,None,:],G[None,:,:]).reshape(-1,4)
d=np.abs(P[:,None,:]-G[None,:,:]).max(-1); assert (d.min(1)<1e-9).all()  # closure
idx=d.argmin(1).reshape(120,120)
inv=np.array([np.abs(conj(g)[None,:]-G).max(1).argmin() for g in G])
# (1) Codex 2I counterexample: normalization reverses ranking with exact 2I directions
q=np.array([1.,0,0,0]); kA=np.array([.25,0,0,0]); kB=np.array([1.,1,1,1])
dots=[q@kA,q@kB]; unit=[q@(kA/np.linalg.norm(kA)),q@(kB/np.linalg.norm(kB))]
kB_unit_in_2I=bool(np.abs(G-kB/2).max(1).min()<1e-12)
# (2) orbit factorization: dot(c(gQ)a_i, c(gK)a_j) == Re(conj(a_i) c(gQ^-1 gK) a_j), 480x480 pairs
A=np.array([[1,0,0,0],[.6,.8,0,0],[1/3,2/3,2/3,0],[2/3,1/3,2/3,0]]); assert np.allclose((A**2).sum(1),1)
codes=qmul(G[:,None,:],A[None,:,:]).reshape(-1,4)  # code (g,i) -> c(g) a_i
distinct=len({tuple(np.round(c,9)) for c in codes})
full=codes@codes.T
tab=np.empty((4,120,4))
for i in range(4):
    for r in range(120):
        tab[i,r,:]=qmul(conj(A[i])[None,:],qmul(G[r][None,:],A))[:,0]
gq=np.repeat(np.arange(120),4); ai=np.tile(np.arange(4),120)
rel=idx[inv[gq][:,None],gq[None,:]]
red=tab[ai[:,None],rel,ai[None,:]]
out={"elements":len(G),"closure":True,
     "counterexample_2I":{"dots":dots,"unit_dots":unit,"reversed":bool((dots[0]<dots[1]) and (unit[0]>unit[1])),"kB_direction_in_2I":kB_unit_in_2I},
     "orbit_codes_distinct":distinct,"pairs_checked":int(full.size),"table_entries":int(tab.size),
     "max_abs_discrepancy":float(np.abs(full-red).max())}
out["script"]="orbit_check.py (director re-derivation, float64, numpy "+np.__version__+")"
print(json.dumps(out,indent=1))
