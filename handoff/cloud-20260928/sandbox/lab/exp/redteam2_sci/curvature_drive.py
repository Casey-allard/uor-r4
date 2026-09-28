"""redteam2_sci R5: zero-training "does the objective want curvature?" test at the flat limit (demonstrated on the stand-in).

Near the flat limit the conversion scores are  s_k = s_0 + k * F(q,k) + O(k^2)   (verified in taylor_check.py), with
  key-norm  : F = (|q|^2-|k|^2)^2/8 + |q-k|^4/24
  intrinsic : F = (|q|^2-|k|^2)^2/8 + |q-k|^4/24 - |k|^4/6        (all divided by sqrt(d), like the dot score)
so dL/dk at k = 0 is exact from one forward/backward pass. Use the dimensionless curvature t = k * m, m = median |k|^2
of the layer (t ~ 1 is where the geometry is genuinely hyperbolic). g = dL/dt < 0 means the objective pulls the head
toward curvature at first order; g >= 0 means gradient descent keeps (or returns) it to the flat limit.
Objectives: next-byte NLL on training windows (data), and KL to the unconverted teacher (self-KD, g must be 0 at t=0:
sanity check). Also the first-order predicted NLL change at t = 0.1 and t = 1 (g * t), in bits/byte.
"""
import json, math, sys, pickle
import numpy as np
import jax, jax.numpy as jnp

S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
sys.path.insert(0, f"{S}/lab/exp/lead_conv")
import conv as C  # noqa: E402
TL = C.TL
teacher = jax.tree_util.tree_map(jnp.asarray, pickle.load(open(f"{C.LEAD}/full_attn_s0_params.pkl", "rb")))
train = np.frombuffer(open(C.WIKI + "train.txt", "rb").read(), dtype=np.uint8)
T = 128
mask = jnp.tril(jnp.ones((T, T), bool))


def feat(q, k, kind):
    q2 = jnp.sum(q * q, -1, keepdims=True); k2 = jnp.swapaxes(jnp.sum(k * k, -1, keepdims=True), 1, 2)
    qk = q @ jnp.swapaxes(k, 1, 2)
    dist2 = q2 + k2 - 2 * qk
    F = (q2 - k2) ** 2 / 8 + dist2 ** 2 / 24
    if kind == "intrinsic":
        F = F - k2 ** 2 / 6
    return F


def forward(p, tokens, ts, meds, kind):
    x = p["emb"][tokens]
    for li, b in enumerate(p["blocks"]):
        xn = TL.rmsnorm(x, b["n1"])
        q, k = TL.rope(xn @ b["Wq"]), TL.rope(xn @ b["Wk"])
        d = q.shape[-1]
        s = (q @ jnp.swapaxes(k, 1, 2) + (ts[li] / meds[li]) * feat(q, k, kind)) / math.sqrt(d)
        s = jnp.where(mask, s, -1e30)
        v = xn @ b["Wv"]; g = jax.nn.silu(xn @ b["Wg"])
        x = x + ((jax.nn.softmax(s, -1) @ v) * g) @ b["Wo"]
        y = TL.rmsnorm(x, b["n2"])
        x = x + (jax.nn.silu(y @ b["W1"]) * (y @ b["W2"])) @ b["W3"]
    return TL.rmsnorm(x, p["nf"]) @ p["out"]


rng = np.random.default_rng(11)
batches = []
for _ in range(8):
    idx = rng.integers(0, len(train) - T - 1, 32)
    batches.append((np.stack([train[i:i + T] for i in idx]).astype(np.int32),
                    np.stack([train[i + 1:i + T + 1] for i in idx]).astype(np.int32)))
# median |k|^2 per layer (teacher)
meds = []
x = teacher["emb"][batches[0][0]]
for b in teacher["blocks"]:
    xn = TL.rmsnorm(x, b["n1"]); k = TL.rope(xn @ b["Wk"]); q = TL.rope(xn @ b["Wq"])
    meds.append(float(np.median(np.sum(np.asarray(k) ** 2, -1))))
    s = jnp.where(mask, q @ jnp.swapaxes(k, 1, 2) / math.sqrt(q.shape[-1]), -1e30)
    v = xn @ b["Wv"]; g = jax.nn.silu(xn @ b["Wg"])
    x = x + ((jax.nn.softmax(s, -1) @ v) * g) @ b["Wo"]
    y = TL.rmsnorm(x, b["n2"]); x = x + (jax.nn.silu(y @ b["W1"]) * (y @ b["W2"])) @ b["W3"]
meds = jnp.asarray(meds)
out = {"median_k2": [float(m) for m in meds]}
for kind in ("intrinsic", "key_norm"):
    def nll(ts, xb, yb):
        lg = forward(teacher, xb, ts, meds, kind)
        return jnp.mean(jax.nn.logsumexp(lg, -1) - jnp.take_along_axis(lg, yb[..., None], -1)[..., 0]) / math.log(2)
    def kd(ts, xb, yb):
        tl = jax.nn.log_softmax(forward(teacher, xb, jnp.zeros(3), meds, kind), -1)
        sl = jax.nn.log_softmax(forward(teacher, xb, ts, meds, kind), -1)
        return jnp.mean(jnp.sum(jnp.exp(tl) * (tl - sl), -1)) / math.log(2)
    gn, gk = jax.jit(jax.grad(nll)), jax.jit(jax.grad(kd))
    ln = jax.jit(nll)
    G = np.array([np.asarray(gn(jnp.zeros(3), xb, yb)) for xb, yb in batches])      # (batches, layers)
    K = np.array([np.asarray(gk(jnp.zeros(3), xb, yb)) for xb, yb in batches])
    # finite NLL change with the EXACT curved score (conv.scores 'hpol' = intrinsic, 'hkb' = key-norm), one layer at a
    # time at t = 0.1 and t = 1 (other layers at log eps = -12, numerically the flat limit)
    var = "hpol" if kind == "intrinsic" else "hkb"
    def exact_nll(logs3, xb, yb):
        cv = [{"logb": jnp.array(0.0), "delta": jnp.array(0.0), "logs": jnp.array(l)} for l in logs3]
        lg = C.forward(teacher, cv, xb, var)
        return float(jnp.mean(jax.nn.logsumexp(lg, -1) - jnp.take_along_axis(lg, yb[..., None], -1)[..., 0]) / math.log(2))
    base = np.mean([exact_nll([-12.0] * 3, xb, yb) for xb, yb in batches[:4]])
    fin = {}
    for li in range(3):
        for t in (0.1, 1.0):
            l3 = [-12.0] * 3; l3[li] = 0.5 * math.log(t / float(meds[li]))
            fin[f"layer{li}_t{t}"] = round(float(np.mean([exact_nll(l3, xb, yb) for xb, yb in batches[:4]]) - base), 5)
    out[kind] = {"dNLL_dt_bits_per_byte_mean": [round(float(v), 5) for v in G.mean(0)],
                 "dNLL_dt_sd_over_batches": [round(float(v), 5) for v in G.std(0)],
                 "frac_batches_negative": [round(float(v), 3) for v in (G < 0).mean(0)],
                 "dKL_dt_selfKD_mean": [float(v) for v in K.mean(0)],
                 "finite_dNLL_single_layer": fin, "base_bpb_train_windows": round(base, 4)}
    print(kind, json.dumps(out[kind]), flush=True)
json.dump(out, open(f"{S}/lab/exp/redteam2_sci/curvature_drive.json", "w"), indent=1)
