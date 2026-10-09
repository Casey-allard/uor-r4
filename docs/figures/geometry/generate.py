#!/usr/bin/env python3
"""Generate the UOR-R4 geometry illustrations (standard library only).

Every coordinate is computed from the mathematics (quaternions, stereographic
projection, Hopf map, 600-cell / H4 Coxeter plane, Z[phi], zeta zeros, primes).
Run:  python3 docs/figures/geometry/generate.py   (deterministic output)
"""
import math, os, itertools, html
from math import cos, sin, pi, sqrt, log

OUT = os.path.dirname(os.path.abspath(__file__))
PHI = (1 + sqrt(5)) / 2
PHIB = (1 - sqrt(5)) / 2
INK, MUTED, FAINT, GRID = "#1f2933", "#5f6b7a", "#d3dae2", "#8a97a6"
BLUE, SKY, GREEN, ORANGE, VERM, PINK, PURPLE = "#0072B2", "#56B4E9", "#009E73", "#E69F00", "#D55E00", "#CC79A7", "#7A4FA3"
TEN = [ORANGE, SKY, GREEN, BLUE, VERM, PINK, PURPLE, "#8C8C1A", "#1B9AAA", "#8B5A2B"]
FONT = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"


def f(x):
    s = f"{x:.2f}"
    if "." in s:
        s = s.rstrip("0").rstrip(".")
    return "0" if s in ("-0", "") else s


def esc(s):
    return html.escape(s, quote=False)


def hexrgb(c):
    return tuple(int(c[i:i + 2], 16) for i in (1, 3, 5))


def mix(c1, c2, t):
    a, b = hexrgb(c1), hexrgb(c2)
    return "#%02x%02x%02x" % tuple(int(round(a[i] + (b[i] - a[i]) * t)) for i in range(3))


def ramp(stops, t, cyclic=False):
    n = len(stops) if cyclic else len(stops) - 1
    t = (t % 1.0) if cyclic else min(1, max(0, t))
    x = t * n
    i = min(int(x), n - 1)
    return mix(stops[i], stops[(i + 1) % len(stops)], x - i)


class Fig:
    def __init__(self, fname, title, desc, caption):
        self.fname, self.title, self.desc, self.caption = fname, title, desc, caption
        assert all(len(l) <= 128 for l in caption), caption
        self.b = []

    def add(self, s):
        self.b.append(s)

    def line(self, x1, y1, x2, y2, stroke=INK, w=1, op=1, dash=None):
        d = f' stroke-dasharray="{dash}"' if dash else ""
        self.add(f'<line x1="{f(x1)}" y1="{f(y1)}" x2="{f(x2)}" y2="{f(y2)}" stroke="{stroke}" stroke-width="{f(w)}" stroke-opacity="{f(op)}" stroke-linecap="round"{d}/>')

    def circle(self, x, y, r, fill="none", stroke="none", w=1, op=1, fop=1):
        self.add(f'<circle cx="{f(x)}" cy="{f(y)}" r="{f(r)}" fill="{fill}" fill-opacity="{f(fop)}" stroke="{stroke}" stroke-width="{f(w)}" stroke-opacity="{f(op)}"/>')

    def rect(self, x, y, w, h, fill="none", stroke="none", sw=1, rx=0, op=1, fop=1):
        self.add(f'<rect x="{f(x)}" y="{f(y)}" width="{f(w)}" height="{f(h)}" rx="{f(rx)}" fill="{fill}" fill-opacity="{f(fop)}" stroke="{stroke}" stroke-width="{f(sw)}" stroke-opacity="{f(op)}"/>')

    def text(self, x, y, s, size=14, fill=INK, anchor="start", weight="normal", style="normal", op=1):
        st = f' font-style="{style}"' if style != "normal" else ""
        self.add(f'<text x="{f(x)}" y="{f(y)}" font-size="{size}" fill="{fill}" fill-opacity="{f(op)}" text-anchor="{anchor}" font-weight="{weight}"{st}>{esc(s)}</text>')

    def path(self, d, stroke=INK, w=1, op=1, fill="none", fop=1, dash=None):
        da = f' stroke-dasharray="{dash}"' if dash else ""
        self.add(f'<path d="{d}" fill="{fill}" fill-opacity="{f(fop)}" stroke="{stroke}" stroke-width="{f(w)}" stroke-opacity="{f(op)}" stroke-linecap="round" stroke-linejoin="round"{da}/>')

    def poly(self, pts, **kw):
        self.path("M" + " L".join(f"{f(x)} {f(y)}" for x, y in pts), **kw)

    def arrow(self, x1, y1, x2, y2, color, w=2.5, head=11, op=1):
        L = math.hypot(x2 - x1, y2 - y1) or 1
        ux, uy = (x2 - x1) / L, (y2 - y1) / L
        self.line(x1, y1, x2 - ux * head * 0.6, y2 - uy * head * 0.6, color, w, op)
        px, py = -uy, ux
        self.add(f'<polygon points="{f(x2)},{f(y2)} {f(x2 - ux * head + px * head * .38)},{f(y2 - uy * head + py * head * .38)} {f(x2 - ux * head - px * head * .38)},{f(y2 - uy * head - py * head * .38)}" fill="{color}" fill-opacity="{f(op)}"/>')

    def save(self):
        head = (f'<?xml version="1.0" encoding="UTF-8"?>\n<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 700" width="900" height="630" '
                f'role="img" aria-labelledby="t d" font-family="{FONT}">\n<title id="t">{esc(self.title)}</title>\n<desc id="d">{esc(self.desc)}</desc>\n'
                '<rect x="2" y="2" width="996" height="696" rx="22" fill="#ffffff" stroke="#d0d7de" stroke-width="1.5"/>\n')
        t = f'<text x="40" y="56" font-size="27" font-weight="700" fill="{INK}">{esc(self.title)}</text>\n'
        cap = "".join(f'<text x="40" y="{(648 if len(self.caption) < 3 else 634) + 21 * i}" font-size="14" fill="{MUTED}">{esc(l)}</text>\n' for i, l in enumerate(self.caption))
        with open(os.path.join(OUT, self.fname), "w", encoding="utf-8") as fh:
            fh.write(head + t + "\n".join(self.b) + "\n" + cap + "</svg>\n")


# ---------------------------------------------------------------- 3-D helpers
def dot(a, b): return sum(x * y for x, y in zip(a, b))
def cross(a, b): return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])
def norm(a): return sqrt(dot(a, a))
def unit(a): n = norm(a); return tuple(x / n for x in a)
def add(a, b): return tuple(x + y for x, y in zip(a, b))
def scl(a, s): return tuple(x * s for x in a)


class View:
    """Orthographic camera at (az, el) looking at the origin, z up."""
    def __init__(self, az, el, scale, cx, cy):
        a, e = math.radians(az), math.radians(el)
        c = (cos(e) * cos(a), cos(e) * sin(a), sin(e))
        self.fwd = scl(c, -1)
        self.r = (-sin(a), cos(a), 0.0)
        self.u = cross(self.r, self.fwd)
        self.s, self.cx, self.cy = scale, cx, cy

    def __call__(self, p):
        return (self.cx + self.s * dot(p, self.r), self.cy - self.s * dot(p, self.u), dot(p, self.fwd))


def depth_path(fig, view, pts, color, width, op=(0.12, 1.0), wr=(0.6, 1.0), bins=5, dmax=1.0, dash=None, close=False):
    P = [view(p) for p in pts]
    if close:
        P.append(P[0])
    runs, cur, cb = [], [], None
    for i in range(len(P) - 1):
        d = (P[i][2] + P[i + 1][2]) / 2
        b = min(bins - 1, int(min(1, max(0, 0.5 - d / (2 * dmax))) * bins))
        if b != cb:
            if cur:
                runs.append((cb, cur))
            cur, cb = [(P[i][0], P[i][1])], b
        cur.append((P[i + 1][0], P[i + 1][1]))
    if cur:
        runs.append((cb, cur))
    for b, pl in sorted(runs, key=lambda r: r[0]):
        t = b / max(1, bins - 1)
        fig.poly(pl, stroke=color, w=wr[0] + (wr[1] - wr[0]) * t, op=op[0] + (op[1] - op[0]) * t, dash=dash)


