"""Lab lead scratch: convert a TRAINED dot-product attention LM to hyperbolic (Lorentz) attention.

Teacher: textlm 'attn' model (3 layers, width 128, single-head RoPE attention, SwiGLU MLP, byte-level WikiText-2),
params in exp/lead/full_attn_s0_params.pkl. The converted model keeps every weight; only the score function of each
attention layer changes:
  dot     : q.k / sqrt(d)                                   (teacher; control)
  lorentz : beta * (delta - arcosh(q0 k0 - <q,k>))          (q, k lifted to the hyperboloid after RoPE)
  cos     : beta * cos(q, k)
  euclid  : beta * (delta - |q - k|)
Softmax is shift-invariant per query row and there is no NoRead slot, so delta has no effect here; beta is fitted per
layer by least squares to the teacher's row-centred scores (zero-shot), then trained.
Stages: (0) teacher; (1) zero-shot converted; (2) KD, train Wq, Wk, beta only; (3) KD, train all weights.
KD loss = KL(teacher || student) per position over the 256 byte classes; evaluation = bits/byte at the training length.
"""
import argparse, json, math, pickle, sys, time
import numpy as np
import jax
import jax.numpy as jnp

LEAD = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/exp/lead"
sys.path.insert(0, LEAD)
import textlm as TL  # noqa: E402

WIKI = "/home/user/uor-r4/research/ai-research/ai-router/router-research/data/lm_proxy/raw/wikitext2/"


def scores(q, k, var, prm, d):
    kt = jnp.swapaxes(k, 1, 2)
    if var == "dot":
        return q @ kt / math.sqrt(d)
    b = jnp.exp(prm["logb"])
    if var == "gromov":
        # Gromov product (q|k)_o = (d(o,q) + d(o,k) - d(q,k)) / 2; d(o,q) is row-constant and dropped.
        # In a rooted tree it is the depth of the lowest common ancestor of q and k.
        eps = jnp.exp(prm.get("logs", 0.0))
        a, bb = q * eps, k * eps
        a2 = jnp.sum(a * a, -1, keepdims=True)
        b2 = jnp.sum(bb * bb, -1, keepdims=True)
        xm = a2 / (1.0 + jnp.sqrt(1.0 + a2))
        ym = jnp.swapaxes(b2 / (1.0 + jnp.sqrt(1.0 + b2)), 1, 2)
        z = jnp.maximum(xm + ym + xm * ym - a @ jnp.swapaxes(bb, 1, 2), 1e-10)
        dist = jnp.log1p(z + jnp.sqrt(z * (z + 2.0)))
        rk = jnp.log1p(ym + jnp.sqrt(ym * (ym + 2.0) + 1e-20))
        return b * 0.5 * (rk - dist)
    if var in ("hpol", "hkb"):
        eps = jnp.exp(prm.get("logs", 0.0))
        a, bb = q * eps, k * eps
        a2 = jnp.sum(a * a, -1, keepdims=True)
        b2 = jnp.sum(bb * bb, -1, keepdims=True)
        xm = a2 / (1.0 + jnp.sqrt(1.0 + a2))                 # x0 - 1, stable
        ym = b2 / (1.0 + jnp.sqrt(1.0 + b2))                 # y0 - 1, stable
        ymT = jnp.swapaxes(ym, 1, 2)
        z = xm + ymT + xm * ymT - a @ jnp.swapaxes(bb, 1, 2)   # -<x,y>_L - 1 >= 0
        z = jnp.maximum(z, 1e-10)
        dist = jnp.log1p(z + jnp.sqrt(z * (z + 2.0)))
        rk = jnp.log1p(ymT + jnp.sqrt(ymT * (ymT + 2.0) + 1e-20))
        if var == "hkb":   # arch2's Euclidean key-norm bias: [|k|^2 - d_kappa(q,k)^2] / (2 sqrt d), kappa = eps^2
            return b / math.sqrt(d) * (jnp.swapaxes(b2, 1, 2) - dist * dist) / (2.0 * eps * eps)
        return b / math.sqrt(d) * (rk * rk - dist * dist) / (2.0 * eps * eps)
    if var == "lorentz":
        sc = jnp.exp(prm.get("logs", 0.0))
        q, k = q * sc, k * sc
        kt = jnp.swapaxes(k, 1, 2)
        q0 = jnp.sqrt(1.0 + jnp.sum(q * q, -1, keepdims=True))
        k0 = jnp.sqrt(1.0 + jnp.sum(k * k, -1, keepdims=True))
        z = q0 * jnp.swapaxes(k0, 1, 2) - q @ kt
        dist = jnp.arccosh(jnp.maximum(z, 1.0 + 1e-6))
        return b * (prm["delta"] - dist)
    if var == "cos":
        qn = q * jax.lax.rsqrt(jnp.sum(q * q, -1, keepdims=True) + 1e-6)
        kn = k * jax.lax.rsqrt(jnp.sum(k * k, -1, keepdims=True) + 1e-6)
        return b * (qn @ jnp.swapaxes(kn, 1, 2))
    if var == "euclid":
        q2 = jnp.sum(q * q, -1, keepdims=True)
        k2 = jnp.swapaxes(jnp.sum(k * k, -1, keepdims=True), 1, 2)
        dist = jnp.sqrt(jnp.maximum(q2 + k2 - 2.0 * (q @ kt), 1e-6))
        return b * (prm["delta"] - dist)
    raise ValueError(var)


