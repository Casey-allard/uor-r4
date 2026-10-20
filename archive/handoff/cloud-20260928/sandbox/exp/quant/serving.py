"""Serving-angle experiment: attention/MIPS scores from codebook tables.

Keys are stored with a 4-D gain-shape code (2I = 600-cell shape, Lloyd gain) after a
randomized Hadamard rotation (add/sub + shift only for d=64). Queries are either kept at
full precision (asymmetric, per-query lookup tables) or quantized too (symmetric), in
which case every block score is ghat_q * ghat_k * <c_q, c_k>, and for 2I x 2I the Gram
value <c_q, c_k> takes only 9 values {0, +-1/2, +-phi/2, +-1/(2 phi), +-1}.

usage: python3 serving.py <dataset> [out.json]
"""
import json
import math
import os
import sys
import time
import numpy as np

from quantlib import PHI, Scalar, lloyd1d, rotation, qmul, LatticeBall
from qcodecs import D, make_data, ip_argmax, LloydScalarQ, LogNormQ, nearest_by_l2
from sweep import get_codes, get_vq4

HERE = os.path.dirname(os.path.abspath(__file__))


def recall_at(S, Sh, k=10):
    t = np.argsort(-S, axis=1)[:, :k]
    a = np.argsort(-Sh, axis=1)[:, :k]
    rec = np.mean([len(set(x) & set(y)) / k for x, y in zip(t, a)])
    r1 = np.mean([x[0] in set(y) for x, y in zip(t, a)])
    return float(rec), float(r1)


def attn_tv(S, Sh, nkeys=256, target_maxw=0.3, rng=None):
    """Mean total-variation distance between softmax attention over 256 keys (exact vs
    approximate logits), temperature set so the exact attention has mean max weight ~0.3."""
    rng = np.random.default_rng(0) if rng is None else rng
    sub = rng.choice(S.shape[1], nkeys, replace=False)
    A, B = S[:, sub], Sh[:, sub]
    lo, hi = 1e-3, 100.0
    for _ in range(60):
        beta = math.sqrt(lo * hi)
        z = beta * A
        p = np.exp(z - z.max(1, keepdims=True))
        p /= p.sum(1, keepdims=True)
        if p.max(1).mean() > target_maxw:
            hi = beta
        else:
            lo = beta
    beta = math.sqrt(lo * hi)

    def sm(M):
        z = beta * M
        p = np.exp(z - z.max(1, keepdims=True))
        return p / p.sum(1, keepdims=True)
    return float(0.5 * np.abs(sm(A) - sm(B)).sum(1).mean())


def ip_stats(S, Sh):
    e = Sh - S
    sc = np.mean(np.abs(S))
    return float(np.mean(np.abs(e)) / sc), float((Sh * S).sum() / (S * S).sum())


class GSEnc:
    """Gain-shape block encoder (abs gains, no vector norm): returns shape ids and gain ids."""

    def __init__(self, C, Lg, Btrain):
        self.C = C
        _, p = ip_argmax(Btrain, C)
        self.g = Scalar(lloyd1d(p, Lg))

    def encode(self, B):
        idx, p = ip_argmax(B, self.C)
        return idx, self.g.index(p)

    def decode(self, idx, gi):
        return self.g.c[gi][:, None] * self.C[idx]

    @property
    def bits(self):
        return math.log2(len(self.C) * len(self.g.c))


