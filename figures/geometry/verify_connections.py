#!/usr/bin/env python3
"""Verify the conversions between the UOR-R4 geometric objects (standard library only).

Every check is exact: octonions and Clifford matrices use integers, icosians use
Z[phi] = {m + n*phi} with coordinates stored doubled (Z[phi]/2), E8 uses integer
Gram values.  Each claim prints PASS or FAIL; docs/geometry.md ("How the
geometric pieces connect") reports only claims that pass.

Run:  python3 docs/figures/geometry/verify_connections.py
"""
import itertools, os, random, re, sys
from fractions import Fraction

RESULTS = []


def claim(cid, text, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {cid}  {text}" + (f"  [{detail}]" if detail else ""))


# ------------------------------------------------------------ source facts
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))


def rust_triples(rel, pat):
    try:
        src = open(os.path.join(ROOT, rel), encoding="utf-8").read()
    except OSError:
        return None
    m = re.search(pat + r"[^=]*=\s*\[(.*?)\];", src, re.S)
    return [tuple(int(x) for x in t) for t in re.findall(r"[\[(]\s*(\d+),\s*(\d+),\s*(\d+)\s*[\])]", m.group(1))] if m else None


SPIRAL = "crates/uor-r4-core/src/spiralcore_operator.rs"
FANO = [(1, 2, 4), (2, 3, 5), (3, 4, 6), (4, 5, 7), (5, 6, 1), (6, 7, 2), (7, 1, 3)]
PRIMES = [5, 7, 11, 13, 17, 19]
sc = rust_triples(SPIRAL, r"OCTONION_FANO_CYCLES")
cd = rust_triples("crates/uor-r4-core/src/transformerless/cd_space.rs", r"FANO_TRIPLES")
try:
    pm = re.search(r"CANONICAL_SIX_PRIME_VALUES[^=]*=\s*\[([^\]]*)\]", open(os.path.join(ROOT, SPIRAL)).read())
    src_primes = [int(x) for x in pm.group(1).split(",")]
except (OSError, AttributeError):
    src_primes = None
claim("S1", "spiralcore_operator.rs and cd_space.rs use the Fano cycles (124)(235)(346)(457)(561)(672)(713)",
      sc == FANO and cd == FANO)
claim("S2", "spiralcore_operator.rs six-prime registry prefix is 5, 7, 11, 13, 17, 19", src_primes == PRIMES)

# ------------------------------------------------------------ 1. octonions
# basis product e_a e_b = SIGN[a][b] * e_IDX[a][b], a, b in 0..7 (e_0 = 1)
SIGN = [[0] * 8 for _ in range(8)]
IDX = [[0] * 8 for _ in range(8)]
for a in range(8):
    SIGN[0][a] = SIGN[a][0] = 1
    IDX[0][a] = IDX[a][0] = a
for a in range(1, 8):
    SIGN[a][a], IDX[a][a] = -1, 0
for (i, j, k) in FANO:
    for (x, y, z) in ((i, j, k), (j, k, i), (k, i, j)):
        SIGN[x][y], IDX[x][y] = 1, z
        SIGN[y][x], IDX[y][x] = -1, z


def omul(x, y):
    r = [0] * 8
    for a in range(8):
        if x[a]:
            for b in range(8):
                if y[b]:
                    r[IDX[a][b]] += SIGN[a][b] * x[a] * y[b]
    return r


def basis(a):
    v = [0] * 8; v[a] = 1; return v


pairs = [frozenset(p) for p in itertools.combinations(range(1, 8), 2)]
claim("O1", "the 7 lines form a Fano plane (each pair of the 7 units lies on exactly one line)",
      all(sum(set(p) <= set(l) for l in FANO) == 1 for p in pairs))
rng = random.Random(20261009)
vecs = [[rng.randint(-3, 3) for _ in range(8)] for _ in range(300)]
nrm = lambda v: sum(t * t for t in v)
alt = all(omul(omul(x, x), y) == omul(x, omul(x, y)) and omul(omul(y, x), x) == omul(y, omul(x, x))
          for x, y in zip(vecs, vecs[1:]))
