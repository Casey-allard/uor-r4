#!/usr/bin/env python3
"""Generate figures for Angular Manifold Routing paper.

Produces:
  figures/fig_scaling_law.pdf  -- log-log scaling law (Figure 1 in paper)

Requires: matplotlib, numpy
"""
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import os

OUT_DIR = os.path.join(os.path.dirname(__file__), "figures")
os.makedirs(OUT_DIR, exist_ok=True)

# ── Figure 1: Scaling law ────────────────────────────────────────────────────
# Parametric laws from INC-0169 (K=25-400) and INC-0170 (K=600-5000)
# TRANS ORIG canonical: eff_buckets = 2.957 * K^0.572
# TRANS PERM:           eff_buckets = 1.664 * K^0.814
# DENSE:                eff_buckets = K^1.0
#
# Confirmed data points (from experiment results):
ORIG_PTS = [  # (K, eff_b) from INC-0166..0170 confirmed runs
    (25,   8.4),
    (50,  13.5),
    (100, 20.7),
    (200, 31.8),
    (400, 46.0),
    (600, 57.0),
    (1000, 79.5),
    (2000, 117.0),
    (5000, 193.0),
]
PERM_PTS = [  # TRANS PERM (structure-destroyed control)
    (25,   17.1),
    (50,   27.8),
    (100,  46.4),
    (200,  75.3),
    (400, 122.0),
    (600, 150.0),
    (1000, 202.7),
    (2000, 315.0),
    (5000, 509.2),
]

K_fit = np.logspace(np.log10(25), np.log10(5000), 300)
orig_fit  = 2.957 * K_fit ** 0.572   # canonical K=25-400 law
perm_fit  = 1.664 * K_fit ** 0.814
dense_fit = K_fit

fig, ax = plt.subplots(figsize=(5.5, 4.0))

# Fit lines
ax.plot(K_fit, dense_fit,  color="#888888", linewidth=1.4, linestyle="--", zorder=1,
        label=r"Dense  ($\propto K^{1.0}$)")
ax.plot(K_fit, perm_fit,   color="#e07b39", linewidth=1.4, linestyle="-.", zorder=2,
        label=r"Permuted  (${\propto}K^{0.814}$)")
ax.plot(K_fit, orig_fit,   color="#2c7bb6", linewidth=2.0, linestyle="-",  zorder=3,
        label=r"Hopf (TRANS ORIG)  (${\propto}K^{0.572}$)")

# Data points
Kp, ep = zip(*PERM_PTS)
Ko, eo = zip(*ORIG_PTS)
ax.scatter(Kp, ep, color="#e07b39", s=28, zorder=4, marker="s")
ax.scatter(Ko, eo, color="#2c7bb6", s=28, zorder=5, marker="o")

# Annotation: ratio at K=400
ax.annotate("2.21×\ngap at K=400",
            xy=(400, 46), xytext=(430, 75),
            fontsize=7.5, color="#2c7bb6",
            arrowprops=dict(arrowstyle="->", color="#2c7bb6", lw=0.8))

ax.set_xscale("log")
ax.set_yscale("log")
ax.set_xlabel("$K$ (number of routing sectors)", fontsize=10)
ax.set_ylabel("Effective routing paths (eff$_b$)", fontsize=10)
ax.set_title("Hopf routing footprint vs $K$", fontsize=10, fontweight="bold")
ax.legend(fontsize=8.5, loc="upper left", framealpha=0.9)
ax.grid(which="both", linestyle=":", linewidth=0.5, alpha=0.6)

# x-axis ticks
ax.set_xticks([25, 50, 100, 200, 400, 1000, 2000, 5000])
ax.get_xaxis().set_major_formatter(ticker.ScalarFormatter())

plt.tight_layout()
out_path = os.path.join(OUT_DIR, "fig_scaling_law.pdf")
fig.savefig(out_path, bbox_inches="tight", dpi=300)
print(f"Saved: {out_path}")

# Also save PNG for quick inspection
png_path = os.path.join(OUT_DIR, "fig_scaling_law.png")
fig.savefig(png_path, bbox_inches="tight", dpi=150)
print(f"Saved: {png_path}")
plt.close(fig)
