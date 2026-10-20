"""Stage 1 (config sweep, relMSE on validation) + stage 2 (full metrics at matched rates).

usage: python3 sweep.py <dataset> [out.json]
"""
import json
import math
import os
import sys
import time
import numpy as np

from quantlib import h4_codes, random_code, spherical_kmeans, normalize_rows
from qcodecs import (D, NORM_BITS, make_data, LogNormQ, LloydScalarQ, GainShapeQ, Polar2Q,
                    LatticeQ, VQQ, PolarQuantQ, Pre, blocks, metrics, ip_argmax, kmeans)
from quantlib import Scalar, lloyd1d, rotation

TARGETS = [1.5, 2.0, 2.5, 3.0, 4.0]
HERE = os.path.dirname(os.path.abspath(__file__))


# --------------------------------------------------------------------------
# shared, data-oblivious resources (cached)
# --------------------------------------------------------------------------

def get_codes():
    path = os.path.join(HERE, "h4_codes.npz")
    if os.path.exists(path):
        z = np.load(path)
        return {k: z[k] for k in z.files}
    codes = h4_codes()
    rng = np.random.default_rng(7)
    # controls: random codes and spherical k-means codes of matching sizes
    for n in (24, 120, 600, 840, 2760, 6480, 12600):
        codes[f"RND{n}"] = random_code(n, rng)
    U = normalize_rows(rng.standard_normal((200000, 4)))
    for n in (24, 120, 600):
        best = None
        for rep in range(3):
            C = spherical_kmeans(U, n, rng, iters=60)
            _, p = ip_argmax(U, C)
            score = np.mean(p ** 2)
            if best is None or score > best[0]:
                best = (score, C)
        codes[f"SKM{n}"] = best[1]
    np.savez(path, **codes)
    return codes


def get_vq4():
    path = os.path.join(HERE, "vq4_codebooks.npz")
    if os.path.exists(path):
        z = np.load(path)
        return {int(k[2:]): z[k] for k in z.files}
    rng = np.random.default_rng(11)
    Z = rng.standard_normal((40000, D))
    Z = Z * (math.sqrt(D) / np.linalg.norm(Z, axis=1))[:, None]
    B = Z.reshape(-1, 4)[:150000]
    out = {}
    for n in (16, 32, 64, 128, 256, 512, 1024, 2048, 4096):
        out[n] = kmeans(B, n, rng, iters=30 if n <= 1024 else 20)
    np.savez(path, **{f"vq{n}": c for n, c in out.items()})
    return out


# --------------------------------------------------------------------------
# families
# --------------------------------------------------------------------------

class Family:
    def __init__(self, name, rot, norm_mode, k, configs, design, seed=101, center=False):
        self.name, self.k, self.design = name, k, design
        self.pre = Pre(rot, norm_mode, seed, center=center)
        self.configs = configs  # list of (label, factory)
        self.exch = rot != "none"  # positions exchangeable after a random rotation


