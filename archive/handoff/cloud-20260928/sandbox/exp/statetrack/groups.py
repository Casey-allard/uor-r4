"""Finite groups for the state-tracking experiment.

2I  : binary icosahedral group, 120 unit quaternions (600-cell vertices), ~= SL(2,5).
A5  : 2I / {+1,-1}, 60 classes (icosahedral rotation group, smallest non-solvable group).
S5  : symmetric group on 5 points, 120 elements (non-solvable; embeds in no finite subgroup of SU(2)).
Z60 : cyclic group, 60 elements (abelian => solvable control, same class count as A5).

Convention for every word problem: running product p_t = x_t * p_{t-1}, p_0 = identity,
matching the left-multiplication recurrence h_t = q(x_t) (x) h_{t-1}.
"""
import itertools
import numpy as np

PHI = (1.0 + 5.0 ** 0.5) / 2.0


def hamilton_np(q, x):
    """Hamilton product q*x for arrays [...,4] (w,i,j,k)."""
    w, a, b, c = q[..., 0], q[..., 1], q[..., 2], q[..., 3]
    x0, x1, x2, x3 = x[..., 0], x[..., 1], x[..., 2], x[..., 3]
    return np.stack([w * x0 - a * x1 - b * x2 - c * x3,
                     w * x1 + a * x0 + b * x3 - c * x2,
                     w * x2 - a * x3 + b * x0 + c * x1,
                     w * x3 + a * x2 - b * x1 + c * x0], -1)


def _perm_parity(p):
    p = list(p)
    s = 0
    for i in range(len(p)):
        for j in range(i + 1, len(p)):
            if p[i] > p[j]:
                s += 1
    return s % 2


def binary_icosahedral(even=True):
    """Return the 120 unit quaternions of 2I (float64) plus exact Z[phi]/2 coordinates.

    Exact coordinates: each coordinate is (a + b*phi)/2 with integers a,b; returned as int array
    [120,4,2] holding (a,b) of 2*coordinate.
    """
    elems = []   # float
    exact = []   # (a,b) pairs of 2*coord
    # 8: +-1 on one axis
    for ax in range(4):
        for s in (1, -1):
            v = [0.0] * 4
            e = [(0, 0)] * 4
            v[ax] = float(s)
            e[ax] = (2 * s, 0)
            elems.append(v)
            exact.append(e)
    # 16: (+-1/2)^4
    for signs in itertools.product((1, -1), repeat=4):
        elems.append([0.5 * s for s in signs])
        exact.append([(s, 0) for s in signs])
    # 96: even permutations of (phi/2, 1/2, 1/(2 phi), 0) with all sign choices
    # 2*coords: phi -> (0,1); 1 -> (1,0); 1/phi = phi-1 -> (-1,1); 0 -> (0,0)
    base_f = [PHI / 2, 0.5, 1 / (2 * PHI), 0.0]
    base_e = [(0, 1), (1, 0), (-1, 1), (0, 0)]
    for perm in itertools.permutations(range(4)):
        if (_perm_parity(perm) == 0) != even:
            continue
        for signs in itertools.product((1, -1), repeat=3):
            v = [0.0] * 4
            e = [(0, 0)] * 4
            for src in range(4):
                dst = perm[src]
                s = signs[src] if src < 3 else 1
                v[dst] = s * base_f[src]
                e[dst] = (s * base_e[src][0], s * base_e[src][1])
            elems.append(v)
            exact.append(e)
    Q = np.array(elems, dtype=np.float64)
    E = np.array(exact, dtype=np.int64)
    return Q, E


def nearest_index(Q, x):
    """Index of nearest element of Q (rows unit quaternions) to each row of x (max dot)."""
    return np.argmax(x @ Q.T, axis=-1)