def attn_layer(b, x, var, prm):
    B, T, d = x.shape
    v = x @ b["Wv"]
    g = jax.nn.silu(x @ b["Wg"])
    q, k = TL.rope(x @ b["Wq"]), TL.rope(x @ b["Wk"])
    att = scores(q, k, var, prm, d)
    mask = jnp.tril(jnp.ones((T, T), bool))
    att = jnp.where(mask, att, -1e30)
    h = jax.nn.softmax(att, -1) @ v
    return (h * g) @ b["Wo"]


def forward(p, conv, tokens, var):
    x = p["emb"][tokens]
    for li, b in enumerate(p["blocks"]):
        x = x + attn_layer(b, TL.rmsnorm(x, b["n1"]), var, conv[li])
        y = TL.rmsnorm(x, b["n2"])
        x = x + (jax.nn.silu(y @ b["W1"]) * (y @ b["W2"])) @ b["W3"]
    return TL.rmsnorm(x, p["nf"]) @ p["out"]


def qk_rows(p, tokens):
    """Teacher q, k (after RoPE) per layer, running the teacher forward."""
    out = []
    x = p["emb"][tokens]
    for b in p["blocks"]:
        xn = TL.rmsnorm(x, b["n1"])
        out.append((TL.rope(xn @ b["Wq"]), TL.rope(xn @ b["Wk"])))
        x = x + attn_layer(b, xn, "dot", None)
        y = TL.rmsnorm(x, b["n2"])
        x = x + (jax.nn.silu(y @ b["W1"]) * (y @ b["W2"])) @ b["W3"]
    return out


def fit_beta(p, tokens, var):
    """Least-squares beta per layer so beta * f(q,k) matches the teacher's row-centred dot scores (causal pairs).
    For lorentz also grid-search the input scale; returns conv params and the explained-variance fraction per layer."""
    conv, r2s = [], []
    grid = [x * 0.5 for x in range(-8, 11)] if var in ("lorentz", "gromov") else [0.0]
    for q, k in qk_rows(p, tokens):
        d = q.shape[-1]
        T = q.shape[1]
        s = np.asarray(q @ jnp.swapaxes(k, 1, 2)) / math.sqrt(d)
        mask = np.tril(np.ones((T, T), bool))
        best = None
        for logs in grid:
            unit = {"logb": jnp.array(0.0), "delta": jnp.array(0.0), "logs": jnp.array(logs)}
            f = np.asarray(scores(q, k, var, unit, d))       # beta = 1, delta = 0: the raw geometric score
            num = den = tot = 0.0
            for row in range(T):
                m = mask[row]
                sc = s[:, row, m] - s[:, row, m].mean(-1, keepdims=True)
                fc = f[:, row, m] - f[:, row, m].mean(-1, keepdims=True)
                num += float((sc * fc).sum())
                den += float((fc * fc).sum())
                tot += float((sc * sc).sum())
            beta = max(num / max(den, 1e-12), 1e-3)
            r2 = (num * num / max(den, 1e-12)) / max(tot, 1e-12) if num > 0 else 0.0
            if best is None or r2 > best[0]:
                best = (r2, beta, logs)
        r2s.append(best[0])
        conv.append({"logb": jnp.array(math.log(best[1])), "delta": jnp.array(0.0), "logs": jnp.array(best[2])})
    return conv, r2s


