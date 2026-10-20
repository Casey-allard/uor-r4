"""E2: norm drift of the project's quantized lane transports over 256 steps.

Mirrors crates/uor-r4-training/src/joint_model.rs::transport_lanes_with_unit and the legacy
integer contract (crates/uor-r4-integer/src/config.rs): unit = normalize(e0 + alpha*raw) then
quantized to exponent -14 WITHOUT renormalization; state quantized to exponent -11 (clip +-32767).
Pure transport chain (update gate z = 0) isolates the transport's own drift.
"""
import numpy as np
rng = np.random.default_rng(0)
ALPHA = 0.1
def q_unit(u):  # exponent -14, codes clipped to +-32767
    return np.clip(np.round(u * 2**14), -32767, 32767) / 2**14
def q_state(x):  # exponent -11
    return np.clip(np.round(x * 2**11), -32767, 32767) / 2**11
def hamilton(q, x):
    w, a, b, c = q.T; x0, x1, x2, x3 = x.T
    return np.stack([w*x0 - a*x1 - b*x2 - c*x3,
                     w*x1 + a*x0 + b*x3 - c*x2,
                     w*x2 - a*x3 + b*x0 + c*x1,
                     w*x3 + a*x2 - b*x1 + c*x0], axis=1)
def householder_pair(u, x):
    r = x.copy(); r[:, 0] = -r[:, 0]
    return r - 2 * u * np.sum(u * r, axis=1, keepdims=True)
def unit(raw, kind):
    s = ALPHA if kind == 'Q' else ALPHA / np.sqrt(2)
    c = s * raw; c[:, 0] += 1
    return c / np.linalg.norm(c, axis=1, keepdims=True)
B, T = 4096, 256
for sigma in (1.0, 3.0, 10.0):
    for kind in ('Q', 'H'):
        for quant in ('none', 'unit only', 'unit+state'):
            x = rng.normal(size=(B, 4)); x /= np.linalg.norm(x, axis=1, keepdims=True)
            x = x * 4.0  # typical state magnitude within the +-16 range
            x0n = np.linalg.norm(x, axis=1)
            angle = []
            for t in range(T):
                raw = rng.normal(scale=sigma, size=(B, 4))
                u = unit(raw, kind)
                angle.append(np.arccos(np.clip(u[:, 0], -1, 1)))
                if quant != 'none': u = q_unit(u)
                x = hamilton(u, x) if kind == 'Q' else householder_pair(u, x)
                if quant == 'unit+state': x = q_state(x)
            ratio = np.linalg.norm(x, axis=1) / x0n
            ang = np.degrees(np.mean(angle)) * (1 if kind == 'Q' else 2)
            print(f"sigma={sigma:4} {kind} quant={quant:10s} mean step rotation {ang:6.2f} deg | "
                  f"|x_256|/|x_0|: mean {ratio.mean():.6f} std {ratio.std():.2e} "
                  f"min {ratio.min():.6f} max {ratio.max():.6f}")
# product of quantized unit quaternions alone (norm is exactly multiplicative)
u = unit(rng.normal(scale=3.0, size=(B * T, 4)), 'Q')
n = np.linalg.norm(q_unit(u), axis=1).reshape(B, T)
lp = np.log(n).sum(axis=1)
print("quantized unit-quaternion norm per step: mean-1 = %.2e, std = %.2e; "
      "log-norm of 256-step product: mean %.2e std %.2e max|.| %.2e" %
      (n.mean() - 1, n.std(), lp.mean(), lp.std(), np.abs(lp).max()))
