"""Resumable driver: runs st.py jobs with 2 single-thread workers. Usage: python3 run_matrix.py <phase>"""
import json, os, subprocess, sys, time
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
RUNS = os.path.join(HERE, "runs", "main")
STEPS = 1000
LRS = (3e-3, 1e-2, 3e-2)
SWEEP_MODELS = ["quat", "quat_r", "hh", "dprod", "diag", "cplx_u", "lru", "gru"]
EXTRA_MODELS = ["hh_r", "diagneg"]
SAVE = {"quat", "quat_r", "hh", "hh_r", "cplx_u", "dprod"}


def name(task, model, seed, lr):
    return f"{task}_{model}_s{seed}_lr{lr:g}"


def job(task, model, seed, lr, steps=STEPS):
    out = os.path.join(RUNS, name(task, model, seed, lr) + ("" if steps == STEPS else f"_st{steps}"))
    if os.path.exists(out + ".json"):
        return out
    cmd = [sys.executable, os.path.join(HERE, "st.py"), "--task", task, "--model", model, "--seed", str(seed),
           "--lr", str(lr), "--steps", str(steps), "--out", out]
    if model in SAVE:
        cmd.append("--save")
    r = subprocess.run(cmd, capture_output=True, text=True, cwd=HERE)
    with open(os.path.join(RUNS, "driver.log"), "a") as f:
        f.write(r.stdout + r.stderr[-2000:] + "\n")
    return out


def best_lr(model, task="A5"):
    best = None
    for lr in LRS:
        p = os.path.join(RUNS, name(task, model, 0, lr) + ".json")
        if not os.path.exists(p):
            continue
        r = json.load(open(p))
        nll = r["eval"]["32"]["nll_band"]
        if nll != nll:   # NaN
            nll = 1e9
        if best is None or nll < best[0]:
            best = (nll, lr)
    return best[1]


def run(jobs):
    t0 = time.time()
    with ThreadPoolExecutor(2) as ex:
        list(ex.map(lambda j: job(*j), jobs))
    print(f"{len(jobs)} jobs in {time.time()-t0:.0f}s wall", flush=True)


if __name__ == "__main__":
    os.makedirs(RUNS, exist_ok=True)
    phase = sys.argv[1]
    if phase == "sweep":
        run([("A5", m, 0, lr) for m in SWEEP_MODELS + EXTRA_MODELS for lr in LRS])
    elif phase == "seeds":
        def learned(m):
            p = os.path.join(RUNS, name("A5", m, 0, best_lr(m)) + ".json")
            return json.load(open(p))["eval"]["32"]["acc_band"] > 0.5
        jobs = [("A5", "quat_r", 0, 3e-2, 3000),       # slow-vs-incapable check for the repo parameterisation
                ("A5", "gru", 0, best_lr("gru"), 3000)]  # slow-vs-incapable check for the GRU
        # extra A5 seeds only where the sweep produced a non-trivial model (others: 3 LRs all at chance)
        for m in [m for m in SWEEP_MODELS if m != "quat_r" and learned(m)]:
            jobs += [("A5", m, s, best_lr(m)) for s in (1, 2)]
        for m in ["quat", "diag", "cplx_u", "lru"]:
            jobs += [("Z60", m, s, best_lr(m)) for s in (0, 1, 2)]
        jobs.append(("Z60", "gru", 0, best_lr("gru")))
        for m in ["quat", "dprod"]:
            jobs += [("S5", m, s, best_lr(m)) for s in (0, 1, 2)]
        jobs.append(("S5", "gru", 0, best_lr("gru")))
        print(jobs, flush=True)
        run(jobs)
    elif phase == "extra":
        run([("A5", "quat", s, 3e-2) for s in (3, 4)])
    elif phase == "lrs":
        print({m: best_lr(m) for m in SWEEP_MODELS + EXTRA_MODELS})
