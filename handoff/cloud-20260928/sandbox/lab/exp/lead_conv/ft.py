"""Lab lead scratch, conversion stage 2a: after converting the trained dot-product teacher, continue training ON DATA
(next-byte cross-entropy, not distillation) and let the curvature learn.

  dot    : continued training of the teacher itself (control)
  hpol   : exact conversion (flat limit, logs=-6) of the polarized squared-distance score; curvature (logs) learnable
  gromov : Gromov-product score, zero-shot fitted (not exact); curvature learnable
  lorentz: the committed Lorentz read form beta*(delta - d), zero-shot fitted

Decision this informs: does the data pull a converted model away from the flat limit (logs rising) and does that lower
validation bits/byte versus the equal-step dot control? Same data order per seed across variants.
"""
import argparse, json, math, pickle, time
import numpy as np
import jax
import jax.numpy as jnp
import conv as C


def ce(p, cv, xb, yb, var):
    logits = C.forward(p, cv, xb, var)
    lz = jax.nn.logsumexp(logits, -1)
    return jnp.mean(lz - jnp.take_along_axis(logits, yb[..., None], -1)[..., 0])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--var", required=True, choices=["dot", "hpol", "gromov", "lorentz", "hkb"])
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--steps", type=int, default=600)
    ap.add_argument("--lr", type=float, default=3e-4)
    ap.add_argument("--curv_lr", type=float, default=1e-2, help="Adam step size for logs (curvature scale)")
    ap.add_argument("--init_logs", type=float, default=-6.0)
    ap.add_argument("--eval_every", type=int, default=150)
    ap.add_argument("--anneal_to", type=float, default=None, help="raise a floor on logs linearly to this value")
    ap.add_argument("--anneal_steps", type=int, default=300)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    t0 = time.time()
    teacher = jax.tree_util.tree_map(jnp.asarray, pickle.load(open(f"{C.LEAD}/full_attn_s0_params.pkl", "rb")))
    train = np.frombuffer(open(C.WIKI + "train.txt", "rb").read(), dtype=np.uint8)
    valid = np.frombuffer(open(C.WIKI + "valid.txt", "rb").read(), dtype=np.uint8)
    if a.var in ("hpol", "hkb"):
        cv = [{"logb": jnp.array(0.0), "logs": jnp.array(a.init_logs)} for _ in range(3)]
        r2 = None
    elif a.var == "dot":
        cv = [{"logb": jnp.array(0.0), "logs": jnp.array(0.0)} for _ in range(3)]
        r2 = None
    else:
        rng0 = np.random.default_rng(0)
        idx = rng0.integers(0, len(train) - 129, 16)
        calib = jnp.asarray(np.stack([train[i:i + 128] for i in idx]).astype(np.int32))
        fit, r2 = C.fit_beta(teacher, calib, a.var)
        cv = [{"logb": c["logb"], "logs": c["logs"], "delta": c["delta"]} for c in fit]
    params = {"p": teacher, "cv": cv}
    lrs = jax.tree_util.tree_map(lambda _: a.lr, params)
    lrs["cv"] = [{k: (a.curv_lr if k == "logs" else a.lr) for k in c} for c in cv]
    grad = jax.jit(jax.value_and_grad(lambda P, xb, yb: ce(P["p"], P["cv"], xb, yb, a.var)))
    mom = jax.tree_util.tree_map(jnp.zeros_like, params)
    vel = jax.tree_util.tree_map(jnp.zeros_like, params)
    rng = np.random.default_rng(1000 + a.seed)
    log = [{"step": 0, "valid_bpb": C.bpb(params["p"], params["cv"], valid, a.var),
            "logs": [float(c["logs"]) for c in params["cv"]], "beta": [float(jnp.exp(c["logb"])) for c in params["cv"]]}]
    print(json.dumps(log[-1]), flush=True)
    run = []
    for t in range(1, a.steps + 1):
        idx = rng.integers(0, len(train) - 129, 32)
        xb = np.stack([train[i:i + 128] for i in idx]).astype(np.int32)
        yb = np.stack([train[i + 1:i + 129] for i in idx]).astype(np.int32)
        l, g = grad(params, xb, yb)
        mom = jax.tree_util.tree_map(lambda m, gg: 0.9 * m + 0.1 * gg, mom, g)
        vel = jax.tree_util.tree_map(lambda v, gg: 0.999 * v + 0.001 * gg * gg, vel, g)
        params = jax.tree_util.tree_map(
            lambda w, m, v, lr: w - lr * (m / (1 - 0.9 ** t)) / (jnp.sqrt(v / (1 - 0.999 ** t)) + 1e-8),
            params, mom, vel, lrs)
        if a.anneal_to is not None:
            floor = a.init_logs + (a.anneal_to - a.init_logs) * min(1.0, t / a.anneal_steps)
            params["cv"] = [dict(c, logs=jnp.maximum(c["logs"], floor)) for c in params["cv"]]
        run.append(float(l))
        if t % a.eval_every == 0 or t == a.steps:
            log.append({"step": t, "train_bpb": float(np.mean(run[-50:])) / math.log(2),
                        "valid_bpb": C.bpb(params["p"], params["cv"], valid, a.var),
                        "logs": [float(c["logs"]) for c in params["cv"]],
                        "beta": [float(jnp.exp(c["logb"])) for c in params["cv"]]})
            print(json.dumps(log[-1]), flush=True)
    res = {"var": a.var, "seed": a.seed, "steps": a.steps, "lr": a.lr, "curv_lr": a.curv_lr, "init_logs": a.init_logs,
           "anneal_to": a.anneal_to, "anneal_steps": a.anneal_steps,
           "fit_r2": r2, "log": log, "seconds": time.time() - t0}
    json.dump(res, open(a.out, "w"), indent=1)


if __name__ == "__main__":
    main()
