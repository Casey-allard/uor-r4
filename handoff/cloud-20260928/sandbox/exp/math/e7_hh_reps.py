"""E7: which A5 elements are single Householder-pair steps H(u)H(e) (simple rotations whose plane contains e)?"""
import itertools, numpy as np
def pm(p):
    M = np.zeros((5, 5)); M[list(p), range(5)] = 1; return M  # M e_i = e_{p(i)}
# orthonormal basis of the sum-zero subspace (standard 4-dim irrep of A5/S5)
Bz = np.linalg.qr(np.vstack([np.eye(5)[i] - np.eye(5)[i + 1] for i in range(4)]).T)[0]  # 5x4
even = [p for p in itertools.permutations(range(5)) if sum(1 for i in range(5) for j in range(i+1,5) if p[i]>p[j]) % 2 == 0]
e = Bz.T @ ((np.eye(5)[0] - np.eye(5)[1]) / np.sqrt(2))   # e0 - e1 in the 4-dim coordinates
kinds = {}
for p in even:
    R = Bz.T @ pm(p) @ Bz                                   # 4x4 orthogonal, det +1
    moved = np.linalg.matrix_rank(R - np.eye(4), tol=1e-9)   # dim of rotation "support"
    # simple rotation <=> rank(R - I) == 2 ; plane = range(R - I)
    cyc = sorted(len(c) for c in __import__('sympy').combinatorics.Permutation(list(p)).cyclic_form) if False else None
    if moved == 2:
        P = np.linalg.svd(R - np.eye(4))[0][:, :2]
        contains = np.linalg.norm(e - P @ (P.T @ e)) < 1e-9
    else:
        contains = False
    key = ('identity' if moved == 0 else 'simple rotation' if moved == 2 else 'double rotation')
    kinds.setdefault(key, [0, 0]); kinds[key][0] += 1; kinds[key][1] += int(contains)
print("A5 in its standard 4-dim representation (e = (e0-e1)/sqrt2):")
for k, (n, c) in kinds.items():
    print(f"   {k:16s}: {n:2d} elements; plane contains e: {c}")
# generators (0 1 k): all simple with plane through e?
for k in (2, 3, 4):
    p = list(range(5)); p[0], p[1], p[k] = 1, k, 0
    R = Bz.T @ pm(p) @ Bz
    P = np.linalg.svd(R - np.eye(4))[0][:, :2]
    print(f"   3-cycle (0 1 {k}): rank(R-I)={np.linalg.matrix_rank(R-np.eye(4),1e-9)}, plane contains e: {np.linalg.norm(e - P @ (P.T @ e)) < 1e-9}")
