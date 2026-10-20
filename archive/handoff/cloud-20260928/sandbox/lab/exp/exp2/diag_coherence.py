"""exp2 diagnostic: member-to-advertisement cosine, storage-order chunks vs k-means cells (dot MQAR keys)."""
import sys, numpy as np
import index_eval as ie
P, meta = ie.load(sys.argv[1])
rng = np.random.default_rng(11)
for N in (1024, 16384):
    keys, qd, tgt, _ = ie.mqar_store(rng, P, meta, N, 64, (0.0,))
    for c in (16, 64):
        nC = N // c
        for name, lab in (("chunk", np.minimum(np.arange(N) // c, nC - 1)), ("kmeans", ie.kmeans(keys, nC, "dot", rng))):
            cent = ie.bundle_cells(keys, lab, nC, "dot")
            cs = (keys * cent[lab]).sum(1) / (np.linalg.norm(keys, axis=1) * np.linalg.norm(cent[lab], axis=1))
            print(f"N={N} c={c} {name:6s} mean cos(member, advert)={cs.mean():.3f}  (1/sqrt(c)={1/np.sqrt(c):.3f})", flush=True)
