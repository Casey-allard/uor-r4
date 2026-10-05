"""Unprofiled prefill step times -> position-dependent share at full context.

For each steps.json: pool all requests' step_ns by position, take the median
step time per position, fit a Theil-Sen line over positions 8..end (robust to
load spikes), and report t(0), the slope, t(383) and slope*383/t(383), plus
the crossover context where the slope part would reach 30%.
"""
import json
import statistics as st
import sys


def theil_sen(xs, ys):
    slopes = []
    step = max(1, len(xs) // 120)
    sx, sy = xs[::step], ys[::step]
    for i in range(len(sx)):
        for j in range(i + 1, len(sx)):
            slopes.append((sy[j] - sy[i]) / (sx[j] - sx[i]))
    b = st.median(slopes)
    a = st.median([y - b * x for x, y in zip(xs, ys)])
    return a, b


out = {}
for path in sys.argv[1:]:
    d = json.load(open(path))
    by_pos = {}
    for r in d['rows']:
        for p, ns in enumerate(r['step_ns']):
            by_pos.setdefault(p, []).append(ns / 1e6)
    xs = sorted(p for p in by_pos if p >= 8)
    ys = [st.median(by_pos[p]) for p in xs]
    a, b = theil_sen(xs, ys)
    full = d['shape']['context'] - 1
    t_full = a + b * full
    share = b * full / t_full
    cross = 0.3 * a / (0.7 * b) if b > 0 else None
    out[path] = {
        'threads': d['threads'],
        'requests': len(d['rows']),
        'history_ids': [r['history_ids'] for r in d['rows']],
        'median_ms_positions_0_31': round(st.median([v for p in range(32) for v in by_pos[p]]), 2),
        'median_ms_last_32': round(st.median([v for p in range(max(by_pos) - 31, max(by_pos) + 1) for v in by_pos[p]]), 2),
        'fit_ms_at_0': round(a, 3),
        'fit_ms_per_position': round(b, 5),
        'fit_ms_at_383': round(t_full, 3),
        'position_dependent_share_at_383': round(share, 4),
        'context_where_share_reaches_30pct': round(cross) if cross else None,
    }
print(json.dumps(out, indent=1))
