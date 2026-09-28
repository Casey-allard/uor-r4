"""Exact, multiplier-free table-driven serving of trained geometric recurrences.

quat / quat_r (h -> q h) and hh / hh_r (h -> v h v) on A5/2I:
  1. per lane, find the conjugation u (as a rotation R of imaginary parts) that best aligns the learned
     unit quaternions with the STANDARD 2I (Z[phi]/2 coordinates), by Kabsch over every candidate
     image pair of two generator tokens, then snap every token quaternion to its nearest 2I element;
  2. lane state = 2I index (quat) or (left,right) 2I index pair (hh); transitions are exact products
     read from the 120x120 group table (integer lookups only);
  3. BFS the reachable tuple of lane states from the identity; for each reachable tuple evaluate the
     trained float readout ONCE offline -> class table; compile a transition table [state, token].
  Serving = two integer table reads per token (next = trans[state, tok]; pred = cls[state]).
cplx_u on Z60: snap each coordinate to the nearest character exp(2 pi i k g / 60) and do the same.
"""
import json, os, sys, time
import numpy as np
from groups import build_2I, hamilton_np, get_task, sample_batch, PHI

Q2I, E2I, T2I, ID2I, _ = build_2I()
D = 32


def unit(v):
    return v / np.linalg.norm(v, axis=-1, keepdims=True)


def gelu(x):
    # tanh approximation is what jax.nn.gelu uses by default (approximate=True)
    return 0.5 * x * (1 + np.tanh(np.sqrt(2 / np.pi) * (x + 0.044715 * x ** 3)))


def readout_np(p, hflat):
    h = hflat / np.sqrt(np.mean(hflat * hflat, -1, keepdims=True) + 1e-6)
    z = gelu(h @ p["W1"] + p["b1"])
    return z @ p["W2"] + p["b2"]


def token_quats(model, p):
    raw = p["raw"]
    e0 = np.array([1.0, 0, 0, 0])
    if model in ("quat", "hh"):
        return unit(raw)
    if model == "quat_r":
        return unit(e0 + 0.1 * raw)
    if model == "hh_r":
        return unit(e0 + (0.1 / np.sqrt(2)) * raw)
    raise ValueError(model)


def kabsch_batch(S, T):
    """S,T [N,k,3] -> rotations R [N,3,3] minimising sum |R s - t|^2."""
    Hm = np.einsum("nki,nkj->nij", S, T)
    U, _, Vt = np.linalg.svd(Hm)
    d = np.sign(np.linalg.det(np.einsum("nji,nkj->nik", Vt, U)))   # det(V U^T)
    Dm = np.tile(np.eye(3), (len(S), 1, 1))
    Dm[:, 2, 2] = d
    return np.einsum("nji,njk,nlk->nil", Vt, Dm, U)                 # V D U^T


def align_lane(qs):
    """qs [V,4] learned unit quaternions of one lane. Returns (R, snapped idx [V], max residual)."""
    V = len(qs)
    # candidate images for tokens 0 and 1: all 2I pairs
    g0, g1 = np.meshgrid(np.arange(120), np.arange(120), indexing="ij")
    g0, g1 = g0.ravel(), g1.ravel()
    ok = (np.abs(Q2I[g0, 0] - qs[0, 0]) < 0.35) & (np.abs(Q2I[g1, 0] - qs[1, 0]) < 0.35)
    g0, g1 = g0[ok], g1[ok]
    if len(g0) == 0:
        return None, None, np.inf
    s0, s1 = Q2I[g0, 1:], Q2I[g1, 1:]
    t0 = np.broadcast_to(qs[0, 1:], s0.shape)
    t1 = np.broadcast_to(qs[1, 1:], s1.shape)
    S = np.stack([s0, s1, np.cross(s0, s1)], 1)
    T = np.stack([t0, t1, np.cross(t0, t1)], 1)
    R = kabsch_batch(S, T)                                   # R s ~ t
    # aligned learned quats: (re, R^T im)
    im = np.einsum("nji,vj->nvi", R, qs[:, 1:])              # [N,V,3]
    al = np.concatenate([np.broadcast_to(qs[None, :, :1], im.shape[:2] + (1,)), im], -1)
    dots = al @ Q2I.T                                        # [N,V,120]
    idx = dots.argmax(-1)
    res = np.linalg.norm(al - Q2I[idx], axis=-1).max(-1)     # [N]
    b = int(res.argmin())
    return R[b], idx[b], float(res[b])


