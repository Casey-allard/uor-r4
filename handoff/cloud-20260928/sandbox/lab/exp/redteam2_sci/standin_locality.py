"""redteam2_sci R1: how much of the stand-in teacher's attention is content-based?

The lead's stand-in (3-layer, width-128, single-head RoPE attention, byte-level WikiText-2, 1.990 bpb) was used to argue
that "swapping a trained model's score geometry is cheap". If its attention is mostly a fixed local/positional pattern,
the stand-in cannot say anything about SmolLM2's content-addressed heads (induction, retrieval, sinks).

(1) Attention mass by relative distance per layer (teacher, dot score), on training windows.
(2) Zero-shot bits/byte when every layer's attention is replaced by a CONTENT-FREE pattern:
    (a) the teacher's own mean attention-by-distance profile (Toeplitz, per layer) -- best content-free pattern of that form,
    (b) uniform over the last w positions (w = 1, 4, 16),
    (c) no attention at all (attention output zeroed).
No training anywhere. Same evaluator as conv.py (bpb on 256 validation windows of 128 bytes).
"""
import json, math, sys, time
import numpy as np
import jax, jax.numpy as jnp

S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
sys.path.insert(0, f"{S}/lab/exp/lead_conv")
import conv as C          # noqa: E402  (imports the lead's textlm as C.TL)
import pickle             # noqa: E402

T = 128
teacher = jax.tree_util.tree_map(jnp.asarray, pickle.load(open(f"{C.LEAD}/full_attn_s0_params.pkl", "rb")))
train = np.frombuffer(open(C.WIKI + "train.txt", "rb").read(), dtype=np.uint8)
valid = np.frombuffer(open(C.WIKI + "valid.txt", "rb").read(), dtype=np.uint8)
TL = C.TL
mask = jnp.tril(jnp.ones((T, T), bool))


def attn_probs(b, xn):
    q, k = TL.rope(xn @ b["Wq"]), TL.rope(xn @ b["Wk"])
    d = q.shape[-1]
    att = q @ jnp.swapaxes(k, 1, 2) / math.sqrt(d)
    att = jnp.where(mask, att, -1e30)
    return jax.nn.softmax(att, -1), q, k


def forward_pattern(p, tokens, patterns):
    """patterns[li]: None = teacher attention; 'zero' = no attention; else a (T,T) row-stochastic causal matrix."""
    x = p["emb"][tokens]
    for li, b in enumerate(p["blocks"]):
        xn = TL.rmsnorm(x, b["n1"])
        pat = patterns[li]
        if isinstance(pat, str) and pat == "zero":
            a_out = 0.0
        else:
            v = xn @ b["Wv"]
            g = jax.nn.silu(xn @ b["Wg"])
            P = attn_probs(b, xn)[0] if pat is None else pat[None]
            a_out = ((P @ v) * g) @ b["Wo"]
        x = x + a_out
        y = TL.rmsnorm(x, b["n2"])
        x = x + (jax.nn.silu(y @ b["W1"]) * (y @ b["W2"])) @ b["W3"]
    return TL.rmsnorm(x, p["nf"]) @ p["out"]


def bpb(patterns, windows=256):
    stride = (len(valid) - T - 1) // windows
    fwd = jax.jit(lambda xb: forward_pattern(teacher, xb, patterns))
    tot = 0.0
    for s in range(0, windows, 32):
        idx = np.arange(s, min(s + 32, windows)) * stride
        xb = np.stack([valid[i:i + T] for i in idx]).astype(np.int32)
        yb = np.stack([valid[i + 1:i + T + 1] for i in idx]).astype(np.int32)
        lg = fwd(xb)
        lz = jax.nn.logsumexp(lg, -1)
        tot += float(jnp.sum(lz - jnp.take_along_axis(lg, yb[..., None], -1)[..., 0]))
    return tot / (windows * T) / math.log(2)


t0 = time.time()
rng = np.random.default_rng(7)
idx = rng.integers(0, len(train) - T - 1, 64)
xb = jnp.asarray(np.stack([train[i:i + T] for i in idx]).astype(np.int32))

