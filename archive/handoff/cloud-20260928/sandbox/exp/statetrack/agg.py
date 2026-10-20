"""Aggregate runs into markdown tables (mean +- sd and [min,max] over seeds at the selected LR)."""
import glob, json, os, sys
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
RUNS = os.path.join(HERE, "runs", "main")
LRS = (3e-3, 1e-2, 3e-2)
LENS = ("32", "64", "128", "256", "512")
ORDER = ["quat", "quat_r", "hh", "hh_r", "dprod", "gru", "diag", "diagneg", "cplx_u", "lru"]
DESC = {
    "quat": "M1 unit-quaternion lanes h<-q(x)h (free param)",
    "quat_r": "M1r same, repo param q=norm(e0+0.1 raw)",
    "hh": "M5 Householder pair H(v)H(e0) = v h v (free)",
    "hh_r": "M5r same, repo param (0.1/sqrt2)",
    "dprod": "M6 4 gen. Householders/lane, beta in (0,2)",
    "gru": "M4 GRU (nonlinear)",
    "diag": "M2 diag real gated, a in (0,1) (Mamba/GLA-like)",
    "diagneg": "M2n diag real, a in (-1,1) (Grazzi-style)",
    "cplx_u": "M3u diag unit-complex rotations (commutative)",
    "lru": "M3 diag complex decay+injection (LRU-like)",
}


def load(steps=1000):
    rs = [json.load(open(p)) for p in glob.glob(os.path.join(RUNS, "*.json"))]
    return [r for r in rs if r["steps"] == steps]


def best_lr(rs, model):
    best = None
    for r in rs:
        if r["task"] == "A5" and r["model"] == model and r["seed"] == 0:
            nll = r["eval"]["32"]["nll_band"]
            nll = 1e9 if nll != nll else nll
            if best is None or nll < best[0]:
                best = (nll, r["lr"])
    return None if best is None else best[1]


def fmt(v):
    v = np.array(v)
    if len(v) == 1:
        return f"{v[0]:.3f}"
    return f"{v.mean():.3f}±{v.std(ddof=0):.3f} [{v.min():.2f},{v.max():.2f}]"


def table(rs, task):
    lines = [f"| model | lr | n | " + " | ".join(f"acc L{L}" for L in LENS) + " | final@512 | params | train CPU s |",
             "|---|---|---|" + "---|" * (len(LENS) + 3)]
    for m in ORDER:
        lr = best_lr(rs, m)
        sel = [r for r in rs if r["task"] == task and r["model"] == m and (lr is None or abs(r["lr"] - lr) < 1e-12)]
        if not sel:
            continue
        sel.sort(key=lambda r: r["seed"])
        accs = [fmt([r["eval"][L]["acc_band"] for r in sel]) for L in LENS]
        fin = fmt([r["eval"]["512"]["acc_final"] for r in sel])
        cpu = np.mean([r["train_cpu_s"] for r in sel])
        lines.append(f"| {m}: {DESC[m]} | {lr:g} | {len(sel)} | " + " | ".join(accs) +
                     f" | {fin} | {sel[0]['params']} | {cpu:.0f} |")
    return "\n".join(lines)


def sweep_table(rs):
    lines = ["| model | " + " | ".join(f"lr {lr:g}: NLL(16,32] / acc L32 / acc L512" for lr in LRS) + " | chosen |",
             "|---|" + "---|" * (len(LRS) + 1)]
    for m in ORDER:
        cells = []
        for lr in LRS:
            r = [x for x in rs if x["task"] == "A5" and x["model"] == m and x["seed"] == 0 and abs(x["lr"] - lr) < 1e-12]
            if r:
                e = r[0]["eval"]
                cells.append(f"{e['32']['nll_band']:.3f} / {e['32']['acc_band']:.3f} / {e['512']['acc_band']:.3f}")
            else:
                cells.append("-")
        lines.append(f"| {m} | " + " | ".join(cells) + f" | {best_lr(rs, m)} |")
    return "\n".join(lines)


if __name__ == "__main__":
    rs = load()
    extra = load(3000)
    print("## LR sweep (A5, seed 0; selection by in-distribution NLL on positions 17-32)\n")
    print(sweep_table(rs))
    for task in ("A5", "Z60", "S5"):
        print(f"\n## {task}: band accuracy over positions (L/2, L] of 512 test sequences (train length 32)\n")
        print(table(rs, task))
    print("\n## Counts over ALL runs at 1000 steps (any LR/seed): fit = acc L32 >= 0.99; exact@512 = acc L512 >= 0.99\n")
    print("| task | model | runs | fit | exact@512 |\n|---|---|---|---|---|")
    for task in ("A5", "Z60", "S5"):
        for m in ORDER:
            sel = [r for r in rs if r["task"] == task and r["model"] == m]
            if sel:
                fit = sum(r["eval"]["32"]["acc_band"] >= 0.99 for r in sel)
                ex = sum(r["eval"]["512"]["acc_band"] >= 0.99 for r in sel)
                print(f"| {task} | {m} | {len(sel)} | {fit} | {ex} |")
    for r in extra:
        e = r["eval"]
        print(f"\nLonger run: {r['task']} {r['model']} seed {r['seed']} lr {r['lr']} steps {r['steps']}: "
              + ", ".join(f"L{L} {e[L]['acc_band']:.3f}" for L in LENS) + f"; final loss {r['log'][-1][1]:.3f}")
    tot = sum(r["total_cpu_s"] for r in rs + extra)
    print(f"\nruns: {len(rs)}, total CPU (training+eval, all runs): {tot:.0f} s")
