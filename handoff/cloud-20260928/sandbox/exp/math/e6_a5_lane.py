"""E6: can ONE 4-D linear lane learn the A5 word problem (NC1-complete) by gradient descent?

Lane transports (matching crates/uor-r4-training/src/joint_model.rs::transport_lanes):
  Q(alpha): h_t = L(q_t) h_{t-1},        q_t = normalize(e0 + alpha * r[x_t])       (left Hamilton)
  H(alpha): h_t = H(u_t) H(e0) h_{t-1},  u_t = normalize(e0 + alpha/sqrt2 * r[x_t]) (Householder pair)
  D       : h_t = diag(R(th1[x_t]), R(th2[x_t])) h_{t-1}                           (complex diagonal)
Readout: softmax(W [h, h_i h_j (i<=j)] + b) over the 60 elements of A5; labels = running product.
Train length 16; test lengths 16/64/256 (length generalization).
Alphabet 'all60' = every A5 element is a token; 'gen6' = {(012),(013),(014)} and inverses.
"""
import sys, itertools, time
import autograd.numpy as np
from autograd import grad
from autograd.misc.optimizers import adam
import numpy as onp

perms = [p for p in itertools.permutations(range(5))
         if sum(1 for i in range(5) for j in range(i + 1, 5) if p[i] > p[j]) % 2 == 0]
assert len(perms) == 60
pidx = {p: i for i, p in enumerate(perms)}
def compose(a, b):  # (a o b)(i) = a[b[i]]
    return tuple(a[b[i]] for i in range(5))
COMP = onp.array([[pidx[compose(perms[a], perms[b])] for b in range(60)] for a in range(60)])
E = pidx[tuple(range(5))]
def cyc(*c):
    p = list(range(5))
    for k in range(len(c)):
        p[c[k]] = c[(k + 1) % len(c)]
    return pidx[tuple(p)]
GEN6 = [cyc(0, 1, 2), cyc(0, 1, 3), cyc(0, 1, 4)]
GEN6 += [int(onp.nonzero(COMP[g] == E)[0][0]) for g in GEN6]  # inverses

def make_batch(rs, B, L, alphabet):
    toks = rs.integers(0, len(alphabet), size=(B, L))
    elems = onp.array(alphabet)[toks]
    lab = onp.zeros((B, L), int); cur = onp.full(B, E)
    for t in range(L):
        cur = COMP[elems[:, t], cur]  # P_t = x_t o P_{t-1}
        lab[:, t] = cur
    return toks, lab

def trans_mats(params, kind, alpha, V):
    r = params['r']
    if kind == 'D':
        c1, s1 = np.cos(r[:, 0]), np.sin(r[:, 0]); c2, s2 = np.cos(r[:, 1]), np.sin(r[:, 1])
        z = np.zeros(V)
        rows = [np.stack([c1, -s1, z, z], 1), np.stack([s1, c1, z, z], 1),
                np.stack([z, z, c2, -s2], 1), np.stack([z, z, s2, c2], 1)]
        return np.stack(rows, 1)
    scale = alpha if kind == 'Q' else alpha / np.sqrt(2.0)
    e0 = np.array([1.0, 0.0, 0.0, 0.0])
    c = e0[None, :] + scale * r
    u = c / np.sqrt(np.sum(c * c, 1, keepdims=True))
    if kind == 'Q':
        w, a, b, d = u[:, 0], u[:, 1], u[:, 2], u[:, 3]
        rows = [np.stack([w, -a, -b, -d], 1), np.stack([a, w, -d, b], 1),
                np.stack([b, d, w, -a], 1), np.stack([d, -b, a, w], 1)]
        return np.stack(rows, 1)
    # H(u) H(e0): H(e0) = diag(-1,1,1,1)
    He0 = np.diag(np.array([-1.0, 1.0, 1.0, 1.0]))
    Hu = np.eye(4)[None] - 2.0 * u[:, :, None] * u[:, None, :]
    return np.einsum('vij,jk->vik', Hu, He0)

IU = onp.triu_indices(4)
def features(h):
    q = h[:, :, None] * h[:, None, :]
    return np.concatenate([h, q[:, IU[0], IU[1]]], axis=1)

def forward_loss(params, toks, lab, kind, alpha, V):
    A = trans_mats(params, kind, alpha, V)
    h0 = params['h0'] / np.sqrt(np.sum(params['h0'] ** 2))
    h = np.tile(h0[None, :], (toks.shape[0], 1))
    loss = 0.0
    for t in range(toks.shape[1]):
        h = np.einsum('bij,bj->bi', A[toks[:, t]], h)
        logits = np.dot(features(h), params['W'].T) + params['b']
        m = np.max(logits, axis=1, keepdims=True)
        lse = m[:, 0] + np.log(np.sum(np.exp(logits - m), axis=1))
        loss = loss + np.mean(lse - logits[onp.arange(len(lab)), lab[:, t]])
    return loss / toks.shape[1]