def lane_float_state(R, gL, h0, gR=None):
    """float lane state for 2I index gL (and right index gR for hh): (u gL u^-1) h0 (u gR u^-1)."""
    def conj(g):
        return np.concatenate([Q2I[g, :1], Q2I[g, 1:] @ R.T], -1)
    h = hamilton_np(conj(gL), h0)
    if gR is not None:
        h = hamilton_np(h, conj(gR))
    return h


NEG2I = np.argmax(-Q2I @ Q2I.T, axis=1)      # index of -g
STRICT = os.environ.get("STRICT") == "1"
SKIP_FLOAT = os.environ.get("SKIP_FLOAT") == "1"


def lane_consistent(task, idx, left=True):
    """Does the snapped lane (token -> 2I element idx[x]) track the task's A5/2I element?
    BFS over (true element, lane element); consistent iff every true element maps to one {g,-g} pair."""
    tab, ident, toks = task["table"], task["ident"], task["tokens"]
    seen = {(ident, ID2I)}
    frontier = [(ident, ID2I)]
    while frontier:
        new = []
        for (t, g) in frontier:
            for x in range(len(toks)):
                nt = int(tab[toks[x], t])
                ng = int(T2I[idx[x], g]) if left else int(T2I[g, idx[x]])
                if (nt, ng) not in seen:
                    seen.add((nt, ng))
                    new.append((nt, ng))
        frontier = new
    images = {}
    for t, g in seen:
        images.setdefault(t, set()).add(min(g, int(NEG2I[g])))
    return all(len(v) == 1 for v in images.values()), len(seen)


def compile_automaton(model, p, task, cap=300000, res_thresh=0.3):
    V = len(task["tokens"])
    qs = token_quats(model, p)                     # [V,k4,4]
    k4 = qs.shape[1]
    lanes = []
    two_sided = model.startswith("hh")
    for i in range(k4):
        R, idx, res = align_lane(qs[:, i, :])
        rec = dict(R=R, idx=idx, res=res, keep=bool(res < res_thresh))
        if rec["keep"] and task["name"] in ("A5", "2I"):
            rec["consistent"], rec["joint"] = lane_consistent(task, idx, left=True)
        if STRICT:
            rec["keep"] = bool(res < 0.05 and rec.get("consistent", False))
        lanes.append(rec)
    kept = [i for i in range(k4) if lanes[i]["keep"]]
    width = 2 * len(kept) if two_sided else len(kept)
    start = tuple([ID2I] * width)
    index = {start: 0}
    states = [start]
    trans = []
    head = 0
    while head < len(states):
        s = states[head]
        row = []
        for x in range(V):
            ns = []
            for j, i in enumerate(kept):
                g = lanes[i]["idx"][x]
                if two_sided:
                    ns.append(int(T2I[g, s[2 * j]]))        # left:  v L
                    ns.append(int(T2I[s[2 * j + 1], g]))    # right: R v
                else:
                    ns.append(int(T2I[g, s[j]]))
            ns = tuple(ns)
            k = index.get(ns)
            if k is None:
                k = len(states)
                index[ns] = k
                states.append(ns)
                if len(states) > cap:
                    return lanes, None, None, len(states)
            row.append(k)
        trans.append(row)
        head += 1
    trans = np.array(trans, dtype=np.int32)
    S = np.array(states).reshape(len(states), -1)
    hs = []
    for i in range(k4):
        if i in kept:
            j = kept.index(i)
            if two_sided:
                hs.append(lane_float_state(lanes[i]["R"], S[:, 2 * j], p["h0"][i], S[:, 2 * j + 1]))
            else:
                hs.append(lane_float_state(lanes[i]["R"], S[:, j], p["h0"][i]))
        else:   # frozen lane: stays at its initial state
            hs.append(np.broadcast_to(p["h0"][i], (len(states), 4)))
    hflat = np.concatenate(hs, -1)
    cls = readout_np(p, hflat).argmax(-1).astype(np.int32)
    return lanes, trans, cls, len(states)