def sphere_wire(fig, view, R, lats, lons, color=GRID, op=(0.1, 0.55), n=96):
    for la in lats:
        z, rr = sin(math.radians(la)) * R, cos(math.radians(la)) * R
        depth_path(fig, view, [(rr * cos(2 * pi * i / n), rr * sin(2 * pi * i / n), z) for i in range(n)], color, 0.9, op, dmax=R, close=True)
    for lo in lons:
        a = math.radians(lo)
        depth_path(fig, view, [(R * cos(a) * cos(t), R * sin(a) * cos(t), R * sin(t)) for t in [2 * pi * i / n for i in range(n)]], color, 0.9, op, dmax=R, close=True)


def sub(n):
    return "".join("₀₁₂₃₄₅₆₇₈₉"[int(c)] for c in str(n))


# ---------------------------------------------------------------- quaternions
def qmul(a, b):
    aw, ax, ay, az = a; bw, bx, by, bz = b
    return (aw * bw - ax * bx - ay * by - az * bz, aw * bx + ax * bw + ay * bz - az * by,
            aw * by - ax * bz + ay * bw + az * bx, aw * bz + ax * by - ay * bx + az * bw)

def qconj(q): return (q[0], -q[1], -q[2], -q[3])
def qrot(q, v): return qmul(qmul(q, (0,) + tuple(v)), qconj(q))[1:]
def axis_quat(n, theta): return (cos(theta / 2),) + scl(n, sin(theta / 2))


# ================================================================ FIG 1
def fig_quaternion():
    fg = Fig("quaternion-rotation.svg", "A unit quaternion is a 3-D rotation",
             "A sphere wireframe with a rotation axis, a vector v, its image q v q* and the arc it sweeps. A side panel shows that a unit quaternion is a point on the 4-D unit sphere S3 because its four squared components sum to one.",
             ["q = cos(θ/2) + sin(θ/2)(xi + yj + zk) turns every 3-D vector by θ about the axis (x, y, z): v ↦ q v q*.",
              "All unit quaternions together form the 4-D unit sphere S³; q and −q describe the same rotation.",
              "Stack recurrence: h_t = λ_t (u_t · h_{t−1}) + √(1−λ_t²) a_t, with u_t a learned unit quaternion each step."])
    n = unit((0.35, 0.2, 0.91)); theta = math.radians(110)
    q = axis_quat(n, theta)
    v = unit((1.0, 0.12, -0.05)); v2 = qrot(q, v)
    c, s = cos(theta), sin(theta)
    rod = add(add(scl(v, c), scl(cross(n, v), s)), scl(n, dot(n, v) * (1 - c)))
    assert max(abs(a - b) for a, b in zip(v2, rod)) < 1e-12 and abs(norm(v2) - 1) < 1e-12
    R = 200; vw = View(-52, 20, R, 330, 362)
    fg.circle(330, 362, R, "#f6f9fc", FAINT, 1.2)
    sphere_wire(fg, vw, 1.0, [-60, -30, 0, 30, 60], range(0, 180, 30))
    for k, (ax, lb) in enumerate((((1.3, 0, 0), "x"), ((0, 1.3, 0), "y"), ((0, 0, 1.3), "z"))):
        x, y, _ = vw(ax); o = vw((0, 0, 0))
        fg.line(o[0], o[1], x, y, GRID, 1, 0.8)
        fg.text(x + (6 if x >= o[0] else -6), y + 4, lb, 13, MUTED, "start" if x >= o[0] else "end", style="italic")
    a0, a1 = vw(scl(n, -1.25)), vw(scl(n, 1.3)); o = vw((0, 0, 0))
    fg.line(a0[0], a0[1], a1[0], a1[1], ORANGE, 2.6)
    fg.text(a1[0] + 8, a1[1] - 6, "axis (x, y, z)", 15, "#9a6a00", weight="bold")
    cc = scl(n, dot(v, n)); rad = sqrt(1 - dot(v, n) ** 2)
    e1 = unit(add(v, scl(n, -dot(v, n)))); e2 = cross(n, e1)
    ring = lambda t: add(cc, add(scl(e1, rad * cos(t)), scl(e2, rad * sin(t))))
    depth_path(fg, vw, [ring(2 * pi * i / 120) for i in range(120)], SKY, 1.4, (0.25, 0.8), dash="5 4", close=True)
    arc = [qrot(axis_quat(n, theta * i / 40), v) for i in range(41)]
    depth_path(fg, vw, arc, BLUE, 4.2, (0.35, 1.0), bins=4)
    cx_, cy_, _ = vw(cc)
    for tip in (v, v2):
        x, y, _ = vw(tip); fg.line(cx_, cy_, x, y, SKY, 1.2, 0.8, "3 3")
    fg.circle(cx_, cy_, 2.6, GRID)
    for tip, col in ((v, BLUE), (v2, GREEN)):
        x, y, _ = vw(tip); fg.arrow(o[0], o[1], x, y, col, 3)
        fg.circle(x, y, 5, col, "#fff", 1.5)
    xv, yv, _ = vw(v); xw, yw, _ = vw(v2); xm, ym, _ = vw(arc[20])
    fg.text(xv + 10, yv + 18, "v", 19, BLUE, weight="bold", style="italic")
    fg.text(xw + 8, yw - 8, "v′ = q v q*", 17, "#00775a", weight="bold")
    fg.text(xm + (cx_ - xm) * 0.28 - 4, ym + (cy_ - ym) * 0.28 + 4, "θ", 20, BLUE, "middle", style="italic")
    fg.text(40, 98, "the arc swept by v is a rotation by θ = 110°", 14, MUTED)
    # side panel
    px = 640
    fg.rect(px, 88, 322, 520, "#f6f9fc", FAINT, 1.2, 16)
    fg.text(px + 20, 124, "q is a point on S³", 21, INK, weight="bold")
    fg.text(px + 20, 148, "the unit sphere in 4-D", 14, MUTED)
    comps = [("w = cos(θ/2)", q[0], GRID), ("x = n₁ sin(θ/2)", q[1], PINK), ("y = n₂ sin(θ/2)", q[2], SKY), ("z = n₃ sin(θ/2)", q[3], GREEN)]
    for i, (lb, val, col) in enumerate(comps):
        y = 186 + 27 * i
        fg.rect(px + 20, y - 13, 14, 14, col, rx=3)
        fg.text(px + 42, y - 1, lb, 14, INK)
        fg.text(px + 302, y - 1, f"{val:+.3f}", 14, INK, "end")
    fg.text(px + 20, 318, "squares add up to 1", 14, MUTED)
    x0, tot = px + 20, 282.0
    for lb, val, col in comps:
        wd = tot * val * val; fg.rect(x0, 328, wd, 30, col, "#fff", 1.5); x0 += wd
    sq = sum(c_[1] ** 2 for c_ in comps); assert abs(sq - 1) < 1e-12
    fg.text(px + 20, 388, f"w² + x² + y² + z² = {sq:.3f}", 15, INK, weight="bold")
    for i, l in enumerate(["The axis is the vector part (x, y, z);", "the angle comes from w = cos(θ/2).", "", "Every point of S³ is one rotation of", "3-D space, and −q gives the same one,", "so S³ covers the rotations twice."]):
        fg.text(px + 20, 430 + 24 * i, l, 14.5, INK)
    fg.save()


# ================================================================ FIG 2
def stereo(q):
    d = 1 + q[0]
    return (q[1] / d, q[2] / d, q[3] / d)

def clipped(pts4, clip):
    runs, cur = [], []
    for q in pts4:
        if 1 + q[0] > 1e-9 and norm(stereo(q)) <= clip:
            cur.append(stereo(q))
        elif cur:
            runs.append(cur); cur = []
    if cur:
        runs.append(cur)
    return runs

