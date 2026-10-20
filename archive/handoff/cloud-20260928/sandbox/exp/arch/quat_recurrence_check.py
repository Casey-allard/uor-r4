import numpy as np, itertools
rng=np.random.default_rng(0)
def qmul(a,b):  # Hamilton product, last axis = (w,x,y,z); matches joint_model.rs:1751-1770 (left mult)
    w1,x1,y1,z1=np.moveaxis(a,-1,0); w2,x2,y2,z2=np.moveaxis(b,-1,0)
    return np.stack([w1*w2-x1*x2-y1*y2-z1*z2, w1*x2+x1*w2+y1*z2-z1*y2,
                     w1*y2-x1*z2+y1*w2+z1*x2, w1*z2+x1*y2-y1*x2+z1*w2],-1)
def conj(a): return a*np.array([1,-1,-1,-1])
def unit(a): return a/np.linalg.norm(a,axis=-1,keepdims=True)
# check qmul matches the repo's lane formula y = q (x) x
q=unit(rng.normal(size=4)); x=rng.normal(size=4); w,a,b,c=q
y_repo=np.array([w*x[0]-a*x[1]-b*x[2]-c*x[3], w*x[1]+a*x[0]+b*x[3]-c*x[2],
                 w*x[2]-a*x[3]+b*x[0]+c*x[1], w*x[3]+a*x[2]-b*x[1]+c*x[0]])
print("repo lane formula == Hamilton left product:", np.allclose(y_repo,qmul(q,x)))
T,L=200,16
qs=unit(rng.normal(size=(T,L,4))*np.array([1,.3,.3,.3])+np.array([3,0,0,0]))
rs=rng.uniform(0.8,1.0,size=(T,L,1)); bs=rng.normal(size=(T,L,4))
# 1) sequential linear quaternion recurrence h_t = r_t q_t (x) h_{t-1} + b_t
h=np.zeros((L,4)); H=[]
for t in range(T): h=rs[t]*qmul(qs[t],h)+bs[t]; H.append(h)
H=np.array(H)
# 2) frame decomposition: U_t = q_t U_{t-1}; g_t = r_t g_{t-1} + conj(U_t) b_t; h_t = U_t g_t
U=np.tile([1.,0,0,0],(L,1)); g=np.zeros((L,4)); err=0
for t in range(T):
    U=qmul(qs[t],U); g=rs[t]*g+qmul(conj(U),bs[t]); err=max(err,np.abs(qmul(U,g)-H[t]).max())
print("frame decomposition h_t = U_t (x) g_t, g scalar-decay recurrence: max err %.2e"%err)
# 3) associative combine (a,b)o(a',b') for elements A(h)=a h + b with a = r q
def comb(e1,e2):  # apply e1 then e2: e2(e1(h)) = a2 a1 h + (a2 b1 + b2)
    a1,b1=e1; a2,b2=e2; return (qmul(a2,a1), qmul(a2,b1)+b2)
es=[(rs[t]*qs[t],bs[t]) for t in range(T)]
e=[es[i] for i in range(3)]
l=comb(comb(e[0],e[1]),e[2]); r_=comb(e[0],comb(e[1],e[2]))
print("associativity max err %.2e"%max(np.abs(l[0]-r_[0]).max(),np.abs(l[1]-r_[1]).max()))
# Hillis-Steele inclusive scan (log2 T rounds) reproduces sequential states
P=list(es); k=1
while k<T:
    P=[P[i] if i<k else comb(P[i-k],P[i]) for i in range(T)]; k*=2
print("parallel prefix scan (%d rounds) vs sequential: max err %.2e"%(int(np.ceil(np.log2(T))),max(np.abs(P[t][1]-H[t]).max() for t in range(T))))
# 4) matrix-state version (linear attention with quaternion-rotated key frame):
#    S_t = r_t L_{q_t} S_{t-1} + k_t v_t^T ,  o_t = S_t^T qry_t  (key dim = L blocks of 4)
dv=8; K=rng.normal(size=(T,L,4)); Vv=rng.normal(size=(T,dv)); Q=rng.normal(size=(T,L,4))
rsc=rng.uniform(0.85,1.0,size=T)          # scalar decay per head
S=np.zeros((L,4,dv)); O=[]
for t in range(T):
    S=rsc[t]*np.einsum('lij,ljv->liv', np.stack([np.stack([qmul(qs[t,l],e_) for e_ in np.eye(4)],1) for l in range(L)]), S) \
      + np.einsum('li,v->liv',K[t],Vv[t])
    O.append(np.einsum('liv,li->v',S,Q[t]))
O=np.array(O)
# chunk/parallel form: rotate k,q into the cumulative frame, then plain decayed linear attention (Mamba-2/RetNet SSD)
U=np.tile([1.,0,0,0],(L,1)); Kt=[];Qt=[]
for t in range(T):
    U=qmul(qs[t],U); Kt.append(qmul(conj(U),K[t])); Qt.append(qmul(conj(U),Q[t]))
Kt=np.array(Kt).reshape(T,-1); Qt=np.array(Qt).reshape(T,-1)
logc=np.cumsum(np.log(rsc)); D=np.tril(np.exp(logc[:,None]-logc[None,:]))
O2=(D*(Qt@Kt.T))@Vv
print("matrix-state quaternion recurrence == decayed linear attention on frame-rotated q,k: max err %.2e"%np.abs(O-O2).max())
# 5) binary icosahedral group 2I (600-cell vertices) closure and Z[phi]/2 coordinates
phi=(1+5**.5)/2
V=[]
for i in range(4):
    for s in (1,-1): v=np.zeros(4); v[i]=s; V.append(v)
for s in itertools.product((.5,-.5),repeat=4): V.append(np.array(s))
even=[p for p in itertools.permutations(range(4)) if sum(p[i]>p[j] for i in range(4) for j in range(i+1,4))%2==0]
for p in even:
    for s in itertools.product((1,-1),repeat=3):
        base=np.array([0,s[0]*.5,s[1]*phi/2,s[2]/(2*phi)]); v=np.zeros(4)
        for i in range(4): v[p[i]]=base[i]
        V.append(v)
V=np.array(V); print("2I size",len(V),"all unit:",np.allclose(np.linalg.norm(V,axis=1),1))
prods=qmul(V[:,None,:],V[None,:,:]).reshape(-1,4)
dist=np.abs(prods[:,None,:]-V[None,:,:]).max(-1).min(-1)
print("2I closed under Hamilton product (120x120 Cayley table):",dist.max()<1e-9)
G=np.inner(V,V); print("min angle between vertices (deg): %.2f"%np.degrees(np.arccos(np.clip(np.sort(G,1)[:,-2],-1,1))).min())
np.save('/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/exp/arch/two_I.npy',V)