def snapped_float(model, p, lanes, xs):
    """float64 recurrence using the SNAPPED 2I quaternions (aligned frame) for kept lanes; other lanes frozen."""
    B, L = xs.shape
    k4 = len(lanes)
    two_sided = model.startswith("hh")
    qs = []
    for ln in lanes:
        if ln["keep"]:
            g = Q2I[ln["idx"]]
            qs.append(np.concatenate([g[:, :1], g[:, 1:] @ ln["R"].T], -1))
        else:
            qs.append(np.tile(np.array([1.0, 0, 0, 0]), (p["raw"].shape[0], 1)))
    qs = np.stack(qs, 1)                                   # [V,k4,4]
    h = np.broadcast_to(p["h0"], (B,) + p["h0"].shape).copy()
    pred = np.empty((B, L), dtype=np.int64)
    for t in range(L):
        q = qs[xs[:, t]]
        if two_sided:
            h = hamilton_np(hamilton_np(q, h), q)
        else:
            h = hamilton_np(q, h)
        pred[:, t] = readout_np(p, h.reshape(B, -1)).argmax(-1)
    return pred


def serve(trans, cls, xs):
    B, L = xs.shape
    st = np.zeros(B, dtype=np.int64)
    pred = np.empty((B, L), dtype=np.int64)
    for t in range(L):
        st = trans[st, xs[:, t]]
        pred[:, t] = cls[st]
    return pred


def float_forward_np(model, p, xs):
    """numpy float64 re-implementation of the trained recurrence (for drift comparison)."""
    qs = token_quats(model, p)
    B, L = xs.shape
    h = np.broadcast_to(p["h0"], (B,) + p["h0"].shape).copy()
    pred = np.empty((B, L), dtype=np.int64)
    for t in range(L):
        q = qs[xs[:, t]]
        if model.startswith("quat"):
            h = hamilton_np(q, h)
        else:
            h = hamilton_np(hamilton_np(q, h), q)
        pred[:, t] = readout_np(p, h.reshape(B, -1)).argmax(-1)
    return pred


def cplx_snap(p, task):
    """Z60 control: snap each complex coordinate to the nearest character of Z60."""
    theta = p["theta"]                       # [V,m]
    toks = task["tokens"]                    # group elements (ints mod 60)
    n = task["n"]
    k = np.arange(n)
    ideal = 2 * np.pi * np.outer(toks, k) / n                      # [V,n]
    err = np.angle(np.exp(1j * (theta[:, :, None] - ideal[:, None, :])))  # [V,m,n]
    score = np.abs(err).max(0)                                      # [m,n]
    kbest = score.argmin(-1)
    res = score.min(-1)
    if STRICT:
        kbest = np.where(res < 0.1, kbest, 0)      # unconverged coordinate: frozen at its initial phase
    # exact state: element s in Z60 -> coordinate phase 2 pi k s / n
    s = np.arange(n)
    ph = 2 * np.pi * np.outer(s, kbest) / n                         # [n,m]
    h0 = p["h0"][0] + 1j * p["h0"][1]
    hc = np.exp(1j * ph) * h0
    hflat = np.concatenate([hc.real, hc.imag], -1)
    cls = readout_np(p, hflat).argmax(-1)
    table = task["table"]
    trans = np.array([[table[toks[x], st] for x in range(len(toks))] for st in range(n)])
    return trans, cls, res