def great_circle(u, v, N=900):
    return [tuple(cos(t) * a + sin(t) * b for a, b in zip(u, v)) for t in [2 * pi * i / N for i in range(N + 1)]]

def fig_s3():
    fg = Fig("s3-and-transport.svg", "S³ seen in 3-D, and a state moving through it",
             "Great circles of the 4-D unit sphere S3 drawn by stereographic projection to 3-D, where they appear as circles and straight lines, and a path of states h0, h1, h2 and so on produced by multiplying repeatedly by one fixed quaternion, which traces a geodesic.",
             ["Stereographic projection flattens S³ into 3-D (identity at the centre, −1 at infinity): great circles become circles or lines.",
              "Multiplying by one fixed quaternion q again and again moves h₀ → h₁ → h₂ → … by equal arcs along one great circle.",
              "Stack recurrence: h_t = λ_t (u_t · h_{t−1}) + √(1−λ_t²) a_t, with u_t a learned unit quaternion each step."])
    CL = 1.9
    alpha = math.radians(30)
    h0 = unit((0.15, 0.6, -0.5, 0.6))
    def minw(n): return 1 - sqrt(h0[0] ** 2 + dot(n, h0[1:]) ** 2)
    cands = [unit(a) for a in [(0.3, 0.5, 0.8), (1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 0), (1, 0, 1), (0, 1, 1), (1, -1, 1), (1, 2, 3), (-1, 1, 2)]]
    good = [n for n in cands if minw(n) >= 0.45]
    nax, others = good[0], good[1:5]
    q = (cos(alpha),) + scl(nax, sin(alpha))
    hs = [h0]
    for _ in range(12):
        hs.append(qmul(q, hs[-1]))
    assert max(abs(a - b) for a, b in zip(hs[12], h0)) < 1e-9 and all(abs(sum(x * x for x in h) - 1) < 1e-12 for h in hs)
    v1 = qmul((0,) + tuple(nax), h0)
    assert abs(dot(h0, v1)) < 1e-12
    orbit = great_circle(h0, v1)
    assert max(abs(a - b) for a, b in zip(hs[3], [cos(3 * alpha) * a + sin(3 * alpha) * b for a, b in zip(h0, v1)])) < 1e-9
    P3 = [stereo(h) for h in hs[:12]]
    nrm = cross(add(P3[4], scl(P3[0], -1)), add(P3[8], scl(P3[0], -1)))
    az0 = math.degrees(math.atan2(nrm[1], nrm[0])); el0 = math.degrees(math.asin(nrm[2] / norm(nrm)))
    v0 = View(az0 + 28, el0 - 32 if el0 > 0 else el0 + 32, 1, 0, 0)
    sp = [v0(p) for p in P3]; ccx = sum(p[0] for p in sp) / 12; ccy = sum(p[1] for p in sp) / 12
    rr = max(math.hypot(p[0] - ccx, p[1] - ccy) for p in sp)
    sc3 = min(150.0, 150.0 / rr)
    vw = View(az0 + 28, el0 - 32 if el0 > 0 else el0 + 32, sc3, 380 - sc3 * ccx, 372 + sc3 * ccy)
    print("s3 view", round(az0, 1), round(el0, 1), round(sc3, 1))
    sphere_wire(fg, vw, 1.0, [-45, 0, 45], range(0, 180, 45), color=GRID, op=(0.06, 0.28))
    for k, (u, v) in enumerate([((1, 0, 0, 0), (0, 1, 0, 0)), ((1, 0, 0, 0), (0, 0, 1, 0)), ((1, 0, 0, 0), (0, 0, 0, 1)),
                                ((0, 1, 0, 0), (0, 0, 1, 0)), ((0, 1, 0, 0), (0, 0, 0, 1)), ((0, 0, 1, 0), (0, 0, 0, 1))]):
        for run in clipped(great_circle(u, v), CL):
            depth_path(fg, vw, run, "#6b7f93", 1.6, (0.25, 0.85), dmax=CL)
    for n in others:
        v = qmul((0,) + tuple(n), h0)
        for run in clipped(great_circle(h0, v), CL):
            depth_path(fg, vw, run, PURPLE, 1.3, (0.12, 0.55), dmax=CL)
    for run in clipped(orbit, CL):
        depth_path(fg, vw, run, ramp([SKY, BLUE], 0.5), 3.4, (0.5, 1.0), dmax=CL, bins=4)
    pts = [vw(p) for p in P3]
    mx = sum(p[0] for p in pts) / 12; my = sum(p[1] for p in pts) / 12
    for k in range(12):
        a, b = hs[k], hs[k + 1]
        seg = [tuple(cos(alpha * t / 8) * x + sin(alpha * t / 8) * y for x, y in zip(h0, v1)) for t in []]
        # arrowhead midway along the arc from h_k to h_(k+1)
        ang = k * alpha
        pa = vw(stereo(tuple(cos(ang + alpha * 0.46) * x + sin(ang + alpha * 0.46) * y for x, y in zip(h0, v1))))
        pb = vw(stereo(tuple(cos(ang + alpha * 0.58) * x + sin(ang + alpha * 0.58) * y for x, y in zip(h0, v1))))
        L = math.hypot(pb[0] - pa[0], pb[1] - pa[1]) or 1; ux, uy = (pb[0] - pa[0]) / L, (pb[1] - pa[1]) / L
        fg.add(f'<polygon points="{f(pb[0] + ux * 6)},{f(pb[1] + uy * 6)} {f(pb[0] - ux * 5 - uy * 5)},{f(pb[1] - uy * 5 + ux * 5)} {f(pb[0] - ux * 5 + uy * 5)},{f(pb[1] - uy * 5 - ux * 5)}" fill="{INK}" fill-opacity="0.85"/>')
    for k in range(12):
        x, y, _ = pts[k]; col = ramp([SKY, BLUE, VERM], k / 11)
        fg.circle(x, y, 6.5, col, "#fff", 1.8)
        dx, dy = x - mx, y - my; L = math.hypot(dx, dy) or 1
        if k % 2 == 0:
            fg.text(x + dx / L * 22, y + dy / L * 22 + 5, "h" + sub(k), 15, INK, "middle", weight="bold")
    # side panel
    px = 700
    fg.rect(px, 88, 262, 330, "#f6f9fc", FAINT, 1.2, 16)
    fg.text(px + 18, 120, "state transported", 19, INK, weight="bold")
    fg.text(px + 18, 144, "step by step", 19, INK, weight="bold")
    for i, l in enumerate(["h_{k+1} = q · h_k", "q = cos 30° + sin 30° · n", "(n a unit 3-D direction)", "", "each step is the same 30° arc", "of one great circle (geodesic);", "in this view equal arcs look", "unequal: projection stretches", "distances near the pole."]):
        fg.text(px + 18, 180 + 23 * i, l.replace("h_{k+1}", "h(k+1)") if False else l, 14.5, INK if i < 3 else MUTED)
    ly = 450
    for col, lb, w in ((ramp([SKY, BLUE, VERM], .4), "orbit of the state (geodesic)", 3.4), ("#6b7f93", "coordinate great circles", 1.6), (PURPLE, "other geodesics through h₀", 1.3)):
        fg.line(px + 6, ly, px + 40, ly, col, w + 1); fg.text(px + 50, ly + 5, lb, 13.5, INK); ly += 28
    fg.save()


# ================================================================ FIG 3
def hopf(z1, z2):
    w = z1 * z2.conjugate()
    return (2 * w.real, 2 * w.imag, abs(z1) ** 2 - abs(z2) ** 2)

