"""Exact, multiplier-free action of each 2I element on an icosian 4-vector.

Storage: a 4-vector x in the icosian ring (coordinates in (1/2)Z[phi]) is stored as X = 2x, i.e. four
Z[phi] numbers, each an integer pair (a, b) meaning a + b*phi.  For q in 2I, 2q has coordinates in
{0, +-1, +-2, +-phi, +-phi^-1}.  Then  Y = 2(q x) = q X = (L(2q) X) / 2, where L(.) is the left
Hamilton-multiplication matrix.  Primitive integer operations used:
    phi * (a + b phi)      = b + (a+b) phi        -> 1 integer add
    phi^-1 * (a + b phi)   = (b-a) + a phi        -> 1 integer sub   (phi^-1 = phi - 1)
    Z[phi] add/sub         -> 2 integer add/sub
    /2                     -> 2 arithmetic shifts (exact: asserted)
No multiplication instruction, no float.  Verified against float64 for all 120 elements.
"""
import numpy as np
from groups import build_2I, hamilton_np, PHI

Q, E, T, ID, _ = build_2I()     # E[g, coord] = (a, b) of 2*g


class Ops:
    def __init__(self):
        self.addsub = 0
        self.shift = 0
        self.neg = 0


def coef_type(c):
    a, b = c
    if (a, b) == (0, 0):
        return ("zero", 0)
    if b == 0 and abs(a) == 1:
        return ("one", a)
    if b == 0 and abs(a) == 2:
        return ("two", a // 2)
    if a == 0 and abs(b) == 1:
        return ("phi", b)
    if (a, b) in ((-1, 1), (1, -1)):
        return ("iphi", b)      # +-(phi - 1)
    raise ValueError(c)


# L(q) row r, column k -> (index of q coordinate, sign)
LSTRUCT = [[(0, 1), (1, -1), (2, -1), (3, -1)],
           [(1, 1), (0, 1), (3, -1), (2, 1)],
           [(2, 1), (3, 1), (0, 1), (1, -1)],
           [(3, 1), (2, -1), (1, 1), (0, 1)]]


def apply(g, X, ops):
    """Y = 2 * (Q[g] (x) x) given X = 2x, as Z[phi] integer pairs. Counts integer operations."""
    c2 = [coef_type(tuple(E[g, j])) for j in range(4)]
    kinds = {t for t, _ in c2 if t != "zero"}
    Y = []
    if kinds == {"two"}:
        # signed permutation: Y = +-X_k (L(q) has entries +-1 in one position per row)
        for r in range(4):
            for k in range(4):
                j, s = LSTRUCT[r][k]
                t, sg = c2[j]
                if t == "two":
                    sign = s * sg
                    if sign < 0:
                        ops.neg += 2
                        Y.append((-X[k][0], -X[k][1]))
                    else:
                        Y.append(X[k])
        return Y
    cache = {}

    def scaled(t, k):
        key = (t, k)
        if key not in cache:
            a, b = X[k]
            if t == "one":
                cache[key] = (a, b)
            elif t == "phi":
                ops.addsub += 1
                cache[key] = (b, a + b)
            elif t == "iphi":
                ops.addsub += 1
                cache[key] = (b - a, a)
            else:
                raise ValueError(t)
        return cache[key]

    for r in range(4):
        terms = []
        for k in range(4):
            j, s = LSTRUCT[r][k]
            t, sg = c2[j]
            if t == "zero":
                continue
            terms.append((s * sg, scaled(t, k)))
        terms.sort(key=lambda z: -z[0])            # positive term first avoids a negation
        sign, acc = terms[0]
        if sign < 0:
            ops.neg += 2
            acc = (-acc[0], -acc[1])
        for sign, v in terms[1:]:
            ops.addsub += 2
            acc = (acc[0] + v[0], acc[1] + v[1]) if sign > 0 else (acc[0] - v[0], acc[1] - v[1])
        assert acc[0] % 2 == 0 and acc[1] % 2 == 0, "division by 2 not exact"
        ops.shift += 2
        Y.append((acc[0] >> 1, acc[1] >> 1))
    return Y


def to_float(X):
    return np.array([(a + b * PHI) / 2.0 for a, b in X])


def random_icosian(rng, k=3, m=3):
    X = [(0, 0)] * 4
    for _ in range(k):
        g = rng.integers(120)
        n = int(rng.integers(-m, m + 1))
        X = [(X[c][0] + n * int(E[g, c, 0]), X[c][1] + n * int(E[g, c, 1])) for c in range(4)]
    return X


if __name__ == "__main__":
    rng = np.random.default_rng(0)
    per_type = {"A (8: +-1,+-i,+-j,+-k)": [], "B (16: (+-1/2)^4)": [], "C (96: even perms of (phi,1,1/phi,0)/2)": []}
    maxerr = 0.0
    for g in range(120):
        ty = "A (8: +-1,+-i,+-j,+-k)" if g < 8 else ("B (16: (+-1/2)^4)" if g < 24 else "C (96: even perms of (phi,1,1/phi,0)/2)")
        for trial in range(200):
            X = random_icosian(rng)
            ops = Ops()
            Y = apply(g, X, ops)
            ref = hamilton_np(Q[g], to_float(X))
            maxerr = max(maxerr, float(np.abs(to_float(Y) - ref).max()))
            if trial == 0:
                per_type[ty].append((ops.addsub, ops.shift, ops.neg))
    print(f"verified 120 elements x 200 random icosian vectors; max |exact - float64| = {maxerr:.2e}")
    for ty, v in per_type.items():
        v = np.array(v)
        print(f"type {ty}: integer add/sub {v[:,0].min()}-{v[:,0].max()}, shifts {v[:,1].min()}-{v[:,1].max()}, "
              f"negations {v[:,2].min()}-{v[:,2].max()}  (per 4-vector product)")
    # long random walk: coordinates stay bounded (orbit of a vector under a finite group is finite)
    X = random_icosian(rng)
    start = X
    mx = 0
    orbit = set()
    for step in range(20000):
        X = apply(int(rng.integers(120)), X, Ops())
        orbit.add(tuple(X))
        mx = max(mx, max(max(abs(a), abs(b)) for a, b in X))
    print(f"20000-step random walk: max |integer coordinate| = {mx}, distinct states visited = {len(orbit)} (<=120)")
    print("float 4x4 matvec for comparison: 16 multiplies + 12 adds; table form: 1 read of a 120x120 byte table")