def build_families(codes, vq4):
    fams = []
    tq_L = [2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 20, 24, 32]
    for rot in ("qr", "rht"):
        fams.append(Family(f"TQ-{rot}", rot, "vec", 1,
                           [(f"L{L}", (lambda L=L: LloydScalarQ(L))) for L in tq_L], "oblivious"))
    fams.append(Family("SCchan", "none", "none", 1,
                       [(f"L{L}", (lambda L=L: LloydScalarQ(L, symmetric=False)))
                        for L in [2, 3, 4, 5, 6, 8, 10, 12, 16, 24, 32]], "perpos"))
    p2 = [(Lt, Lg) for Lt in (4, 6, 8, 12, 16, 24, 32, 48, 64) for Lg in (1, 2, 3, 4, 6, 8, 12, 16)]
    for rot, design in (("rht", "oblivious"), ("none", "perpos")):
        fams.append(Family(f"P2-{rot}", rot, "vec", 2,
                           [(f"t{Lt}g{Lg}", (lambda Lt=Lt, Lg=Lg: Polar2Q(Lt, Lg))) for Lt, Lg in p2], design))
    pq = [(L1, L234, br) for L1 in (4, 8, 16, 32)
          for L234 in ((2, 2, 2), (4, 2, 2), (4, 4, 2), (4, 4, 4), (8, 4, 4), (8, 8, 8))
          for br in (4, 6, 8)]
    for rot, design in (("rht", "pooled"), ("none", "perpos")):
        fams.append(Family(f"PQ-{rot}", rot, "none", 16,
                           [(f"a{L1}-{L234[0]}{L234[1]}{L234[2]}-r{br}",
                             (lambda L1=L1, L234=L234, br=br: PolarQuantQ(L1, *L234, br)))
                            for L1, L234, br in pq], design))
    h4 = ["2T24", "D4r24", "2I120", "C600", "R720", "G840", "F1200", "G2760", "G6480", "G12600"]
    gl = [1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 24, 32]
    for rot, mode, design in (("rht", "vec", "oblivious"), ("none", "vec", "perpos"),
                              ("rht", "abs", "pooled"), ("none", "abs", "perpos")):
        norm_mode = "vec" if mode == "vec" else "none"
        fams.append(Family(f"GS4-{rot}-{mode}", rot, norm_mode, 4,
                           [(f"{c}g{L}", (lambda c=c, L=L: GainShapeQ(codes[c], L, c))) for c in h4 for L in gl],
                           design))
    rnd = ["RND24", "RND120", "RND600", "RND840", "RND2760", "RND6480", "RND12600"]
    fams.append(Family("GS4rnd-rht-vec", "rht", "vec", 4,
                       [(f"{c}g{L}", (lambda c=c, L=L: GainShapeQ(codes[c], L, c))) for c in rnd for L in gl],
                       "oblivious"))
    fams.append(Family("GS4skm-rht-vec", "rht", "vec", 4,
                       [(f"{c}g{L}", (lambda c=c, L=L: GainShapeQ(codes[c], L, c)))
                        for c in ("SKM24", "SKM120", "SKM600") for L in gl], "oblivious"))
    d4r = [2, 4, 6, 8, 12, 16, 24, 30, 40, 60, 80, 100, 130, 160, 200, 260]
    e8m = [1, 2, 3, 4, 5, 6, 8, 10, 13, 16, 22, 28, 36, 45, 56, 70, 90, 115]
    for rot, design in (("rht", "oblivious"), ("none", "perpos")):
        fams.append(Family(f"D4-{rot}", rot, "vec", 4,
                           [(f"R{r}", (lambda r=r: LatticeQ("D4", r))) for r in d4r], design))
        fams.append(Family(f"E8-{rot}", rot, "vec", 8,
                           [(f"M{m}", (lambda m=m: LatticeQ("E8", 2 * m))) for m in e8m], design))
    fams.append(Family("VQ4km-rht", "rht", "vec", 4,
                       [(f"N{n}", (lambda n=n: VQQ(vq4[n]))) for n in sorted(vq4)], "oblivious"))
    # mean-centered variants (global per-channel mean from training data; no per-vector bits)
    fams.append(Family("TQ-qr-c", "qr", "vec", 1,
                       [(f"L{L}", (lambda L=L: LloydScalarQ(L))) for L in tq_L], "oblivious", center=True))
    fams.append(Family("P2-rht-c", "rht", "vec", 2,
                       [(f"t{Lt}g{Lg}", (lambda Lt=Lt, Lg=Lg: Polar2Q(Lt, Lg))) for Lt, Lg in p2], "oblivious",
                       center=True))
    fams.append(Family("GS4-rht-vec-c", "rht", "vec", 4,
                       [(f"{c}g{L}", (lambda c=c, L=L: GainShapeQ(codes[c], L, c))) for c in h4 for L in gl],
                       "oblivious", center=True))
    fams.append(Family("GS4-rht-abs-c", "rht", "none", 4,
                       [(f"{c}g{L}", (lambda c=c, L=L: GainShapeQ(codes[c], L, c))) for c in h4 for L in gl],
                       "pooled", center=True))
    fams.append(Family("VQ4km-rht-c", "rht", "vec", 4,
                       [(f"N{n}", (lambda n=n: VQQ(vq4[n]))) for n in sorted(vq4)], "oblivious", center=True))
    fams.append(Family("D4-rht-c", "rht", "vec", 4,
                       [(f"R{r}", (lambda r=r: LatticeQ("D4", r))) for r in d4r], "oblivious", center=True))
    fams.append(Family("E8-rht-c", "rht", "vec", 8,
                       [(f"M{m}", (lambda m=m: LatticeQ("E8", 2 * m))) for m in e8m], "oblivious", center=True))
    return fams