def fig_hopf():
    fg = Fig("hopf-fibration.svg", "The Hopf map: a circle behind every point",
             "Left: the 2-sphere with ten colour-coded marked points. Right: the ten corresponding Hopf fibres, which are linked circles in the 3-sphere, drawn by stereographic projection to 3-D in the same colours. Faint circles show the nested tori the fibres sit on.",
             ["Each point on S² is a whole circle in S³; observing only S² loses the position along the circle (the fiber).",
              "The model keeps the fiber separately. Any two fibres are linked circles; one latitude of S² gives a torus of fibres."])
    rings = [(100, [-80 + 40 * i for i in range(5)]), (140, [-80 + 40 * i for i in range(5)])]
    pts = [(b, a) for b, al in rings for a in al]
    def fibre(beta, alp, N=160):
        b, a = math.radians(beta), math.radians(alp); out = []
        for i in range(N + 1):
            ph = 2 * pi * i / N; e = complex(cos(ph), sin(ph))
            out.append((cos(b / 2) * e, sin(b / 2) * complex(cos(a), -sin(a)) * e))
        return out
    def st(z1, z2): return stereo((z1.real, z1.imag, z2.real, z2.imag))
    for beta, alp in pts:                                   # the fibre maps to ONE point of S2
        b, a = math.radians(beta), math.radians(alp)
        P = (sin(b) * cos(a), sin(b) * sin(a), cos(b))
        for z1, z2 in fibre(beta, alp, 24):
            assert max(abs(x - y) for x, y in zip(hopf(z1, z2), P)) < 1e-12 and abs(abs(z1) ** 2 + abs(z2) ** 2 - 1) < 1e-12
    # left S2
    v2 = View(0, -30, 150, 215, 360)
    fg.text(215, 112, "S² — what is observed", 17, INK, "middle", weight="bold")
    fg.circle(215, 360, 150, "#f6f9fc", FAINT, 1.2)
    sphere_wire(fg, v2, 1.0, [-60, -30, 0, 30, 60], range(0, 180, 30), op=(0.08, 0.45))
    for beta, _ in rings:
        la = 90 - beta
        depth_path(fg, v2, [(cos(math.radians(la)) * cos(2 * pi * i / 96), cos(math.radians(la)) * sin(2 * pi * i / 96), sin(math.radians(la))) for i in range(96)], GRID, 1.4, (0.3, 0.9), dash="4 4", close=True)
    marks = []
    for k, (beta, alp) in enumerate(pts):
        b, a = math.radians(beta), math.radians(alp)
        x, y, d = v2((sin(b) * cos(a), sin(b) * sin(a), cos(b))); marks.append((x, y, d))
        assert d < 0   # in front of the camera
        fg.circle(x, y, 9.5, TEN[k], "#fff", 1.8); fg.text(x, y + 4.5, str(k + 1), 12, "#fff", "middle", weight="bold")
    # right: fibres in R^3
    v3 = View(90, 28, 78, 700, 372)
    fg.text(690, 112, "S³ — the fibres (stereographic view)", 17, INK, "middle", weight="bold")
    for beta, _ in rings:
        for j in range(20):
            fb = fibre(beta, -180 + 18 * j, 120)
            depth_path(fg, v3, [st(*p) for p in fb], GRID, 0.8, (0.05, 0.22), bins=3, dmax=3.0)
    order = sorted(range(10), key=lambda k: 0)
    for k in order:
        fb = [st(*p) for p in fibre(*pts[k])]
        assert max(norm(p) for p in fb) < 2.3
        depth_path(fg, v3, fb, "#ffffff", 5.2, (0.9, 0.9), bins=1, dmax=3.0)
        depth_path(fg, v3, fb, TEN[k], 3.0, (0.35, 1.0), wr=(1.6, 3.2), bins=5, dmax=3.0)
    fg.arrow(470, 372, 508, 372, INK, 2, 10)
    fg.text(489, 360, "fibre", 13, MUTED, "middle", style="italic")
    fg.text(489, 396, "π⁻¹", 13, MUTED, "middle", style="italic") if False else None
    fg.text(40, 616, "numbers match points to fibres; dashed rings on S² are the two latitudes whose fibres fill the tori", 13, MUTED)
    fg.save()


# ================================================================ FIG 4
def icosians():
    vs = {}
    def put(v): vs[tuple(round(x, 9) for x in v)] = tuple(v)
    for i in range(4):
        for s in (1, -1):
            v = [0.0] * 4; v[i] = float(s); put(v)
    for sg in itertools.product((0.5, -0.5), repeat=4):
        put(sg)
    base = (PHI / 2, 0.5, 1 / (2 * PHI), 0.0)
    for p in itertools.permutations(range(4)):
        inv = sum(1 for i in range(4) for j in range(i + 1, 4) if p[i] > p[j])
        if inv % 2:
            continue
        for sg in itertools.product((1, -1), repeat=3):
            v = [0.0] * 4
            for i in range(3):
                v[p[i]] = base[i] * sg[i]
            put(v)
    return [vs[k] for k in sorted(vs)]

def matmul(A, B): return [[sum(A[i][k] * B[k][j] for k in range(4)) for j in range(4)] for i in range(4)]
def matvec(A, v): return [sum(A[i][k] * v[k] for k in range(4)) for i in range(4)]

def coxeter_plane(V):
    gram = {(0, 1): -PHI / 2, (1, 2): -0.5, (2, 3): -0.5}
    def ok(ch, cand):
        j = len(ch)
        for i, s in enumerate(ch):
            want = gram.get((i, j), 0.0)
            if abs(dot(s, cand) - want) > 1e-9:
                return False
        return True
    def search(ch):
        if len(ch) == 4:
            return ch
        for cand in V:
            if ok(ch, cand):
                r = search(ch + [cand])
                if r:
                    return r
        return None
    S = search([])
    I = [[float(i == j) for j in range(4)] for i in range(4)]
    refl = [[[I[i][j] - 2 * s[i] * s[j] for j in range(4)] for i in range(4)] for s in S]
    c = matmul(matmul(refl[0], refl[1]), matmul(refl[2], refl[3]))
    P = I
    for k in range(1, 40):
        P = matmul(P, c)
        if max(abs(P[i][j] - I[i][j]) for i in range(4) for j in range(4)) < 1e-9:
            assert k == 30, k       # Coxeter number of H4
            break
    c2 = matmul(c, c); k11 = 2 * cos(2 * pi * 11 / 30)
    K = [[c2[i][j] - k11 * c[i][j] + I[i][j] for j in range(4)] for i in range(4)]
    basis = []
    for e in I:
        w = matvec(K, e)
        for b in basis:
            d = dot(w, b); w = [x - d * y for x, y in zip(w, b)]
        if norm(w) > 1e-6:
            basis.append(unit(w))
    assert len(basis) == 2
    b1, b2 = basis
    cb1 = matvec(c, b1)
    ang = math.atan2(dot(cb1, b2), dot(cb1, b1))
    assert abs(abs(ang) - 2 * pi / 30) < 1e-9
    return b1, b2

