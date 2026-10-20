"""math2 E11b: exact conversion keeps the teacher's blind spot (E11 arm D). Does annealing the completion radius QC down
during fine-tuning (dot limit -> natural hyperbolic lift) let a converted head exceed its dot teacher on the code tree?

Same task/model/teacher as e11_transfer.py (teacher A retrained with the same seed and batches).
Arms (1500 fine-tune steps each from teacher A):
  G  converted norm-completed Lorentz; log QC annealed linearly from ln 1e4 to ln 1 over the first 750 steps
     (the clamps q0 >= sqrt(1+|q|^2), k0 >= sqrt(1+|k|^2) then make it the natural Lorentz lift); free radii rho
  H  direct 'curved' conversion: natural Lorentz lift of the teacher's q, k from step 0 (beta, delta fitted by SGD)
"""
import importlib.util, json, math, sys, time
import numpy as np
import jax, jax.numpy as jnp

S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
spec = importlib.util.spec_from_file_location("e11", f"{S}/lab/exp/math2/e11_transfer.py")
src = open(f"{S}/lab/exp/math2/e11_transfer.py").read().split("out = {}; t0 = time.time()")[0]   # helpers only
ns = {}; exec(src, ns)
geo, train, evaluate, base_params, scores, forward = ns["geo"], ns["train"], ns["evaluate"], ns["base_params"], ns["scores"], ns["forward"]
Q_, DK, D_MODEL, K = ns["Q"], ns["DK"], ns["D_MODEL"], ns["K"]
STEPS = 1500


def loss_sched(p, toks, tgt, logqc):
    pp = dict(p, logQ=0.5 * logqc, logC=0.5 * logqc, lb=p["lb"] + jnp.log(jnp.exp(logqc) / math.sqrt(DK) + 1.0))
    lg = forward(pp, toks, "lor")
    return jnp.mean(jax.nn.logsumexp(lg, -1) - jnp.take_along_axis(lg, tgt[..., None], -1)[..., 0])


def train_anneal(p, steps, seed, qc_hi, qc_lo, anneal_steps):
    m = jax.tree_util.tree_map(jnp.zeros_like, p); v2 = jax.tree_util.tree_map(jnp.zeros_like, p)

    @jax.jit
    def step(p, m, v2, toks, tgt, lr, t, logqc):
        l, gr = jax.value_and_grad(loss_sched)(p, toks, tgt, logqc)
        gn = jnp.sqrt(sum(jnp.sum(x * x) for x in jax.tree_util.tree_leaves(gr)))
        gr = jax.tree_util.tree_map(lambda x: x * jnp.minimum(1.0, 1.0 / (gn + 1e-6)), gr)
        m = jax.tree_util.tree_map(lambda a_, g: 0.9 * a_ + 0.1 * g, m, gr)
        v2 = jax.tree_util.tree_map(lambda a_, g: 0.99 * a_ + 0.01 * g * g, v2, gr)
        p = jax.tree_util.tree_map(lambda p_, mm, vv: p_ - lr * (mm / (1 - 0.9 ** t)) / (jnp.sqrt(vv / (1 - 0.99 ** t)) + 1e-8), p, m, v2)
        return p, m, v2, l
    rng = np.random.default_rng(seed)
    for it in range(1, steps + 1):
        toks, tgt, _ = geo.make_tree_batch(rng, 64, 48, Q_, 512)
        lr = 3e-3 * min(1.0, it / 100) * (0.1 + 0.9 * 0.5 * (1 + math.cos(math.pi * it / steps)))
        frac = min(1.0, it / anneal_steps) if anneal_steps > 0 else 1.0
        logqc = math.log(qc_hi) * (1 - frac) + math.log(qc_lo) * frac
        p, m, v2, l = step(p, m, v2, jnp.asarray(toks), jnp.asarray(tgt), lr, it, logqc)
    return p, logqc


out = {}; t0 = time.time()
pA = train(base_params(0), "dot", STEPS, 0); out["A_dot_scratch"] = evaluate(pA, "dot"); print("A", out["A_dot_scratch"], round(time.time() - t0), flush=True)
QC = 1e4
pG = dict(pA, wr=jnp.zeros(D_MODEL), lb=jnp.array(0.0), delta=jnp.array(math.log(2 * QC)))
pG, lq = train_anneal(pG, STEPS, 1, QC, 1.0, STEPS // 2)
eff = lambda pp, lq: dict(pp, logQ=jnp.array(0.5 * lq), logC=jnp.array(0.5 * lq), lb=pp["lb"] + math.log(math.exp(lq) / math.sqrt(DK) + 1.0))
out["G0_converted_before"] = evaluate(eff(dict(pA, wr=jnp.zeros(D_MODEL), lb=jnp.array(0.0), delta=jnp.array(math.log(2 * QC))), math.log(QC)), "lor")
out["G_converted_annealed"] = evaluate(eff(pG, lq), "lor")
out["G_scalars"] = {"lb": float(pG["lb"]), "delta": float(pG["delta"])}
print("G0", out["G0_converted_before"], "G", out["G_converted_annealed"], out["G_scalars"], round(time.time() - t0), flush=True)
pH = dict(pA, wr=jnp.zeros(D_MODEL), lb=jnp.array(0.0), delta=jnp.array(3.0))
pH, lq = train_anneal(pH, STEPS, 1, 1.0, 1.0, 0)
out["H0_curved_before"] = evaluate(eff(dict(pA, wr=jnp.zeros(D_MODEL), lb=jnp.array(0.0), delta=jnp.array(3.0)), 0.0), "lor")
out["H_curved_conversion"] = evaluate(eff(pH, lq), "lor")
print("H0", out["H0_curved_before"], "H", out["H_curved_conversion"], round(time.time() - t0), flush=True)
json.dump(out, open(f"{S}/lab/exp/math2/e11b_anneal.json", "w"), indent=1)