def evaluate(params, kind, alpha, alphabet, L, n=2000, seed=123):
    rs = onp.random.default_rng(seed)
    toks, lab = make_batch(rs, n, L, alphabet)
    A = onp.array(trans_mats(params, kind, alpha, len(alphabet)))
    h0 = onp.array(params['h0']); h0 = h0 / onp.linalg.norm(h0)
    h = onp.tile(h0, (n, 1)); correct = onp.zeros(L)
    W, b = onp.array(params['W']), onp.array(params['b'])
    for t in range(L):
        h = onp.einsum('bij,bj->bi', A[toks[:, t]], h)
        pred = onp.argmax(onp.array(features(h)) @ W.T + b, axis=1)
        correct[t] = onp.mean(pred == lab[:, t])
    return correct[-1], correct.mean()

def run_curriculum(kind, alpha, alph_name, seed, phases=((2, 400), (4, 400), (8, 600), (16, 600)), B=128, lr=0.02):
    alphabet = list(range(60)) if alph_name == 'all60' else GEN6
    V = len(alphabet)
    rs = onp.random.default_rng(seed)
    params = {'r': rs.normal(size=(V, 4)), 'h0': rs.normal(size=4),
              'W': 0.1 * rs.normal(size=(60, 14)), 'b': onp.zeros(60)}
    from autograd.misc import flatten
    flat, unflat = flatten(params)
    m = onp.zeros_like(flat); v = onp.zeros_like(flat); it = 0
    t0 = time.time()
    for L, steps in phases:
        gfun = grad(lambda f, toks, lab: forward_loss(unflat(f), toks, lab, kind, alpha, V))
        for _ in range(steps):
            toks, lab = make_batch(rs, B, L, alphabet)
            g = onp.array(gfun(flat, toks, lab)); it += 1
            m = 0.9 * m + 0.1 * g; v = 0.999 * v + 0.001 * g * g
            flat = flat - lr * (m / (1 - 0.9 ** it)) / (onp.sqrt(v / (1 - 0.999 ** it)) + 1e-8)
    params = unflat(flat)
    res = {Lt: evaluate(params, kind, alpha, alphabet, Lt) for Lt in (16, 64, 256)}
    tr = forward_loss(params, *make_batch(rs, 512, 16, alphabet), kind, alpha, V)
    print(f"CURRIC {kind} alpha={alpha:<4} {alph_name:5s} seed={seed} train-CE@16 {float(tr):.3f} | acc@last "
          + " ".join(f"L{Lt}:{res[Lt][0]:.3f}" for Lt in res) + " | mean-acc "
          + " ".join(f"L{Lt}:{res[Lt][1]:.3f}" for Lt in res) + f" | {time.time()-t0:.0f}s", flush=True)

def run(kind, alpha, alph_name, seed, steps=2500, L=16, B=256):
    alphabet = list(range(60)) if alph_name == 'all60' else GEN6
    V = len(alphabet)
    rs = onp.random.default_rng(seed)
    params = {'r': rs.normal(size=(V, 4)), 'h0': rs.normal(size=4),
              'W': 0.1 * rs.normal(size=(60, 14)), 'b': onp.zeros(60)}
    g = grad(lambda p, i: forward_loss(p, *batches[i], kind, alpha, V))
    batches = [make_batch(rs, B, L, alphabet) for _ in range(steps)]
    t0 = time.time()
    params = adam(g, params, step_size=0.02, num_iters=steps)
    res = {Lt: evaluate(params, kind, alpha, alphabet, Lt) for Lt in (16, 64, 256)}
    tr = forward_loss(params, *make_batch(rs, 512, L, alphabet), kind, alpha, V)
    print(f"{kind} alpha={alpha:<4} {alph_name:5s} seed={seed} train-CE {float(tr):.3f} | acc@last "
          + " ".join(f"L{Lt}:{res[Lt][0]:.3f}" for Lt in res) + " | mean-acc "
          + " ".join(f"L{Lt}:{res[Lt][1]:.3f}" for Lt in res) + f" | {time.time()-t0:.0f}s", flush=True)

if __name__ == '__main__':
    cfgs = [a.split(':') for a in sys.argv[1:]]
    for kind, alpha, alph, seed in cfgs:
        run_curriculum(kind, float(alpha), alph, int(seed))