# --------------------------------------------------------------------------
# stage 1
# --------------------------------------------------------------------------

def stage1(fam, Xtr, Xval, Ziso, log):
    k = fam.k
    nb = D // k
    if fam.design == "oblivious":  # isotropic model is already centred: do not shift it
        mu = fam.pre.mu
        fam.pre.mu = np.zeros_like(mu)
        Ydes, _ = fam.pre.fwd(Ziso)
        fam.pre.mu = mu
    else:
        Ydes, _ = fam.pre.fwd(Xtr)
    Yval, _ = fam.pre.fwd(Xval)
    den = np.full(len(Yval), float(D)) if fam.pre.norm_mode == "vec" else (Yval ** 2).sum(1)
    Bdes = blocks(Ydes, k)
    Bval = blocks(Yval, k)
    shared = fam.design in ("oblivious", "pooled")
    cache = {}
    out = []
    t0 = time.time()
    for label, factory in fam.configs:
        proto = factory()
        if isinstance(proto, GainShapeQ):
            key = proto.name
            if key not in cache:
                if shared:
                    Bd = Bdes.reshape(-1, k)
                    des = ip_argmax(Bd, proto.C)
                else:
                    des = [ip_argmax(Bdes[:, p, :], proto.C) for p in range(nb)]
                val = [ip_argmax(Bval[:, p, :], proto.C) for p in range(nb)]
                cache = {key: (des, val)}  # keep only one code in memory
            des, val = cache[key]
            if shared:
                q = GainShapeQ(proto.C, proto.Lg, proto.name).fit(None, proj=des[1])
                qs = [q] * nb
            else:
                qs = [GainShapeQ(proto.C, proto.Lg, proto.name).fit(None, proj=des[p][1]) for p in range(nb)]
            contrib = np.array([np.mean(((Bval[:, p, :] - qs[p](Bval[:, p, :], cached=val[p])) ** 2).sum(1) / den)
                                for p in range(nb)])
        else:
            if shared:
                if isinstance(proto, PolarQuantQ) and fam.design == "pooled":
                    # angle codebooks from the isotropic model; log-radius from data
                    Ziso_b = blocks(fam.pre.fwd(Ziso)[0], k).reshape(-1, k)
                    q = proto.fit(Bdes.reshape(-1, k), design=Ziso_b)
                else:
                    q = proto.fit(Bdes.reshape(-1, k))
                qs = [q] * nb
            else:
                qs = [factory().fit(Bdes[:, p, :]) for p in range(nb)]
            contrib = np.array([np.mean(((Bval[:, p, :] - qs[p](Bval[:, p, :])) ** 2).sum(1) / den)
                                for p in range(nb)])
        out.append(dict(label=label, bits=qs[0].bits, contrib=contrib, qs=qs))
    log(f"  stage1 {fam.name}: {len(out)} configs in {time.time() - t0:.1f}s")
    return out


def select(fam, res, target):
    k = fam.k
    nb = D // k
    side = fam.pre.side_bits
    budget = target * D + 1e-9
    best = None
    for a in res:
        if side + math.ceil(nb * a["bits"] - 1e-9) > budget:
            continue
        for b in res:
            if b["bits"] < a["bits"]:
                continue
            # largest number of B positions that fits the budget
            if b["bits"] == a["bits"]:
                mmax = nb if b is not a else 0
            else:
                mmax = int(math.floor((budget - side - nb * a["bits"] + 1e-9) / (b["bits"] - a["bits"])))
                mmax = max(0, min(nb, mmax))
                while mmax > 0 and side + math.ceil((nb - mmax) * a["bits"] + mmax * b["bits"] - 1e-9) > budget:
                    mmax -= 1
            if fam.exch:
                ma, mb = a["contrib"].mean(), b["contrib"].mean()
                m = mmax if mb < ma else 0
                mse = (nb - m) * ma + m * mb
                posB = range(m)
            else:
                ca, cb = a["contrib"], b["contrib"]
                imp = ca - cb
                order = np.argsort(-imp, kind="stable")
                m = min(mmax, int((imp > 0).sum()))
                posB = order[:m]
                mse = ca.sum() - imp[posB].sum()
            if m == 0 and b is not a:
                continue
            bits = side + math.ceil((nb - m) * a["bits"] + m * b["bits"] - 1e-9)
            if best is None or mse < best["pred"] - 1e-15:
                best = dict(a=a, b=b, posB=set(int(p) for p in posB), bits=bits, pred=float(mse))
    return best