def fig_600cell():
    V = icosians()
    assert len(V) == 120 and all(abs(norm(v) - 1) < 1e-12 for v in V)
    E = [(i, j) for i in range(120) for j in range(i + 1, 120) if abs(norm(add(V[i], scl(V[j], -1))) - 1 / PHI) < 1e-9]
    print(f"600-cell: vertices={len(V)} edges={len(E)}")
    assert len(V) == 120 and len(E) == 720
    deg = [0] * 120
    for i, j in E:
        deg[i] += 1; deg[j] += 1
    assert set(deg) == {12}
    b1, b2 = coxeter_plane(V)
    xy = [(dot(v, b1), dot(v, b2)) for v in V]
    rad = [math.hypot(*p) for p in xy]
    rings = sorted(set(round(r, 6) for r in rad))
    assert len(rings) == 4, rings
    rmax = max(rad); S = 238 / rmax; cx, cy = 590, 362
    dep = [sqrt(max(0, 1 - r * r)) for r in rad]       # distance out of the projection plane
    fg = Fig("600-cell-icosians.svg", "The 120 unit icosians: the 600-cell",
             "The 120 vertices of the 600-cell, which are the 120 unit icosians and the root system of H4, projected from 4-D to the plane with the Coxeter-plane projection. The 120 points fall on four concentric rings of 30 with 30-fold symmetry, joined by the 720 edges between vertices at distance one over phi. Depth is shown by opacity.",
             ["During training each step's rotation u_t can be snapped to the nearest of these 120 unit icosians — an exact, finite codebook.",
              "Each vertex has 12 neighbours: 120 × 12 / 2 = 720 edges. Fainter points and edges lie farther out of the plane."])
    bins = {}
    for i, j in E:
        t = 1 - (dep[i] + dep[j]) / 2 / max(dep)
        bins.setdefault(min(5, int(t * 6)), []).append((i, j))
    for b in sorted(bins):
        d = "".join(f"M{f(cx + S * xy[i][0])} {f(cy - S * xy[i][1])}L{f(cx + S * xy[j][0])} {f(cy - S * xy[j][1])}" for i, j in bins[b])
        fg.path(d, stroke="#4a5b6e", w=0.7 + 0.1 * b, op=0.08 + 0.1 * b)
    ringcol = [PINK, ORANGE, GREEN, BLUE]
    for i in sorted(range(120), key=lambda i: -dep[i]):
        k = rings.index(round(rad[i], 6)); t = 1 - dep[i] / max(dep)
        fg.circle(cx + S * xy[i][0], cy - S * xy[i][1], 4.4 + 1.6 * t, ringcol[k], "#fff", 1, fop=0.35 + 0.65 * t)
    counts = [sum(1 for r in rad if round(r, 6) == g) for g in rings]
    fg.rect(40, 92, 232, 168, "#f6f9fc", FAINT, 1.2, 14)
    fg.text(56, 120, "120 vertices", 17, INK, weight="bold")
    for k, (g, cnt) in enumerate(zip(reversed(rings), reversed(counts))):
        kk = rings.index(g); fg.circle(66, 144 + 25 * k, 6, ringcol[kk], "#fff", 1.2)
        fg.text(82, 149 + 25 * k, f"ring {k + 1}: {cnt} points, r = {g:.3f}", 14, INK)
    fg.text(56, 252, "30-fold symmetry (Coxeter number 30)", 12.5, MUTED)
    fg.text(960, 112, "24 = (±1,0,0,0) perms", 13.5, MUTED, "end")
    fg.text(960, 132, "  and (±½,±½,±½,±½)", 13.5, MUTED, "end")
    fg.text(960, 156, "96 = even perms of", 13.5, MUTED, "end")
    fg.text(960, 176, "½(±φ, ±1, ±1/φ, 0)", 13.5, MUTED, "end")
    fg.save()


# ================================================================ FIG 5
def fig_golden():
    fg = Fig("golden-integers.svg", "Exact golden integers Z[φ]",
             "Top: the numbers a plus b phi for small integers a and b placed on the real line, one row per value of b, with their union below. Bottom: the Galois-conjugate embedding that sends the pair a, b to the point a plus b phi and a plus b phi-bar in the plane, a lattice, with the points in a horizontal band projecting to an aperiodic exact chain on the line.",
             ["Icosian coordinates are exact numbers a + bφ (integers a, b); chirality and polarity are exact signs of these coordinates.",
              "Rows b = −3…3 are integers shifted by bφ; below, each a + bφ is paired with its conjugate a + bφ̄ (φ̄ = 1 − φ)."])
    x0, x1, lo, hi = 90, 950, -6.0, 9.0
    X = lambda t: x0 + (t - lo) / (hi - lo) * (x1 - x0)
    bcol = [BLUE, SKY, GREEN, GRID, ORANGE, VERM, PINK]
    fg.text(40, 92, "a + bφ on the real line (φ = 1.6180…)", 15, MUTED)
    for r, b in enumerate(range(-3, 4)):
        y = 118 + 21 * r
        fg.line(x0, y, x1, y, FAINT, 1)
        fg.text(66, y + 4.5, f"b={b}", 12.5, bcol[r] if r != 3 else MUTED, "end", weight="bold")
        for a in range(-14, 16):
            t = a + b * PHI
            if lo <= t <= hi:
                fg.circle(X(t), y, 4.3, bcol[r], "#fff", 1)
    yu = 280
    fg.line(x0, yu, x1, yu, INK, 1.4)
    allv = sorted(set(round(a + b * PHI, 9) for a in range(-12, 13) for b in range(-12, 13) if lo <= a + b * PHI <= hi))
    for t in allv:
        fg.line(X(t), yu - 9, X(t), yu + 9, "#44596e", 1, 0.8)
    fg.text(66, yu + 4.5, "all", 12.5, INK, "end", weight="bold")
    for t, lb, lvl in ((0, "0", 0), (1, "1", 0), (PHI, "φ", 0), (sqrt(5), "√5 = 2φ−1", 1), (PHI * PHI, "φ² = φ+1", 0), (-1 / PHI, "−1/φ = 1−φ", 1)):
        fg.line(X(t), 108, X(t), yu + 14, VERM, 1, 0.5, "3 3")
        fg.text(X(t), yu + 30 + 15 * lvl, lb, 13.5, "#9a3a00", "middle", weight="bold")
    # lattice
    sc, ox, oy = 19.0, 300, 478
    fg.text(40, 358, "(a, b) ↦ (a + bφ, a + bφ̄)", 15, MUTED)
    N = 4; win = PHI / 2
    fg.rect(ox - 11 * sc, oy - win * sc, 22 * sc, 2 * win * sc, SKY, rx=0, fop=0.2)
    fg.line(ox - 11 * sc, oy, ox + 11 * sc, oy, INK, 1.2); fg.line(ox, oy - 7 * sc, ox, oy + 7 * sc, INK, 1.2)
    P = {(a, b): (a + b * PHI, a + b * PHIB) for a in range(-N, N + 1) for b in range(-N, N + 1)}
    for (a, b), (x, y) in P.items():
        for da, db in ((1, 0), (0, 1)):
            if (a + da, b + db) in P:
                x2, y2 = P[(a + da, b + db)]
                fg.line(ox + sc * x, oy - sc * y, ox + sc * x2, oy - sc * y2, GRID, 0.6, 0.35)
    inside = []
    for (a, b), (x, y) in sorted(P.items()):
        ins = abs(y) <= win
        fg.circle(ox + sc * x, oy - sc * y, 3.6 if ins else 3, BLUE if ins else GRID, "#fff", 0.8, fop=1 if ins else 0.7)
        if ins:
            inside.append(x)
    xs = sorted(inside)
    gaps = sorted(set(round(b - a, 6) for a, b in zip(xs, xs[1:]) if -6 < a < 6 and -6 < b < 6))
    print("band gaps:", gaps)
    for x in xs:
        fg.line(ox + sc * x, oy - 5, ox + sc * x, oy + 5, VERM, 1.6)
    fg.text(ox + 11 * sc + 4, oy + 4, "x = a + bφ", 12.5, MUTED)
    fg.text(ox + 6, oy - 7 * sc + 4, "x′ = a + bφ̄", 12.5, MUTED)
    fg.text(ox - 11 * sc, oy - win * sc - 6, "band |x′| ≤ φ/2: its points project to an aperiodic chain", 12.5, "#1d6fa0")
    # algebra
    px = 600
    fg.rect(px, 372, 360, 236, "#f6f9fc", FAINT, 1.2, 14)
    for a, b, c_, d in ((2, -1, 3, 5), (-4, 7, 1, -2)):
        prod = (a * c_ + b * d, a * d + b * c_ + b * d)
        assert abs((a + b * PHI) * (c_ + d * PHI) - (prod[0] + prod[1] * PHI)) < 1e-9
        assert abs((a + b * PHI) * (a + b * PHIB) - (a * a + a * b - b * b)) < 1e-9
    assert abs(PHI * PHI - PHI - 1) < 1e-12
    for i, (l, bold) in enumerate([("Z[φ] = { a + bφ : a, b ∈ ℤ },  φ² = φ + 1", True),
                                   ("closed under + and ×:", False), ("(a+bφ)(c+dφ) = (ac+bd) + (ad+bc+bd)φ", False), ("",False),
                                   ("conjugate: φ̄ = 1 − φ = −1/φ  (√5 ↦ −√5)", False),
                                   ("norm N(a+bφ) = a² + ab − b²  ∈ ℤ", False), ("", False),
                                   ("every product, sum and norm stays an", False), ("exact integer pair — never a rounded float.", False)]):
        fg.text(px + 16, 402 + 24 * i, l, 14.5, INK, weight="bold" if bold else "normal")
    fg.save()


