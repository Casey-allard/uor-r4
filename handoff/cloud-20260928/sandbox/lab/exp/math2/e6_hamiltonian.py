"""math2 E6: checks for the 'Hamiltonian heatmap' proposals (numpy, 1 thread, seconds).

(a) Lie-group leapfrog on S^3 (unit quaternions) for H = |p|^2/2 + V(q), V(q) = -kappa*<q, key>: exact norm, bounded energy
    error, time reversibility; explicit Euler for contrast. Then: does a conservative query-particle ever settle on a key?
    (no: it oscillates); with friction (heavy ball) it converges -> retrieval needs dissipation.
(b) Conservative (isometric) vs dissipative superposed memory: h = sum_t lambda^(T-t) P_t v_t with random rotations P_t;
    recall item tau by P_tau^T h and nearest-codeword decoding. Capacity vs T/d, and recall profile vs age.
(c) Gibbs read over hyperbolic distance for keys uniform in a hyperbolic ball of H^n: attention mass on the nearest
    keys vs beta; localization threshold near the volume-growth rate beta_c = n-1 (query at the centre).
"""
import numpy as np, json
rng = np.random.default_rng(0)
out = {}


def qmul(a, b):
    w1, x1, y1, z1 = a; w2, x2, y2, z2 = b
    return np.array([w1*w2 - x1*x2 - y1*y2 - z1*z2, w1*x2 + x1*w2 + y1*z2 - z1*y2,
                     w1*y2 - x1*z2 + y1*w2 + z1*x2, w1*z2 + x1*y2 - y1*x2 + z1*w2])


def qexp(v):                      # v in R^3 (pure quaternion) -> unit quaternion
    th = np.linalg.norm(v)
    return np.array([1.0, 0, 0, 0]) if th < 1e-15 else np.concatenate([[np.cos(th)], np.sin(th) * v / th])


def grad_alg(q, key, kappa):      # -dV along right-trivialized su(2): V = -kappa <q,key>; d/dt <exp(tw) q, key> at 0
    # derivative of <exp(t w) q, key> = <w*q, key>, w pure; gradient components for w = e1,e2,e3
    g = np.array([np.dot(qmul(np.concatenate([[0], e]), q), key) for e in np.eye(3)])
    return kappa * g              # force = -grad V = +kappa * g


def energy(q, p, key, kappa):
    return 0.5 * p @ p - kappa * q @ key


# (a) leapfrog vs Euler on S^3
key = rng.normal(size=4); key /= np.linalg.norm(key)
q0 = rng.normal(size=4); q0 /= np.linalg.norm(q0); p0 = rng.normal(size=3) * 0.5
kappa, h, N = 1.0, 0.05, 20000
q, p = q0.copy(), p0.copy(); E0 = energy(q, p, key, kappa); emax = 0.0; dmin = []
for n in range(N):
    p = p + 0.5 * h * grad_alg(q, key, kappa)
    q = qmul(qexp(h * p), q)
    p = p + 0.5 * h * grad_alg(q, key, kappa)
    emax = max(emax, abs(energy(q, p, key, kappa) - E0)); dmin.append(np.arccos(np.clip(abs(q @ key), -1, 1)))
norm_err = abs(np.linalg.norm(q) - 1)
qf, pf = q.copy(), -p.copy()      # reverse
for n in range(N):
    pf = pf + 0.5 * h * grad_alg(qf, key, kappa); qf = qmul(qexp(h * pf), qf); pf = pf + 0.5 * h * grad_alg(qf, key, kappa)
rev_err = float(np.linalg.norm(qf - q0) + np.linalg.norm(-pf - p0))
qe, pe = q0.copy(), p0.copy()
for n in range(N):                # explicit Euler in the embedding (renormalized) for contrast
    f = grad_alg(qe, key, kappa); qe = qmul(qexp(h * pe), qe); pe = pe + h * f
e_euler = abs(energy(qe, pe, key, kappa) - E0)
dm = np.array(dmin)
out["a_leapfrog"] = {"steps": N, "h": h, "norm_error": float(norm_err), "max_energy_error": float(emax),
                     "reversibility_error": rev_err, "euler_final_energy_error": float(e_euler),
                     "angle_to_key_last_2000_min_max_deg": [float(np.degrees(dm[-2000:].min())), float(np.degrees(dm[-2000:].max()))]}
# with friction: heavy-ball / damped leapfrog (conformal symplectic)
for gamma in (0.5, 2.0):
    q, p = q0.copy(), p0.copy(); steps_to = None
    for n in range(N):
        p = np.exp(-gamma * h / 2) * p; p = p + 0.5 * h * grad_alg(q, key, kappa)
        q = qmul(qexp(h * p), q)
        p = p + 0.5 * h * grad_alg(q, key, kappa); p = np.exp(-gamma * h / 2) * p
        if steps_to is None and np.degrees(np.arccos(np.clip(abs(q @ key), -1, 1))) < 1.0: steps_to = n + 1
    out[f"a_damped_gamma{gamma}"] = {"steps_to_within_1deg_of_key": steps_to}

# (b) conservative vs dissipative superposed memory
def random_rotation(d):
    A = rng.normal(size=(d, d)); Q, R = np.linalg.qr(A); return Q * np.sign(np.diag(R))

d, K = 64, 256
code = rng.normal(size=(K, d)) / np.sqrt(d)
res_b = {}
for lam in (1.0, 0.97, 0.9):
    for T in (8, 32, 64, 128, 256):
        trials, correct_by_age = 40, np.zeros(T)
        for _ in range(trials):
            ids = rng.integers(0, K, size=T); Ps = [random_rotation(d) for _ in range(T)]
            hvec = sum((lam ** (T - 1 - t)) * Ps[t] @ code[ids[t]] for t in range(T))
            for t in range(T):
                est = Ps[t].T @ hvec
                correct_by_age[T - 1 - t] += float(np.argmax(code @ est) == ids[t])
        acc = correct_by_age / trials
        res_b[f"lam{lam}_T{T}"] = {"mean_recall": float(acc.mean()), "recall_age0": float(acc[0]),
                                   "recall_oldest": float(acc[-1])}
out["b_superposed_memory_d64_K256"] = res_b

# (c) hyperbolic Gibbs read: keys uniform in a ball of radius R in H^n, query at the centre
def hyp_ball_radii(n, R, N):      # radial density ~ sinh^(n-1) r on [0,R]; inverse-CDF by rejection
    out_r = []
    while len(out_r) < N:
        r = rng.uniform(0, R, size=4 * N)
        acc = rng.uniform(size=4 * N) < (np.sinh(r) / np.sinh(R)) ** (n - 1)
        out_r.extend(r[acc].tolist())
    return np.array(out_r[:N])

res_c = {}
for n in (4, 8):
    R = 10.0 if n == 4 else 6.0
    rad = hyp_ball_radii(n, R, 200000)
    for beta in (0.5 * (n - 1), 0.8 * (n - 1), 1.0 * (n - 1), 1.25 * (n - 1), 2.0 * (n - 1)):
        w = np.exp(-beta * rad); w /= w.sum()
        res_c[f"n{n}_R{R}_beta{beta:.2f}"] = {"mass_within_1": float(w[rad < 1].sum()), "mass_beyond_R_minus_1": float(w[rad > R - 1].sum())}
out["c_gibbs_read_uniform_hyperbolic_ball"] = res_c
json.dump(out, open("/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/math2/e6_hamiltonian.json", "w"), indent=1)
print(json.dumps(out, indent=1))