claim("O2", "alternative: (xx)y = x(xy) and (yx)x = y(xx) on 299 random integer octonion pairs", alt)
claim("O3", "composition algebra: |xy|^2 = |x|^2 |y|^2 on 299 random integer pairs (a valid octonion product)",
      all(nrm(omul(x, y)) == nrm(x) * nrm(y) for x, y in zip(vecs, vecs[1:])))
nz, coll_zero, total = 0, True, 0
for i, j, k in itertools.permutations(range(1, 8), 3):
    total += 1
    assoc = [p - q for p, q in zip(omul(omul(basis(i), basis(j)), basis(k)), omul(basis(i), omul(basis(j), basis(k))))]
    collinear = any({i, j, k} == set(l) for l in FANO)
    if any(assoc):
        nz += 1
        coll_zero &= not collinear
    else:
        coll_zero &= collinear
claim("O4", "non-associative: 168 of 210 ordered triples of distinct units have a non-zero associator, "
      "exactly the non-collinear ones", nz == 168 and total == 210 and coll_zero, f"{nz}/{total}")
quat_ok = True
for (i, j, k) in FANO:
    span = {0, i, j, k}
    quat_ok &= all(IDX[a][b] in span for a in span for b in span)
    quat_ok &= (SIGN[i][j], IDX[i][j]) == (1, k) and (SIGN[j][k], IDX[j][k]) == (1, i) and (SIGN[k][i], IDX[k][i]) == (1, j)
    quat_ok &= all(omul(omul(basis(a), basis(b)), basis(c)) == omul(basis(a), omul(basis(b), basis(c)))
                   for a in span for b in span for c in span)
claim("O5", "each line {i,j,k} spans an associative subalgebra with e_i e_j = e_k, e_j e_k = e_i, e_k e_i = e_j "
      "(a copy of the quaternions; its unit sphere is S3)", quat_ok)