# ================================================================ FIG 6
ZEROS = [14.134725142, 21.022039639, 25.010857580, 30.424876126, 32.935061588, 37.586178159, 40.918719012, 43.327073281, 48.005150881, 49.773832478]
PHASE_STOPS = [BLUE, SKY, "#F0E442", ORANGE, VERM, PINK, PURPLE]

def fig_zeta():
    fg = Fig("zeta-phases.svg", "Fixed zeta-zero phases as position signatures",
             "Ten small unit circles, one per nontrivial zeta zero gamma-k, each showing the phase e to the i gamma-k log n of positions n equals 1 to 12 as dots; below, a colour table of the same phases where every position n gives a different column of ten phases.",
             ["Fixed phases from the zeta zeros γ: each pair of primes gets the phase γ·(log p − log q).",
              "The model uses them as a fixed lookup table of distinct, non-repeating offsets; they are not learned.",
              "Dot n sits at angle γ·ln n (mod 2π); in the table, colour = phase, so each column is one position’s signature."])
    NS = list(range(1, 13)); ncol = lambda n: ramp([ORANGE, VERM, PURPLE, BLUE], (n - 1) / 11)
    ang = lambda g, n: (g * log(n)) % (2 * pi)
    for k, g in enumerate(ZEROS):
        cx = 40 + 92 + 184 * (k % 5); cy = 168 + 168 * (k // 5); r = 56
        fg.text(cx, cy - r - 12, f"γ{sub(k + 1)} = {g:.4f}", 14, INK, "middle", weight="bold")
        fg.circle(cx, cy, r, "#f6f9fc", FAINT, 1.4)
        fg.line(cx - r, cy, cx + r, cy, FAINT, 0.8); fg.line(cx, cy - r, cx, cy + r, FAINT, 0.8)
        pts = [(cx + r * cos(ang(g, n)), cy - r * sin(ang(g, n))) for n in NS]
        fg.poly(pts, stroke=GRID, w=0.9, op=0.5)
        for n, (x, y) in zip(NS, pts):
            fg.circle(x, y, 5.4, ncol(n), "#fff", 1)
            if n in (1, 2, 3, 5, 7, 11):
                dx, dy = x - cx, y - cy; L = math.hypot(dx, dy) or 1
                fg.text(x + dx / L * 13, y + dy / L * 13 + 4, str(n), 11, INK, "middle")
    ty, cw, chh, tx = 466, 42, 13.5, 120
    fg.text(40, ty - 26, "phase of n at each zero", 13.5, MUTED)
    for j, n in enumerate(NS):
        fg.text(tx + cw * j + cw / 2, ty - 10, f"n={n}" if n == 1 else str(n), 12.5, INK, "middle", weight="bold")
    for k, g in enumerate(ZEROS):
        fg.text(tx - 8, ty + chh * k + 11, f"γ{sub(k + 1)}", 11.5, MUTED, "end")
        for j, n in enumerate(NS):
            fg.rect(tx + cw * j + 1, ty + chh * k + 1, cw - 2, chh - 2, ramp(PHASE_STOPS, ang(g, n) / (2 * pi), True), rx=2)
    cols = [tuple(round(ang(g, n), 6) for g in ZEROS) for n in NS]
    assert len(set(cols)) == 12
    mind = min(max(abs((a - b + pi) % (2 * pi) - pi) for a, b in zip(c1, c2)) for c1, c2 in itertools.combinations(cols, 2))
    print(f"min over position pairs of max phase gap: {mind:.3f} rad")
    wx, wy, wr = 800, 536, 62
    for i in range(48):
        a0, a1 = 2 * pi * i / 48, 2 * pi * (i + 1.04) / 48
        pa = (wx + wr * cos(a0), wy - wr * sin(a0)); pb = (wx + wr * cos(a1), wy - wr * sin(a1))
        pc = (wx + 0.58 * wr * cos(a1), wy - 0.58 * wr * sin(a1)); pd = (wx + 0.58 * wr * cos(a0), wy - 0.58 * wr * sin(a0))
        c = ramp(PHASE_STOPS, (i + .5) / 48, True)
        fg.path(f"M{f(pa[0])} {f(pa[1])}L{f(pb[0])} {f(pb[1])}L{f(pc[0])} {f(pc[1])}L{f(pd[0])} {f(pd[1])}Z", stroke=c, w=0.5, fill=c)
    for a, lb in ((0, "0"), (pi / 2, "π/2"), (pi, "π"), (3 * pi / 2, "3π/2")):
        fg.text(wx + (wr + 14) * cos(a), wy - (wr + 14) * sin(a) + 4, lb, 12, MUTED, "middle")
    fg.text(wx, wy + wr + 28, "colour = phase γ·ln n", 13, MUTED, "middle")
    fg.save()


# ================================================================ FIG 7
def factor(n):
    out, p = {}, 2
    while n > 1:
        while n % p == 0:
            out[p] = out.get(p, 0) + 1; n //= p
        p += 1
    return out

def fig_primes():
    fg = Fig("prime-addressing.svg", "Prime addressing: an address is a product of primes",
             "The integers 1 to 60 on a grid with primes highlighted. The composite address 30 is joined by curves to its prime coordinates 2, 3 and 5, and all multiples of 5, the records sharing that factor, are ringed. A table of exponent vectors shows that a gcd is the component-wise minimum of exponents.",
             ["An address is a product of primes; a read admits records that share a factor (gcd > 1) or the longest matching prime n-let,",
              "exactly, not by approximate similarity.",
              "Prime exponents are the coordinates: gcd is the component-wise minimum, so shared factors are read off exactly."])
    isp = lambda n: n > 1 and all(n % d for d in range(2, int(sqrt(n)) + 1))
    gx, gy, cw, ch = 50, 138, 58, 46
    cell = lambda n: (gx + ((n - 1) % 10) * cw, gy + ((n - 1) // 10) * ch)
    pc = {2: GREEN, 3: PINK, 5: ORANGE}
    fg.text(50, 438, "records 1 … 60  (blue = prime; ring = multiple of 5, shares a factor with 30)", 13.5, MUTED)
    for n in range(1, 61):
        x, y = cell(n); p = isp(n)
        fg.rect(x + 2, y + 2, cw - 4, ch - 4, "#d6eaf8" if p else ("#eef1f4" if n == 1 else "#ffffff"), BLUE if p else FAINT, 1.4 if p else 1, 7)
        if n % 5 == 0:
            fg.rect(x + 2, y + 2, cw - 4, ch - 4, "none", ORANGE, 3.2, 7)
        fg.text(x + cw / 2, y + ch / 2 + 6, str(n), 17, BLUE if p else INK, "middle", weight="bold" if p or n == 30 else "normal")
    x30, y30 = cell(30)
    fg.rect(x30 + 2, y30 + 2, cw - 4, ch - 4, "#fff4cf", INK, 2.6, 7)
    fg.text(x30 + cw / 2, y30 + ch / 2 + 6, "30", 17, INK, "middle", weight="bold")
    for p, col in pc.items():
        x, y = cell(p)
        fg.rect(x + 2, y + 2, cw - 4, ch - 4, "none", col, 3.4, 7)
        sx, sy = x30 + cw - 2, y30 + ch / 2; ex, ey = x + cw / 2, y + 2
        top = gy - 44 - 8 * list(pc).index(p)
        d = f"M{f(sx)} {f(sy)}C{f(sx + 46)} {f(sy)} {f(sx + 40)} {f(top)} {f(ex + 40)} {f(top)}L{f(ex)} {f(top)}L{f(ex)} {f(ey - 2)}"
        fg.path(d, stroke="#ffffff", w=6, op=0.9)
        fg.path(d, stroke=col, w=2.8)
        fg.add(f'<polygon points="{f(ex)},{f(ey)} {f(ex - 5)},{f(ey - 10)} {f(ex + 5)},{f(ey - 10)}" fill="{col}"/>')
    # right: factor card
    px = 690
    fg.rect(px, 92, 272, 332, "#f6f9fc", FAINT, 1.2, 16)
    fg.text(px + 18, 124, "address 30", 19, INK, weight="bold")
    fg.rect(px + 18, 142, 64, 44, "#fff4cf", INK, 2.4, 8); fg.text(px + 50, 172, "30", 22, INK, "middle", weight="bold")
    fg.text(px + 96, 172, "=", 22, INK, "middle")
    for i, (p, col) in enumerate(pc.items()):
        bx = px + 112 + 52 * i
        fg.rect(bx, 142, 44, 44, col, rx=8, fop=0.9); fg.text(bx + 22, 172, str(p), 22, "#fff", "middle", weight="bold")
        if i < 2: fg.text(bx + 48, 172, "×", 14, INK, "middle")
    assert 2 * 3 * 5 == 30 and factor(30) == {2: 1, 3: 1, 5: 1}
    fg.text(px + 18, 214, "exponent vector over primes", 13.5, MUTED)
    for i, p in enumerate([2, 3, 5, 7, 11, 13]):
        e = factor(30).get(p, 0); bx = px + 18 + 38 * i
        fg.rect(bx, 224, 34, 38, pc.get(p, "#ffffff") if e else "#ffffff", FAINT if not e else "none", 1.2, 6, fop=0.9)
        fg.text(bx + 17, 241, str(e), 16, "#fff" if e else MUTED, "middle", weight="bold"); fg.text(bx + 17, 256, str(p), 11, "#fff" if e else MUTED, "middle")
    fg.text(px + 18, 292, "read: “records containing factor 5”", 14, INK, weight="bold")
    mult = [n for n in range(1, 61) if n % 5 == 0]
    fg.text(px + 18, 316, ", ".join(map(str, mult[:6])) + ",", 14, "#9a6a00")
    fg.text(px + 18, 336, ", ".join(map(str, mult[6:])) + f"   ({len(mult)} of 60)", 14, "#9a6a00")
    fg.text(px + 18, 372, "no scoring, no nearest neighbour:", 13.5, MUTED)
    fg.text(px + 18, 392, "n is in the answer iff 5 | n.", 13.5, MUTED)
    # table
    ty, tx = 452, 50
    heads = ["record", "2", "3", "5", "7", "gcd with 30", "shared primes"]
    xs = [0, 90, 150, 210, 270, 350, 520]
    for h, x in zip(heads, xs):
        fg.text(tx + x + 6, ty + 14, h, 13, MUTED, weight="bold")
    for r, n in enumerate([30, 12, 45, 35, 49]):
        y = ty + 24 + 27 * r; fa, f30 = factor(n), factor(30)
        g = {p: min(fa.get(p, 0), f30.get(p, 0)) for p in (2, 3, 5, 7)}
        gv = 1
        for p, e in g.items(): gv *= p ** e
        assert gv == math.gcd(n, 30)
        fg.rect(tx, y, 690 - tx + 30, 24, "#f6f9fc" if r % 2 == 0 else "#ffffff", rx=5)
        fg.text(tx + xs[0] + 6, y + 17, f"{n} = " + " × ".join(f"{p}" + (sub(e) if False else (f"^{e}" if e > 1 else "")) for p, e in sorted(fa.items())), 13.5, INK, weight="bold")
        for j, p in enumerate((2, 3, 5, 7)):
            e = fa.get(p, 0)
            fg.text(tx + xs[1 + j] + 16, y + 17, str(e), 14, pc.get(p, INK) if e else "#aab4c0", "middle", weight="bold" if e else "normal")
        fg.text(tx + xs[5] + 6, y + 17, str(gv), 14, INK, weight="bold")
        fg.text(tx + xs[6] + 6, y + 17, " · ".join(str(p) for p in (2, 3, 5, 7) if g[p]) or "none (coprime)", 13.5, MUTED)
    fg.save()


def fig_next_token():
    fg = Fig("next-token.svg", "How the next token is predicted",
             "Left-to-right pipeline of the geometric stack: a token id selects an embedding row; sixteen layers in the pattern rrarrarrarrarrar alternate quaternion-rotation recurrence (r) with reads over earlier positions (a); a final norm gives logits over the 4,096 tokens, a softmax in the float form, and an optional copy head routed by prime products mixes in a copy distribution to give the next-token probabilities. A band notes that the served form uses integer table reads and a greedy argmax over integer scores.",
             ["Geometry: state carried by quaternion rotation, the past read by a Lorentz score, copying routed by primes. Float form shown.",
              "Example numbers are illustrative. Served form: D11 integer table reads for steps 2-6, greedy argmax, no vocabulary softmax."])
    SOFT = "#f6f9fc"

    def badge(x, y, n, col=INK):
        fg.circle(x, y, 10, col)
        fg.text(x, y + 4.5, str(n), 12, "#ffffff", "middle", weight="bold")

    def box(x, y, w, h, fill=SOFT, stroke=FAINT, sw=1.4):
        fg.rect(x, y, w, h, fill, stroke, sw, 12)

    # ---- row 1: token -> embedding -> layers
    box(40, 160, 90, 70)
    fg.text(85, 184, "token id", 13, MUTED, "middle")
    fg.text(85, 205, "x_t", 20, INK, "middle", weight="bold")
    fg.text(85, 221, "byte-BPE", 11.5, MUTED, "middle")
    fg.arrow(130, 195, 155, 195, GRID, 2, 9)
    box(155, 160, 140, 70)
    fg.text(225, 182, "embedding E", 13, INK, "middle", weight="bold")
    fg.text(225, 200, "row E[x_t]: 4,096 × d", 11.5, MUTED, "middle")
    fg.text(225, 216, "(d = 1536 at 214M)", 11.5, MUTED, "middle")
    badge(155, 160, 1, GRID)
    fg.arrow(295, 195, 320, 195, GRID, 2, 9)

    box(320, 90, 640, 210, "#fbfcfd")
    fg.text(336, 113, "16 layers, repeated pattern:", 13, INK, weight="bold")
    for i, ch in enumerate("rrarrarrarrarrar"):
        fg.text(540 + 12 * i, 114, ch, 15, BLUE if ch == "r" else ORANGE, weight="bold")
    fg.text(745, 113, "(r = recurrence, a = read)", 11.5, MUTED)
    # r layer
    fg.rect(336, 128, 290, 134, "#eef6fc", BLUE, 1.6, 10)
    badge(336, 128, 2, BLUE)
    fg.text(354, 147, "r layer: carry the state by rotation", 13, BLUE, weight="bold")
    cx, cy = 392, 196
    fg.circle(cx, cy, 28, "#ffffff", BLUE, 1.4)
    a0, a1 = 0.0, -1.15
    fg.arrow(cx, cy, cx + 24 * cos(a0), cy + 24 * sin(a0), MUTED, 2, 8)
    fg.arrow(cx, cy, cx + 24 * cos(a1), cy + 24 * sin(a1), BLUE, 2.6, 8)
    fg.path(f"M{f(cx + 13)} {f(cy)}A13 13 0 0 0 {f(cx + 13 * cos(a1))} {f(cy + 13 * sin(a1))}", stroke=VERM, w=1.8)
    fg.text(450, 172, "u_t: unit quaternion", 11.5, INK)
    fg.text(450, 188, "optional snap to 1 of 120", 11.5, MUTED)
    fg.text(450, 203, "icosians (training)", 11.5, MUTED)
    fg.text(450, 219, "λ_t gate; a_t conv input", 11.5, INK)
    fg.text(350, 240, "grey: h_{t−1}    blue: u_t · h_{t−1}", 11, MUTED)
    fg.text(350, 256, "h_t = λ_t (u_t · h_{t−1}) + √(1−λ_t²) a_t", 12.5, INK, weight="bold")
    # a layer
    fg.arrow(628, 180, 672, 180, GRID, 2, 8)
    fg.arrow(672, 206, 628, 206, GRID, 2, 8)
    fg.rect(674, 128, 270, 134, "#fff8e8", ORANGE, 1.6, 10)
    badge(674, 128, 3, ORANGE)
    fg.text(692, 147, "a layer: read the past", 13, "#9a6a00", weight="bold")
    ys = 196
    fg.add(f'<circle cx="700" cy="{ys}" r="8" fill="none" stroke="{GRID}" stroke-width="1.4" stroke-dasharray="3 2"/>')
    xs = [740, 780, 820, 860]
    for x in xs:
        fg.circle(x, ys, 8, ORANGE, fop=0.35)
    fg.circle(912, ys, 10, ORANGE)
    fg.text(912, ys + 4, "t", 12, "#ffffff", "middle", weight="bold")
    for x, w in zip(xs, (1.2, 2.0, 4.6, 3.0)):
        fg.path(f"M{f(912)} {f(ys - 10)}Q{f((912 + x) / 2)} {f(ys - 58)} {f(x)} {f(ys - 9)}", stroke=ORANGE, w=w, op=0.85)
    fg.text(700, ys + 24, "NoRead", 10.5, MUTED, "middle")
    fg.text(800, ys + 24, "earlier positions", 10.5, MUTED, "middle")
    fg.text(692, 240, "score: Dot or Lorentz −β·arcosh(1+e)", 12, INK)
    fg.text(692, 256, "+ age term; softmax mixes earlier states", 12, INK)
    fg.text(640, 284, "after every layer: SwiGLU MLP + residual", 12, MUTED, "middle")
    badge(320, 284, 4, GRID)

    # ---- connector to row 2
    fg.path("M640 300L640 326L85 326", stroke=GRID, w=2)
    fg.arrow(85, 326, 85, 356, GRID, 2, 9)
    fg.text(100, 320, "h_t from the last layer", 11.5, MUTED)

    # ---- row 2: norm -> logits -> softmax -> mix -> p
    box(40, 358, 90, 62)
    fg.text(85, 384, "final norm", 13, INK, "middle", weight="bold")
    fg.text(85, 402, "norm(h_t)", 11.5, MUTED, "middle")
    fg.arrow(130, 389, 158, 389, GRID, 2, 9)
    z = [("mat", 1.6), ("cat", 1.9), ("sat", 0.9), ("dog", 1.3), ("on", 0.2)]
    ez = [math.exp(v) for _, v in z]
    soft = [e / sum(ez) for e in ez]
    copy = {"mat": 0.85, "sat": 0.15}
    g = 0.6
    fin = [(1 - g) * soft[i] + g * copy.get(z[i][0], 0.0) for i in range(len(z))]
    box(158, 352, 196, 108)
    badge(158, 352, 5, GRID)
    fg.text(172, 372, "logits z_t = norm(h_t) · Eᵀ", 12, INK, weight="bold")
    base = 436
    for i, (name, v) in enumerate(z):
        bx = 174 + 31 * i
        fg.rect(bx, base - v * 22, 22, v * 22, BLUE, rx=3, fop=0.85)
        fg.text(bx + 11, base + 13, name, 10.5, MUTED, "middle")
    fg.text(342, base, "…", 14, MUTED, "middle")
    fg.text(342, base + 13, "4,096", 9.5, MUTED, "middle")
    fg.arrow(354, 389, 380, 389, GRID, 2, 9)
    box(380, 364, 84, 50)
    fg.text(422, 386, "softmax", 13, INK, "middle", weight="bold")
    fg.text(422, 402, "float form", 11, MUTED, "middle")
    fg.arrow(464, 389, 524, 389, GRID, 2, 9)
    fg.circle(560, 389, 32, "#e6f6ef", GREEN, 2.2)
    fg.text(560, 394, "mix", 14, GREEN, "middle", weight="bold")
    fg.text(560, 346, "p(v) = (1−g_t)·softmax(z_t)[v] + g_t·p_copy(v)", 12, INK, "middle", weight="bold")
    fg.arrow(592, 389, 640, 389, GRID, 2, 9)
    box(640, 358, 320, 102)
    fg.text(656, 378, "p(next token)", 13.5, INK, weight="bold")
    for i, (name, v) in enumerate(z):
        bx = 664 + 54 * i
        hh = fin[i] * 90
        col = GREEN if name == "mat" else BLUE
        fg.rect(bx, base - hh, 34, hh, col, rx=3, fop=0.85)
        fg.text(bx + 17, base + 13, name, 10.5, MUTED, "middle")
    fg.text(944, 378, "served: greedy argmax", 11, MUTED, "end")

    # ---- copy head
    box(340, 474, 440, 112, "#f2faf6", GREEN, 1.6)
    badge(340, 474, 6, GREEN)
    fg.text(358, 494, "optional copy head", 13, "#00704f", weight="bold")
    fg.text(520, 494, "gate g_t = sigmoid(w_g·h_t + b_g)", 12, INK)
    cells = ["the", "cat", "sat", "on", "the", "mat", "on the"]
    x0, cwid, cy0 = 366, 50, 524
    for i, c in enumerate(cells):
        x = x0 + 54 * i
        cur = i == 6
        hit = i == 5
        wmatch = i in (3, 4)
        fg.rect(x, cy0, cwid + (4 if cur else 0), 24, "#fff4cf" if cur else ("#d9f0e6" if hit else "#ffffff"),
                INK if cur else (GREEN if hit else (BLUE if wmatch else FAINT)), 2 if (cur or hit or wmatch) else 1.2, 5)
        fg.text(x + (cwid + (4 if cur else 0)) / 2, cy0 + 16, c, 11.5, INK, "middle", weight="bold" if cur or hit else "normal")
    fg.path(f"M{x0 + 54 * 6 + 24} {cy0}Q{x0 + 54 * 6 - 10} {cy0 - 34} {x0 + 54 * 5 + 25} {cy0 - 2}", stroke=GREEN, w=3)
    fg.add(f'<polygon points="{x0 + 54 * 5 + 25},{cy0 - 1} {x0 + 54 * 5 + 18},{cy0 - 9} {x0 + 54 * 5 + 31},{cy0 - 8}" fill="{GREEN}"/>')
    fg.text(x0 + 54 * 5 + 25, cy0 + 33, "p_copy(mat)", 10.5, "#00704f", "middle")
    fg.text(x0 + 54 * 3.5 + 24, cy0 + 33, "window match", 10.5, BLUE, "middle")
    fg.text(358, 573, "sources chosen by a learned score, or the exact prime route: gcd of prime products of the last", 11, MUTED)
    fg.text(358, 584, "≤ 6 tokens vs earlier windows (or the longest n-let)", 11, MUTED)
    fg.arrow(560, 474, 560, 424, GREEN, 2.4, 9)
    fg.text(570, 454, "g_t, p_copy", 11.5, "#00704f")
    fg.text(800, 492, "Example values are", 11.5, MUTED, style="italic")
    fg.text(800, 508, "illustrative, not a", 11.5, MUTED, style="italic")
    fg.text(800, 524, "measured output.", 11.5, MUTED, style="italic")

    # ---- serving band
    fg.rect(40, 596, 920, 24, "#e6f6ef", GREEN, 1.2, 12)
    badge(58, 608, 7, GREEN)
    fg.text(504, 612, "served: steps 2–6 as D11 integer table reads (no multiplier, no float); token = greedy argmax over i32 scores, no vocabulary softmax", 11.5, "#00704f", "middle", weight="bold")
    fg.save()


if __name__ == "__main__":
    for fn in (fig_quaternion, fig_s3, fig_hopf, fig_600cell, fig_golden, fig_zeta, fig_primes, fig_next_token):
        fn()
    print("wrote 8 figures to", OUT)