def float_long_jax(meta, p, xs, chunk=64):
    """float32 JAX forward of the trained model (same code path as training) -> predictions."""
    import jax, jax.numpy as jnp
    import st
    fwd = jax.jit(st.make_forward(meta["model"]))
    pj = {k: jnp.asarray(v) for k, v in p.items()}
    out = []
    for i in range(0, len(xs), chunk):
        logits = fwd(pj, jnp.asarray(xs[i:i + chunk]))
        out.append(np.asarray(jnp.argmax(logits, -1)).T)
    return np.concatenate(out, 0)


LONG = 4096


if __name__ == "__main__":
    runs = sys.argv[1:]
    out = []
    for r in runs:
        meta = json.load(open(r + ".json"))
        p = dict(np.load(r + ".npz"))
        task = get_task(meta["task"], meta["gens"])
        V = len(task["tokens"])
        rng = np.random.default_rng(12345)
        xs, ys = sample_batch(rng, task, 512, 512)      # same test set as st.evaluate
        t0 = time.process_time()
        if meta["model"] == "cplx_u":
            trans, cls, res = cplx_snap(p, task)
            mode = "integer table"
            lane_info = None
            nstates = len(cls)
            lane_res = res.tolist()
            # serve: state = group element index; start at identity
            B, L = xs.shape
            st = np.full(B, task["ident"])
            pred = np.empty((B, L), dtype=np.int64)
            for t in range(L):
                st = trans[st, xs[:, t]]
                pred[:, t] = cls[st]
        else:
            lanes, trans, cls, nstates = compile_automaton(meta["model"], p, task)
            lane_res = [l["res"] for l in lanes]
            lane_info = [dict(res=round(l["res"], 4), keep=l["keep"], consistent=l.get("consistent"),
                              joint=l.get("joint")) for l in lanes]
            if trans is None:
                mode = "snapped-float64 (BFS cap exceeded)"
                pred = snapped_float(meta["model"], p, lanes, xs)
            else:
                mode = "integer table"
                pred = serve(trans, cls, xs)
        t_comp = time.process_time() - t0
        corr = pred == ys
        # very long sequences: exact table vs float32 model
        rngL = np.random.default_rng(999)
        xl, yl = sample_batch(rngL, task, 32, LONG)
        if mode != "integer table":
            pl = snapped_float(meta["model"], p, lanes, xl)
        elif meta["model"] == "cplx_u":
            stl = np.full(32, task["ident"])
            pl = np.empty_like(xl)
            for t in range(LONG):
                stl = trans[stl, xl[:, t]]
                pl[:, t] = cls[stl]
        else:
            pl = serve(trans, cls, xl)
        pf = float_long_jax(meta, p, xl) if not SKIP_FLOAT else None
        long_band = slice(LONG // 2, LONG)
        long = dict(table=float((pl == yl)[:, long_band].mean()),
                    float32=None if pf is None else float((pf == yl)[:, long_band].mean()))
        bands = {L: float(corr[:, L // 2:L].mean()) for L in (32, 64, 128, 256, 512)}
        fl = {int(k): v["acc_band"] for k, v in meta["eval"].items()}
        rec = dict(run=os.path.basename(r), model=meta["model"], task=meta["task"], nstates=int(nstates),
                   lane_residual=[round(x, 4) for x in lane_res], table_acc=bands, float_acc=fl,
                   table_final512=float(corr[:, -1].mean()), compile_cpu_s=t_comp,
                   acc_band_2048_4096=long, lanes=lane_info, mode=mode, strict=STRICT,
                   kept=None if lane_info is None else sum(l["keep"] for l in lane_info),
                   kept_coords=None if lane_info is not None else int(sum(1 for x in lane_res if x < 0.1)))
        out.append(rec)
        print(json.dumps(rec), flush=True)
    if out:
        fn = "snap_results_strict.jsonl" if STRICT else "snap_results.jsonl"
        with open(os.path.join(os.path.dirname(runs[0]), "..", fn), "a") as f:
            for rec in out:
                f.write(json.dumps(rec) + "\n")