def stage2(fam, sel, normq, Xdb, Q):
    k = fam.k
    nb = D // k
    Ydb, ndb = fam.pre.fwd(Xdb)
    nhat = normq(ndb) if fam.pre.norm_mode == "vec" else ndb
    Bdb = blocks(Ydb, k)
    Yhat = np.empty_like(Bdb)
    groups = {}
    for p in range(nb):
        q = sel["b"]["qs"][p] if p in sel["posB"] else sel["a"]["qs"][p]
        groups.setdefault(id(q), (q, []))[1].append(p)
    for q, ps in groups.values():
        B = Bdb[:, ps, :].reshape(-1, k)
        Yhat[:, ps, :] = q(B).reshape(len(Bdb), len(ps), k)
    Xh = fam.pre.inv(Yhat.reshape(len(Xdb), D), nhat)
    m = metrics(Xdb, Xh, Q)
    m["bits_per_dim"] = sel["bits"] / D
    m["config"] = (sel["a"]["label"] if not sel["posB"] or sel["a"] is sel["b"]
                   else f"{sel['a']['label']}x{nb - len(sel['posB'])}+{sel['b']['label']}x{len(sel['posB'])}")
    m["pred_val_relmse"] = sel["pred"]
    return m


# --------------------------------------------------------------------------
# single-config families: TurboQuant-prod (MSE + 1-bit QJL residual) and plain scalar
# --------------------------------------------------------------------------

def tq_prod(Xtr, Xeval, L, normq, seed=202):
    pre = Pre("qr", "vec", 101)
    rng = np.random.default_rng(seed)
    S = rng.standard_normal((D, D))
    Ziso = np.random.default_rng(5).standard_normal((20000, D))
    Ydes, _ = pre.fwd(Ziso)
    q = LloydScalarQ(L).fit(Ydes.reshape(-1, 1))
    # residual-norm quantizer fitted on design data
    rdes = np.linalg.norm(Ydes - q(Ydes.reshape(-1, 1)).reshape(Ydes.shape), axis=1)
    rq = LogNormQ(8).fit(rdes)
    Y, n = pre.fwd(Xeval)
    Yh = q(Y.reshape(-1, 1)).reshape(Y.shape)
    r = Y - Yh
    gam = rq(np.linalg.norm(r, axis=1))
    z = np.sign(r @ S.T)
    z[z == 0] = 1
    Yh = Yh + (math.sqrt(math.pi / 2) / D) * gam[:, None] * (z @ S)
    bits = math.ceil(D * math.log2(L) - 1e-9) + D + 8 + NORM_BITS
    return pre.inv(Yh, normq(n)), bits


def sc_pow2(X, L):
    Qm = (L - 1) // 2
    mx = np.max(np.abs(X), axis=1)
    e0 = np.ceil(np.log2(np.maximum(mx, 1e-30) / Qm))
    best = None
    for de in (0, 1, 2):
        s = 2.0 ** (e0 - de)
        Xh = np.clip(np.round(X / s[:, None]), -Qm, Qm) * s[:, None]
        err = ((X - Xh) ** 2).sum(1)
        if best is None:
            best = (err, Xh)
        else:
            better = err < best[0]
            best = (np.where(better, err, best[0]), np.where(better[:, None], Xh, best[1]))
    bits = math.ceil(D * math.log2(L) - 1e-9) + 6
    return best[1], bits


def sc_opt(X, L, sq):
    Qm = (L - 1) // 2
    mx = np.max(np.abs(X), axis=1) / Qm
    best = None
    for f in np.geomspace(0.2, 1.0, 24):
        s = sq(mx * f)
        Xh = np.clip(np.round(X / s[:, None]), -Qm, Qm) * s[:, None]
        err = ((X - Xh) ** 2).sum(1)
        if best is None:
            best = (err, Xh)
        else:
            better = err < best[0]
            best = (np.where(better, err, best[0]), np.where(better[:, None], Xh, best[1]))
    bits = math.ceil(D * math.log2(L) - 1e-9) + 8
    return best[1], bits