def bpb(p, conv, valid, var, T=128, windows=256):
    stride = (len(valid) - T - 1) // windows
    tot = 0.0
    fwd = jax.jit(lambda xb: forward(p, conv, xb, var))
    for s in range(0, windows, 32):
        idx = np.arange(s, min(s + 32, windows)) * stride
        xb = np.stack([valid[i:i + T] for i in idx]).astype(np.int32)
        yb = np.stack([valid[i + 1:i + T + 1] for i in idx]).astype(np.int32)
        logits = fwd(xb)
        lz = jax.nn.logsumexp(logits, -1)
        tgt = jnp.take_along_axis(logits, yb[..., None], -1)[..., 0]
        tot += float(jnp.sum(lz - tgt))
    return tot / (windows * T) / math.log(2)


def kd_train(teacher, p, conv, train, var, trainable, steps, lr, seed, T=128, B=32):
    """KL(teacher || student) with Adam on the selected leaves; returns updated (p, conv)."""
    params = {"p": p, "conv": conv}
    def select(tree):
        return {"p": {"blocks": [{k: v for k, v in b.items() if k in trainable.get("block", ())} for b in tree["p"]["blocks"]],
                      **({k: tree["p"][k] for k in trainable.get("top", ())})},
                "conv": [{k: c[k] for k in trainable.get("conv", ())} for c in tree["conv"]]}
    def merge(full, part):
        out = {"p": dict(full["p"]), "conv": [dict(c) for c in full["conv"]]}
        out["p"]["blocks"] = [dict(b) for b in full["p"]["blocks"]]
        for i, b in enumerate(part["p"]["blocks"]):
            out["p"]["blocks"][i].update(b)
        for k in trainable.get("top", ()):
            out["p"][k] = part["p"][k]
        for i, c in enumerate(part["conv"]):
            out["conv"][i].update(c)
        return out
    tfwd = jax.jit(lambda xb: jax.nn.log_softmax(forward(teacher, [None] * len(teacher["blocks"]), xb, "dot"), -1))
    def loss(part, full, xb, tlog):
        m = merge(full, part)
        slog = jax.nn.log_softmax(forward(m["p"], m["conv"], xb, var), -1)
        return jnp.mean(jnp.sum(jnp.exp(tlog) * (tlog - slog), -1))
    grad = jax.jit(jax.value_and_grad(loss))
    part = select(params)
    mom = jax.tree_util.tree_map(jnp.zeros_like, part)
    vel = jax.tree_util.tree_map(jnp.zeros_like, part)
    rng = np.random.default_rng(seed)
    last = []
    for t in range(1, steps + 1):
        idx = rng.integers(0, len(train) - T - 1, B)
        xb = np.stack([train[i:i + T] for i in idx]).astype(np.int32)
        l, gr = grad(part, params, xb, tfwd(xb))
        mom = jax.tree_util.tree_map(lambda m, g: 0.9 * m + 0.1 * g, mom, gr)
        vel = jax.tree_util.tree_map(lambda v, g: 0.999 * v + 0.001 * g * g, vel, gr)
        part = jax.tree_util.tree_map(lambda w, m, v: w - lr * (m / (1 - 0.9 ** t)) / (jnp.sqrt(v / (1 - 0.999 ** t)) + 1e-8),
                                      part, mom, vel)
        last.append(float(l))
    out = merge(params, part)
    return out["p"], out["conv"], float(np.mean(last[-50:]))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--var", required=True, choices=["dot", "lorentz", "cos", "euclid", "hpol", "gromov", "hkb"])
    ap.add_argument("--steps", type=int, default=400)
    ap.add_argument("--init_logs", type=float, default=-6.0)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    teacher = pickle.load(open(f"{LEAD}/full_attn_s0_params.pkl", "rb"))
    teacher = jax.tree_util.tree_map(jnp.asarray, teacher)
    train = np.frombuffer(open(WIKI + "train.txt", "rb").read(), dtype=np.uint8)
    valid = np.frombuffer(open(WIKI + "valid.txt", "rb").read(), dtype=np.uint8)
    res = {"var": a.var, "steps": a.steps}
    t0 = time.time()
    res["teacher_bpb"] = bpb(teacher, [None] * 3, valid, "dot")
    rng = np.random.default_rng(0)
    idx = rng.integers(0, len(train) - 129, 16)
    calib = jnp.asarray(np.stack([train[i:i + 128] for i in idx]).astype(np.int32))
    if a.var in ("hpol", "hkb"):
        conv = [{"logb": jnp.array(0.0), "delta": jnp.array(0.0), "logs": jnp.array(a.init_logs)} for _ in range(3)]
        res["fit_r2"] = None
    elif a.var == "dot":
        conv = [{"logb": jnp.array(0.0), "delta": jnp.array(0.0), "logs": jnp.array(0.0)} for _ in range(3)]
        res["fit_r2"] = [1.0] * 3
    else:
        conv, res["fit_r2"] = fit_beta(teacher, calib, a.var)
    res["beta_fit"] = [float(jnp.exp(c["logb"])) for c in conv]
    res["logs_fit"] = [float(c["logs"]) for c in conv]
    res["zero_shot_bpb"] = bpb(teacher, conv, valid, a.var)
    print(json.dumps({k: res[k] for k in ("teacher_bpb", "fit_r2", "beta_fit", "logs_fit", "zero_shot_bpb")}), flush=True)
    p2, c2, l2 = kd_train(teacher, teacher, conv, train, a.var,
                          {"block": ("Wq", "Wk"), "conv": ("logb", "logs") if a.var in ("lorentz", "hpol", "gromov", "hkb") else (("logb",) if a.var != "dot" else ())}, a.steps, 1e-3, 1)
    res["qk_kd_bpb"] = bpb(p2, c2, valid, a.var)
    res["qk_kd_final_kl"] = l2
    res["beta_after_qk_kd"] = [float(jnp.exp(c["logb"])) for c in c2]
    res["logs_after_qk_kd"] = [float(c["logs"]) for c in c2]
    print(json.dumps({"qk_kd_bpb": res["qk_kd_bpb"], "kl": l2}), flush=True)
    allb = ("Wq", "Wk", "Wv", "Wg", "Wo", "W1", "W2", "W3", "n1", "n2", "hn")
    p3, c3, l3 = kd_train(teacher, p2, c2, train, a.var,
                          {"block": allb, "top": ("emb", "out", "nf"), "conv": ("logb", "logs") if a.var in ("lorentz", "hpol", "gromov", "hkb") else (("logb",) if a.var != "dot" else ())},
                          a.steps, 5e-4, 2)
    res["all_kd_bpb"] = bpb(p3, c3, valid, a.var)
    res["all_kd_final_kl"] = l3
    res["seconds"] = time.time() - t0
    print(json.dumps({"all_kd_bpb": res["all_kd_bpb"], "kl": l3, "seconds": res["seconds"]}), flush=True)
    json.dump(res, open(a.out, "w"), indent=1)


if __name__ == "__main__":
    main()