def build_2I():
    for even in (True, False):
        Q, E = binary_icosahedral(even)
        assert Q.shape == (120, 4)
        assert np.allclose(np.linalg.norm(Q, axis=1), 1.0)
        P = hamilton_np(Q[:, None, :], Q[None, :, :])        # P[a,b] = Q[a]*Q[b]
        idx = nearest_index(Q, P.reshape(-1, 4)).reshape(120, 120)
        err = np.abs(P - Q[idx]).max()
        if err < 1e-9:
            # identity index
            ident = int(np.argmax(Q @ np.array([1.0, 0, 0, 0])))
            return Q, E, idx.astype(np.int64), ident, even
    raise RuntimeError("no closed 2I construction found")


def verify_group(table, ident):
    n = table.shape[0]
    # Latin square
    for r in range(n):
        assert len(set(table[r])) == n
        assert len(set(table[:, r])) == n
    assert all(table[ident, a] == a and table[a, ident] == a for a in range(n))
    # associativity on all triples (vectorised)
    lhs = table[table[:, :, None], np.arange(n)[None, None, :]]   # (a*b)*c
    rhs = table[np.arange(n)[:, None, None], table[None, :, :]]    # a*(b*c)
    assert np.array_equal(lhs, rhs), "not associative"
    return True


def build_A5_from_2I(Q, table2I):
    """A5 = 2I/{+-1}. class id for each 2I element, and 60x60 table."""
    neg = nearest_index(Q, -Q)
    cls = -np.ones(120, dtype=np.int64)
    c = 0
    rep = []
    for g in range(120):
        if cls[g] < 0:
            cls[g] = c
            cls[neg[g]] = c
            rep.append(g)
            c += 1
    assert c == 60
    rep = np.array(rep)
    tabA5 = cls[table2I[rep[:, None], rep[None, :]]]
    return cls, rep, tabA5


def build_S5():
    perms = list(itertools.permutations(range(5)))
    index = {p: i for i, p in enumerate(perms)}
    n = len(perms)
    tab = np.zeros((n, n), dtype=np.int64)
    for a, pa in enumerate(perms):
        for b, pb in enumerate(perms):
            comp = tuple(pa[pb[i]] for i in range(5))   # (a o b)(i) = a(b(i))
            tab[a, b] = index[comp]
    ident = index[tuple(range(5))]
    parity = np.array([_perm_parity(p) for p in perms])
    return tab, ident, perms, parity


def build_Zn(n):
    a = np.arange(n)
    return (a[:, None] + a[None, :]) % n, 0


def derived_series_sizes(table, ident):
    """Sizes of the derived series G > G' > G'' ... (solvable iff reaches 1)."""
    n = table.shape[0]
    inv = np.zeros(n, dtype=np.int64)
    for a in range(n):
        inv[a] = int(np.where(table[a] == ident)[0][0])
    current = set(range(n))
    sizes = [n]
    for _ in range(10):
        comms = set()
        cur = list(current)
        for a in cur:
            for b in cur:
                comms.add(int(table[table[inv[a], inv[b]], table[a, b]]))
        # subgroup generated by commutators
        gen = set(comms)
        frontier = list(gen)
        while frontier:
            new = []
            for a in frontier:
                for b in list(gen):
                    for c in (int(table[a, b]), int(table[b, a])):
                        if c not in gen:
                            gen.add(c)
                            new.append(c)
            frontier = new
        if len(gen) == len(current):
            sizes.append(len(gen))
            break
        current = gen
        sizes.append(len(gen))
        if len(gen) == 1:
            break
    return sizes


TASKS = {}


def generated_size(table, gens, ident):
    seen = {ident}
    frontier = [ident]
    while frontier:
        new = []
        for g in frontier:
            for a in gens:
                h = int(table[a, g])
                if h not in seen:
                    seen.add(h)
                    new.append(h)
        frontier = new
    return len(seen)


def tv_from_uniform(table, gens, ident, t):
    n = table.shape[0]
    dist = np.zeros(n)
    dist[ident] = 1.0
    for _ in range(t):
        nd = np.zeros(n)
        for a in gens:
            np.add.at(nd, table[a], dist / len(gens))
        dist = nd
    return 0.5 * np.abs(dist - 1.0 / n).sum()


