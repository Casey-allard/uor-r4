"""exp2 diagnostic: why does two-level routing plateau?  Coarse-level recall under different routing rules / sizes."""
import sys, numpy as np
sys.argv = [sys.argv[0]]
import index_eval as ie
P, meta = ie.load("runs/mq_dot.npz")
rng = np.random.default_rng(5)
N = 16384
keys, qd, tgt, _ = ie.mqar_store(rng, P, meta, N, 512, (0.0, 1.0))
print("mean |q|/|k_t|:", float((np.linalg.norm(qd[0.0], axis=1) / np.linalg.norm(keys[tgt], axis=1)).mean()))
for c in (16,):
    nF = N // c
    labF = ie.kmeans(keys, nF, "dot", rng)
    centF = ie.bundle_cells(keys, labF, nF, "dot")
    for g in (8, 32):
        nCo = nF // g
        par = ie.kmeans(centF, nCo, "dot", rng)
        labCo = par[labF]
        centCo = ie.bundle_cells(keys, labCo, nCo, "dot")
        for noise in (0.0, 1.0):
            q = qd[noise]
            tc = labCo[tgt]
            rules = {"mips": q @ centCo.T,
                     "l2": -((centCo ** 2).sum(1)[None] - 2 * q @ centCo.T),
                     "cos": (q @ centCo.T) / np.linalg.norm(centCo, axis=1)[None]}
            for rn, sc in rules.items():
                rk = (sc > sc[np.arange(len(q)), tc][:, None]).sum(1)
                print(f"g={g} coarse={nCo} noise={noise} rule={rn:4s} coarse-recall@p: " +
                      " ".join(f"p{p}:{(rk < p).mean():.3f}" for p in (1, 2, 4, 8, 16) if p < nCo))
        # fine-level (one-level) recall under the three rules
    for noise in (0.0, 1.0):
        q = qd[noise]; tcF = labF[tgt]
        rules = {"mips": q @ centF.T, "l2": -((centF ** 2).sum(1)[None] - 2 * q @ centF.T),
                 "cos": (q @ centF.T) / np.linalg.norm(centF, axis=1)[None]}
        for rn, sc in rules.items():
            rk = (sc > sc[np.arange(len(q)), tcF][:, None]).sum(1)
            print(f"one-level fine={nF} noise={noise} rule={rn:4s} cell-recall@p: " +
                  " ".join(f"p{p}:{(rk < p).mean():.3f}" for p in (1, 4, 8, 16, 32)))
    sizes = np.bincount(labF, minlength=nF); print("fine size min/median/max", sizes.min(), np.median(sizes), sizes.max())
    print("coarse sizes (g=32 last) min/median/max", np.bincount(labCo).min(), np.median(np.bincount(labCo)), np.bincount(labCo).max())