def relmse(X, Xh):
    return float(np.mean(((X - Xh) ** 2).sum(1) / (X ** 2).sum(1)))


def run(dataset, outpath, only=None):
    logf = open(outpath + ".log", "a")

    def log(s):
        print(s, flush=True)
        logf.write(s + "\n")
        logf.flush()

    t0 = time.time()
    codes = get_codes()
    vq4 = get_vq4()
    log(f"[{dataset}] resources ready {time.time() - t0:.1f}s")
    Xtr = make_data(dataset, 8000, np.random.default_rng(1))
    Xval = make_data(dataset, 2000, np.random.default_rng(2))
    Xdb = make_data(dataset, 10000, np.random.default_rng(3))
    Q = make_data(dataset, 100, np.random.default_rng(4))
    Ziso = np.random.default_rng(5).standard_normal((8000, D))
    normq = LogNormQ().fit(np.linalg.norm(Xtr, axis=1))
    fams = build_families(codes, vq4)
    result = dict(dataset=dataset, stage1={}, stage2={})
    if os.path.exists(outpath):
        result = json.load(open(outpath))
    fams = [f for f in fams if only is None or f.name in only]
    for fam in fams:
        fam.pre.fit_center(Xtr)
        res = stage1(fam, Xtr, Xval, Ziso, log)
        result["stage1"][fam.name] = [
            dict(label=r["label"], bits_per_dim=(fam.pre.side_bits + math.ceil((D // fam.k) * r["bits"] - 1e-9)) / D,
                 relmse=float(r["contrib"].sum())) for r in res]
        for tgt in TARGETS:
            sel = select(fam, res, tgt)
            if sel is None:
                continue
            m = stage2(fam, sel, normq, Xdb, Q)
            result["stage2"].setdefault(fam.name, {})[str(tgt)] = m
        log(f"  done {fam.name} ({time.time() - t0:.0f}s): " + ", ".join(
            f"{t}:{result['stage2'].get(fam.name, {}).get(str(t), {}).get('relmse', float('nan')):.4f}" for t in TARGETS))
        del res
        with open(outpath, "w") as f:
            json.dump(result, f, indent=1)
    # single-config families
    sq = LogNormQ(8).fit(np.max(np.abs(Xtr), axis=1) / 7)
    for fname, fn, Ls in (
        ("TQprod-qr", lambda X, L: tq_prod(Xtr, X, L, normq), [2, 3, 4, 5, 6, 8, 12]),
        ("SC-pow2", lambda X, L: sc_pow2(X, L), [3, 5, 7, 9, 11, 15, 21, 31]),
        ("SC-opt", lambda X, L: sc_opt(X, L, sq), [3, 5, 7, 9, 11, 15, 21, 31]),
    ):
        if only is not None and fname not in only:
            continue
        st = []
        for L in Ls:
            Xh, bits = fn(Xval, L)
            st.append(dict(label=f"L{L}", bits_per_dim=bits / D, relmse=relmse(Xval, Xh), L=L))
        result["stage1"][fname] = [{k: v for k, v in s.items() if k != "L"} for s in st]
        for tgt in TARGETS:
            ok = [s for s in st if s["bits_per_dim"] <= tgt + 1e-9]
            if not ok:
                continue
            b = min(ok, key=lambda s: s["relmse"])
            Xh, bits = fn(Xdb, b["L"])
            m = metrics(Xdb, Xh, Q)
            m["bits_per_dim"] = bits / D
            m["config"] = b["label"]
            m["pred_val_relmse"] = b["relmse"]
            result["stage2"].setdefault(fname, {})[str(tgt)] = m
        log(f"  done {fname} ({time.time() - t0:.0f}s)")
    with open(outpath, "w") as f:
        json.dump(result, f, indent=1)
    log(f"[{dataset}] total {time.time() - t0:.1f}s")


if __name__ == "__main__":
    ds = sys.argv[1]
    only = set(sys.argv[2].split(",")) if len(sys.argv) > 2 and sys.argv[2] != "all" else None
    tag = sys.argv[3] if len(sys.argv) > 3 else "all"
    out = os.path.join(HERE, f"results_{ds}_{tag}.json")
    run(ds, out, only)