def choose_gens(table, ident, k, seed, tries=400):
    """k random non-identity elements generating the group; pick the fastest-mixing of `tries` draws
    (TV distance from uniform after 16 steps)."""
    n = table.shape[0]
    rng = np.random.default_rng(seed)
    best = None
    for _ in range(tries):
        g = sorted(rng.choice([i for i in range(n) if i != ident], size=k, replace=False).tolist())
        if generated_size(table, g, ident) != n:
            continue
        tv = tv_from_uniform(table, g, ident, 16)
        if best is None or tv < best[0]:
            best = (tv, g)
    return best[1]


def get_task(name, gens=3):
    """dict(table, ident, n, tokens): tokens are group-element indices used as the input alphabet
    (gens=0 -> all elements), classes are all group elements (running product)."""
    key = (name, gens)
    if key in TASKS:
        return TASKS[key]
    if name in ("A5", "2I"):
        Q, E, t2, id2, even = build_2I()
        if name == "2I":
            table, ident = t2, id2
        else:
            cls, rep, tA5 = build_A5_from_2I(Q, t2)
            table, ident = tA5, int(cls[id2])
    elif name == "S5":
        table, ident, _, _ = build_S5()
    elif name == "Z60":
        table, ident = build_Zn(60)
    else:
        raise ValueError(name)
    n = table.shape[0]
    tokens = list(range(n)) if gens == 0 else choose_gens(table, ident, gens, seed=2026)
    d = dict(table=table, ident=ident, n=n, tokens=np.array(tokens, dtype=np.int64), name=name)
    TASKS[key] = d
    return d


def sample_batch(rng, task, batch, length):
    """token ids [B,L] (index into task['tokens']); targets [B,L] running product x_t*...*x_1."""
    tab, ident, toks = task["table"], task["ident"], task["tokens"]
    x = rng.integers(0, len(toks), size=(batch, length))
    y = np.empty_like(x)
    p = np.full(batch, ident)
    for t in range(length):
        p = tab[toks[x[:, t]], p]
        y[:, t] = p
    return x, y


if __name__ == "__main__":
    Q, E, t2, id2, even = build_2I()
    print("2I built; even-permutation family:", even, "identity idx", id2)
    verify_group(t2, id2)
    print("2I: Latin square, identity, associativity over all 120^3 triples: OK")
    # exact coords consistent with float coords
    fl = (E[..., 0] + E[..., 1] * PHI) / 2.0
    print("exact Z[phi]/2 coords match float:", np.abs(fl - Q).max() < 1e-12)
    cls, rep, tA5 = build_A5_from_2I(Q, t2)
    verify_group(tA5, int(cls[id2]))
    print("A5: 60 elements, group axioms OK")
    tS5, iS5, perms, parity = build_S5()
    verify_group(tS5, iS5)
    print("S5: 120 elements, group axioms OK")
    tZ, iZ = build_Zn(60)
    verify_group(tZ, iZ)
    for nm, tb, ii in (("A5", tA5, int(cls[id2])), ("2I", t2, id2), ("S5", tS5, iS5), ("Z60", tZ, iZ)):
        print(nm, "derived series sizes:", derived_series_sizes(tb, ii))
    # real parts of 2I (conjugacy-invariant)
    re = np.round(Q[:, 0], 6)
    vals, cnt = np.unique(re, return_counts=True)
    print("2I real-part spectrum:", dict(zip(vals.tolist(), cnt.tolist())))
    # nearest-neighbour angle of 600-cell
    G = Q @ Q.T
    np.fill_diagonal(G, -2)
    print("600-cell min geodesic angle (deg):", np.degrees(np.arccos(G.max())))

    for nm in ("A5", "Z60", "S5", "2I"):
        tk = get_task(nm, 3)
        tvs = [round(tv_from_uniform(tk["table"], tk["tokens"], tk["ident"], t), 4) for t in (8, 16, 32, 64)]
        print(nm, "3 generator tokens", tk["tokens"].tolist(), "TV from uniform at t=8,16,32,64:", tvs)