# (1) attention by distance, teacher forward
out = {"distance_profile": [], "qk_norms": []}
x = teacher["emb"][xb]
toeplitz = []
dist = np.arange(T)[:, None] - np.arange(T)[None, :]          # t - j  (>= 0 on the causal part)
for li, b in enumerate(teacher["blocks"]):
    xn = TL.rmsnorm(x, b["n1"])
    P, q, k = attn_probs(b, xn)
    P = np.asarray(P)                                          # (B,T,T)
    prof = np.zeros(T)
    cnt = np.zeros(T)
    for dd in range(T):
        m = dist == dd
        prof[dd] = P[:, m].mean() if m.any() else 0.0
    bins = {"d0": prof[0], "d1": prof[1], "d2-3": prof[2:4].sum(), "d4-7": prof[4:8].sum(), "d8-15": prof[8:16].sum(),
            "d16-31": prof[16:32].sum(), "d32-127": prof[32:].sum()}
    # expected mass per bin for a query at the last position (t = T-1) and averaged over rows t >= 32
    rows = np.arange(32, T)
    row_mass = {}
    for name, (lo, hi) in {"d0": (0, 1), "d1": (1, 2), "d2-3": (2, 4), "d4-7": (4, 8), "d8-15": (8, 16),
                           "d16-31": (16, 32), "d32+": (32, T)}.items():
        mm = (dist[rows] >= lo) & (dist[rows] < hi)
        row_mass[name] = float((P[:, rows, :] * mm[None]).sum(-1).mean())
    ent = float((-(P * np.log(P + 1e-30)).sum(-1))[:, rows].mean())
    out["distance_profile"].append({"layer": li, "row_mass_rows32plus": row_mass, "entropy_nats_rows32plus": ent})
    out["qk_norms"].append({"layer": li, "q_rms": float(np.sqrt((np.asarray(q) ** 2).sum(-1).mean())),
                            "k_rms": float(np.sqrt((np.asarray(k) ** 2).sum(-1).mean()))})
    # Toeplitz content-free pattern from the mean profile: A[t, j] ∝ prof[t-j], causal, row-normalized
    A = np.where(dist >= 0, prof[np.clip(dist, 0, T - 1)], 0.0)
    A = A / A.sum(1, keepdims=True)
    toeplitz.append(jnp.asarray(A, jnp.float32))
    # advance x with the teacher's own layer
    v = xn @ b["Wv"]; g = jax.nn.silu(xn @ b["Wg"])
    x = x + ((jnp.asarray(P) @ v) * g) @ b["Wo"]
    y = TL.rmsnorm(x, b["n2"])
    x = x + (jax.nn.silu(y @ b["W1"]) * (y @ b["W2"])) @ b["W3"]
print(json.dumps(out), flush=True)


def uniform_last(w):
    A = np.where((dist >= 0) & (dist < w), 1.0, 0.0)
    return jnp.asarray(A / A.sum(1, keepdims=True), jnp.float32)


res = {}
res["teacher"] = bpb([None, None, None]); print("teacher", res["teacher"], flush=True)
res["toeplitz_all_layers"] = bpb(toeplitz); print("toeplitz all", res["toeplitz_all_layers"], flush=True)
for li in range(3):
    pats = [None, None, None]; pats[li] = toeplitz[li]
    res[f"toeplitz_layer{li}_only"] = bpb(pats); print("toeplitz layer", li, res[f"toeplitz_layer{li}_only"], flush=True)
for w in (1, 4, 16):
    U = uniform_last(w)
    res[f"uniform_last{w}_all"] = bpb([U, U, U]); print("uniform", w, res[f"uniform_last{w}_all"], flush=True)
res["no_attention_all"] = bpb(["zero", "zero", "zero"]); print("zero", res["no_attention_all"], flush=True)
out["bpb"] = res
out["seconds"] = time.time() - t0
json.dump(out, open(f"{S}/lab/exp/redteam2_sci/standin_locality.json", "w"), indent=1)
print(json.dumps(res), round(time.time() - t0, 1))
