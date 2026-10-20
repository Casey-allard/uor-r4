import numpy as np, math, sys
sys.argv=['x']
exec(open('golden_gates.py').read().split("def main():")[0])
c = two_i(); T=np.array([0.0,2+PHI,1.0,1.0]); That=T/math.sqrt(7+5*PHI)
g1 = canon(ham(ham(c[:,None,None,:],That[None,None,None,:]),c[None,:,None,:]).reshape(-1,4))
g01 = canon(np.concatenate([g1, c]))
test = haar(200000)
ang_id = np.degrees(2*np.arccos(np.clip(np.abs(test[:,0]),0,1)))
near = test[ang_id < 20][:20000]
print("test rotations within 20 deg of identity:", len(near))
for name, code in [("2I", canon(c)), ("GG T-count<=1", g01), ("random same N", canon(haar(len(g01))))]:
    a = nearest_angle_deg(near, code)
    print(f"{name:16s} N={len(code):5d}  near-identity mean={a.mean():6.2f}  max={a.max():6.2f} deg")