# ------------------------------------------------------------ Z[phi]
def zm(x, y): return (x[0] * y[0] + x[1] * y[1], x[0] * y[1] + x[1] * y[0] + x[1] * y[1])
def za(x, y): return (x[0] + y[0], x[1] + y[1])
def zn(x): return (-x[0], -x[1])
def zf(x): return x[0] + x[1] * (1 + 5 ** 0.5) / 2
def zhalf(x):
    assert x[0] % 2 == 0 and x[1] % 2 == 0, x
    return (x[0] // 2, x[1] // 2)
Z0, PHI, ONE = (0, 0), (0, 1), (1, 0)
INVPHI = (-1, 1)  # phi - 1 = 1/phi


def zsum(ts):
    r = Z0
    for t in ts: r = za(r, t)
    return r


def raw_qmul(A, B):
    a0, a1, a2, a3 = A; b0, b1, b2, b3 = B
    return (zsum([zm(a0, b0), zn(zm(a1, b1)), zn(zm(a2, b2)), zn(zm(a3, b3))]),
            zsum([zm(a0, b1), zm(a1, b0), zm(a2, b3), zn(zm(a3, b2))]),
            zsum([zm(a0, b2), zn(zm(a1, b3)), zm(a2, b0), zm(a3, b1)]),
            zsum([zm(a0, b3), zm(a1, b2), zn(zm(a2, b1)), zm(a3, b0)]))


def qmul(A, B):  # A, B, result: doubled coordinates (X = 2q)
    return tuple(zhalf(c) for c in raw_qmul(A, B))


def qconj(A): return (A[0], zn(A[1]), zn(A[2]), zn(A[3]))


# ------------------------------------------------------------ 2. icosians
I = set()
for i in range(4):
    for s in (2, -2):
        v = [Z0] * 4; v[i] = (s, 0); I.add(tuple(v))
for s in itertools.product((1, -1), repeat=4):
    I.add(tuple((t, 0) for t in s))
even = [p for p in itertools.permutations(range(4)) if sum(p[a] > p[b] for a in range(4) for b in range(a + 1, 4)) % 2 == 0]
for perm in even:
    for s in itertools.product((1, -1), repeat=3):
        base = [(0, s[0]), (s[1], 0), (-s[2], s[2]), Z0]  # 2 * (phi/2, 1/2, 1/(2 phi), 0) with signs
        I.add(tuple(base[perm[k]] for k in range(4)))
I = sorted(I)
Iset = set(I)
E = ((2, 0), Z0, Z0, Z0)
claim("I1", "the 120 unit icosians (coordinates in Z[phi]/2) are closed under multiplication: the group 2I of order 120",
      len(I) == 120 and all(qmul(a, b) in Iset for a in I for b in I) and all(qmul(a, qconj(a)) == E for a in I))


def re4(A, B):  # 4 * Re(conj(a) b) = 4 * (a . b) in Z[phi]
    return zsum([zm(x, y) for x, y in zip(A, B)])


SHELL_VALUES = sorted({re4(E, q) for q in I}, key=zf, reverse=True)
EXPECT = [1, 12, 20, 12, 30, 12, 20, 12, 1]
shells_ok = all([sum(1 for q in I if re4(p, q) == v) for v in SHELL_VALUES] == EXPECT for p in I)
claim("I2", "from every one of the 120 elements the others fall in 9 shells of sizes 1,12,20,12,30,12,20,12,1 "
      "(Re(p^-1 q) in {1, phi/2, 1/2, 1/(2phi), 0, ...})", len(SHELL_VALUES) == 9 and shells_ok)
edges600 = sum(1 for a, b in itertools.combinations(I, 2) if re4(a, b) == SHELL_VALUES[1])
claim("I3", "the 720 edges of the 600-cell are exactly the shell-1 pairs (angle 36 degrees, Re = phi/2)",
      edges600 == 720 and SHELL_VALUES[1] == (0, 2), f"{edges600} pairs")

# ------------------------------------------------------------ 3. axes and A5
def im(A): return A[1:]
def cross(u, v): return (za(zm(u[1], v[2]), zn(zm(u[2], v[1]))), za(zm(u[2], v[0]), zn(zm(u[0], v[2]))), za(zm(u[0], v[1]), zn(zm(u[1], v[0]))))
def parallel(u, v): return cross(u, v) == (Z0, Z0, Z0)
def dot3(u, v): return zsum([zm(x, y) for x, y in zip(u, v)])
def rot(A, v):  # 4 * (a v a^-1), only directions matter
    return raw_qmul(raw_qmul(A, (Z0,) + tuple(v)), qconj(A))[1:]


def rot_order(A):
    r = A[0]  # 2 Re(a)
    if r in ((2, 0), (-2, 0)): return 1
    if r == Z0: return 2
    if r in ((1, 0), (-1, 0)): return 3
    return 5


def axes_of(order):
    out = []
    for q in I:
        if rot_order(q) == order and not any(parallel(im(q), a) for a in out):
            out.append(im(q))
    return sorted(out, key=lambda a: [-zf(t) for t in a])


A5 = {}
for q in I:
    key = min(q, tuple(zn(t) for t in q))
    A5[key] = q
F5, F3, F2 = axes_of(5), axes_of(3), axes_of(2)
counts = {o: sum(1 for q in A5.values() if rot_order(q) == o) for o in (1, 2, 3, 5)}
claim("A1", "2I/{+-1} has 60 elements: 1 identity, 15 half-turns, 20 order-3 and 24 order-5 rotations (the group A5)",
      len(A5) == 60 and counts == {1: 1, 2: 15, 3: 20, 5: 24}, str(counts))
claim("A2", "the rotations have 6 five-fold, 10 three-fold and 15 two-fold axes", (len(F5), len(F3), len(F2)) == (6, 10, 15))


def perm_of(q):
    return tuple(next(b for b in range(6) if parallel(rot(q, F5[a]), F5[b])) for a in range(6))


def parity(p):
    return sum(p[a] > p[b] for a in range(6) for b in range(a + 1, 6)) % 2


PERMS = {k: perm_of(q) for k, q in A5.items()}
G = set(PERMS.values())
two_trans = all(len({(p[a], p[b]) for p in G}) == 30 for a in range(6) for b in range(6) if a != b)
claim("A3", "A5 acts faithfully on the 6 five-fold axes by even permutations, 2-transitively (as PSL(2,5) on the projective line over F5)",
      len(G) == 60 and all(parity(p) == 0 for p in G) and two_trans)
half = [q for q in A5.values() if rot_order(q) == 2]
fixed = {}
for q in half:
    fx = tuple(a for a in range(6) if PERMS[min(q, tuple(zn(t) for t in q))][a] == a)
    fixed[im(q)] = fx
claim("A4", "each of the 15 half-turns fixes exactly 2 of the 6 five-fold axes, and the 15 fixed pairs are distinct: "
      "a bijection half-turns <-> edges of K6", all(len(v) == 2 for v in fixed.values()) and len(set(fixed.values())) == 15)
claim("A5", "the 30 order-4 icosians are exactly the 15 half-turns times {+-1}",
      sum(1 for q in I if rot_order(q) == 2) == 30)
frames = [t for t in itertools.combinations(list(fixed), 3)
          if all(dot3(u, v) == Z0 for u, v in itertools.combinations(t, 2))]
matchings = [frozenset(frozenset(fixed[u]) for u in t) for t in frames]
claim("A6", "the 5 orthogonal frames of two-fold axes are 5 perfect matchings of K6 that partition its 15 edges "
      "(a synthematic total)", len(frames) == 5 and all(len(set().union(*m)) == 6 for m in matchings)
      and len(set().union(*matchings)) == 15)
share, disjoint = {}, {}
hq = {im(q): q for q in half}
for u, v in itertools.combinations(list(fixed), 2):
    o = rot_order(qmul(hq[u], hq[v]))
    d = share if set(fixed[u]) & set(fixed[v]) else disjoint
    d[o] = d.get(o, 0) + 1
claim("A7", "half-turns do not compose like K6 paths: for edges sharing a vertex the product is always order 5, never a half-turn",
      set(share) == {5}, f"sharing {share}, disjoint {disjoint}")
labellings = set(itertools.permutations(range(6)))
orbits = 0
while labellings:
    lab = labellings.pop()
    orbits += 1
    for g in G:
        labellings.discard(tuple(g[lab[i]] for i in range(6)))
claim("A8", "of the 720 labellings of the 6 axes by the 6 primes, rotations identify only 60 at a time: 12 inequivalent labellings",
      orbits == 12, f"{orbits} orbits")

# ------------------------------------------------------------ 4. primes, K6, bivectors
semis = [(i, j, PRIMES[i] * PRIMES[j]) for i, j in itertools.combinations(range(6), 2)]


def factor(n):
    out, p = [], 2
    while p * p <= n:
        while n % p == 0: out.append(p); n //= p
        p += 1
    return out + ([n] if n > 1 else [])


claim("P1", "the 15 products p*q (p<q from the registry) are distinct square-free semiprimes whose factor pairs are the 15 edges of K6",
      len({s for _, _, s in semis}) == 15 and all(factor(s) == [PRIMES[i], PRIMES[j]] for i, j, s in semis))


def L(i):
    M = [[0] * 8 for _ in range(8)]
    for b in range(8):
        M[IDX[i][b]][b] = SIGN[i][b]
    return tuple(map(tuple, M))


def mm(A, B): return tuple(tuple(sum(A[r][k] * B[k][c] for k in range(8)) for c in range(8)) for r in range(8))
def neg(A): return tuple(tuple(-x for x in row) for row in A)
ID = tuple(tuple(int(r == c) for c in range(8)) for r in range(8))
Ls = {i: L(i) for i in range(1, 7)}
cliff = all(mm(Ls[i], Ls[i]) == neg(ID) for i in Ls) and all(mm(Ls[i], Ls[j]) == neg(mm(Ls[j], Ls[i])) for i in Ls for j in Ls if i != j)
claim("C1", "left multiplications L_1..L_6 by e_1..e_6 satisfy the Cl(0,6) relations L_i^2 = -1, L_i L_j = -L_j L_i", cliff)
Bv = {(i, j): mm(Ls[i], Ls[j]) for i, j in itertools.combinations(range(1, 7), 2)}
b_ok = len(set(Bv.values())) == 15 and all(mm(B, B) == neg(ID) for B in Bv.values())
claim("C2", "the 15 bivectors B_ij = L_i L_j (i<j in 1..6) are distinct, B_ij^2 = -1, so B_ij^-1 = B_ij^3", b_ok)
def Bs(i, j): return Bv[(i, j)] if i < j else neg(Bv[(j, i)])


path_ok = all(mm(Bs(i, j), Bs(j, k)) == neg(Bs(i, k)) for i, j, k in itertools.permutations(range(1, 7), 3))
claim("C3", "bivectors compose like K6 paths: B_ij B_jk = -B_ik, mirroring (p_i p_j)(p_j p_k)/p_j^2 = p_i p_k", path_ok)
grp, frontier = {ID}, [ID]
while frontier:
    nxt = []
    for g in frontier:
        for B in Bv.values():
            h = mm(g, B)
            if h not in grp: grp.add(h); nxt.append(h)
    frontier = nxt
mono = {}
for S in itertools.chain.from_iterable(itertools.combinations(range(1, 7), r) for r in (0, 2, 4, 6)):
    M = ID
    for i in S: M = mm(M, Ls[i])
    mono[M] = S; mono[neg(M)] = S
def prod(xs):
    r = 1
    for x in xs: r *= x
    return r


def sqfree(S): return prod(PRIMES[i - 1] for i in S)
def sqfree_part(n): return prod(p for p in set(factor(n)) if factor(n).count(p) % 2)


hom = all(sqfree(set(mono[a]) ^ set(mono[b])) == sqfree_part(sqfree(mono[a]) * sqfree(mono[b])) and
          set(mono[mm(a, b)]) == set(mono[a]) ^ set(mono[b])
          for a in grp for b in grp if a in mono and b in mono)
claim("C4", "the bivectors generate a group of order 64 = {+-L_S : |S| even}; S -> product of its primes is a homomorphism "
      "onto the 32 square-free products of an even number of registry primes, kernel {+-1}",
      len(grp) == 64 and set(grp) == set(mono) and hom and len({sqfree(S) for S in mono.values()}) == 32)

# ------------------------------------------------------------ 5. E8
roots = set()
for i, j in itertools.combinations(range(8), 2):
    for s, t in itertools.product((2, -2), repeat=2):
        v = [0] * 8; v[i], v[j] = s, t; roots.add(tuple(v))
for s in itertools.product((1, -1), repeat=8):
    if s.count(-1) % 2 == 0: roots.add(s)
def mv(M, v): return tuple(sum(M[r][c] * v[c] for c in range(8)) for r in range(8))
claim("E1", "the standard E8 roots (D8 plus even half-integer, 240) are preserved by all 15 bivectors B_ij and by L_1..L_6",
      len(roots) == 240 and all({mv(M, v) for v in roots} == roots for M in list(Bv.values()) + list(Ls.values())))
H = [q for q in I] + [tuple(zm(PHI, t) for t in q) for q in I]


def ip(A, B):  # <x,y> = 2 * rational part (basis 1, phi) of the dot product; X = 2x so dot = re4/4
    m = re4(A, B)[0]
    assert m % 2 == 0
    return m // 2


dist_ok = all(sorted(ip(r, s) for s in H) == sorted([2] + [1] * 56 + [0] * 126 + [-1] * 56 + [-2]) for r in H)
HS = set(H)


def reflect(x, r):
    c = (-ip(x, r), 0)
    return tuple(za(a, zm(c, b)) for a, b in zip(x, r))


refl_ok = all(reflect(x, r) in HS for r in H for x in H)
coords = [[Fraction(c) for t in q for c in t] for q in H]


def rank(rows):
    rows = [r[:] for r in rows]; rk = 0
    for c in range(len(rows[0])):
        piv = next((r for r in range(rk, len(rows)) if rows[r][c] != 0), None)
        if piv is None: continue
        rows[rk], rows[piv] = rows[piv], rows[rk]
        for r in range(len(rows)):
            if r != rk and rows[r][c] != 0:
                f = rows[r][c] / rows[rk][c]
                rows[r] = [a - f * b for a, b in zip(rows[r], rows[rk])]
        rk += 1
    return rk


claim("E2", "H4 + phi H4: the 240 quaternions 2I and phi*2I, with <x,y> = 2 * (rational part of x.y in the basis 1, phi), "
      "are 240 distinct norm-2 vectors of rank 8 with E8 inner-product counts 1/56/126/56/1, closed under reflections (the E8 roots)",
      len(set(H)) == 240 and rank(coords) == 8 and dist_ok and refl_ok)

# ------------------------------------------------------------ 6. Hopf
def hopf_raw(A):  # 4 * (2(ac+bd), 2(bc-ad), a^2+b^2-c^2-d^2), formula of UnitS3Q30::hopf
    a, b, c, d = A
    return (zm((2, 0), za(zm(a, c), zm(b, d))), zm((2, 0), za(zm(b, c), zn(zm(a, d)))),
            zsum([zm(a, a), zm(b, b), zn(zm(c, c)), zn(zm(d, d))]))


img = {}
for q in I:
    img.setdefault(hopf_raw(q), []).append(q)
fibre_sizes = sorted({len(v) for v in img.values()})
n_img = len(img)
# measured: the image is one orbit of 2I under left and under right multiplication, each fibre is a coset U*q0 with
# U = {+-1, +-s}, s^2 = -1 (stabiliser of order 4 = preimage of a half-turn, so a two-fold-type orbit)
ONE = ((2, 0), Z0, Z0, Z0)
one_orbit = all(hopf_raw(qmul(g, I[5])) in img and hopf_raw(qmul(I[5], g)) in img for g in I)
fibre_cosets = True
for v in img.values():
    q0 = v[0]
    U = [qmul(q, qconj(q0)) for q in v]
    fibre_cosets &= ONE in U and all(qmul(u, u) in (tuple((zn(c) if k == 0 else c) for k, c in enumerate(ONE)), ONE) for u in U)
orbit_name = {12: "five-fold (stabiliser 10)", 20: "three-fold (stabiliser 6)", 30: "two-fold (stabiliser 4)"}.get(n_img, "unknown")
on_F2 = sum(1 for p in img if any(parallel(p, ax) for ax in F2))
claim("H1", f"measured: the Hopf map (2(ac+bd), 2(bc-ad), a^2+b^2-c^2-d^2) sends 2I onto {n_img} points, fibres of size "
      f"{fibre_sizes} = cosets of {{+-1, +-s}} with s^2 = -1; one 2I-orbit, of two-fold type ({orbit_name})",
      fibre_sizes == [4] and n_img == 30 and one_orbit and fibre_cosets and orbit_name.startswith("two-fold"),
      f"image={n_img}, fibres={fibre_sizes}; only {on_F2} of 30 image points lie on a two-fold axis of the script's frame "
      f"(axis directions are therefore not asserted)")
orb = lambda v: len({rot(q, v) for q in I})
claim("H2", "a basepoint on a five-fold / three-fold / two-fold axis has an orbit of 12 / 20 / 30 points under 2I",
      (orb(F5[0]), orb(F3[0]), orb(F2[0])) == (12, 20, 30))

# ------------------------------------------------------------ 7. bits
fano_lines = [set(l) for l in FANO]
good = [p for p in itertools.permutations(range(1, 8))
        if all(p[a - 1] ^ p[b - 1] ^ p[c - 1] == 0 for a, b, c in FANO)]
claim("V1", "relabelling the 7 units by nonzero 3-bit codes turns every Fano line into {a, b, a XOR b} in 168 ways "
      "(|GL(3,2)|), so e_i e_j = +-e_(i XOR j): a signed XOR", len(good) == 168, f"first: {dict(zip(range(1, 8), good[0]))}")
noncomm = sum(1 for a in I for b in I if qmul(a, b) != qmul(b, a))
claim("V2", "XOR binding on bit codes is commutative; 2I is not (ordered pairs with ab != ba)", noncomm > 0, f"{noncomm} of 14400")

# ------------------------------------------------------------ chosen labelling (printed for the doc)
def zs(x):
    m, n = x
    if n == 0: return str(m)
    ph = ("" if n == 1 else "-" if n == -1 else str(n)) + "φ"
    return ph if m == 0 else f"{m}{'+' if n > 0 else ''}{ph}"


print("\nLabelling used in docs/geometry.md (prime -> five-fold axis, unnormalised, doubled coordinates):")
for p, ax in zip(PRIMES, F5):
    print(f"  {p:>2} -> ({', '.join(zs(t) for t in ax)})")
print("Semiprime -> half-turn axis under that labelling:")
for i, j, s in semis:
    u = next(u for u, fx in fixed.items() if fx == (i, j))
    print(f"  {s:>3} = {PRIMES[i]}*{PRIMES[j]} -> ({', '.join(zs(t) for t in u)})")
print(f"\n{sum(RESULTS)}/{len(RESULTS)} PASS")
sys.exit(0 if all(RESULTS) else 1)
