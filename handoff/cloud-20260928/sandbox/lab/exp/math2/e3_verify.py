"""Verify the probe's rebuilt queries/keys reproduce the model's own read masses."""
import json, sys, numpy as np
pre = sys.argv[1]
P = json.load(open(pre + ".params.json")); T, r = P["context"], P["read_width"]; n = P["dump"] * T
q = np.fromfile(pre + ".query.f32", "<f4").reshape(n, r).astype(np.float64)
k = np.fromfile(pre + ".key.f32", "<f4").reshape(n, r).astype(np.float64)
nl = np.fromfile(pre + ".null.f32", "<f4").astype(np.float64)
m = np.fromfile(pre + ".mass.f32", "<f4").reshape(n, T).astype(np.float64)
age = np.array(P["age"])
err = 0.0
for w in range(P["dump"]):
    Q, K = q[w*T:(w+1)*T], k[w*T:(w+1)*T]
    for t in range(1, T):
        kk = K[:t]
        if P["geometry"] == "Lorentz":
            z = np.sqrt(1 + (Q[t]**2).sum()) * np.sqrt(1 + (kk**2).sum(1)) - kk @ Q[t]
            z = np.maximum(z, 1 + 1e-6)
            s = P["beta"] * (P["offset"] - np.arccosh(z))
        else:
            s = kk @ Q[t] / np.sqrt(r)
        s = s + age[t - 1 - np.arange(t)]
        logits = np.concatenate([[nl[w*T+t]], s]); p = np.exp(logits - logits.max()); p /= p.sum()
        err = max(err, np.abs(p[1:] - m[w*T+t, :t]).max())
print(pre.split("/")[-1], P["geometry"], "beta", P["beta"], "offset", P["offset"], "max |mass rebuild - model| =", err)