def run(dataset, outpath, center=False):
    t0 = time.time()
    codes = get_codes()
    vq4 = get_vq4()
    V = codes["2I120"]
    R = rotation("rht", D, np.random.default_rng(101))
    Xtr = make_data(dataset, 8000, np.random.default_rng(1)) @ R.T
    Xdb = make_data(dataset, 10000, np.random.default_rng(3)) @ R.T
    Q = make_data(dataset, 100, np.random.default_rng(4)) @ R.T  # rotation preserves <q,k>
    S = Q @ Xdb.T
    mu = Xtr.mean(0) if center else np.zeros(D)
    # centred coding: <q, k> = <q_c, k_c> + <mu, k_c> + <q, mu>; the per-key term <mu, khat_c> is one
    # scalar stored at write time, the per-query term is constant per query (no effect on ranking/softmax)
    Qfull = Q
    Xtr, Xdb, Q = Xtr - mu, Xdb - mu, Q - mu
    kcorr = {"v": np.zeros(len(Xdb))}
    nb = D // 4
    Btr = Xtr.reshape(-1, 4)
    Bq = Q.reshape(-1, 4)
    Bdb = Xdb.reshape(-1, 4)
    out = dict(dataset=dataset, rows=[])

    # ---- Gram table check for 2I x 2I
    G = V @ V.T
    vals = np.unique(np.round(G, 9))
    out["gram_2I_values"] = [float(v) for v in vals]
    # group-multiplication form: <a,b> = Re(a * conj(b)); class of a*conj(b) is one of 9
    conj = V * np.array([1, -1, -1, -1])
    prod = qmul(V[:, None, :], conj[None, :, :])
    assert np.allclose(prod[..., 0], G)

    def add(key_desc, key_bits, q_desc, Sh, extra=None):
        Sh = Sh + kcorr["v"][None, :] + (Qfull @ mu)[:, None]
        rec, r1 = recall_at(S, Sh)
        ipa, slope = ip_stats(S, Sh)
        row = dict(keys=key_desc, key_bits_per_dim=key_bits, query=q_desc, recall10=rec, r1at10=r1,
                   ip_abs=ipa, ip_slope=slope, attn_tv=attn_tv(S, Sh))
        if extra:
            row.update(extra)
        out["rows"].append(row)
        print(f"{key_desc:28s} {key_bits:5.2f}b  {q_desc:34s} rec10={rec:.3f} r1@10={r1:.2f} "
              f"ip_abs={ipa:.3f} slope={slope:.3f} tv={row['attn_tv']:.3f}", flush=True)

    add("exact", 32.0, "exact", S.copy() - (Qfull @ mu)[:, None])

    for code, Lg in (("2I120", 2), ("2I120", 4), ("2I120", 8), ("C600", 4), ("G2760", 8)):
        Ck = codes[code]
        kenc = GSEnc(Ck, Lg, Btr)
        kid, kgi = kenc.encode(Bdb)
        Khat = kenc.decode(kid, kgi).reshape(len(Xdb), D)
        kcorr['v'] = Khat @ mu
        kb = kenc.bits * nb / D
        kdesc = f"{code} x g{Lg} (abs, rht)"
        Gk = Ck @ Ck.T
        ngram = len(np.unique(np.round(Gk, 9)))
        # (a) full precision query: per-query table T[b, c] = <q_b, c>  (ADC)
        T = np.einsum("qbk,ck->qbc", Q.reshape(len(Q), nb, 4), Ck)
        kid_r = kid.reshape(len(Xdb), nb)
        kg_r = kenc.g.c[kgi].reshape(len(Xdb), nb)
        Sh = np.zeros((len(Q), len(Xdb)))
        for b in range(nb):
            Sh += T[:, b, :][:, kid_r[:, b]] * kg_r[:, b][None, :]
        assert np.allclose(Sh, Q @ Khat.T)
        add(kdesc, kb, "full precision (ADC table)", Sh)
        # (b) query quantized with the same code (same gains / 16 gains / shape only)
        for qLg, qdesc in ((Lg, f"same code+gains ({ngram} Gram values)"), (16, f"same code, g16 query"),
                           (1, "same code, shape only (g1)")):
            qenc = kenc if qLg == Lg else GSEnc(Ck, qLg, Btr)
            qid, qgi = qenc.encode(Bq)
            qid_r = qid.reshape(len(Q), nb)
            qg_r = qenc.g.c[qgi].reshape(len(Q), nb)
            Sh = np.zeros((len(Q), len(Xdb)))
            for b in range(nb):
                Sh += Gk[qid_r[:, b]][:, kid_r[:, b]] * qg_r[:, b][:, None] * kg_r[:, b][None, :]
            add(kdesc, kb, qdesc, Sh)
        # (c) query quantized with a finer H4 code (Gram table Nq x Nk, still finite)
        if code == "2I120":
            for cq in ("G840", "G2760"):
                qenc = GSEnc(codes[cq], 16, Btr)
                qid, qgi = qenc.encode(Bq)
                Gq = codes[cq] @ Ck.T
                nvals = len(np.unique(np.round(Gq, 9)))
                qid_r = qid.reshape(len(Q), nb)
                qg_r = qenc.g.c[qgi].reshape(len(Q), nb)
                Sh = np.zeros((len(Q), len(Xdb)))
                for b in range(nb):
                    Sh += Gq[qid_r[:, b]][:, kid_r[:, b]] * qg_r[:, b][:, None] * kg_r[:, b][None, :]
                add(kdesc, kb, f"{cq} x g16 query ({nvals} Gram values)", Sh)
        # (d) integerized symmetric path: Z[phi] accumulation with integer gains
        if code == "2I120" and Lg == 4:
            G = Gk
            qid, qgi = kenc.encode(Bq)
            qid_r = qid.reshape(len(Q), nb)
            qgi_r = qgi.reshape(len(Q), nb)
            kgi_r = kgi.reshape(len(Xdb), nb)
            gint = np.round(kenc.g.c * 16).astype(np.int64)
            cand = {}
            for a in range(-2, 3):
                for bb in range(-2, 3):
                    cand[round((a + bb * PHI) / 2, 9)] = (a, bb)
            A_tab = np.vectorize(lambda v: cand[round(v, 9)][0])(np.round(G, 9)).astype(np.int64)
            B_tab = np.vectorize(lambda v: cand[round(v, 9)][1])(np.round(G, 9)).astype(np.int64)
            P_tab = np.outer(gint, gint)
            A = np.zeros((len(Q), len(Xdb)), dtype=np.int64)
            Bz = np.zeros((len(Q), len(Xdb)), dtype=np.int64)
            for b in range(nb):
                gp = P_tab[qgi_r[:, b]][:, kgi_r[:, b]]
                A += A_tab[qid_r[:, b]][:, kid_r[:, b]] * gp
                Bz += B_tab[qid_r[:, b]][:, kid_r[:, b]] * gp
            phi_fx = int(round(PHI * 2 ** 16))
            Sint = A * 2 ** 16 + Bz * phi_fx
            Sfl = Sint.astype(np.float64) / (2.0 ** 16 * 2.0 * 256.0)
            add(kdesc, kb, "same code+gains, integer Z[phi] accumulate", Sfl,
                extra=dict(note="A,B int64 sums of table entries; phi as 16-bit fixed point"))

    # ---- comparisons at ~2 bits/dim: scalar TQ (rht, per-coordinate Lloyd, abs) and 4-D k-means VQ
    for L in (4, 8, 16):
        sq = LloydScalarQ(L).fit(Xtr.reshape(-1, 1))
        Khat = sq(Xdb.reshape(-1, 1)).reshape(Xdb.shape)
        kcorr['v'] = Khat @ mu
        add(f"scalar Lloyd L{L} (abs, rht)", math.log2(L), "full precision", Q @ Khat.T)
        Qh = sq(Q.reshape(-1, 1)).reshape(Q.shape)
        add(f"scalar Lloyd L{L} (abs, rht)", math.log2(L), f"same scalar L{L} (LxL product table)", Qh @ Khat.T)
        if L < 16:
            sq16 = LloydScalarQ(16).fit(Xtr.reshape(-1, 1))
            Qh = sq16(Q.reshape(-1, 1)).reshape(Q.shape)
            add(f"scalar Lloyd L{L} (abs, rht)", math.log2(L), "scalar L16 (16xL table)", Qh @ Khat.T)
    # 4-D k-means VQ with 256 codewords (trained on these rotated blocks)
    from qcodecs import kmeans
    C256 = kmeans(Btr[:60000], 256, np.random.default_rng(3), iters=25)
    kid = nearest_by_l2(Bdb, C256)
    Khat = C256[kid].reshape(Xdb.shape)
    kcorr['v'] = Khat @ mu
    add("VQ4 k-means 256 (abs, rht)", 2.0, "full precision", Q @ Khat.T)
    qid = nearest_by_l2(Bq, C256)
    Qh = C256[qid].reshape(Q.shape)
    add("VQ4 k-means 256 (abs, rht)", 2.0, "same VQ (256x256 Gram table)", Qh @ Khat.T)
    # E8 lattice ball (scale tuned), ~2 bits/dim: M=5 -> 56881 points (1.97 bits/dim)
    lat = LatticeBall("E8", 10).fit(Xtr.reshape(-1, 8))
    Khat = lat(Xdb.reshape(-1, 8)).reshape(Xdb.shape)
    kcorr['v'] = Khat @ mu
    add("E8 ball M5 (abs, rht)", math.log2(lat.N) / 8, "full precision", Q @ Khat.T)
    Qh = lat(Q.reshape(-1, 8)).reshape(Q.shape)
    add("E8 ball M5 (abs, rht)", math.log2(lat.N) / 8, "same E8 code (integer dot)", Qh @ Khat.T)
    lat = LatticeBall("E8", 32).fit(Xtr.reshape(-1, 8))
    Khat = lat(Xdb.reshape(-1, 8)).reshape(Xdb.shape)
    kcorr['v'] = Khat @ mu
    add("E8 ball M16 (abs, rht)", math.log2(lat.N) / 8, "full precision", Q @ Khat.T)
    Qh = lat(Q.reshape(-1, 8)).reshape(Q.shape)
    add("E8 ball M16 (abs, rht)", math.log2(lat.N) / 8, "same E8 code (integer dot)", Qh @ Khat.T)
    out["seconds"] = time.time() - t0
    with open(outpath, "w") as f:
        json.dump(out, f, indent=1)
    print(f"done {time.time() - t0:.1f}s")


if __name__ == "__main__":
    ds = sys.argv[1]
    center = len(sys.argv) > 2 and sys.argv[2] == "center"
    run(ds, os.path.join(HERE, f"serving_{ds}{'_c' if center else ''}.json"), center=center)
