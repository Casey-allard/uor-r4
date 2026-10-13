#!/usr/bin/env python3
"""
Route 1 (certified numerics): Arb / ball-arithmetic verifier utilities.

This script is intended to support *certificate-side* rigorous evaluation:
  - build the finite Γ-certificate at σ0 (Definitions def:certificate-operator/transfer)
  - evaluate the scalar transfer output J_cert,N(s) with complex ball arithmetic
  - (next step) combine with an arithmetic evaluator for J_N(s) and verify
      sup_R |J_N - J_cert,N| ≤ m_R/4

NOTE: The arithmetic-side evaluator for J_N(s) depends on the manuscript's outer
normalizer O(s). This script currently focuses on the certified certificate path
and rectangle covering infrastructure; see TODO "arith side" below.
"""

from __future__ import annotations

from collections import deque
from dataclasses import dataclass
from typing import Any, Dict, Iterable, List, Tuple

import argparse
import json
import math
from pathlib import Path
import sys
import time

import flint
from flint import arb, arb_mat, acb, acb_mat
import numpy as np


# -------------------------
# Small utilities
# -------------------------


def primes_upto(n: int) -> List[int]:
    if n < 2:
        return []
    sieve = bytearray(b"\x01") * (n + 1)
    sieve[:2] = b"\x00\x00"
    r = int(math.isqrt(n))
    for p in range(2, r + 1):
        if sieve[p]:
            step = p
            start = p * p
            sieve[start : n + 1 : step] = b"\x00" * (((n - start) // step) + 1)
    return [i for i in range(n + 1) if sieve[i]]


_PRIMES_CACHE: Dict[int, List[int]] = {}


def cached_primes_upto(n: int) -> List[int]:
    """
    Cache primes lists since certified computations may call prime loops many times.
    """
    key = int(n)
    if key not in _PRIMES_CACHE:
        _PRIMES_CACHE[key] = primes_upto(key)
    return _PRIMES_CACHE[key]


def acb_from_box(re_mid: float, re_rad: float, im_mid: float, im_rad: float) -> acb:
    return acb(arb(re_mid, re_rad), arb(im_mid, im_rad))


def acb_midpoint(z: acb) -> acb:
    """
    Return the midpoint of an `acb` box as a point `acb` (zero radii).

    This is intentionally *non-rigorous* when used to evaluate functions that vary
    across the box; it exists to support the exploratory "midpoint" outer mode,
    which would otherwise catastrophically overestimate (or even include 0) when
    fed wide complex boxes.
    """
    return acb(arb(z.real.mid()), arb(z.imag.mid()))


def _arb_zero() -> arb:
    return arb(0)


def _arb_one() -> arb:
    return arb(1)


# -------------------------
# Printed window Fourier transform (ψ̂_cert)
# -------------------------


def beta_interval(u: arb) -> arb:
    """
    β(u) = exp(-1/(u(1-u))) for u in (0,1), else 0.

    For interval input u, we safely intersect with (0,1) to avoid invalid
    evaluation outside the support.
    """
    lo = float(u.lower())
    hi = float(u.upper())
    if hi <= 0.0 or lo >= 1.0:
        return arb(0)

    # Clamp to [0,1] and evaluate on the intersection, but avoid dividing by an
    # interval that touches 0 or 1 (u(1-u)=0).
    a = max(lo, 0.0)
    b = min(hi, 1.0)
    if a >= b:
        return arb(0)

    # A tiny clamp away from endpoints for safe evaluation; we union with 0 to
    # cover the (true) β(0)=β(1)=0 values.
    eps = 1e-12
    aa = max(a, eps)
    bb = min(b, 1.0 - eps)
    if aa >= bb:
        # Entire intersection is within eps of an endpoint (0 or 1).
        # Use the crude but safe enclosure β ∈ [0, β(eps)].
        ue = arb(eps)
        val = (-arb(1) / (ue * (arb(1) - ue))).exp()
        return val.union(arb(0))

    mid = 0.5 * (aa + bb)
    rad = 0.5 * (bb - aa)
    uu = arb(mid, rad)
    denom = uu * (arb(1) - uu)
    val = (-arb(1) / denom).exp()

    if lo < eps or hi > 1.0 - eps:
        return val.union(arb(0))
    return val


@dataclass(frozen=True)
class PsiHatEngine:
    """
    Certified evaluator for ψ̂_cert(ξ) for the printed window ψ_cert = (1/12)ψ.

    Uses a rigorous interval Riemann sum for integrals of the smooth bump β
    appearing in the definition of the smooth step S.

    References:
      - Printed window and Lemma lem:psi-cert-Cwin in Riemann-Christmas-v1.tex
    """

    n_steps: int = 4000  # uniform partition for [0,1]

    # Precomputed B1 = ∫_0^1 β(u) du as an interval.
    B1: arb = arb(0)  # set by build()

    # Precomputed Iu = ∫_0^1 u β(u) du as an interval (for xi=0 limit).
    Iu: arb = arb(0)  # set by build()

    @staticmethod
    def _beta_point(u: float) -> arb:
        if u <= 0.0 or u >= 1.0:
            return arb(0)
        uu = arb(u)
        return (-arb(1) / (uu * (arb(1) - uu))).exp()

    @staticmethod
    def _beta_range_on(u0: float, u1: float) -> arb:
        """
        Certified enclosure of β on the interval [u0,u1] using unimodality
        (β increases on (0,1/2], decreases on [1/2,1)).
        """
        a = min(u0, u1)
        b = max(u0, u1)
        if b <= 0.0 or a >= 1.0:
            return arb(0)
        a = max(a, 0.0)
        b = min(b, 1.0)
        if a >= b:
            return arb(0)

        ba = PsiHatEngine._beta_point(a)
        bb = PsiHatEngine._beta_point(b)
        bmin = ba if float(ba) <= float(bb) else bb
        bmax = bb if float(ba) <= float(bb) else ba
        if a <= 0.5 <= b:
            bm = PsiHatEngine._beta_point(0.5)
            if float(bm) > float(bmax):
                bmax = bm
        return bmin.union(bmax)

    @staticmethod
    def build(n_steps: int = 4000) -> "PsiHatEngine":
        h = 1.0 / n_steps
        B1 = arb(0)
        Iu = arb(0)
        for k in range(n_steps):
            u0 = k * h
            u1 = (k + 1) * h
            b = PsiHatEngine._beta_range_on(u0, u1)
            B1 += b
            mid = (k + 0.5) * h
            u_interval = arb(mid, 0.5 * h)
            Iu += u_interval * b
        B1 *= arb(h)
        Iu *= arb(h)
        return PsiHatEngine(n_steps=n_steps, B1=B1, Iu=Iu)

    @property
    def m_cert(self) -> arb:
        """
        m_cert = ∫ ψ_cert = 1/4 exactly (Lemma lem:psi-cert-Cwin).

        We return the exact rational 1/4 as an arb.
        """
        return arb(1) / arb(4)

    def _beta_trig_integrals(self, xi: arb) -> Tuple[arb, arb]:
        """
        Return (Jc, Js) where:
          Jc = ∫_0^1 β(u) cos(xi*u) du
          Js = ∫_0^1 β(u) sin(xi*u) du
        """
        h = 1.0 / self.n_steps
        Jc = arb(0)
        Js = arb(0)
        for k in range(self.n_steps):
            u0 = k * h
            u1 = (k + 1) * h
            b = PsiHatEngine._beta_range_on(u0, u1)
            mid = (k + 0.5) * h
            interval = arb(mid, 0.5 * h)
            arg = xi * interval
            Jc += b * arg.cos()
            Js += b * arg.sin()
        Jc *= arb(h)
        Js *= arb(h)
        return Jc, Js

    def psihat_cert(self, xi_in: arb) -> arb:
        """
        Compute ψ̂_cert(ξ) for real ξ (as arb interval).

        Since ψ_cert is real even and nonnegative, ψ̂_cert is real even.
        """
        xi = abs(xi_in)
        # Exact at 0:
        if xi.contains(0):
            return self.m_cert

        Jc, Js = self._beta_trig_integrals(xi)

        # sin(xi)/xi and cos(xi)/xi appear; xi does not contain 0 here.
        sin_over_x = xi.sin() / xi
        cos_over_x = xi.cos() / xi

        # I_c = ∫_0^1 S(u) cos(xi u) du
        # I_s = ∫_0^1 S(u) sin(xi u) du
        Ic = sin_over_x - (Js / (xi * self.B1))
        Is = (-cos_over_x) + (Jc / (xi * self.B1))

        two_xi = arb(2) * xi
        out = (sin_over_x + two_xi.cos() * Ic + two_xi.sin() * Is) / arb(6)
        return out


# -------------------------
# Arithmetic side: det2 prime product and xi/zeta (certified)
# -------------------------


def det2_prime_trunc(s: acb, primes: List[int]) -> acb:
    """
    det₂(I - A_P(s)) for the diagonal prime operator truncated to primes<=P:
      det₂ = ∏_{p<=P} (1 - p^{-s}) * exp(p^{-s}).
    This is exact for the prime-truncated approximant A_P.
    """
    # IMPORTANT: compute via log-sum, not multiplicative product.
    #
    # For `acb` inputs with nontrivial radii, the direct product dramatically
    # overestimates (dependency blow-up) and can swallow 0, which then poisons
    # downstream divisions and Cayley transforms.
    #
    # Since |p^{-s}| < 1 for Re(s) > 0, each (1 - p^{-s}) stays in the disk
    # centered at 1 of radius < 1; in particular it avoids 0 and stays away
    # from the principal-log branch cut. Thus `log(1 - p^{-s})` is safe.
    logdet = acb(0)
    for p in primes:
        ps = acb(p) ** (-s)
        logdet += (acb(1) - ps).log() + ps
    return logdet.exp()


def _progress(msg: str, enabled: bool) -> None:
    if enabled:
        print(msg, file=sys.stderr, flush=True)


def _read_json(path: str) -> Any:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _write_json(path: str, data: Any) -> None:
    def sanitize(x: Any) -> Any:
        # Produce strict JSON (no NaN/Infinity) for machine-checkable artifacts.
        if isinstance(x, float):
            if math.isnan(x) or math.isinf(x):
                return None
            return x
        if isinstance(x, dict):
            return {k: sanitize(v) for k, v in x.items()}
        if isinstance(x, (list, tuple)):
            return [sanitize(v) for v in x]
        return x

    data2 = sanitize(data)
    Path(path).write_text(
        json.dumps(data2, indent=2, sort_keys=True, allow_nan=False) + "\n", encoding="utf-8"
    )


def prime_tail_bound_rosser_schoenfeld(P: int, alpha: float) -> float:
    """
    Explicit tail bound (as used in the manuscript) for alpha>1 and P>=17:

      sum_{p>P} p^{-alpha} <= 1.25506 * alpha / ((alpha-1) log P) * P^{1-alpha}
    """
    if P < 17:
        P = 17
    if alpha <= 1.0:
        return float("inf")
    C = 1.25506
    return C * alpha / ((alpha - 1.0) * math.log(P)) * (P ** (1.0 - alpha))


def det2_full_enclosure(
    s: acb,
    P_cut: int,
    k_max: int = 40,
    *,
    progress: bool = False,
    progress_every: int = 500,
) -> acb:
    """
    Certified enclosure for det₂(I-A(s)) using primes<=P_cut plus a prime-tail bound.

    We use the absolutely convergent representation (σ>1/2):
      log det₂(I-A(s)) = Σ_p (log(1 - p^{-s}) + p^{-s})
                      = - Σ_p Σ_{k>=2} p^{-k s} / k.

    Tail bound: |tail| <= Σ_{k=2..k_max} (1/k) Σ_{p>P_cut} p^{-kσ}  +  (remaining k>k_max term),
    where σ = inf Re(s) on the input ball.

    Note: this is coarse but purely explicit and unconditional.
    """
    sigma_lo = float(s.real.lower())
    if sigma_lo <= 0.5:
        raise ValueError("det2_full_enclosure requires Re(s) > 1/2")

    # For wide balls in the imaginary direction, computing det2 via exp(logdet)
    # can yield an enclosure that includes 0 purely from phase overestimation.
    # Many callers (notably the outer build) only need log|det2|, i.e. Re(logdet),
    # so we factor out a helper that returns an enclosure for logdet itself.
    return logdet2_full_enclosure(
        s,
        P_cut=P_cut,
        k_max=k_max,
        progress=progress,
        progress_every=progress_every,
    ).exp()


def logdet2_full_enclosure(
    s: acb,
    P_cut: int,
    k_max: int = 40,
    *,
    progress: bool = False,
    progress_every: int = 500,
) -> acb:
    """
    Certified enclosure for log det₂(I-A(s)) for Re(s)>1/2.

    Uses the absolutely convergent series
      log det₂ = - Σ_p Σ_{k>=2} p^{-k s} / k
    truncated to primes<=P_cut and k<=k_max, with explicit tail bounds:
      - primes>P_cut for k>=2 (Rosser–Schoenfeld prime tail),
      - k>k_max for primes<=P_cut (geometric tail using |p^{-s}|=p^{-sigma}).

    Returns an `acb` ball for log det₂; then log|det₂| is `logdet.real`.
    """
    sigma_lo = float(s.real.lower())
    if sigma_lo <= 0.5:
        raise ValueError("logdet2_full_enclosure requires Re(s) > 1/2")

    primes = [p for p in cached_primes_upto(P_cut) if p >= 2]

    logdet = acb(0)
    tail_head = 0.0
    t0 = time.time()
    for i, p in enumerate(primes, start=1):
        ps = acb(p) ** (-s)
        power = ps * ps  # k=2
        for k in range(2, k_max + 1):
            logdet -= power / acb(k)
            if k != k_max:
                power *= ps

        r = p ** (-sigma_lo)
        tail_head += (r ** (k_max + 1)) / ((k_max + 1) * (1.0 - r))

        if progress and progress_every > 0 and (i % progress_every == 0):
            dt = time.time() - t0
            _progress(f"[det2] primes {i}/{len(primes)} (elapsed {dt:.1f}s)", enabled=True)

    tail = 0.0
    for k in range(2, k_max + 1):
        alpha = k * sigma_lo
        tail += prime_tail_bound_rosser_schoenfeld(P_cut, alpha) / k

    alpha0 = (k_max + 1) * sigma_lo
    rem = prime_tail_bound_rosser_schoenfeld(P_cut, alpha0)
    rem *= (1.0 / (k_max + 1)) * (1.0 / (1.0 - (2.0 ** (-sigma_lo))))
    tail += rem

    tail += tail_head

    rad = arb(tail)
    return logdet + acb(arb(0, rad), arb(0, rad))


def det2_logabs_bound_sigma(sigma: float, P_cut: int) -> arb:
    """
    Coarse but robust bound for det₂ on a vertical line:

      For s = sigma + i t with sigma > 1/2, write x_p = p^{-s}, r_p = |x_p| = p^{-sigma}.
      Then
        log((1-x_p) e^{x_p}) = - Σ_{k>=2} x_p^k / k
      so
        |log((1-x_p)e^{x_p})| <= Σ_{k>=2} r_p^k/k = -log(1-r_p) - r_p =: E(r_p).
      Hence
        |log det₂(I-A(s))| <= Σ_p E(p^{-sigma}) =: E_total(sigma),
      and therefore
        log|det₂| ∈ [-E_total, +E_total].

    We compute E_total by summing primes<=P_cut exactly and bounding the tail using
      E(r) <= r^2 / (2(1-r))  for 0<r<1
    and the explicit prime tail bound for Σ_{p>P} p^{-2sigma}.
    """
    if sigma <= 0.5:
        raise ValueError("sigma must be > 1/2")
    primes = [p for p in cached_primes_upto(P_cut) if p >= 2]
    sig = float(sigma)
    # Sum E(p^{-sigma}) over primes<=P_cut
    E_sum = 0.0
    for p in primes:
        r = p ** (-sig)
        # E(r) = -log(1-r) - r
        E_sum += -math.log(1.0 - r) - r

    # Tail: E(r) <= r^2 / (2(1-r)) <= r^2 / (2(1-2^{-sigma}))
    c = 1.0 / (2.0 * (1.0 - (2.0 ** (-sig))))
    tail_p2 = prime_tail_bound_rosser_schoenfeld(P_cut, 2.0 * sig)
    E_sum += c * tail_p2
    return arb(E_sum)


def xi_completed(s: acb) -> acb:
    """
    ξ(s) = 1/2 s(s-1) π^{-s/2} Γ(s/2) ζ(s).
    """
    half = acb(arb(1) / arb(2))
    pi = acb.pi()
    return (
        half
        * s
        * (s - acb(1))
        * (pi ** (-(s * half)))
        * (s * half).gamma()
        * s.zeta()
    )


def compensator_B(s: acb) -> acb:
    """
    ζ-compensator used in the manuscript's ζ-gauge:

      B(s) = s/(s-1).

    In the ζ-gauge
      J(s) = det2(I-A(s)) / (O(s) ζ(s)) * B(s),
    the factor 1/ζ has a simple zero at s=1, while B has a simple pole at s=1,
    so J is regular and typically nonzero at s=1.

    On the boundary line Re(s)=1/2 one has |B|=1, so B does not change boundary
    modulus (it only shifts the boundary phase by an explicit rational term).
    """
    return s / (s - acb(1))


def J_arith_raw_xi(s: acb, primes: List[int]) -> acb:
    """
    Raw arithmetic proxy (no outer): J_raw = det2_P(s) / ξ(s).

    WARNING: this is *not* the manuscript's J_N unless the outer normalizer is 1.
    It is useful as a first certified falsification / compatibility check.
    """
    return det2_prime_trunc(s, primes) / xi_completed(s)


def J_arith_raw_zeta(s: acb, primes: List[int]) -> acb:
    """
    Raw ζ-normalized proxy (no outer): J_raw = det2_P(s) / ζ(s) * B(s).
    With B(s)=s/(s-1), this cancels the simple zero of 1/ζ at s=1, making J_raw
    regular at s=1.

    WARNING: still no outer; this is a diagnostic gauge.
    """
    return det2_prime_trunc(s, primes) / s.zeta() * compensator_B(s)


def J_arith_outer_zeta(
    s: acb,
    primes: List[int],
    outer: "OuterNormalizerEngineLike",
) -> acb:
    """
    Option 2-style (computable) ζ-gauge normalizer:
      J(s) = det2_P(s) / (O(s) ζ(s)) * B(s)
    where O(s) is analytic and zero-free (outer-like).
    """
    return det2_prime_trunc(s, primes) / (outer.O(s) * s.zeta()) * compensator_B(s)


class OuterNormalizerEngineLike:
    """
    Protocol-ish helper for duck-typing the normalizer engines.
    """

    def O(self, s: acb) -> acb:  # pragma: no cover
        raise NotImplementedError


@dataclass
class OuterNormalizerEngineCanonicalStub(OuterNormalizerEngineLike):
    """
    Placeholder for the manuscript's *canonical* outer normalizer O_can on Ω={Re(s)>1/2}.

    This is intentionally NOT implemented yet: evaluating O_can(s) requires a rigorous
    reduction from boundary a.e. data on Re(s)=1/2 (with ζ-zeros) to a computable,
    certified enclosure at interior points (see `FF_CANONICAL_ARTIFACT_PLAN.md`).
    """

    def build(self) -> None:
        # No-op placeholder.
        return

    def O(self, s: acb) -> acb:
        raise NotImplementedError(
            "canonical outer normalizer is not implemented yet (see FF_CANONICAL_ARTIFACT_PLAN.md)"
        )

    def O_box(self, s_box: acb) -> acb:
        # For compatibility with theta_certify_rect which prefers O_box if present.
        raise NotImplementedError(
            "canonical outer normalizer is not implemented yet (see FF_CANONICAL_ARTIFACT_PLAN.md)"
        )


# -------------------------
# Certificate transfer function (finite Γ-certificate)
# -------------------------


def weights_geometric(n_mode: int) -> List[arb]:
    """
    w_n = (1/19) * (17/19)^(n-1), n=1..n_mode
    """
    w: List[arb] = []
    base = arb(17) / arb(19)
    scale = arb(1) / arb(19)
    pow_ = arb(1)
    for _n in range(1, n_mode + 1):
        w.append(scale * pow_)
        pow_ *= base
    return w


def weights_geometric_param(n_mode: int, r: float) -> List[arb]:
    """
    General geometric weights with Σ_{n>=1} w_n = 1/2:

      w_n = ((1-r)/2) * r^(n-1),  n>=1,  0<=r<1.

    This is useful for diagnostics when exploring certificate coupling strength.
    """
    rr = float(r)
    if not (0.0 <= rr < 1.0):
        raise ValueError("weights_geometric_param requires 0 <= r < 1")
    w: List[arb] = []
    scale = (arb(1) - arb(rr)) / arb(2)
    base = arb(rr)
    pow_ = arb(1)
    for _n in range(1, n_mode + 1):
        w.append(scale * pow_)
        pow_ *= base
    return w


def weights_pointmass(n_mode: int) -> List[arb]:
    """
    Extreme diagnostic choice: w_1=1/2, w_n=0 for n>=2 (still Σ w_n=1/2).
    """
    if n_mode <= 0:
        raise ValueError("n_mode must be positive")
    w = [arb(0) for _ in range(n_mode)]
    w[0] = arb(1) / arb(2)
    return w


@dataclass(frozen=True)
class ScaledPsiHatEngine:
    """
    Diagnostic wrapper: scale the printed ψ_cert by a constant factor c>0.

    This scales both the Fourier transform ψ̂_cert and the mass m_cert consistently:
      ψ̂_scaled = c ψ̂_cert,   m_scaled = c m_cert.

    WARNING: this does NOT match the printed manuscript unless c=1. It is used
    only to explore sensitivity of the certificate transfer to window scaling.
    """

    base: PsiHatEngine
    scale: float

    @property
    def m_cert(self) -> arb:
        return arb(self.scale) * self.base.m_cert

    def psihat_cert(self, xi_in: arb) -> arb:
        return arb(self.scale) * self.base.psihat_cert(xi_in)


def chol_lower_spd(S: arb_mat) -> arb_mat:
    """
    Basic Cholesky factorization for real symmetric SPD matrix S:
      S = L * L^T with L lower triangular.

    This is a straightforward ball-arithmetic implementation; it expects S to be
    SPD with a comfortable gap.
    """
    n = S.nrows()
    L = arb_mat(n, n)
    for i in range(n):
        for j in range(i + 1):
            s = S[i, j]
            for k in range(j):
                s -= L[i, k] * L[j, k]
            if i == j:
                L[i, j] = s.sqrt()
            else:
                L[i, j] = s / L[j, j]
    return L


def sqrtm_newton_spd(M: arb_mat, n_iter: int = 12) -> arb_mat:
    """
    Matrix square root for SPD real matrix M using Newton iteration:
      X_{k+1} = 1/2 * (X_k + M * X_k^{-1})

    Returns X with X^2 ≈ M.
    """
    n = M.nrows()
    # Start from identity (always invertible); for SPD M, Newton converges to √M.
    X = arb_mat(n, n)
    for i in range(n):
        X[i, i] = arb(1)
    half = arb(1) / arb(2)
    for _ in range(n_iter):
        Xinv = X.inv()
        X = half * (X + (M * Xinv))
    return X


def _binom_half_sequence(n_terms: int) -> List[arb]:
    """
    Return [c_0, c_1, ..., c_n] where
      sqrt(1 - x) = Σ_{k>=0} c_k x^k
    so c_0=1 and c_k = -|binom(1/2,k)| for k>=1.
    """
    coeffs: List[arb] = [arb(1)]
    binom = arb(1)  # binom(1/2,0)
    half = arb(1) / arb(2)
    for k in range(1, n_terms + 1):
        binom = binom * (half - arb(k - 1)) / arb(k)  # binom(1/2,k)
        coeffs.append(-abs(binom))
    return coeffs


def sqrt_I_minus_series_mat(G: arb_mat, q: float, n_terms: int) -> arb_mat:
    """
    Compute D ≈ (I - G)^{1/2} via the binomial series:
      (I - G)^{1/2} = Σ_{k>=0} c_k G^k,  c_0=I, c_k=-|binom(1/2,k)|.

    We return an enclosure for the true value assuming only that ‖G_true‖ ≤ q < 1.
    The remainder is bounded in operator norm by:
      tail ≤ |binom(1/2,n+1)| q^{n+1} / (1-q).
    We inflate each entry by this tail bound (conservative but safe).
    """
    if not (0.0 <= q < 1.0):
        raise ValueError("q must satisfy 0 <= q < 1")
    n = G.nrows()
    if n != G.ncols():
        raise ValueError("G must be square")

    # Identity
    D = arb_mat(n, n)
    for i in range(n):
        D[i, i] = arb(1)

    coeffs = _binom_half_sequence(n_terms)
    power = arb_mat(G)  # G^1
    for k in range(1, n_terms + 1):
        D += coeffs[k] * power
        power = power * G

    # Tail bound using q
    half = arb(1) / arb(2)
    binom = arb(1)
    for k in range(1, n_terms + 2):
        binom = binom * (half - arb(k - 1)) / arb(k)
    # now binom = binom(1/2,n_terms+1)
    qarb = arb(q)
    tail = abs(binom) * (qarb ** (n_terms + 1)) / (arb(1) - qarb)

    # Inflate each entry by ±tail
    inflate = arb(0, tail.upper())
    for i in range(n):
        for j in range(n):
            D[i, j] = D[i, j] + inflate
    return D


def sqrt_I_minus_apply_inner(M: arb_mat, S: arb_mat, q: float, n_terms: int) -> arb:
    """
    Compute the scalar
        a = < (I - T)^{1/2} g, g >
    where T is represented by the matrix M in the coordinate basis b = (g, u_1,...,u_k),
    and S is the Gram matrix of that basis:
        <B v, B w> = v^T S w.

    Here g corresponds to the coefficient vector e0.

    We evaluate the binomial series on T and bound the tail using ‖T‖ ≤ q < 1.
    """
    if not (0.0 <= q < 1.0):
        raise ValueError("q must satisfy 0 <= q < 1")
    n = M.nrows()
    if n != M.ncols() or S.nrows() != n or S.ncols() != n:
        raise ValueError("M and S must be same-size square matrices")

    coeffs = _binom_half_sequence(n_terms)

    # s0 = S e0 is the first column of S
    s0 = [S[i, 0] for i in range(n)]

    # v_k = M^k e0 (start v_0 = e0)
    v = [arb(0) for _ in range(n)]
    v[0] = arb(1)

    out = arb(0)
    for k in range(0, n_terms + 1):
        # inner_k = v^T s0
        inner = arb(0)
        for i in range(n):
            inner += v[i] * s0[i]
        out += coeffs[k] * inner

        # v <- M v
        if k != n_terms:
            v_next = [arb(0) for _ in range(n)]
            for i in range(n):
                s = arb(0)
                for j in range(n):
                    s += M[i, j] * v[j]
                v_next[i] = s
            v = v_next

    # Tail: |<T^k g,g>| ≤ ‖T‖^k ≤ q^k, so
    # |tail| ≤ Σ_{k>n_terms} |binom(1/2,k)| q^k ≤ |binom(1/2,n+1)| q^{n+1}/(1-q).
    half = arb(1) / arb(2)
    binom = arb(1)
    for k in range(1, n_terms + 2):
        binom = binom * (half - arb(k - 1)) / arb(k)
    qarb = arb(q)
    tail = abs(binom) * (qarb ** (n_terms + 1)) / (arb(1) - qarb)

    return out + arb(0, tail.upper())


def approx_lambda_max_midpoint(G: arb_mat, n_iter: int = 80) -> float:
    """
    Heuristic (non-rigorous) spectral-norm proxy: power iteration on midpoints
    for a real symmetric matrix G.
    """
    k = G.nrows()
    if k == 0:
        return 0.0
    v = [1.0 for _ in range(k)]
    for _ in range(n_iter):
        wv = [0.0 for _ in range(k)]
        for i in range(k):
            s = 0.0
            for j in range(k):
                s += float(G[i, j].mid()) * v[j]
            wv[i] = s
        norm = math.sqrt(sum(x * x for x in wv))
        if norm == 0.0:
            return 0.0
        v = [x / norm for x in wv]
    # Rayleigh quotient
    wv = [0.0 for _ in range(k)]
    for i in range(k):
        s = 0.0
        for j in range(k):
            s += float(G[i, j].mid()) * v[j]
        wv[i] = s
    num = sum(v[i] * wv[i] for i in range(k))
    den = sum(v[i] * v[i] for i in range(k))
    if den == 0.0:
        return 0.0
    return num / den


@dataclass
class Certificate:
    sigma0: float
    P: int
    n_mode: int
    psihat: Any
    weights: List[arb] | None = None
    amp_shift: float = 0.5
    coeff_mode: str = "printed"  # printed | primepower_sqrt | primepower_inv
    default_port: str = "const"  # const | eigmax (diagnostic)

    # Built data
    idx: List[Tuple[int, int]] | None = None
    G: arb_mat | None = None  # Gram matrix Γ*Γ on state space (k×k)
    D: arb_mat | None = None  # sqrt(I - Γ*Γ) on state space (k×k)
    x: List[arb] | None = None  # Γ* g (k-vector, real)
    a: arb | None = None  # <Δ g, g> (scalar, real)
    # Cached alternate port parameters (computed on demand)
    eigmax_a: arb | None = None
    eigmax_x: List[arb] | None = None

    def build(self) -> None:
        primes = [p for p in primes_upto(self.P) if p >= 2]
        w = weights_geometric(self.n_mode) if self.weights is None else self.weights
        if len(w) < self.n_mode:
            raise ValueError("weights list must have length >= n_mode")
        sigma = arb(self.sigma0)
        amp_pow = -(sigma + arb(self.amp_shift))

        idx: List[Tuple[int, int]] = []
        for p in primes:
            for n in range(1, self.n_mode + 1):
                idx.append((p, n))
        k = len(idx)

        # Precompute frequencies ξ_i = n log p (arb), and ψ̂ at needed ξ diffs.
        logs: Dict[int, arb] = {p: arb(p).log() for p in primes}
        xis: List[arb] = [arb(0) for _ in range(k)]
        for i, (p, n) in enumerate(idx):
            xis[i] = arb(n) * logs[p]

        cache: Dict[Tuple[float, float], arb] = {}

        def get_psihat(xi: arb) -> arb:
            xi_abs = abs(xi)
            key = (round(float(xi_abs), 14), round(float(xi_abs.rad()), 24))
            if key not in cache:
                cache[key] = self.psihat.psihat_cert(xi_abs)
            return cache[key]

        # Build Gram matrix G = Γ*Γ and x = Γ* g.
        G = arb_mat(k, k)
        x_vec: List[arb] = [arb(0) for _ in range(k)]
        m_cert = self.psihat.m_cert
        sqrt_m = m_cert.sqrt()

        if self.coeff_mode not in {"printed", "primepower_sqrt", "primepower_inv"}:
            raise ValueError(f"unknown coeff_mode: {self.coeff_mode}")

        for i, (p, n) in enumerate(idx):
            wn = w[n - 1]
            if self.coeff_mode == "printed":
                coeff = wn * (arb(p) ** amp_pow)  # p^{-(σ0+amp_shift)}
            else:
                # Diagnostic model: attach a prime-power attenuation consistent with the
                # det2 prime-power frequencies e^{-it n log p} and decay p^{-nσ}.
                # Also include a mild 1/sqrt(n) or 1/n factor to mimic the 1/k weights
                # in log det2 = -Σ_p Σ_{k>=2} p^{-ks}/k, while keeping ℓ^2 control.
                pn_pow = arb(p) ** (amp_pow * arb(n))  # p^{-n(σ0+amp_shift)}
                if self.coeff_mode == "primepower_sqrt":
                    coeff = wn * pn_pow / arb(n).sqrt()
                else:
                    coeff = wn * pn_pow / arb(n)

            x_vec[i] = coeff * get_psihat(xis[i]) / sqrt_m
            for j, (q, m) in enumerate(idx):
                wm = w[m - 1]
                if self.coeff_mode == "printed":
                    coeff_j = wm * (arb(q) ** amp_pow)
                else:
                    qm_pow = arb(q) ** (amp_pow * arb(m))
                    if self.coeff_mode == "primepower_sqrt":
                        coeff_j = wm * qm_pow / arb(m).sqrt()
                    else:
                        coeff_j = wm * qm_pow / arb(m)
                G[i, j] = coeff * coeff_j * get_psihat(xis[i] - xis[j])

        # Build D = (I - Γ*Γ)^{1/2} using a binomial power series.
        # Default (printed) mode: rely on the audited spectral gap δ ≥ 0.72
        # (Proposition prop:delta-cert-06), which implies ‖Γ*Γ‖ ≤ 1-δ ≤ 0.28.
        #
        # Diagnostic modes (changed weights / scaled ψ / changed amp shift):
        # fall back to a heuristic midpoint power-iteration estimate for ‖G‖.
        is_printed = (
            self.weights is None
            and float(self.amp_shift) == 0.5
            and isinstance(self.psihat, PsiHatEngine)
        )
        if is_printed:
            delta_gap = 0.72
            q = 1.0 - delta_gap
        else:
            lam = approx_lambda_max_midpoint(G, n_iter=120)
            q = min(0.999, max(0.0, float(lam)) + 1e-6)
            if q >= 1.0:
                raise ValueError(
                    f"diagnostic certificate produced ‖G‖≈{lam:.6g} (not a contraction); "
                    "adjust --cert-psi-scale/--cert-weights-mode/--cert-amp-shift"
                )

        D = sqrt_I_minus_series_mat(G, q=q, n_terms=30)

        # Compute a = <Δ g, g> with Δ=(I-ΓΓ*)^{1/2} via the same series, without
        # explicit Cholesky/sqrtm: we only need the scalar <sqrt(I-T) g,g> where
        # T=ΓΓ*.
        # Work in the coordinate basis b0=g, b_i=u_i=Γ e_i with Gram matrix S.
        S = arb_mat(k + 1, k + 1)
        S[0, 0] = arb(1)
        for i in range(k):
            S[0, i + 1] = x_vec[i]
            S[i + 1, 0] = x_vec[i]
        for i in range(k):
            for j in range(k):
                S[i + 1, j + 1] = G[i, j]

        # Operator matrix M for T=ΓΓ* in basis b (acts on coefficient vectors).
        M = arb_mat(k + 1, k + 1)
        for i in range(k):
            M[i + 1, 0] = x_vec[i]
        for i in range(k):
            for j in range(k):
                M[i + 1, j + 1] = G[i, j]

        a = sqrt_I_minus_apply_inner(M, S, q=q, n_terms=40)

        self.idx = idx
        self.G = G
        self.D = D
        self.x = x_vec
        self.a = a
        # Reset cached alternate ports (depend on G/D).
        self.eigmax_a = None
        self.eigmax_x = None

    def _theta(self, z: acb) -> acb:
        assert self.D is not None and self.x is not None and self.a is not None
        k = self.D.nrows()
        # Build complex matrix M = I - z D.
        M = acb_mat(k, k)
        for i in range(k):
            for j in range(k):
                M[i, j] = -z * acb(self.D[i, j])
            M[i, i] += acb(1)
        R = M.inv()
        # y = R * x
        y = [acb(0) for _ in range(k)]
        for i in range(k):
            s = acb(0)
            for j in range(k):
                s += R[i, j] * acb(self.x[j])
            y[i] = s
        # x^T y (x is real)
        xty = acb(0)
        for i in range(k):
            xty += acb(self.x[i]) * y[i]
        return acb(self.a) - z * xty

    def _J_cert_const(self, s: acb) -> acb:
        """
        Return J_cert,N(s) (scalar Herglotz function) using the manuscript port g_cert
        (the built-in constant-function scalarization):
          2J = (1+θ)/(1-θ) where θ(z)=<Θ(z)g,g>, z=z_{σ0}(s).
        """
        sigma0 = arb(self.sigma0)
        z = (s - acb(sigma0)) / (s + acb(sigma0))
        th = self._theta(z)
        twoJ = (acb(1) + th) / (acb(1) - th)
        return twoJ / acb(2)

    def J_cert(self, s: acb) -> acb:
        """
        Return J_cert,N(s) (scalar Herglotz function) for the selected certificate port.

        By default this is the manuscript port ("const"), but diagnostics can switch to
        alternative scalarizations (e.g. "eigmax") via `default_port`.
        """
        if self.default_port == "const":
            return self._J_cert_const(s)
        return self.J_cert_port(s, port=self.default_port)

    # -------------------------
    # Alternate scalar ports (diagnostic)
    # -------------------------

    def _theta_from_ax(self, z: acb, a: arb, x: List[arb]) -> acb:
        """
        Evaluate θ(z) = a - z * <(I - z D)^{-1} x, x>, where D is the built defect
        matrix on the state space, a is a real scalar, and x is a real state vector.
        """
        assert self.D is not None
        k = self.D.nrows()
        if len(x) != k:
            raise ValueError("x has wrong length for certificate state space")

        M = acb_mat(k, k)
        for i in range(k):
            for j in range(k):
                M[i, j] = -z * acb(self.D[i, j])
            M[i, i] += acb(1)
        R = M.inv()

        y = [acb(0) for _ in range(k)]
        for i in range(k):
            s = acb(0)
            for j in range(k):
                s += R[i, j] * acb(x[j])
            y[i] = s

        xty = acb(0)
        for i in range(k):
            xty += acb(x[i]) * y[i]
        return acb(a) - z * xty

    def _power_iteration_eigmax(self, G_mid: List[List[float]], n_iter: int = 64) -> List[float]:
        """
        Very simple power iteration on a real symmetric matrix (midpoint approximation)
        to obtain a dominant eigenvector direction. Used only for diagnostics.
        """
        n = len(G_mid)
        v = [1.0 for _ in range(n)]
        for _ in range(n_iter):
            w = [0.0 for _ in range(n)]
            for i in range(n):
                s = 0.0
                row = G_mid[i]
                for j in range(n):
                    s += row[j] * v[j]
                w[i] = s
            # normalize
            norm = math.sqrt(sum(x * x for x in w))
            if norm == 0.0:
                break
            v = [x / norm for x in w]
        return v

    def J_cert_port(self, s: acb, *, port: str = "const") -> acb:
        """
        Compute a scalar certificate transfer using an alternate port (scalarization).

        - port="const": the manuscript port g_cert (constant function), i.e. J_cert(s).
        - port="eigmax": port along a state vector aligned with the dominant eigendirection
          of G=Γ*Γ (so θ(0)≈sqrt(1-λ_max), producing a nontrivial scale).

        NOTE: this is used by the main verifier if `default_port` is set to a non-const
        value (e.g. via a CLI flag).
        """
        if port == "const":
            return self._J_cert_const(s)

        assert self.G is not None and self.D is not None
        k = self.D.nrows()
        if self.G.nrows() != k or self.G.ncols() != k:
            raise ValueError("certificate internal matrices have inconsistent sizes")

        if port != "eigmax":
            raise ValueError(f"unknown certificate port: {port}")

        # Cache the eigmax port parameters (a,x) so rectangle covers don't redo power iteration.
        if self.eigmax_a is None or self.eigmax_x is None:
            # Build a midpoint float matrix for power iteration.
            G_mid: List[List[float]] = [[0.0 for _ in range(k)] for _ in range(k)]
            for i in range(k):
                for j in range(k):
                    G_mid[i][j] = float(self.G[i, j].mid())

            v = self._power_iteration_eigmax(G_mid, n_iter=64)

            # Convert v into an arb vector c, and compute:
            #   denom = c^T G c = ||Γ c||^2
            #   x = (G c)/sqrt(denom) = Γ^* g where g = Γ c / ||Γ c||
            #   a = <Δ g,g> = (c^T D G c)/denom   (using ΔΓ = ΓD and g in Ran Γ)
            c = [arb(vi) for vi in v]

            # Gc
            Gc: List[arb] = [arb(0) for _ in range(k)]
            for i in range(k):
                s_i = arb(0)
                for j in range(k):
                    s_i += self.G[i, j] * c[j]
                Gc[i] = s_i

            denom = arb(0)
            for i in range(k):
                denom += c[i] * Gc[i]
            if denom.contains(0):
                raise ValueError("degenerate eigmax port: denom encloses 0")
            inv_sqrt_denom = (denom.sqrt()) ** (-1)

            x_vec = [Gc[i] * inv_sqrt_denom for i in range(k)]

            # DGc
            DGc: List[arb] = [arb(0) for _ in range(k)]
            for i in range(k):
                s_i = arb(0)
                for j in range(k):
                    s_i += self.D[i, j] * Gc[j]
                DGc[i] = s_i

            num = arb(0)
            for i in range(k):
                num += c[i] * DGc[i]
            a = num / denom

            self.eigmax_a = a
            self.eigmax_x = x_vec

        assert self.eigmax_a is not None and self.eigmax_x is not None
        a = self.eigmax_a
        x_vec = self.eigmax_x

        sigma0 = arb(self.sigma0)
        z = (s - acb(sigma0)) / (s + acb(sigma0))
        th = self._theta_from_ax(z, a, x_vec)
        twoJ = (acb(1) + th) / (acb(1) - th)
        return twoJ / acb(2)


# -------------------------
# Rectangle cover + bounds
# -------------------------


@dataclass(frozen=True)
class Rect:
    sigma_min: float
    sigma_max: float
    t_min: float
    t_max: float


@dataclass
class OuterNormalizerEngine(OuterNormalizerEngineLike):
    """
    Computable normalizer (Option 2 scaffolding):

    Build a *rigorous* enclosure for an analytic, zero-free normalizer
      O(s) = exp( logO(s) )
    where logO is defined via a Cauchy-type integral on a fixed boundary line
      Re(s) = sigma_ref > 1/2.

    This is intended to replace the manuscript's global outer-normalizer-with-limit
    by a *computable* far-field normalizer (still analytic and nonvanishing).

    WARNING: This is computationally expensive; it is a closure module.
    """

    sigma_ref: float
    T: float
    n_intervals: int
    P_cut: int
    k_max: int = 40

    # cached boundary intervals (t mid, t rad) and u(t) enclosures
    t_intervals: List[arb] | None = None
    u_intervals: List[arb] | None = None

    def build(self) -> None:
        if self.sigma_ref <= 0.5:
            raise ValueError("sigma_ref must be > 1/2")
        if self.T <= 0:
            raise ValueError("T must be positive")
        if self.n_intervals <= 0:
            raise ValueError("n_intervals must be positive")

        h = (2.0 * self.T) / self.n_intervals
        t_intervals: List[arb] = []
        u_intervals: List[arb] = []
        sig = arb(self.sigma_ref)
        E = det2_logabs_bound_sigma(self.sigma_ref, P_cut=self.P_cut)
        logabs_det2 = arb(0, E)  # log|det2| ∈ [-E, +E]

        for j in range(self.n_intervals):
            a = -self.T + j * h
            b = -self.T + (j + 1) * h
            mid = 0.5 * (a + b)
            rad = 0.5 * (b - a)
            tI = arb(mid, rad)
            t_intervals.append(tI)

            # Boundary point for parameter t: s_b = sigma_ref - i t
            sb = acb(sig, -tI)
            xi = xi_completed(sb)
            logabs_xi = abs(xi).log()
            # u(t) = log|det2/xi| ∈ log|det2| - log|xi|
            u = logabs_det2 - logabs_xi
            u_intervals.append(u)

        self.t_intervals = t_intervals
        self.u_intervals = u_intervals

    def logO(self, s: acb) -> acb:
        """
        logO(s) := (1/(π i)) ∫_{-T}^{T} ( 1/(t - w) - t/(1+t^2) ) u(t) dt
        where w = i(s - sigma_ref).

        We evaluate the integral rigorously by interval enclosure on each subinterval.
        """
        assert self.t_intervals is not None and self.u_intervals is not None
        sig = arb(self.sigma_ref)
        w = acb(0, 1) * (s - acb(sig))  # w = i(s - sigma_ref)

        h = arb((2.0 * self.T) / self.n_intervals)
        total = acb(0)
        for tI, uI in zip(self.t_intervals, self.u_intervals):
            tC = acb(tI)  # real interval embedded as complex
            kernel = (acb(1) / (tC - w)) - (tC / (acb(1) + tC * tC))
            total += kernel * acb(uI) * acb(h)

        pi = acb.pi()
        return (-acb(0, 1) / pi) * total  # 1/(π i) = -i/π

    def O(self, s: acb) -> acb:
        return self.logO(s).exp()


@dataclass
class OuterNormalizerEngineZetaRigorous(OuterNormalizerEngineLike):
    """
    Computable far-field normalizer in a ζ-gauge (Option B).

    This engine builds a *rigorous* enclosure for an analytic, zero-free normalizer
      O_ff(s) = exp(logO_ff(s))
    on the far half-plane Re(s) > sigma_ref > 1/2, where logO_ff is defined via a
    truncated Cauchy-type integral on the boundary line Re(s)=sigma_ref.

    Boundary data:
      u(t) := log|det2(I-A)(sigma_ref - i t)| - log|ζ(sigma_ref - i t)| + log|B(sigma_ref - i t)|
    with B(s)=s/(s-1).

    Notes:
    - det2 is evaluated with a certified enclosure (prime head + explicit tail).
    - ζ is evaluated with Arb ball arithmetic on each t-interval; if the enclosure
      contains 0, log|ζ| cannot be certified on that interval (refine outer-n / raise prec).
    - This normalizer is intended for *far-field certified numerics* and avoids
      Γ-factor instability inherent in ξ at large |t|.
    """

    sigma_ref: float
    T: float
    n_intervals: int
    P_cut: int
    k_max: int = 40
    progress: bool = False
    progress_every: int = 20
    max_depth: int = 12
    min_width: float = 1e-3

    t_intervals: List[arb] | None = None
    u_intervals: List[arb] | None = None
    dt_intervals: List[arb] | None = None

    def build(self) -> None:
        if self.sigma_ref <= 0.5:
            raise ValueError("sigma_ref must be > 1/2")
        if self.T <= 0:
            raise ValueError("T must be positive")
        if self.n_intervals <= 0:
            raise ValueError("n_intervals must be positive")

        if self.max_depth < 0:
            raise ValueError("max_depth must be nonnegative")
        if self.min_width <= 0:
            raise ValueError("min_width must be positive")

        # Adaptive refinement: start from a uniform partition and bisect any
        # subinterval on which Arb's enclosure for |ζ(s)| contains 0.
        #
        # Performance: computing log|det2| is far more expensive than evaluating ζ.
        # We compute log|det2| once on the initial coarse interval and reuse that
        # enclosure on all bisection children.
        h0 = (2.0 * self.T) / self.n_intervals
        work: List[Tuple[float, float, int, arb | None]] = []
        for j in range(self.n_intervals):
            a = -self.T + j * h0
            b = -self.T + (j + 1) * h0
            work.append((a, b, 0, None))

        sig = arb(self.sigma_ref)
        t0 = time.time()
        t_intervals: List[arb] = []
        u_intervals: List[arb] = []
        dt_intervals: List[arb] = []

        leaves = 0
        while work:
            a, b, depth, logabs_det2_cached = work.pop()
            width = b - a
            if width <= 0:
                continue
            mid = 0.5 * (a + b)
            rad = 0.5 * width
            tI = arb(mid, rad)
            sb = acb(sig, -tI)  # boundary point: sigma_ref - i t

            if logabs_det2_cached is None:
                logdet2 = logdet2_full_enclosure(sb, P_cut=self.P_cut, k_max=self.k_max)
                logabs_det2_cached = logdet2.real

            # For the manuscript arithmetic ratio
            #   F(s) = det2(I-A(s)) / ζ(s) * B(s),   B(s)=s/(s-1),
            # the canonical outer uses boundary modulus |F| on Re(s)=1/2.
            #
            # Here we build a *computable* outer on the boundary line Re(s)=sigma_ref>1/2
            # using the modulus data:
            #   u(t) := log|F(sigma_ref - i t)|
            #        = log|det2| - log|ζ| + log|B|.
            #
            # NOTE: An earlier version used log|det2/(ζ B)| which is *not* the same off
            # the critical line (since |B|≠1). The choice here matches the manuscript's
            # F-ratio (and the midpoint zeta outer implementation below).
            zeta_val = sb.zeta()
            abs_zeta = abs(zeta_val)
            if abs_zeta.contains(0):
                if depth < self.max_depth and width > self.min_width:
                    m = mid
                    work.append((m, b, depth + 1, logabs_det2_cached))
                    work.append((a, m, depth + 1, logabs_det2_cached))
                    continue
                raise ValueError(
                    "zeta enclosure contains 0 on boundary even after refinement: "
                    f"sigma_ref={self.sigma_ref}, t∈[{a:.17g},{b:.17g}], "
                    f"width={width:.3g}, depth={depth}. "
                    "Increase --prec, increase --outer-n, increase --outer-max-depth, "
                    "or decrease --outer-min-width; if persistent, consider shifting sigma_ref."
                )

            logabs_zeta = abs_zeta.log()
            logabs_B = abs(compensator_B(sb)).log()
            t_intervals.append(tI)
            u_intervals.append(logabs_det2_cached - logabs_zeta + logabs_B)
            dt_intervals.append(arb(width))
            leaves += 1

            if self.progress and self.progress_every > 0 and (leaves % self.progress_every == 0):
                dt = time.time() - t0
                _progress(
                    f"[outer_zeta] leaves {leaves} (stack {len(work)}, elapsed {dt:.1f}s)",
                    enabled=True,
                )

        self.t_intervals = t_intervals
        self.u_intervals = u_intervals
        self.dt_intervals = dt_intervals

    def logO(self, s: acb) -> acb:
        assert (
            self.t_intervals is not None
            and self.u_intervals is not None
            and self.dt_intervals is not None
        )
        sig = arb(self.sigma_ref)
        w = acb(0, 1) * (s - acb(sig))  # w = i(s - sigma_ref)

        total = acb(0)
        for tI, uI, dtI in zip(self.t_intervals, self.u_intervals, self.dt_intervals):
            tC = acb(tI)
            kernel = (acb(1) / (tC - w)) - (tC / (acb(1) + tC * tC))
            total += kernel * acb(uI) * acb(dtI)

        pi = acb.pi()
        return (-acb(0, 1) / pi) * total

    def O(self, s: acb) -> acb:
        return self.logO(s).exp()

    def logO_midpoint(self, s: acb) -> acb:
        """
        Midpoint-evaluated (non-rigorous) variant of logO(s).

        This is intended for diagnostics when `eval_mode="midpoint"`: it avoids
        catastrophic enclosure blow-up (and exp(...) enclosing 0) that can occur
        when evaluating the Cauchy integral on wide balls.
        """
        assert (
            self.t_intervals is not None
            and self.u_intervals is not None
            and self.dt_intervals is not None
        )
        s0 = acb_midpoint(s)
        sig = arb(self.sigma_ref)
        w = acb(0, 1) * (s0 - acb(sig))

        total = acb(0)
        for tI, uI, dtI in zip(self.t_intervals, self.u_intervals, self.dt_intervals):
            t_mid = arb(tI.mid())
            u_mid = arb(uI.mid())
            dt_mid = arb(dtI.mid())
            tC = acb(t_mid)
            kernel = (acb(1) / (tC - w)) - (tC / (acb(1) + tC * tC))
            total += kernel * acb(u_mid) * acb(dt_mid)

        pi = acb.pi()
        return (-acb(0, 1) / pi) * total

    def O_midpoint(self, s: acb) -> acb:
        return self.logO_midpoint(s).exp()

    # -------------------------
    # Cache helpers (outer-build-only mode)
    # -------------------------

    def _cache_params(self) -> Dict[str, Any]:
        return {
            "sigma_ref": float(self.sigma_ref),
            "T": float(self.T),
            "n_intervals": int(self.n_intervals),
            "P_cut": int(self.P_cut),
            "k_max": int(self.k_max),
            "max_depth": int(self.max_depth),
            "min_width": float(self.min_width),
        }

    def save_cache(self, path: str) -> None:
        if self.t_intervals is None or self.u_intervals is None or self.dt_intervals is None:
            raise ValueError("outer cache save requested before build()")
        data = {
            "kind": "OuterNormalizerEngineZetaRigorous",
            "version": 1,
            "params": self._cache_params(),
            "n_leaves": int(len(self.t_intervals)),
            "t_intervals": [str(x) for x in self.t_intervals],
            "u_intervals": [str(x) for x in self.u_intervals],
            "dt_intervals": [str(x) for x in self.dt_intervals],
        }
        _write_json(path, data)

    def load_cache(self, path: str, *, strict: bool = True) -> None:
        data = _read_json(path)
        if not isinstance(data, dict):
            raise ValueError("outer cache is not a JSON object")
        if data.get("kind") != "OuterNormalizerEngineZetaRigorous":
            raise ValueError(f"unexpected outer cache kind: {data.get('kind')!r}")
        if int(data.get("version", -1)) != 1:
            raise ValueError(f"unsupported outer cache version: {data.get('version')!r}")

        params = data.get("params", {})
        want = self._cache_params()
        mismatches: List[str] = []
        for k, v_want in want.items():
            v = params.get(k)
            if v is None:
                mismatches.append(f"{k}: missing")
                continue
            if isinstance(v_want, float):
                if not math.isclose(float(v), float(v_want), rel_tol=0.0, abs_tol=1e-15):
                    mismatches.append(f"{k}: cache={v} want={v_want}")
            else:
                if int(v) != int(v_want):
                    mismatches.append(f"{k}: cache={v} want={v_want}")
        if mismatches and strict:
            raise ValueError("outer cache parameter mismatch:\n  " + "\n  ".join(mismatches))
        if mismatches and not strict:
            _progress(
                "[outer_cache] WARNING: parameter mismatch; using cache anyway:\n  "
                + "\n  ".join(mismatches),
                enabled=True,
            )

        try:
            t_list = [arb(s) for s in data["t_intervals"]]
            u_list = [arb(s) for s in data["u_intervals"]]
            dt_list = [arb(s) for s in data["dt_intervals"]]
        except Exception as e:
            raise ValueError(f"failed to parse outer cache arb data: {e}") from e
        if not (len(t_list) == len(u_list) == len(dt_list)):
            raise ValueError("outer cache lists have inconsistent lengths")
        self.t_intervals = t_list
        self.u_intervals = u_list
        self.dt_intervals = dt_list


@dataclass
class OuterNormalizerEngineZetaProjectRigorous(OuterNormalizerEngineLike):
    """
    Rigorous (interval) version of the phase-sensitive ζ-gauge projection normalizer.

    Boundary data on Re(s)=sigma_ref:
      f(t) := det2(I-A)(sigma_ref - i t) / ζ(sigma_ref - i t) * B(sigma_ref - i t),
      B(s)=s/(s-1).

    Normalizer (truncated):
      O_proj(s) := (1/(π i)) ∫_{-T}^{T} ( 1/(t - w) - t/(1+t^2) ) f(t) dt,
      w := i(s - sigma_ref).

    Notes:
    - Unlike an exp(…) outer, O_proj is NOT automatically zero-free; we will certify
      nonvanishing on the specific rectangle cover by checking |O_proj| does not
      enclose 0 on each box.
    - Division by ζ on the boundary requires that ζ does not enclose 0 on each
      boundary interval; we enforce this by adaptive bisection (like the ζ-gauge outer).
    """

    sigma_ref: float
    T: float
    n_intervals: int
    P_cut: int
    det2_mode: str = "enclosure"  # enclosure | trunc
    k_max: int = 40
    progress: bool = False
    progress_every: int = 20
    max_depth: int = 12
    min_width: float = 1e-3
    # Optional: refine until f(t) enclosures are reasonably tight (helps avoid
    # catastrophic overestimation when integrating oscillatory complex boundary data).
    f_rel_tol: float = 0.0
    f_abs_tol: float = 0.0
    det2_cache_children: bool = True

    t_intervals: List[arb] | None = None
    f_intervals: List[acb] | None = None
    dt_intervals: List[arb] | None = None

    def _cache_params(self) -> Dict[str, Any]:
        return {
            "sigma_ref": float(self.sigma_ref),
            "T": float(self.T),
            "n_intervals": int(self.n_intervals),
            "P_cut": int(self.P_cut),
            "det2_mode": str(self.det2_mode),
            "k_max": int(self.k_max),
            "max_depth": int(self.max_depth),
            "min_width": float(self.min_width),
            "f_rel_tol": float(self.f_rel_tol),
            "f_abs_tol": float(self.f_abs_tol),
            "det2_cache_children": bool(self.det2_cache_children),
        }

    def save_cache(self, path: str) -> None:
        if self.t_intervals is None or self.f_intervals is None or self.dt_intervals is None:
            raise ValueError("outer cache save requested before build()")
        data = {
            "kind": "OuterNormalizerEngineZetaProjectRigorous",
            "version": 1,
            "params": self._cache_params(),
            "n_leaves": int(len(self.t_intervals)),
            "t_intervals": [str(x) for x in self.t_intervals],
            "f_intervals_re": [str(z.real) for z in self.f_intervals],
            "f_intervals_im": [str(z.imag) for z in self.f_intervals],
            "dt_intervals": [str(x) for x in self.dt_intervals],
        }
        _write_json(path, data)

    def load_cache(self, path: str, *, strict: bool = True) -> None:
        data = _read_json(path)
        if not isinstance(data, dict):
            raise ValueError("outer cache is not a JSON object")
        if data.get("kind") != "OuterNormalizerEngineZetaProjectRigorous":
            raise ValueError(f"unexpected outer cache kind: {data.get('kind')!r}")
        if int(data.get("version", -1)) != 1:
            raise ValueError(f"unsupported outer cache version: {data.get('version')!r}")

        params = data.get("params", {})
        want = self._cache_params()
        mismatches: List[str] = []
        for k, v_want in want.items():
            v = params.get(k)
            if v is None:
                mismatches.append(f"{k}: missing")
                continue
            if isinstance(v_want, float):
                if not math.isclose(float(v), float(v_want), rel_tol=0.0, abs_tol=1e-15):
                    mismatches.append(f"{k}: cache={v} want={v_want}")
            elif isinstance(v_want, bool):
                if bool(v) != bool(v_want):
                    mismatches.append(f"{k}: cache={v} want={v_want}")
            elif isinstance(v_want, int):
                if int(v) != int(v_want):
                    mismatches.append(f"{k}: cache={v} want={v_want}")
            else:
                if str(v) != str(v_want):
                    mismatches.append(f"{k}: cache={v} want={v_want}")
        if mismatches and strict:
            raise ValueError("outer cache parameter mismatch:\n  " + "\n  ".join(mismatches))
        if mismatches and not strict:
            _progress(
                "[outer_cache] WARNING: parameter mismatch; using cache anyway:\n  "
                + "\n  ".join(mismatches),
                enabled=True,
            )

        try:
            t_list = [arb(s) for s in data["t_intervals"]]
            re_list = [arb(s) for s in data["f_intervals_re"]]
            im_list = [arb(s) for s in data["f_intervals_im"]]
            dt_list = [arb(s) for s in data["dt_intervals"]]
        except Exception as e:
            raise ValueError(f"failed to parse outer cache arb data: {e}") from e
        if not (len(t_list) == len(re_list) == len(im_list) == len(dt_list)):
            raise ValueError("outer cache lists have inconsistent lengths")
        self.t_intervals = t_list
        self.f_intervals = [acb(re, im) for re, im in zip(re_list, im_list)]
        self.dt_intervals = dt_list

    def build(self) -> None:
        if self.sigma_ref <= 0.5:
            raise ValueError("sigma_ref must be > 1/2")
        if self.T <= 0:
            raise ValueError("T must be positive")
        if self.n_intervals <= 0:
            raise ValueError("n_intervals must be positive")
        if self.max_depth < 0:
            raise ValueError("max_depth must be nonnegative")
        if self.min_width <= 0:
            raise ValueError("min_width must be positive")
        if self.det2_mode not in {"enclosure", "trunc"}:
            raise ValueError("det2_mode must be 'enclosure' or 'trunc'")

        h0 = (2.0 * self.T) / self.n_intervals
        # (a,b,depth, det2_cached)
        work: List[Tuple[float, float, int, acb | None]] = []
        for j in range(self.n_intervals):
            a = -self.T + j * h0
            b = -self.T + (j + 1) * h0
            work.append((a, b, 0, None))

        sig = arb(self.sigma_ref)
        primes_big = [p for p in cached_primes_upto(self.P_cut) if p >= 2] if self.det2_mode == "trunc" else []
        t0 = time.time()
        t_intervals: List[arb] = []
        f_intervals: List[acb] = []
        dt_intervals: List[arb] = []

        leaves = 0
        while work:
            a, b, depth, det2_cached = work.pop()
            width = b - a
            if width <= 0:
                continue
            mid = 0.5 * (a + b)
            rad = 0.5 * width
            tI = arb(mid, rad)
            sb = acb(sig, -tI)

            if det2_cached is None:
                if self.det2_mode == "enclosure":
                    det2_cached = det2_full_enclosure(sb, P_cut=self.P_cut, k_max=self.k_max)
                else:
                    det2_cached = det2_prime_trunc(sb, primes_big)

            zeta_val = sb.zeta()
            if abs(zeta_val).contains(0):
                if depth < self.max_depth and width > self.min_width:
                    m = mid
                    det2_child = det2_cached if self.det2_cache_children else None
                    work.append((m, b, depth + 1, det2_child))
                    work.append((a, m, depth + 1, det2_child))
                    continue
                raise ValueError(
                    "zeta enclosure contains 0 on boundary even after refinement: "
                    f"sigma_ref={self.sigma_ref}, t∈[{a:.17g},{b:.17g}], "
                    f"width={width:.3g}, depth={depth}. "
                    "Increase --prec, increase --outer-n, increase --outer-max-depth, "
                    "or decrease --outer-min-width; if persistent, consider shifting sigma_ref."
                )

            fI = det2_cached / zeta_val * compensator_B(sb)
            # Optional tightening heuristic: if fI is too wide relative to its midpoint,
            # bisect further to avoid O(s) enclosures swallowing 0 during integration.
            if (self.f_rel_tol > 0.0 or self.f_abs_tol > 0.0) and (depth < self.max_depth) and (
                width > self.min_width
            ):
                try:
                    rad = max(float(fI.real.rad()), float(fI.imag.rad()))
                    f_mid = acb_midpoint(fI)
                    mag = float(abs(f_mid))
                    thresh = float(self.f_abs_tol) + float(self.f_rel_tol) * max(1.0, mag)
                    if rad > thresh:
                        m = mid
                        det2_child = det2_cached if self.det2_cache_children else None
                        work.append((m, b, depth + 1, det2_child))
                        work.append((a, m, depth + 1, det2_child))
                        continue
                except Exception:
                    # If anything goes wrong extracting radii, fall back to accepting fI.
                    pass

            t_intervals.append(tI)
            f_intervals.append(fI)
            dt_intervals.append(arb(width))
            leaves += 1

            if self.progress and self.progress_every > 0 and (leaves % self.progress_every == 0):
                dt = time.time() - t0
                _progress(
                    f"[outer_zeta_proj] leaves {leaves} (stack {len(work)}, elapsed {dt:.1f}s)",
                    enabled=True,
                )

        self.t_intervals = t_intervals
        self.f_intervals = f_intervals
        self.dt_intervals = dt_intervals

    def O(self, s: acb) -> acb:
        assert (
            self.t_intervals is not None
            and self.f_intervals is not None
            and self.dt_intervals is not None
        )
        sig = arb(self.sigma_ref)
        w = acb(0, 1) * (s - acb(sig))
        total = acb(0)
        for tI, fI, dtI in zip(self.t_intervals, self.f_intervals, self.dt_intervals):
            tC = acb(tI)
            kernel = (acb(1) / (tC - w)) - (tC / (acb(1) + tC * tC))
            total += kernel * fI * acb(dtI)
        pi = acb.pi()
        return (-acb(0, 1) / pi) * total

    def O_midpoint(self, s: acb) -> acb:
        # Midpoint-evaluated diagnostic variant.
        assert (
            self.t_intervals is not None
            and self.f_intervals is not None
            and self.dt_intervals is not None
        )
        s0 = acb_midpoint(s)
        sig = arb(self.sigma_ref)
        w = acb(0, 1) * (s0 - acb(sig))
        total = acb(0)
        for tI, fI, dtI in zip(self.t_intervals, self.f_intervals, self.dt_intervals):
            t_mid = arb(tI.mid())
            f_mid = acb(arb(fI.real.mid()), arb(fI.imag.mid()))
            dt_mid = arb(dtI.mid())
            tC = acb(t_mid)
            kernel = (acb(1) / (tC - w)) - (tC / (acb(1) + tC * tC))
            total += kernel * f_mid * acb(dt_mid)
        pi = acb.pi()
        return (-acb(0, 1) / pi) * total


@dataclass
class OuterNormalizerEngineMidpoint(OuterNormalizerEngineLike):
    """
    Non-rigorous-but-ball-arithmetic exploratory normalizer.

    Uses midpoint sampling (t nodes) for u(t)=log|det2/xi| and trapezoidal/quadrature,
    avoiding catastrophic overestimation from evaluating special functions on wide t-balls.

    This is meant as a *directional* check: if this does not reduce the attachment error,
    then the fully certified interval-integration implementation is unlikely to succeed.
    """

    sigma_ref: float
    T: float
    n_nodes: int
    P_cut: int

    t_nodes: List[arb] | None = None
    u_nodes: List[arb] | None = None

    def build(self) -> None:
        if self.sigma_ref <= 0.5:
            raise ValueError("sigma_ref must be > 1/2")
        if self.T <= 0:
            raise ValueError("T must be positive")
        if self.n_nodes <= 1:
            raise ValueError("n_nodes must be >= 2")

        # Use equally spaced nodes including endpoints.
        h = (2.0 * self.T) / (self.n_nodes - 1)
        sig = arb(self.sigma_ref)
        primes_big = [p for p in cached_primes_upto(self.P_cut) if p >= 2]

        t_nodes: List[arb] = []
        u_nodes: List[arb] = []
        for j in range(self.n_nodes):
            t = -self.T + j * h
            tA = arb(t)
            t_nodes.append(tA)

            sb = acb(sig, arb(-t))
            # det2 truncated at P_cut (midpoint sampling)
            logdet = acb(0)
            for p in primes_big:
                ps = acb(p) ** (-sb)
                logdet += (acb(1) - ps).log() + ps
            det2_tr = logdet.exp()
            F = det2_tr / xi_completed(sb)
            u_nodes.append(abs(F).log())

        self.t_nodes = t_nodes
        self.u_nodes = u_nodes

    def logO(self, s: acb) -> acb:
        assert self.t_nodes is not None and self.u_nodes is not None
        # IMPORTANT: midpoint mode is *exploratory*; evaluate at the midpoint of
        # the input box to avoid catastrophic enclosure blow-up.
        s0 = acb_midpoint(s)
        sig = arb(self.sigma_ref)
        w = acb(0, 1) * (s0 - acb(sig))
        pi = acb.pi()

        h = arb((2.0 * self.T) / (self.n_nodes - 1))
        total = acb(0)
        for j, (tA, uA) in enumerate(zip(self.t_nodes, self.u_nodes)):
            tC = acb(tA)
            kernel = (acb(1) / (tC - w)) - (tC / (acb(1) + tC * tC))
            weight = h
            if j == 0 or j == len(self.t_nodes) - 1:
                weight = weight / acb(2)  # trapezoid endpoint half-weight
            total += kernel * acb(uA) * acb(weight)

        return (-acb(0, 1) / pi) * total

    def O(self, s: acb) -> acb:
        # midpoint mode is exploratory; keep evaluation at the midpoint.
        return self.logO(s).exp()


@dataclass
class OuterNormalizerEngineMidpointZeta(OuterNormalizerEngineLike):
    """
    Exploratory normalizer using ζ (no Γ-factor), meant to be more stable at large |t|.

    Builds u(t) = log| det2_{P_cut}(sigma_ref - i t) / ζ(sigma_ref - i t) * B(s) |
    on equally spaced nodes and uses the same Cauchy-type integral for logO(s).
    """

    sigma_ref: float
    T: float
    n_nodes: int
    P_cut: int

    t_nodes: List[arb] | None = None
    u_nodes: List[arb] | None = None

    def build(self) -> None:
        if self.sigma_ref <= 0.5:
            raise ValueError("sigma_ref must be > 1/2")
        if self.T <= 0:
            raise ValueError("T must be positive")
        if self.n_nodes <= 1:
            raise ValueError("n_nodes must be >= 2")

        h = (2.0 * self.T) / (self.n_nodes - 1)
        sig = arb(self.sigma_ref)
        primes_big = [p for p in cached_primes_upto(self.P_cut) if p >= 2]

        t_nodes: List[arb] = []
        u_nodes: List[arb] = []
        for j in range(self.n_nodes):
            t = -self.T + j * h
            tA = arb(t)
            t_nodes.append(tA)

            sb = acb(sig, arb(-t))
            # det2 truncated at P_cut (midpoint sampling)
            logdet = acb(0)
            for p in primes_big:
                ps = acb(p) ** (-sb)
                logdet += (acb(1) - ps).log() + ps
            det2_tr = logdet.exp()

            F = det2_tr / sb.zeta() * compensator_B(sb)
            u_nodes.append(abs(F).log())

        self.t_nodes = t_nodes
        self.u_nodes = u_nodes

    def logO(self, s: acb) -> acb:
        assert self.t_nodes is not None and self.u_nodes is not None
        # IMPORTANT: midpoint mode is *exploratory*; evaluate at the midpoint of
        # the input box to avoid catastrophic enclosure blow-up.
        s0 = acb_midpoint(s)
        sig = arb(self.sigma_ref)
        w = acb(0, 1) * (s0 - acb(sig))
        pi = acb.pi()

        h = arb((2.0 * self.T) / (self.n_nodes - 1))
        total = acb(0)
        for j, (tA, uA) in enumerate(zip(self.t_nodes, self.u_nodes)):
            tC = acb(tA)
            kernel = (acb(1) / (tC - w)) - (tC / (acb(1) + tC * tC))
            weight = h
            if j == 0 or j == len(self.t_nodes) - 1:
                weight = weight / acb(2)
            total += kernel * acb(uA) * acb(weight)

        return (-acb(0, 1) / pi) * total

    def O(self, s: acb) -> acb:
        # midpoint mode is exploratory; keep evaluation at the midpoint.
        return self.logO(s).exp()


@dataclass
class OuterNormalizerEngineMidpointZetaLog(OuterNormalizerEngineLike):
    """
    Exploratory *phase-corrected* ζ-gauge normalizer.

    Motivation:
      The Option B ζ-gauge normalizer uses only boundary modulus data u(t)=log|F(t)|,
      which cannot control the *phase* of the arithmetic Cayley/Herglotz field.
      This engine instead builds boundary data

        ℓ(t) := log( F(σ_ref - i t) ),   where  F(s) = det2_{P_cut}(s)/ζ(s) * B(s),

      using midpoint sampling and a simple argument-unwrapping pass along t.

    Definition (truncated):
      log O(s) := (1/(π i)) ∫_{-T}^{T} ( 1/(t - w) - t/(1+t^2) ) ℓ(t) dt,
      w := i(s - σ_ref).

    If ℓ(t) were the boundary values of an analytic Hardy-class function, the Cauchy
    integral would reconstruct it (up to the chosen normalization), so O(s) would
    approximately cancel both magnitude and phase of F(s) and make
      J(s)=F(s)/O(s)
    close to 1 (hence Herglotz-like).

    WARNING:
      This is midpoint/unwrapped and is *diagnostic only*. It is intended to decide
      whether adding phase information is the right mathematical direction before
      investing in a fully certified argument-principle / branch-tracking version.
    """

    sigma_ref: float
    T: float
    n_nodes: int
    P_cut: int
    unwrap: bool = True

    t_nodes: List[arb] | None = None
    log_nodes: List[acb] | None = None

    def _cache_params(self) -> Dict[str, Any]:
        return {
            "sigma_ref": float(self.sigma_ref),
            "T": float(self.T),
            "n_nodes": int(self.n_nodes),
            "P_cut": int(self.P_cut),
            "unwrap": bool(self.unwrap),
        }

    def save_cache(self, path: str) -> None:
        if self.t_nodes is None or self.log_nodes is None:
            raise ValueError("outer cache save requested before build()")
        data = {
            "kind": "OuterNormalizerEngineMidpointZetaLog",
            "version": 1,
            "params": self._cache_params(),
            "n_nodes": int(len(self.t_nodes)),
            "t_nodes": [str(x) for x in self.t_nodes],
            "log_nodes_re": [str(z.real) for z in self.log_nodes],
            "log_nodes_im": [str(z.imag) for z in self.log_nodes],
        }
        _write_json(path, data)

    def load_cache(self, path: str, *, strict: bool = True) -> None:
        data = _read_json(path)
        if not isinstance(data, dict):
            raise ValueError("outer cache is not a JSON object")
        if data.get("kind") != "OuterNormalizerEngineMidpointZetaLog":
            raise ValueError(f"unexpected outer cache kind: {data.get('kind')!r}")
        if int(data.get("version", -1)) != 1:
            raise ValueError(f"unsupported outer cache version: {data.get('version')!r}")

        params = data.get("params", {})
        want = self._cache_params()
        mismatches: List[str] = []
        for k, v_want in want.items():
            v = params.get(k)
            if v is None:
                mismatches.append(f"{k}: missing")
                continue
            if isinstance(v_want, float):
                if not math.isclose(float(v), float(v_want), rel_tol=0.0, abs_tol=1e-15):
                    mismatches.append(f"{k}: cache={v} want={v_want}")
            else:
                if v != v_want:
                    mismatches.append(f"{k}: cache={v} want={v_want}")
        if mismatches and strict:
            raise ValueError("outer cache parameter mismatch:\n  " + "\n  ".join(mismatches))
        if mismatches and not strict:
            _progress(
                "[outer_cache] WARNING: parameter mismatch; using cache anyway:\n  "
                + "\n  ".join(mismatches),
                enabled=True,
            )

        try:
            t_list = [arb(s) for s in data["t_nodes"]]
            re_list = [arb(s) for s in data["log_nodes_re"]]
            im_list = [arb(s) for s in data["log_nodes_im"]]
        except Exception as e:
            raise ValueError(f"failed to parse outer cache arb data: {e}") from e
        if not (len(t_list) == len(re_list) == len(im_list)):
            raise ValueError("outer cache lists have inconsistent lengths")
        self.t_nodes = t_list
        self.log_nodes = [acb(re, im) for re, im in zip(re_list, im_list)]

    def build(self) -> None:
        if self.sigma_ref <= 0.5:
            raise ValueError("sigma_ref must be > 1/2")
        if self.T <= 0:
            raise ValueError("T must be positive")
        if self.n_nodes <= 1:
            raise ValueError("n_nodes must be >= 2")

        h = (2.0 * self.T) / (self.n_nodes - 1)
        sig = arb(self.sigma_ref)
        primes_big = [p for p in cached_primes_upto(self.P_cut) if p >= 2]

        t_nodes: List[arb] = []
        log_nodes: List[acb] = []

        # Build principal logs at nodes.
        for j in range(self.n_nodes):
            t = -self.T + j * h
            tA = arb(t)
            t_nodes.append(tA)

            sb = acb(sig, arb(-t))
            # det2 truncated at P_cut (midpoint sampling)
            logdet = acb(0)
            for p in primes_big:
                ps = acb(p) ** (-sb)
                logdet += (acb(1) - ps).log() + ps
            det2_tr = logdet.exp()

            F = det2_tr / sb.zeta() * compensator_B(sb)
            logF = F.log()  # principal branch
            log_nodes.append(logF)

        # Unwrap the imaginary part (argument) along t to reduce 2π jumps.
        if self.unwrap and len(log_nodes) >= 2:
            two_pi = 2.0 * math.pi
            k_offset = 0
            prev_im = float(log_nodes[0].imag)
            for i in range(1, len(log_nodes)):
                im = float(log_nodes[i].imag) + two_pi * k_offset
                diff = im - prev_im
                if diff > math.pi:
                    while diff > math.pi:
                        k_offset -= 1
                        im -= two_pi
                        diff = im - prev_im
                elif diff < -math.pi:
                    while diff < -math.pi:
                        k_offset += 1
                        im += two_pi
                        diff = im - prev_im
                if k_offset != 0:
                    log_nodes[i] = log_nodes[i] + acb(0, arb(two_pi * k_offset))
                prev_im = im

        self.t_nodes = t_nodes
        self.log_nodes = log_nodes

    def logO(self, s: acb) -> acb:
        assert self.t_nodes is not None and self.log_nodes is not None
        # midpoint mode is *exploratory*; evaluate at the midpoint of the input box
        # to avoid catastrophic enclosure blow-up.
        s0 = acb_midpoint(s)
        sig = arb(self.sigma_ref)
        w = acb(0, 1) * (s0 - acb(sig))
        pi = acb.pi()

        h = arb((2.0 * self.T) / (self.n_nodes - 1))
        total = acb(0)
        for j, (tA, logA) in enumerate(zip(self.t_nodes, self.log_nodes)):
            tC = acb(tA)
            kernel = (acb(1) / (tC - w)) - (tC / (acb(1) + tC * tC))
            weight = h
            if j == 0 or j == len(self.t_nodes) - 1:
                weight = weight / acb(2)  # trapezoid endpoint half-weight
            total += kernel * logA * acb(weight)

        return (-acb(0, 1) / pi) * total

    def O(self, s: acb) -> acb:
        return self.logO(s).exp()


@dataclass
class OuterNormalizerEngineMidpointZetaProject(OuterNormalizerEngineLike):
    """
    Exploratory ζ-gauge normalizer that *directly* Cauchy-projects complex boundary values.

    Build boundary samples
      f(t) := det2_{P_cut}(σ_ref - i t) / ζ(σ_ref - i t) * B(σ_ref - i t),
    and define the analytic normalizer by a truncated Cauchy-type integral
      O(s) := (1/(π i)) ∫_{-T}^{T} ( 1/(t - w) - t/(1+t^2) ) f(t) dt,
      w := i(s - σ_ref).

    This is a phase-sensitive analogue of Option B: it uses *complex* boundary values
    (so it can cancel phase, not just modulus), without requiring complex logarithms
    (no branch tracking). Empirically this can pull Θ=(2J-1)/(2J+1) back inside the
    unit disk at heights where the modulus-only outer leaves Θ outside.

    WARNING:
      Midpoint/truncated/discretized. Also, unlike an outer exp(…) normalizer, this
      O(s) is not guaranteed zero-free. Treat as diagnostic until we have a certified
      nonvanishing argument for O on the needed region.
    """

    sigma_ref: float
    T: float
    n_nodes: int
    P_cut: int

    t_nodes: List[arb] | None = None
    f_nodes: List[acb] | None = None

    def _cache_params(self) -> Dict[str, Any]:
        return {
            "sigma_ref": float(self.sigma_ref),
            "T": float(self.T),
            "n_nodes": int(self.n_nodes),
            "P_cut": int(self.P_cut),
        }

    def save_cache(self, path: str) -> None:
        if self.t_nodes is None or self.f_nodes is None:
            raise ValueError("outer cache save requested before build()")
        data = {
            "kind": "OuterNormalizerEngineMidpointZetaProject",
            "version": 1,
            "params": self._cache_params(),
            "n_nodes": int(len(self.t_nodes)),
            "t_nodes": [str(x) for x in self.t_nodes],
            "f_nodes_re": [str(z.real) for z in self.f_nodes],
            "f_nodes_im": [str(z.imag) for z in self.f_nodes],
        }
        _write_json(path, data)

    def load_cache(self, path: str, *, strict: bool = True) -> None:
        data = _read_json(path)
        if not isinstance(data, dict):
            raise ValueError("outer cache is not a JSON object")
        if data.get("kind") != "OuterNormalizerEngineMidpointZetaProject":
            raise ValueError(f"unexpected outer cache kind: {data.get('kind')!r}")
        if int(data.get("version", -1)) != 1:
            raise ValueError(f"unsupported outer cache version: {data.get('version')!r}")

        params = data.get("params", {})
        want = self._cache_params()
        mismatches: List[str] = []
        for k, v_want in want.items():
            v = params.get(k)
            if v is None:
                mismatches.append(f"{k}: missing")
                continue
            if isinstance(v_want, float):
                if not math.isclose(float(v), float(v_want), rel_tol=0.0, abs_tol=1e-15):
                    mismatches.append(f"{k}: cache={v} want={v_want}")
            else:
                if int(v) != int(v_want):
                    mismatches.append(f"{k}: cache={v} want={v_want}")
        if mismatches and strict:
            raise ValueError("outer cache parameter mismatch:\n  " + "\n  ".join(mismatches))
        if mismatches and not strict:
            _progress(
                "[outer_cache] WARNING: parameter mismatch; using cache anyway:\n  "
                + "\n  ".join(mismatches),
                enabled=True,
            )

        try:
            t_list = [arb(s) for s in data["t_nodes"]]
            re_list = [arb(s) for s in data["f_nodes_re"]]
            im_list = [arb(s) for s in data["f_nodes_im"]]
        except Exception as e:
            raise ValueError(f"failed to parse outer cache arb data: {e}") from e
        if not (len(t_list) == len(re_list) == len(im_list)):
            raise ValueError("outer cache lists have inconsistent lengths")
        self.t_nodes = t_list
        self.f_nodes = [acb(re, im) for re, im in zip(re_list, im_list)]

    def build(self) -> None:
        if self.sigma_ref <= 0.5:
            raise ValueError("sigma_ref must be > 1/2")
        if self.T <= 0:
            raise ValueError("T must be positive")
        if self.n_nodes <= 1:
            raise ValueError("n_nodes must be >= 2")

        h = (2.0 * self.T) / (self.n_nodes - 1)
        sig = arb(self.sigma_ref)
        primes_big = [p for p in cached_primes_upto(self.P_cut) if p >= 2]

        t_nodes: List[arb] = []
        f_nodes: List[acb] = []
        for j in range(self.n_nodes):
            t = -self.T + j * h
            tA = arb(t)
            t_nodes.append(tA)

            sb = acb(sig, arb(-t))
            # det2 truncated at P_cut (midpoint sampling)
            logdet = acb(0)
            for p in primes_big:
                ps = acb(p) ** (-sb)
                logdet += (acb(1) - ps).log() + ps
            det2_tr = logdet.exp()

            zeta_val = sb.zeta()
            if abs(zeta_val).contains(0):
                raise ValueError(
                    "midpoint zeta_proj boundary sample has zeta enclosure containing 0: "
                    f"sigma_ref={self.sigma_ref}, t={t}. Increase --prec, increase --outer-n, or shift --outer-sigma-ref."
                )
            f_nodes.append(det2_tr / zeta_val * compensator_B(sb))

        self.t_nodes = t_nodes
        self.f_nodes = f_nodes

    def O(self, s: acb) -> acb:
        assert self.t_nodes is not None and self.f_nodes is not None
        # midpoint mode is *exploratory*; evaluate at midpoint to avoid ball blow-up
        s0 = acb_midpoint(s)
        sig = arb(self.sigma_ref)
        w = acb(0, 1) * (s0 - acb(sig))
        pi = acb.pi()

        h = arb((2.0 * self.T) / (self.n_nodes - 1))
        total = acb(0)
        for j, (tA, fA) in enumerate(zip(self.t_nodes, self.f_nodes)):
            tC = acb(tA)
            kernel = (acb(1) / (tC - w)) - (tC / (acb(1) + tC * tC))
            weight = h
            if j == 0 or j == len(self.t_nodes) - 1:
                weight = weight / acb(2)
            total += kernel * fA * acb(weight)

        return (-acb(0, 1) / pi) * total

    def O_box(self, s: acb) -> acb:
        """
        Box-evaluated (rigorous w.r.t. the input ball) variant of O(s).

        This uses the *discrete* zeta_proj normalizer defined by the built (t_nodes, f_nodes)
        and evaluates the kernel on the full input ball `s` (no midpoint projection).
        """
        assert self.t_nodes is not None and self.f_nodes is not None
        sig = arb(self.sigma_ref)
        w = acb(0, 1) * (s - acb(sig))  # w = i(s - σ_ref)
        pi = acb.pi()

        h = arb((2.0 * self.T) / (self.n_nodes - 1))
        total = acb(0)
        for j, (tA, fA) in enumerate(zip(self.t_nodes, self.f_nodes)):
            tC = acb(tA)
            kernel = (acb(1) / (tC - w)) - (tC / (acb(1) + tC * tC))
            weight = h
            if j == 0 or j == len(self.t_nodes) - 1:
                weight = weight / acb(2)
            total += kernel * fA * acb(weight)

        return (-acb(0, 1) / pi) * total


def cover_rect(R: Rect, n_sigma: int, n_t: int) -> List[acb]:
    """
    Cover R by n_sigma × n_t acb rectangles (complex boxes), returned as acb balls
    with separate real/imag radii.
    """
    if n_sigma <= 0 or n_t <= 0:
        raise ValueError("n_sigma and n_t must be positive")
    ds = (R.sigma_max - R.sigma_min) / n_sigma
    dt = (R.t_max - R.t_min) / n_t
    boxes: List[acb] = []
    for i in range(n_sigma):
        s0 = R.sigma_min + i * ds
        s1 = R.sigma_min + (i + 1) * ds
        re_mid = 0.5 * (s0 + s1)
        re_rad = 0.5 * (s1 - s0)
        for j in range(n_t):
            t0 = R.t_min + j * dt
            t1 = R.t_min + (j + 1) * dt
            im_mid = 0.5 * (t0 + t1)
            im_rad = 0.5 * (t1 - t0)
            boxes.append(acb_from_box(re_mid, re_rad, im_mid, im_rad))
    return boxes


def arb_lower(x: arb) -> float:
    # Convert an Arb lower endpoint to a Python float, then round *down* to preserve safety.
    return math.nextafter(float(x.lower()), -float("inf"))


def arb_upper(x: arb) -> float:
    # Convert an Arb upper endpoint to a Python float, then round *up* to preserve safety.
    return math.nextafter(float(x.upper()), float("inf"))


def cayley_theta_from_J(J: acb) -> acb:
    """
    Cayley transform from the right half-plane to the unit disk:
      Θ = (2J - 1)/(2J + 1).

    If Re(2J) ≥ 0, then |Θ| ≤ 1, with strict inequality when Re(2J) > 0.
    """
    twoJ = J * acb(2)
    return (twoJ - acb(1)) / (twoJ + acb(1))


def pseudohyperbolic_phi(theta_a: acb, theta_c: acb) -> acb:
    """
    Disk automorphism φ_{θc}(θa) = (θa-θc)/(1-conj(θc) θa).

    If |theta_c| < 1 and |φ| < 1, then |theta_a| < 1 (Möbius invariance of the disk).
    """
    denom = acb(1) - theta_c.conjugate() * theta_a
    return (theta_a - theta_c) / denom


def _safe_abs_upper(z: acb) -> float:
    """Return an upper bound for |z| as a float, or +inf if it cannot be certified."""
    try:
        a = abs(z)
        hi = arb_upper(a)
        if math.isnan(hi):
            return float("inf")
        return hi
    except Exception:
        return float("inf")


def _denom_contains_zero(z: acb) -> bool:
    """
    Does the complex box enclosure contain 0?

    IMPORTANT: Do NOT use `abs(z).contains(0)` here. For non-point `acb` enclosures,
    some `abs` implementations return a coarse `[0, upper]` enclosure even when the
    complex rectangle is bounded away from 0. The correct test for membership of 0
    in an `acb` rectangle is: 0 ∈ Re(z) and 0 ∈ Im(z).
    """
    try:
        return bool(z.real.contains(0) and z.imag.contains(0))
    except Exception:
        # If we cannot even form an enclosure, treat as unsafe.
        return True


def _abs_lower_acb(z: acb) -> float:
    """
    A cheap lower bound on |z| for a complex rectangle enclosure (acb).
    Returns 0 if the rectangle intersects the origin.
    """
    try:
        re_lo = float(z.real.lower())
        re_hi = float(z.real.upper())
        im_lo = float(z.imag.lower())
        im_hi = float(z.imag.upper())
        re_min = 0.0 if (re_lo <= 0.0 <= re_hi) else min(abs(re_lo), abs(re_hi))
        im_min = 0.0 if (im_lo <= 0.0 <= im_hi) else min(abs(im_lo), abs(im_hi))
        return math.hypot(re_min, im_min)
    except Exception:
        return 0.0


def _acb_inv_safe(z: acb) -> acb:
    """
    Safe enclosure for 1/z for an `acb` rectangle.

    We avoid relying on `abs(z)` lower bounds, because some complex-ball absolute-value
    implementations are extremely coarse and can return a 0 lower bound even when the
    rectangle is bounded away from 0, which can trigger NaNs in division.
    """
    if _denom_contains_zero(z):
        raise ZeroDivisionError("complex denominator encloses 0")

    # Build a *certified* positive interval [den_lo, den_hi] enclosing |z|^2
    # using only rectangle endpoint bounds (avoids ball-multiplication blow-up).
    re_lo = float(z.real.lower())
    re_hi = float(z.real.upper())
    im_lo = float(z.imag.lower())
    im_hi = float(z.imag.upper())

    abs_lo = _abs_lower_acb(z)
    if abs_lo <= 0.0:
        raise ZeroDivisionError("could not certify |den| lower bound > 0")
    den_lo = abs_lo * abs_lo

    re_abs_hi = max(abs(re_lo), abs(re_hi))
    im_abs_hi = max(abs(im_lo), abs(im_hi))
    den_hi = (re_abs_hi * re_abs_hi) + (im_abs_hi * im_abs_hi)
    if not (den_hi >= den_lo) or math.isnan(den_hi) or math.isinf(den_hi):
        raise ZeroDivisionError("failed to build a finite |den|^2 upper bound")

    den_mid = 0.5 * (den_lo + den_hi)
    den_rad = 0.5 * (den_hi - den_lo)
    den = arb(den_mid, den_rad)  # interval [den_lo, den_hi]

    return acb(z.real, -z.imag) / acb(den)


def _acb_div_safe(num: acb, den: acb) -> acb:
    """Safe enclosure for num/den using `_acb_inv_safe`."""
    return num * _acb_inv_safe(den)


def _parse_csv_floats(s: str) -> List[float]:
    out: List[float] = []
    for part in (s or "").split(","):
        part = part.strip()
        if not part:
            continue
        out.append(float(part))
    return out


def _parse_csv_ints(s: str) -> List[int]:
    out: List[int] = []
    for part in (s or "").split(","):
        part = part.strip()
        if not part:
            continue
        out.append(int(part))
    return out


def _parse_csv_strs(s: str) -> List[str]:
    out: List[str] = []
    for part in (s or "").split(","):
        part = part.strip()
        if not part:
            continue
        out.append(part)
    return out


def _build_outer_midpoint_for_gauge(
    gauge: str,
    *,
    sigma_ref: float,
    T: float,
    n_nodes: int,
    P_cut: int,
) -> OuterNormalizerEngineLike | None:
    """
    Construct and build a midpoint outer engine appropriate for a given gauge.
    Returns None for gauges that do not use an outer normalizer.
    """
    if gauge == "raw_xi" or gauge == "raw_zeta":
        return None
    if gauge == "canonical_zeta":
        raise ValueError(
            "gauge canonical_zeta requires the manuscript canonical outer normalizer, "
            "which is not implemented (see FF_CANONICAL_ARTIFACT_PLAN.md)"
        )
    if gauge == "outer_xi":
        eng: OuterNormalizerEngineLike = OuterNormalizerEngineMidpoint(
            sigma_ref=float(sigma_ref),
            T=float(T),
            n_nodes=max(2, int(n_nodes)),
            P_cut=int(P_cut),
        )
        eng.build()
        return eng
    if gauge == "outer_zeta":
        eng = OuterNormalizerEngineMidpointZeta(
            sigma_ref=float(sigma_ref),
            T=float(T),
            n_nodes=max(2, int(n_nodes)),
            P_cut=int(P_cut),
        )
        eng.build()
        return eng
    if gauge == "outer_zeta_log":
        eng = OuterNormalizerEngineMidpointZetaLog(
            sigma_ref=float(sigma_ref),
            T=float(T),
            n_nodes=max(2, int(n_nodes)),
            P_cut=int(P_cut),
            unwrap=True,
        )
        eng.build()
        return eng
    if gauge == "outer_zeta_proj":
        eng = OuterNormalizerEngineMidpointZetaProject(
            sigma_ref=float(sigma_ref),
            T=float(T),
            n_nodes=max(2, int(n_nodes)),
            P_cut=int(P_cut),
        )
        eng.build()
        return eng
    raise ValueError(f"unknown gauge for midpoint outer: {gauge}")


def _theta_hi_arith_on_rect(
    *,
    primes: List[int],
    R: Rect,
    n_sigma: int,
    n_t: int,
    gauge: str,
    outer: OuterNormalizerEngineLike | None,
    det2_mode: str,
    det2_P_cut: int,
    det2_k_max: int,
) -> Dict[str, Any]:
    """
    Diagnostic sweep metric: theta_hi = sup_R |Theta(J_arith)| (evaluated at box midpoints).
    Also returns basic denominator diagnostics and worst-point location.
    """
    if det2_mode not in {"trunc", "enclosure"}:
        raise ValueError(f"unknown det2_mode: {det2_mode}")
    boxes = cover_rect(R, n_sigma=n_sigma, n_t=n_t)

    theta_hi = 0.0
    worst: Dict[str, Any] | None = None

    need_zeta = gauge in {"raw_zeta", "outer_zeta", "outer_zeta_log", "outer_zeta_proj", "canonical_zeta"}
    need_xi = gauge in {"raw_xi", "outer_xi"}
    need_outer = gauge in {"outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj", "canonical_zeta"}

    zeta_contains0 = 0
    xi_contains0 = 0
    O_contains0 = 0
    min_abs_zeta = float("inf")
    min_abs_xi = float("inf")
    min_abs_O = float("inf")

    for box in boxes:
        s = acb_midpoint(box)

        # det2
        if det2_mode == "trunc":
            det2_val = det2_prime_trunc(s, primes)
        else:
            det2_val = det2_full_enclosure(s, P_cut=int(det2_P_cut), k_max=int(det2_k_max))

        zeta_val: acb | None = None
        if need_zeta:
            zeta_val = acb_midpoint(s.zeta())
            az = abs(zeta_val)
            if az.contains(0):
                zeta_contains0 += 1
            min_abs_zeta = min(min_abs_zeta, max(0.0, arb_lower(az)))

        xi_val: acb | None = None
        if need_xi:
            xi_val = xi_completed(s)
            ax = abs(xi_val)
            if ax.contains(0):
                xi_contains0 += 1
            min_abs_xi = min(min_abs_xi, max(0.0, arb_lower(ax)))

        O_val: acb | None = None
        if need_outer:
            if outer is None:
                raise ValueError(f"gauge {gauge} requires an outer engine, but outer=None")
            if hasattr(outer, "O_midpoint"):
                O_val = getattr(outer, "O_midpoint")(s)
            else:
                O_val = acb_midpoint(outer.O(s))
            aO = abs(O_val)
            if aO.contains(0):
                O_contains0 += 1
            min_abs_O = min(min_abs_O, max(0.0, arb_lower(aO)))

        # J_arith
        if gauge == "raw_xi":
            assert xi_val is not None
            Ja = det2_val / xi_val
        elif gauge == "raw_zeta":
            assert zeta_val is not None
            Ja = det2_val / zeta_val * compensator_B(s)
        elif gauge == "outer_xi":
            assert xi_val is not None and O_val is not None
            Ja = det2_val / (O_val * xi_val)
        elif gauge in {"outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
            assert zeta_val is not None and O_val is not None
            Ja = det2_val / (O_val * zeta_val) * compensator_B(s)
        else:
            raise ValueError(f"unknown gauge: {gauge}")

        # Theta(J_arith), with safety guard for 2J+1 denominator
        denom_theta = Ja * acb(2) + acb(1)
        if _denom_contains_zero(denom_theta):
            theta_hi = float("inf")
            worst = {
                "sigma": float(s.real),
                "t": float(s.imag),
                "reason": "2J+1 encloses 0 (Theta undefined on this point enclosure)",
            }
            break
        theta = cayley_theta_from_J(Ja)
        th = _safe_abs_upper(theta)
        if th > theta_hi:
            theta_hi = th
            worst = {"sigma": float(s.real), "t": float(s.imag), "theta_hi": th}

    return {
        "theta_hi": theta_hi,
        "worst": worst,
        "denoms": {
            "need_zeta": need_zeta,
            "need_xi": need_xi,
            "need_outer": need_outer,
            "zeta_abs_contains0_points": zeta_contains0,
            "xi_abs_contains0_points": xi_contains0,
            "O_abs_contains0_points": O_contains0,
            "min_abs_zeta_lower": (None if min_abs_zeta == float("inf") else min_abs_zeta),
            "min_abs_xi_lower": (None if min_abs_xi == float("inf") else min_abs_xi),
            "min_abs_O_lower": (None if min_abs_O == float("inf") else min_abs_O),
        },
    }


def run_gauge_sweep(args: Any) -> None:
    """
    Search a structured family of gauges/parameters and report theta_hi margins on a target rectangle.

    This is a diagnostic tool: it evaluates J_arith at box midpoints.
    """
    gauges = _parse_csv_strs(args.sweep_gauges)
    if not gauges:
        raise ValueError("--sweep-gauges produced an empty list")
    sigma_refs = _parse_csv_floats(args.sweep_sigma_refs)
    Ts = _parse_csv_floats(args.sweep_Ts)
    P_cuts = _parse_csv_ints(args.sweep_P_cuts)
    if not sigma_refs:
        sigma_refs = [float(args.outer_sigma_ref)]
    if not Ts:
        Ts = [float(args.outer_T)]
    if not P_cuts:
        P_cuts = [int(args.outer_P_cut)]

    rect = Rect(
        sigma_min=float(args.sweep_rect_sigma_min),
        sigma_max=float(args.sweep_rect_sigma_max),
        t_min=float(args.sweep_rect_t_min),
        t_max=float(args.sweep_rect_t_max),
    )
    n_sigma = int(args.sweep_cover_n_sigma)
    n_t = int(args.sweep_cover_n_t)
    if n_sigma <= 0 or n_t <= 0:
        raise ValueError("sweep cover sizes must be positive")

    det2_mode = str(args.sweep_det2_mode)
    det2_kmax = int(args.arith_det2_kmax)
    step = float(args.sweep_node_step)
    if step <= 0:
        raise ValueError("--sweep-node-step must be positive")

    max_cases = int(args.sweep_max_cases)
    if max_cases <= 0:
        raise ValueError("--sweep-max-cases must be positive")
    time_limit = float(args.sweep_time_limit)
    t_start = time.time()

    # Cache primes and built outers by (gauge,sigma_ref,T,n_nodes,P_cut).
    primes_cache: Dict[int, List[int]] = {}
    outer_cache: Dict[Tuple[str, float, float, int, int], OuterNormalizerEngineLike | None] = {}

    rows: List[Dict[str, Any]] = []
    cases = 0

    print()
    print("[sweep] rectangle:", f"[{rect.sigma_min},{rect.sigma_max}] × [{rect.t_min},{rect.t_max}]")
    print("[sweep] gauges:", ", ".join(gauges))
    print("[sweep] sigma_refs:", sigma_refs)
    print("[sweep] Ts:", Ts)
    print("[sweep] P_cuts:", P_cuts)
    print("[sweep] cover:", f"{n_sigma}×{n_t} (midpoint evaluation)")
    print("[sweep] det2_mode:", det2_mode)
    print()

    for gauge in gauges:
        # Gauges without an outer normalizer: just one config per P_cut (det2 truncation set).
        if gauge in {"raw_xi", "raw_zeta"}:
            for P_cut in P_cuts:
                if cases >= max_cases:
                    break
                if time_limit > 0 and (time.time() - t_start) >= time_limit:
                    break
                cases += 1

                primes = primes_cache.get(P_cut)
                if primes is None:
                    primes = [p for p in cached_primes_upto(int(P_cut)) if p >= 2]
                    primes_cache[P_cut] = primes

                t0 = time.time()
                info = _theta_hi_arith_on_rect(
                    primes=primes,
                    R=rect,
                    n_sigma=n_sigma,
                    n_t=n_t,
                    gauge=gauge,
                    outer=None,
                    det2_mode=det2_mode,
                    det2_P_cut=int(P_cut),
                    det2_k_max=det2_kmax,
                )
                dt = time.time() - t0
                th = float(info["theta_hi"])
                margin = (None if math.isinf(th) or math.isnan(th) else (1.0 - th))
                row = {
                    "gauge": gauge,
                    "outer": None,
                    "sigma_ref": None,
                    "T": None,
                    "n_nodes": None,
                    "P_cut": int(P_cut),
                    "theta_hi": th,
                    "margin_1_minus_theta_hi": margin,
                    "seconds": dt,
                    **info,
                }
                rows.append(row)
                print(
                    f"[sweep] {cases:4d}  gauge={gauge:>12}  P_cut={P_cut:<6d}  "
                    f"theta_hi={th:.6g}  margin={('' if margin is None else f'{margin:.3g}'):<9}  "
                    f"dt={dt:.2f}s"
                )
            continue

        # Outer gauges: cartesian product sigma_ref × T × P_cut, with n_nodes derived from node-step.
        for sigma_ref in sigma_refs:
            for T in Ts:
                for P_cut in P_cuts:
                    if cases >= max_cases:
                        break
                    if time_limit > 0 and (time.time() - t_start) >= time_limit:
                        break
                    cases += 1

                    # Derive n_nodes from step (keep boundary sampling density roughly constant across T).
                    n_nodes = max(2, int(round((2.0 * float(T)) / step)) + 1)

                    primes = primes_cache.get(P_cut)
                    if primes is None:
                        primes = [p for p in cached_primes_upto(int(P_cut)) if p >= 2]
                        primes_cache[P_cut] = primes

                    key = (gauge, float(sigma_ref), float(T), int(n_nodes), int(P_cut))
                    outer = outer_cache.get(key)
                    if outer is None and key not in outer_cache:
                        # Build outer (can be expensive).
                        outer = _build_outer_midpoint_for_gauge(
                            gauge,
                            sigma_ref=float(sigma_ref),
                            T=float(T),
                            n_nodes=int(n_nodes),
                            P_cut=int(P_cut),
                        )
                        outer_cache[key] = outer
                    else:
                        outer = outer_cache.get(key)

                    t0 = time.time()
                    info = _theta_hi_arith_on_rect(
                        primes=primes,
                        R=rect,
                        n_sigma=n_sigma,
                        n_t=n_t,
                        gauge=gauge,
                        outer=outer,
                        det2_mode=det2_mode,
                        det2_P_cut=int(P_cut),
                        det2_k_max=det2_kmax,
                    )
                    dt = time.time() - t0
                    th = float(info["theta_hi"])
                    margin = (None if math.isinf(th) or math.isnan(th) else (1.0 - th))
                    row = {
                        "gauge": gauge,
                        "outer": (None if outer is None else type(outer).__name__),
                        "sigma_ref": float(sigma_ref),
                        "T": float(T),
                        "n_nodes": int(n_nodes),
                        "P_cut": int(P_cut),
                        "theta_hi": th,
                        "margin_1_minus_theta_hi": margin,
                        "seconds": dt,
                        **info,
                    }
                    rows.append(row)
                    print(
                        f"[sweep] {cases:4d}  gauge={gauge:>12}  sig_ref={sigma_ref:<7g}  T={T:<6g}  "
                        f"n={n_nodes:<5d}  P_cut={P_cut:<6d}  "
                        f"theta_hi={th:.6g}  margin={('' if margin is None else f'{margin:.3g}'):<9}  "
                        f"dt={dt:.2f}s"
                    )

    # Rank and show top results.
    finite_rows = [r for r in rows if not (math.isnan(float(r["theta_hi"])) or math.isinf(float(r["theta_hi"])))]
    finite_rows.sort(key=lambda r: float(r["theta_hi"]))
    top_k = min(int(args.sweep_top), len(finite_rows))
    print()
    print(f"[sweep] done. cases={cases}, finite={len(finite_rows)}, elapsed={time.time()-t_start:.1f}s")
    print(f"[sweep] top {top_k} by theta_hi:")
    for i in range(top_k):
        r = finite_rows[i]
        print(
            f"  #{i+1:02d}  theta_hi={float(r['theta_hi']):.8g}  gauge={r['gauge']}  "
            f"sig_ref={r.get('sigma_ref')}  T={r.get('T')}  n={r.get('n_nodes')}  P_cut={r.get('P_cut')}"
        )

    if args.sweep_out is not None:
        _write_json(str(args.sweep_out), {"rect": vars(rect), "rows": rows})
        print(f"[sweep] wrote JSON: {args.sweep_out}")


def theta_certify_rect(args: Any) -> Dict[str, Any]:
    """
    Fully rigorous (ball arithmetic) attempt to certify:
      - ζ(s) does not vanish on the rectangle cover (so 1/ζ is safe),
      - O(s) does not vanish (for outer gauges),
      - |Theta(J_arith(s))| < 1 on the rectangle,
    using adaptive bisection with hard caps so runs cannot explode.
    """
    gauge = str(args.arith_gauge)
    if gauge == "canonical_zeta":
        raise ValueError(
            "--theta-certify does not support --arith-gauge canonical_zeta yet "
            "(canonical outer normalizer not implemented; see FF_CANONICAL_ARTIFACT_PLAN.md)"
        )
    if gauge not in {"raw_xi", "raw_zeta", "outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
        raise ValueError(f"--theta-certify: unsupported gauge {gauge!r}")
    if str(args.eval_mode) != "rigorous":
        raise ValueError("--theta-certify requires --eval-mode rigorous (box evaluation)")

    R = Rect(
        sigma_min=float(args.rect_sigma_min),
        sigma_max=float(args.rect_sigma_max),
        t_min=float(args.rect_t_min),
        t_max=float(args.rect_t_max),
    )

    # det2 settings (arith side)
    det2_mode = str(args.arith_det2_mode)
    det2_kmax = int(args.arith_det2_kmax)
    arith_P_cut = int(args.P if args.arith_P_cut is None else args.arith_P_cut)
    primes = [p for p in cached_primes_upto(arith_P_cut) if p >= 2]

    # Build outer (if needed)
    outer_engine: OuterNormalizerEngineLike | None = None
    if gauge in {"outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
        outer_mode = str(getattr(args, "outer_mode", "midpoint"))
        if outer_mode == "midpoint":
            outer_engine = _build_outer_midpoint_for_gauge(
                gauge,
                sigma_ref=float(args.outer_sigma_ref),
                T=float(args.outer_T),
                n_nodes=int(max(2, args.outer_n)),
                P_cut=int(args.outer_P_cut),
            )
        elif outer_mode == "rigorous":
            if gauge == "outer_xi":
                outer_engine = OuterNormalizerEngine(
                    sigma_ref=float(args.outer_sigma_ref),
                    T=float(args.outer_T),
                    n_intervals=int(args.outer_n),
                    P_cut=int(args.outer_P_cut),
                )
                # Optional cache is not implemented for this engine.
                outer_engine.build()
            elif gauge == "outer_zeta":
                outer_engine = OuterNormalizerEngineZetaRigorous(
                    sigma_ref=float(args.outer_sigma_ref),
                    T=float(args.outer_T),
                    n_intervals=int(args.outer_n),
                    P_cut=int(args.outer_P_cut),
                    k_max=int(args.arith_det2_kmax),
                    progress=bool(args.progress),
                    max_depth=int(args.outer_max_depth),
                    min_width=float(args.outer_min_width),
                )
                if args.outer_cache_load is not None:
                    outer_engine.load_cache(
                        str(args.outer_cache_load), strict=not bool(args.outer_cache_ignore_mismatch)
                    )
                else:
                    outer_engine.build()
                if args.outer_cache_save is not None:
                    outer_engine.save_cache(str(args.outer_cache_save))
            elif gauge == "outer_zeta_proj":
                outer_engine = OuterNormalizerEngineZetaProjectRigorous(
                    sigma_ref=float(args.outer_sigma_ref),
                    T=float(args.outer_T),
                    n_intervals=int(args.outer_n),
                    P_cut=int(args.outer_P_cut),
                    det2_mode=str(args.arith_det2_mode),
                    k_max=int(args.arith_det2_kmax),
                    progress=bool(args.progress),
                    max_depth=int(args.outer_max_depth),
                    min_width=float(args.outer_min_width),
                    f_rel_tol=float(args.outer_proj_rel_tol),
                    f_abs_tol=float(args.outer_proj_abs_tol),
                    det2_cache_children=not bool(args.outer_proj_recompute_det2),
                )
                if args.outer_cache_load is not None:
                    outer_engine.load_cache(
                        str(args.outer_cache_load), strict=not bool(args.outer_cache_ignore_mismatch)
                    )
                else:
                    outer_engine.build()
                if args.outer_cache_save is not None:
                    outer_engine.save_cache(str(args.outer_cache_save))
            else:
                raise ValueError(f"--theta-certify does not support gauge={gauge!r} with --outer-mode rigorous")
        else:
            raise ValueError(f"unknown --outer-mode {outer_mode!r}")

    # Adaptive refinement parameters
    init_ns = int(args.theta_init_n_sigma)
    init_nt = int(args.theta_init_n_t)
    min_sigma_w = float(args.theta_min_sigma_width)
    min_t_w = float(args.theta_min_t_width)
    max_boxes = int(args.theta_max_boxes)
    time_limit = float(args.theta_time_limit)
    if init_ns <= 0 or init_nt <= 0:
        raise ValueError("--theta-init-n-sigma/--theta-init-n-t must be positive")
    if min_sigma_w <= 0 or min_t_w <= 0:
        raise ValueError("--theta-min-sigma-width/--theta-min-t-width must be positive")
    if max_boxes <= 0:
        raise ValueError("--theta-max-boxes must be positive")
    if time_limit < 0:
        raise ValueError("--theta-time-limit must be nonnegative")

    # Box representation: (sigma0, sigma1, t0, t1, depth)
    stack: List[Tuple[float, float, float, float, int]] = []

    # Running counters (can be resumed)
    processed = 0
    refined = 0
    passed = 0
    theta_hi = 0.0
    prev_elapsed = 0.0

    # Optional resume from a previous, time-capped run.
    if getattr(args, "theta_resume_in", None) is not None:
        data = _read_json(str(args.theta_resume_in))
        if not isinstance(data, dict) or data.get("kind") != "theta_certify_artifact":
            raise ValueError("--theta-resume-in is not a theta_certify_artifact JSON")
        resume = data.get("resume", {})
        if not isinstance(resume, dict) or "stack" not in resume:
            raise ValueError("--theta-resume-in missing resume.stack")
        st = resume.get("stack", [])
        if not isinstance(st, list):
            raise ValueError("resume.stack is not a list")
        for row in st:
            if not (isinstance(row, list) or isinstance(row, tuple)) or len(row) != 5:
                raise ValueError("resume.stack entries must be [sig0,sig1,t0,t1,depth]")
            sig0, sig1, t0, t1, depth = row
            stack.append((float(sig0), float(sig1), float(t0), float(t1), int(depth)))

        # Restore running counters (purely for reporting).
        try:
            prev = data.get("results", {})
            if isinstance(prev, dict):
                theta_hi = max(theta_hi, float(prev.get("theta_hi", 0.0) or 0.0))
                passed = int(prev.get("passed_boxes", 0) or 0)
                processed = int(prev.get("processed_boxes", 0) or 0)
                refined = int(prev.get("refined_splits", 0) or 0)
                prev_elapsed = float(prev.get("elapsed_seconds", 0.0) or 0.0)
        except Exception:
            # If anything goes wrong parsing counters, just resume with fresh counters.
            passed = 0
            processed = 0
            refined = 0
            theta_hi = 0.0
            prev_elapsed = 0.0
        print(f"[theta] RESUME: loaded {len(stack)} pending boxes from {args.theta_resume_in}")
    else:
        ds0 = (R.sigma_max - R.sigma_min) / init_ns
        dt0 = (R.t_max - R.t_min) / init_nt
        for i in range(init_ns):
            a = R.sigma_min + i * ds0
            b = R.sigma_min + (i + 1) * ds0
            for j in range(init_nt):
                c = R.t_min + j * dt0
                d = R.t_min + (j + 1) * dt0
                stack.append((a, b, c, d, 0))

    t_slice_start = time.time()
    min_abs_zeta = float("inf")
    min_abs_O = float("inf")
    zeta_contains0 = 0
    O_contains0 = 0
    failures: List[Dict[str, Any]] = []

    def split_box(sig0: float, sig1: float, t0: float, t1: float, depth: int, *, prefer_t: bool) -> None:
        nonlocal refined
        ws = sig1 - sig0
        wt = t1 - t0
        if ws <= 0 or wt <= 0:
            return
        # Choose split direction.
        split_t = prefer_t and (wt > min_t_w)
        if not split_t:
            # Otherwise split the larger dimension if possible.
            if wt > ws and wt > min_t_w:
                split_t = True
        if split_t:
            m = 0.5 * (t0 + t1)
            stack.append((sig0, sig1, m, t1, depth + 1))
            stack.append((sig0, sig1, t0, m, depth + 1))
        else:
            m = 0.5 * (sig0 + sig1)
            stack.append((m, sig1, t0, t1, depth + 1))
            stack.append((sig0, m, t0, t1, depth + 1))
        refined += 1

    print()
    print("[theta] certify |Theta_arith| < 1 on:", f"[{R.sigma_min},{R.sigma_max}] × [{R.t_min},{R.t_max}]")
    print("[theta] gauge:", gauge, "| det2_mode:", det2_mode, "| outer:", (None if outer_engine is None else type(outer_engine).__name__))
    print(
        "[theta] caps:",
        f"time_limit={time_limit}s, max_boxes={max_boxes}, min_sigma_w={min_sigma_w}, min_t_w={min_t_w}",
    )

    while stack:
        if time_limit > 0 and (time.time() - t_slice_start) >= time_limit:
            failures.append({"reason": "time_limit", "processed": processed, "stack": len(stack)})
            break
        if processed + len(stack) > max_boxes:
            failures.append({"reason": "max_boxes", "processed": processed, "stack": len(stack)})
            break

        sig0, sig1, t0, t1, depth = stack.pop()
        processed += 1
        ws = sig1 - sig0
        wt = t1 - t0
        s_box = acb_from_box(0.5 * (sig0 + sig1), 0.5 * ws, 0.5 * (t0 + t1), 0.5 * wt)

        # Denominators first: zeta / outer.
        need_zeta = gauge in {"raw_zeta", "outer_zeta", "outer_zeta_log", "outer_zeta_proj", "canonical_zeta"}
        need_xi = gauge in {"raw_xi", "outer_xi"}
        need_outer = gauge in {"outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj", "canonical_zeta"}

        zeta_val: acb | None = None
        if need_zeta:
            zeta_val = s_box.zeta()
            if _denom_contains_zero(zeta_val):
                zeta_contains0 += 1
                if ws <= min_sigma_w and wt <= min_t_w:
                    failures.append({"reason": "zeta_contains0_min_width", "sigma": [sig0, sig1], "t": [t0, t1]})
                    continue
                split_box(sig0, sig1, t0, t1, depth, prefer_t=True)
                continue
            min_abs_zeta = min(min_abs_zeta, _abs_lower_acb(zeta_val))

        xi_val: acb | None = None
        if need_xi:
            xi_val = xi_completed(s_box)
            ax = abs(xi_val)
            if ax.contains(0):
                # Conservatively refine (xi zeros are not expected in the far strip; this is for safety).
                if ws <= min_sigma_w and wt <= min_t_w:
                    failures.append({"reason": "xi_contains0_min_width", "sigma": [sig0, sig1], "t": [t0, t1]})
                    continue
                split_box(sig0, sig1, t0, t1, depth, prefer_t=True)
                continue

        O_val: acb | None = None
        if need_outer:
            assert outer_engine is not None
            if hasattr(outer_engine, "O_box"):
                O_val = getattr(outer_engine, "O_box")(s_box)
            else:
                # Fallback: try O(s_box) (may be midpoint-style and not rigorous).
                O_val = outer_engine.O(s_box)
            if _denom_contains_zero(O_val):
                O_contains0 += 1
                if ws <= min_sigma_w and wt <= min_t_w:
                    failures.append({"reason": "O_contains0_min_width", "sigma": [sig0, sig1], "t": [t0, t1]})
                    continue
                split_box(sig0, sig1, t0, t1, depth, prefer_t=True)
                continue
            min_abs_O = min(min_abs_O, _abs_lower_acb(O_val))

        # det2
        if det2_mode == "trunc":
            det2_val = det2_prime_trunc(s_box, primes)
        else:
            det2_val = det2_full_enclosure(s_box, P_cut=arith_P_cut, k_max=det2_kmax)

        # Certified Schur check in a numerically stable way:
        #
        # We avoid forming J(s) itself on wide boxes (division on wide enclosures can
        # catastrophically overestimate, and the ζ-gauge compensator B(s)=s/(s-1) has
        # a pole at s=1 even though the full ratio is regular there). Instead, use
        # the stable (X±Y) form:
        #   Θ = (X - Y)/(X + Y).
        #
        # For ζ-gauges (manuscript normalization B(s)=s/(s-1)):
        #   J = det2/(O ζ) * s/(s-1)
        #   Θ = ( 2 det2*s  -  (s-1) O ζ ) / ( 2 det2*s  +  (s-1) O ζ )
        # i.e. X = 2 det2*s and Y = (s-1)Oζ (or Y=(s-1)ζ if no outer).
        if gauge in {"raw_zeta", "outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
            assert zeta_val is not None
            X = acb(2) * det2_val * s_box
            Y0 = zeta_val if gauge == "raw_zeta" else (O_val * zeta_val)  # type: ignore[operator]
            Y = Y0 * (s_box - acb(1))
        else:
            assert xi_val is not None
            X = acb(2) * det2_val
            Y = xi_val if gauge == "raw_xi" else (O_val * xi_val)  # type: ignore[operator]

        denom = X + Y
        if _denom_contains_zero(denom):
            if ws <= min_sigma_w and wt <= min_t_w:
                failures.append({"reason": "theta_denom_contains0_min_width", "sigma": [sig0, sig1], "t": [t0, t1]})
                continue
            split_box(sig0, sig1, t0, t1, depth, prefer_t=True)
            continue

        try:
            theta = (X - Y) / denom
        except Exception:
            if ws <= min_sigma_w and wt <= min_t_w:
                failures.append({"reason": "theta_div_exception_min_width", "sigma": [sig0, sig1], "t": [t0, t1]})
                continue
            split_box(sig0, sig1, t0, t1, depth, prefer_t=True)
            continue

        # Certified strict disk inclusion: upper endpoint must be < 1.
        #
        # IMPORTANT: update `theta_hi` for *every* box where we successfully compute Θ,
        # even if the box does not certify. This avoids the misleading situation where
        # passed_boxes=0 but theta_hi stays at 0.
        abs_theta = abs(theta)
        try:
            abs_theta_u = float(abs_theta.upper())
            if not (abs_theta_u < 1.0):
                if ws <= min_sigma_w and wt <= min_t_w:
                    failures.append(
                        {
                            "reason": "theta_not_lt_one_min_width",
                            "sigma": [sig0, sig1],
                            "t": [t0, t1],
                            "abs_theta_upper": abs_theta_u,
                        }
                    )
                    continue
                split_box(sig0, sig1, t0, t1, depth, prefer_t=True)
                continue
        except Exception:
            if ws <= min_sigma_w and wt <= min_t_w:
                failures.append({"reason": "theta_abs_exception_min_width", "sigma": [sig0, sig1], "t": [t0, t1]})
                continue
            split_box(sig0, sig1, t0, t1, depth, prefer_t=True)
            continue

        # We computed Θ on this box; update diagnostic maximum upper bound.
        theta_hi = max(theta_hi, abs_theta_u)

        # Box certified.
        passed += 1

        if args.progress and (passed % int(args.theta_progress_every)) == 0:
            dt = prev_elapsed + (time.time() - t_slice_start)
            print(
                f"[theta] passed {passed} boxes (processed {processed}, stack {len(stack)}, "
                f"theta_hi≈{theta_hi:.6g}, elapsed {dt:.1f}s)"
            )

    ok = (len(failures) == 0) and (not stack)
    dt = prev_elapsed + (time.time() - t_slice_start)
    print()
    print(f"[theta] DONE. ok={ok}  passed_boxes={passed}  processed={processed}  refined_splits={refined}  elapsed={dt:.1f}s")
    if ok:
        print(f"[theta] theta_hi ≤ {theta_hi:.9g}  margin≥ {max(0.0, 1.0-theta_hi):.3g}")
    else:
        print(f"[theta] theta_hi_seen (diagnostic) ≤ {theta_hi:.9g}")
    if need_zeta:
        print(f"[theta] zeta: contains0_hits={zeta_contains0}, min_abs_lower≈{(None if min_abs_zeta==float('inf') else min_abs_zeta):.6g}")
    if need_outer:
        print(f"[theta] outer: contains0_hits={O_contains0}, min_abs_lower≈{(None if min_abs_O==float('inf') else min_abs_O):.6g}")
    if failures:
        print(f"[theta] failures: {len(failures)} (showing up to 5)")
        for f in failures[:5]:
            print("  -", f)

    out = {
        "kind": "theta_certify_artifact",
        "version": 1,
        "args": vars(args),
        "rect": {"sigma_min": R.sigma_min, "sigma_max": R.sigma_max, "t_min": R.t_min, "t_max": R.t_max},
        "results": {
            "ok": ok,
            "theta_hi": theta_hi,
            # Only meaningful as a certificate when ok=true.
            "margin_1_minus_theta_hi": (None if (not ok) or math.isnan(theta_hi) else (1.0 - theta_hi)),
            "passed_boxes": passed,
            "processed_boxes": processed,
            "refined_splits": refined,
            "elapsed_seconds": dt,
        },
        "denominators": {
            "need_zeta": need_zeta,
            "need_outer": need_outer,
            "zeta_contains0_hits": zeta_contains0,
            "O_contains0_hits": O_contains0,
            "min_abs_zeta_lower": (None if min_abs_zeta == float("inf") else min_abs_zeta),
            "min_abs_O_lower": (None if min_abs_O == float("inf") else min_abs_O),
        },
        "failures": failures,
        "resume": (None if ok else {"stack": [[a, b, c, d, depth] for (a, b, c, d, depth) in stack]}),
    }
    if args.theta_out is not None:
        _write_json(str(args.theta_out), out)
        print(f"[theta] wrote JSON: {args.theta_out}")
    return out


def theta_certify_cover(args: Any) -> None:
    """
    Run `theta_certify_rect` over a JSON-provided list of rectangles and write a single bundle.

    Cover JSON formats accepted:
      - a list of rect objects: [{"sigma_min":..., "sigma_max":..., "t_min":..., "t_max":...}, ...]
      - an object with key "rects": {"rects": [...]}  (extra keys ignored)
    """
    if args.theta_out is None:
        raise ValueError("--theta-certify-cover requires --theta-out <bundle.json>")

    cover = _read_json(str(args.theta_certify_cover))
    rects_in: Any
    if isinstance(cover, list):
        rects_in = cover
    elif isinstance(cover, dict) and isinstance(cover.get("rects"), list):
        rects_in = cover.get("rects")
    else:
        raise ValueError("--theta-certify-cover expects a JSON list of rects or an object with key 'rects'")

    rects: List[Rect] = []
    for r in rects_in:
        if not isinstance(r, dict):
            raise ValueError("cover rect entries must be objects")
        rects.append(
            Rect(
                sigma_min=float(r["sigma_min"]),
                sigma_max=float(r["sigma_max"]),
                t_min=float(r["t_min"]),
                t_max=float(r["t_max"]),
            )
        )

    artifacts: List[Dict[str, Any]] = []
    ok_all = True
    theta_hi_max = 0.0
    t0 = time.time()
    for i, R in enumerate(rects):
        print()
        print(f"[theta_cover] rectangle {i+1}/{len(rects)}: [{R.sigma_min},{R.sigma_max}] × [{R.t_min},{R.t_max}]")
        # Clone args and override rect fields; disable per-rect output unless user also asked for it.
        args2 = argparse.Namespace(**vars(args))
        args2.rect_sigma_min = R.sigma_min
        args2.rect_sigma_max = R.sigma_max
        args2.rect_t_min = R.t_min
        args2.rect_t_max = R.t_max
        args2.theta_resume_in = None  # cover runner does not manage per-rect resume automatically
        args2.theta_out = None
        art = theta_certify_rect(args2)
        artifacts.append(art)
        ok_i = bool(art.get("results", {}).get("ok"))
        ok_all = ok_all and ok_i
        try:
            theta_hi_max = max(theta_hi_max, float(art.get("results", {}).get("theta_hi", 0.0) or 0.0))
        except Exception:
            pass

    bundle = {
        "kind": "theta_certify_cover_bundle",
        "version": 1,
        "args": vars(args),
        "rects": [{"sigma_min": r.sigma_min, "sigma_max": r.sigma_max, "t_min": r.t_min, "t_max": r.t_max} for r in rects],
        "results": {
            "ok": ok_all,
            "theta_hi_max": theta_hi_max,
            "margin_1_minus_theta_hi_max": (None if (not ok_all) or math.isnan(theta_hi_max) else (1.0 - theta_hi_max)),
            "elapsed_seconds": (time.time() - t0),
        },
        "artifacts": artifacts,
    }
    _write_json(str(args.theta_out), bundle)
    print()
    print(f"[theta_cover] DONE. ok={ok_all}  theta_hi_max≤{theta_hi_max:.9g}  elapsed={bundle['results']['elapsed_seconds']:.1f}s")
    print(f"[theta_cover] wrote JSON bundle: {args.theta_out}")


def certify_margin(cert: Certificate, R: Rect, n_sigma: int, n_t: int) -> Tuple[float, float]:
    """
    Compute a rigorous lower bound on m_R = inf_R Re(2 J_cert,N(s)).
    Returns (m_lower, m_upper) where m_R ∈ [m_lower, m_upper].
    """
    boxes = cover_rect(R, n_sigma=n_sigma, n_t=n_t)
    m_lo = float("inf")
    m_hi = float("inf")
    for box in boxes:
        twoJ = cert.J_cert(box) * acb(2)
        re = twoJ.real
        lo = arb_lower(re)
        hi = arb_upper(re)
        if math.isnan(lo) or math.isnan(hi):
            return float("nan"), float("nan")
        m_lo = min(m_lo, lo)
        m_hi = min(m_hi, hi)
    return m_lo, m_hi


def certify_sup_error(
    cert: Certificate,
    primes: List[int],
    R: Rect,
    n_sigma: int,
    n_t: int,
    gauge: str,
    outer: OuterNormalizerEngineLike | None = None,
) -> Tuple[float, float]:
    """
    Certified enclosure of sup_R |J_arith - J_cert|.

    Returns (sup_lower, sup_upper) for the enclosure; sup_lower is usually 0.
    """
    boxes = cover_rect(R, n_sigma=n_sigma, n_t=n_t)
    sup_lo = 0.0
    sup_hi = 0.0
    for box in boxes:
        Jc = cert.J_cert(box)
        if gauge == "raw_xi":
            Ja = J_arith_raw_xi(box, primes)
        elif gauge == "raw_zeta":
            Ja = J_arith_raw_zeta(box, primes)
        elif gauge == "outer_xi":
            if outer is None:
                raise ValueError("outer_xi gauge requires an OuterNormalizerEngine")
            # J_arith := det2_P(s) / (O(s) ξ(s))
            det2P = det2_prime_trunc(box, primes)
            Ja = det2P / (outer.O(box) * xi_completed(box))
        elif gauge in {"outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
            if outer is None:
                raise ValueError("outer_zeta gauge requires an OuterNormalizerEngine-like")
            Ja = J_arith_outer_zeta(box, primes, outer)
        else:
            raise ValueError(f"unknown gauge: {gauge}")
        diff = Ja - Jc
        r = abs(diff)  # arb (radius+mid) for modulus
        hi = arb_upper(r)
        if math.isnan(hi):
            return float("nan"), float("nan")
        sup_hi = max(sup_hi, hi)
    return sup_lo, sup_hi


def certify_margin_and_sup_error(
    cert: Certificate,
    primes: List[int],
    R: Rect,
    n_sigma: int,
    n_t: int,
    gauge: str,
    outer: OuterNormalizerEngineLike | None = None,
    eval_mode: str = "rigorous",
    *,
    progress: bool = False,
    progress_every: int = 2000,
    det2_mode: str = "trunc",
    det2_P_cut: int | None = None,
    det2_k_max: int = 40,
) -> Tuple[float, float, float, float, float, float]:
    """
    Compute, in a *single* rectangle-cover pass:
      - a rigorous enclosure for m_R = inf_R Re(2 J_cert), and
      - a rigorous enclosure for sup_R |J_arith - J_cert|.

    Also compute two Cayley/Schur diagnostics for the arithmetic side:
      - theta_hi: sup_R |Θ_arith| where Θ=(2J-1)/(2J+1)
      - phi_hi:   sup_R |φ| where φ=(Θ_arith-Θ_cert)/(1-conj(Θ_cert)Θ_arith)

    This avoids the biggest cost in repeated runs: evaluating J_cert (matrix inversions)
    twice for the same cover (once for margin, once for error).
    """
    boxes = cover_rect(R, n_sigma=n_sigma, n_t=n_t)
    m_lo = float("inf")
    m_hi = float("inf")
    err_lo = 0.0
    err_hi = 0.0
    theta_hi = 0.0
    phi_hi = 0.0

    if eval_mode not in {"rigorous", "midpoint"}:
        raise ValueError(f"unknown eval_mode: {eval_mode}")
    if det2_mode not in {"trunc", "enclosure"}:
        raise ValueError(f"unknown det2_mode: {det2_mode}")
    if det2_mode == "enclosure" and det2_P_cut is None:
        raise ValueError("det2_mode='enclosure' requires det2_P_cut")

    t0 = time.time()
    for bi, box in enumerate(boxes, start=1):
        s_eval = box if eval_mode == "rigorous" else acb_midpoint(box)
        # Certificate side (shared)
        Jc = cert.J_cert(s_eval)
        # Cayley( J_cert ) is well-defined on the far half-plane, but guard anyway.
        theta_c = cayley_theta_from_J(Jc) if not _denom_contains_zero(Jc * acb(2) + acb(1)) else acb(0, arb(0, 0))
        twoJc = Jc * acb(2)
        re = twoJc.real
        lo = arb_lower(re)
        hi = arb_upper(re)
        if math.isnan(lo) or math.isnan(hi):
            return (
                float("nan"),
                float("nan"),
                float("nan"),
                float("nan"),
                float("nan"),
                float("nan"),
            )
        m_lo = min(m_lo, lo)
        m_hi = min(m_hi, hi)

        # Arithmetic side
        # Choose det2 evaluation mode
        if det2_mode == "trunc":
            det2_val = det2_prime_trunc(s_eval, primes)
        else:
            det2_val = det2_full_enclosure(
                s_eval,
                P_cut=int(det2_P_cut),
                k_max=int(det2_k_max),
                progress=progress,
            )

        if gauge == "raw_xi":
            Ja = det2_val / xi_completed(s_eval)
        elif gauge == "raw_zeta":
            Ja = det2_val / s_eval.zeta() * compensator_B(s_eval)
        elif gauge == "outer_xi":
            if outer is None:
                raise ValueError("outer_xi gauge requires an OuterNormalizerEngine-like")
            if eval_mode == "midpoint" and hasattr(outer, "O_midpoint"):
                O_val = getattr(outer, "O_midpoint")(s_eval)
            else:
                O_val = outer.O(s_eval)
                if eval_mode == "midpoint":
                    O_val = acb_midpoint(O_val)
            Ja = det2_val / (O_val * xi_completed(s_eval))
        elif gauge in {"outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
            if outer is None:
                raise ValueError("outer_zeta gauge requires an OuterNormalizerEngine-like")
            if eval_mode == "midpoint" and hasattr(outer, "O_midpoint"):
                O_val = getattr(outer, "O_midpoint")(s_eval)
            elif eval_mode == "rigorous" and hasattr(outer, "O_box"):
                O_val = getattr(outer, "O_box")(s_eval)
            else:
                O_val = outer.O(s_eval)
                if eval_mode == "midpoint":
                    O_val = acb_midpoint(O_val)
            zeta_val = s_eval.zeta()
            if eval_mode == "midpoint":
                zeta_val = acb_midpoint(zeta_val)
            Ja = det2_val / (O_val * zeta_val) * compensator_B(s_eval)
        else:
            raise ValueError(f"unknown gauge: {gauge}")

        diff = Ja - Jc
        r = abs(diff)
        r_hi = arb_upper(r)
        if math.isnan(r_hi):
            return (
                float("nan"),
                float("nan"),
                float("nan"),
                float("nan"),
                float("nan"),
                float("nan"),
            )
        err_hi = max(err_hi, r_hi)

        # Cayley/Schur diagnostics (arith side)
        theta_a: acb | None = None
        th_hi = float("inf")
        denom_theta = Ja * acb(2) + acb(1)
        if not _denom_contains_zero(denom_theta):
            theta_a = cayley_theta_from_J(Ja)
            th_hi = _safe_abs_upper(theta_a)
        theta_hi = max(theta_hi, th_hi)

        # Pseudohyperbolic witness relative to Θ_cert. If the denominator encloses 0,
        # we cannot certify φ, so treat as +inf (automatic failure for pass_metric=pseudohyp).
        ph_hi = float("inf")
        if theta_a is not None:
            denom_phi = acb(1) - theta_c.conjugate() * theta_a
            if not _denom_contains_zero(denom_phi):
                phi = (theta_a - theta_c) / denom_phi
                ph_hi = _safe_abs_upper(phi)
        phi_hi = max(phi_hi, ph_hi)

        if progress and progress_every > 0 and (bi % progress_every == 0):
            dt = time.time() - t0
            _progress(
                f"[cover] boxes {bi}/{len(boxes)} (elapsed {dt:.1f}s, m_lo≈{m_lo:.3g}, "
                f"err_hi≈{err_hi:.3g}, |Theta|_hi≈{theta_hi:.3g}, |phi|_hi≈{phi_hi:.3g})",
                enabled=True,
            )

    return m_lo, m_hi, err_lo, err_hi, theta_hi, phi_hi


def certify_attachment_refine(
    cert: Certificate,
    primes: List[int],
    R: Rect,
    n_sigma: int,
    n_t: int,
    gauge: str,
    outer: OuterNormalizerEngineLike | None = None,
    max_refines: int = 6,
    eval_mode: str = "rigorous",
    *,
    progress: bool = False,
    det2_mode: str = "trunc",
    det2_P_cut: int | None = None,
    det2_k_max: int = 40,
) -> Tuple[float, float, float, float, float, float, int, int]:
    """
    Refine the rectangle cover until we can (a) certify a positive margin lower bound
    and (b) obtain a finite sup error enclosure.

    Returns:
      (m_lo, m_hi, err_lo, err_hi, theta_hi, phi_hi, ns, nt)
    """
    ns = n_sigma
    nt = n_t
    if eval_mode == "midpoint":
        m_lo, m_hi, err_lo, err_hi, theta_hi, phi_hi = certify_margin_and_sup_error(
            cert,
            primes,
            R,
            n_sigma=ns,
            n_t=nt,
            gauge=gauge,
            outer=outer,
            eval_mode="midpoint",
            progress=progress,
            det2_mode=det2_mode,
            det2_P_cut=det2_P_cut,
            det2_k_max=det2_k_max,
        )
        return m_lo, m_hi, err_lo, err_hi, theta_hi, phi_hi, ns, nt

    if max_refines < 0:
        raise ValueError("max_refines must be nonnegative")

    # Always run at least one pass (even if max_refines=0), then refine if needed.
    for _ in range(max_refines + 1):
        m_lo, m_hi, err_lo, err_hi, theta_hi, phi_hi = certify_margin_and_sup_error(
            cert,
            primes,
            R,
            n_sigma=ns,
            n_t=nt,
            gauge=gauge,
            outer=outer,
            eval_mode=eval_mode,
            progress=progress,
            det2_mode=det2_mode,
            det2_P_cut=det2_P_cut,
            det2_k_max=det2_k_max,
        )
        if (
            not (
                math.isnan(m_lo)
                or math.isnan(m_hi)
                or math.isnan(err_hi)
                or math.isnan(theta_hi)
                or math.isnan(phi_hi)
            )
            and m_lo > 0.0
            and err_hi >= 0.0
        ):
            return m_lo, m_hi, err_lo, err_hi, theta_hi, phi_hi, ns, nt
        ns *= 2
        nt *= 2
    return m_lo, m_hi, err_lo, err_hi, theta_hi, phi_hi, ns, nt


def certify_margin_refine(
    cert: Certificate,
    R: Rect,
    n_sigma: int,
    n_t: int,
    max_refines: int = 6,
) -> Tuple[float, float, int, int]:
    """
    Heuristic refinement wrapper: interval matrix inversions can yield NaNs or
    overly-wide enclosures on coarse boxes. We refine until we get a finite,
    positive lower bound (as expected for a strict Herglotz certificate).
    """
    ns = n_sigma
    nt = n_t
    for _ in range(max_refines):
        m_lo, m_hi = certify_margin(cert, R, n_sigma=ns, n_t=nt)
        if not (math.isnan(m_lo) or math.isnan(m_hi)) and m_lo > 0.0:
            return m_lo, m_hi, ns, nt
        ns *= 2
        nt *= 2
    return m_lo, m_hi, ns, nt


# -------------------------
# Phase audit (diagnostic)
# -------------------------


def _unwrap_phases(phases: List[float]) -> List[float]:
    """
    Simple 2π-unwrapping for a list of principal arguments in (-π, π].
    """
    if not phases:
        return []
    two_pi = 2.0 * math.pi
    out = [float(phases[0])]
    offset = 0.0
    prev = float(phases[0])
    for p in phases[1:]:
        p = float(p)
        dp = p - prev
        if dp > math.pi:
            offset -= two_pi
        elif dp < -math.pi:
            offset += two_pi
        out.append(p + offset)
        prev = p
    return out


def _sliding_window_max_osc(
    t_vals: List[float],
    phase_unwrapped: List[float],
    window_len: float,
) -> Tuple[float, Dict[str, Any] | None]:
    """
    Return (max_osc, witness) where max_osc is
      max_{windows [t0,t0+window_len]} (max phase - min phase)
    computed on the discrete grid.

    witness includes t_start/t_end and the argmin/argmax sample locations.
    """
    n = len(t_vals)
    if n == 0:
        return 0.0, None
    if n != len(phase_unwrapped):
        raise ValueError("t_vals and phase_unwrapped must have the same length")
    if window_len <= 0:
        return 0.0, {"reason": "window_len<=0", "max_osc": 0.0}

    minq: deque[int] = deque()
    maxq: deque[int] = deque()

    j = 0
    best_osc = -float("inf")
    best: Dict[str, Any] | None = None

    eps = 1e-15

    for i in range(n):
        # Extend window right endpoint.
        t_end = t_vals[i] + window_len + eps
        while j < n and t_vals[j] <= t_end:
            v = phase_unwrapped[j]
            while minq and phase_unwrapped[minq[-1]] >= v:
                minq.pop()
            minq.append(j)
            while maxq and phase_unwrapped[maxq[-1]] <= v:
                maxq.pop()
            maxq.append(j)
            j += 1

        # Remove indices left of the window.
        while minq and minq[0] < i:
            minq.popleft()
        while maxq and maxq[0] < i:
            maxq.popleft()

        if not minq or not maxq:
            continue

        osc = phase_unwrapped[maxq[0]] - phase_unwrapped[minq[0]]
        if osc > best_osc:
            best_osc = osc
            j_last = max(i, j - 1)
            best = {
                "t_start": float(t_vals[i]),
                "t_end": float(t_vals[j_last]),
                "i_start": int(i),
                "i_end": int(j_last),
                "t_argmin": float(t_vals[minq[0]]),
                "t_argmax": float(t_vals[maxq[0]]),
                "argmin": float(phase_unwrapped[minq[0]]),
                "argmax": float(phase_unwrapped[maxq[0]]),
                "osc": float(osc),
            }

    if best_osc == -float("inf"):
        return 0.0, {"reason": "no windows", "max_osc": 0.0}
    return float(best_osc), best


def _arith_eval_point_mid(
    *,
    s: acb,
    primes: List[int],
    gauge: str,
    outer: OuterNormalizerEngineLike | None,
    det2_mode: str,
    det2_P_cut: int,
    det2_k_max: int,
) -> acb:
    """
    Midpoint-style point evaluation for the arithmetic gauge at an exact point `s`.

    This is intended for diagnostics (phase audit): we use midpoints for ζ / outers to
    avoid catastrophic overestimation.
    """
    if det2_mode not in {"trunc", "enclosure"}:
        raise ValueError(f"unknown det2_mode: {det2_mode}")

    need_zeta = gauge in {"raw_zeta", "outer_zeta", "outer_zeta_log", "outer_zeta_proj", "canonical_zeta"}
    need_xi = gauge in {"raw_xi", "outer_xi"}
    need_outer = gauge in {"outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj", "canonical_zeta"}

    # det2
    if det2_mode == "trunc":
        det2_val = det2_prime_trunc(s, primes)
    else:
        det2_val = acb_midpoint(det2_full_enclosure(s, P_cut=int(det2_P_cut), k_max=int(det2_k_max)))

    zeta_val: acb | None = None
    if need_zeta:
        zeta_val = acb_midpoint(s.zeta())

    xi_val: acb | None = None
    if need_xi:
        xi_val = acb_midpoint(xi_completed(s))

    O_val: acb | None = None
    if need_outer:
        if outer is None:
            raise ValueError(f"gauge {gauge} requires an outer engine, but outer=None")
        if hasattr(outer, "O_midpoint"):
            O_val = getattr(outer, "O_midpoint")(s)
        else:
            O_val = acb_midpoint(outer.O(s))

    if gauge == "raw_xi":
        assert xi_val is not None
        return det2_val / xi_val
    if gauge == "raw_zeta":
        assert zeta_val is not None
        return det2_val / zeta_val * compensator_B(s)
    if gauge == "outer_xi":
        assert xi_val is not None and O_val is not None
        return det2_val / (O_val * xi_val)
    if gauge in {"outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
        assert zeta_val is not None and O_val is not None
        return det2_val / (O_val * zeta_val) * compensator_B(s)
    raise ValueError(f"unknown gauge: {gauge}")


def audit_phase(args: Any) -> None:
    """
    Empirical phase oscillation audit for the selected arithmetic gauge.

    For each sigma in `--audit-phase-sigmas`, sample t on a uniform grid and compute
      max_{t-windows of length 1/sigma} (max arg - min arg)
    after 2π-unwrapping. This is intended as a sanity check for the "phase budget".
    """
    gauge = str(args.arith_gauge)
    det2_mode = str(args.arith_det2_mode)

    # Pick sigmas list.
    sigmas: List[float]
    if args.audit_phase_sigmas is not None and str(args.audit_phase_sigmas).strip():
        sigmas = _parse_csv_floats(str(args.audit_phase_sigmas))
    elif args.scan_sigma is not None:
        sigmas = [float(args.scan_sigma)]
    else:
        a = float(args.rect_sigma_min)
        b = float(args.rect_sigma_max)
        sigmas = [a, 0.5 * (a + b), b]

    if not sigmas:
        raise ValueError("audit_phase produced an empty sigma list")
    for sig in sigmas:
        if sig <= 0:
            raise ValueError("audit sigmas must be positive")

    t_min = float(args.rect_t_min if args.audit_phase_t_min is None else args.audit_phase_t_min)
    t_max = float(args.rect_t_max if args.audit_phase_t_max is None else args.audit_phase_t_max)
    dt = float(args.audit_phase_t_step)
    if dt <= 0:
        raise ValueError("--audit-phase-t-step must be positive")
    if t_max < t_min:
        raise ValueError("--audit-phase-t-max must be >= --audit-phase-t-min")

    obj = str(args.audit_phase_object)
    if obj not in {"J", "theta"}:
        raise ValueError("--audit-phase-object must be 'J' or 'theta'")

    budget = None if args.audit_phase_budget_rad is None else float(args.audit_phase_budget_rad)
    arith_P_cut = int(args.P if args.arith_P_cut is None else args.arith_P_cut)
    primes = [p for p in cached_primes_upto(arith_P_cut) if p >= 2]

    # Build/load outer engine if required.
    outer_engine: OuterNormalizerEngineLike | None = None
    if gauge in {"outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
        if gauge == "outer_zeta_log" and str(args.outer_mode) != "midpoint":
            raise ValueError("--arith-gauge outer_zeta_log requires --outer-mode midpoint (diagnostic only)")

        print(
            f"[audit_phase] building O(s) from boundary line Re(s)={args.outer_sigma_ref} "
            f"with T={args.outer_T}, n={args.outer_n}, P_cut={args.outer_P_cut} ..."
        )

        if str(args.outer_mode) == "rigorous":
            if gauge == "outer_xi":
                outer_engine = OuterNormalizerEngine(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_intervals=args.outer_n,
                    P_cut=args.outer_P_cut,
                )
            elif gauge == "outer_zeta":
                outer_engine = OuterNormalizerEngineZetaRigorous(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_intervals=args.outer_n,
                    P_cut=args.outer_P_cut,
                    k_max=int(args.arith_det2_kmax),
                    progress=bool(args.progress),
                    max_depth=int(args.outer_max_depth),
                    min_width=float(args.outer_min_width),
                )
            elif gauge == "outer_zeta_proj":
                outer_engine = OuterNormalizerEngineZetaProjectRigorous(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_intervals=args.outer_n,
                    P_cut=args.outer_P_cut,
                    det2_mode=str(args.arith_det2_mode),
                    k_max=int(args.arith_det2_kmax),
                    progress=bool(args.progress),
                    max_depth=int(args.outer_max_depth),
                    min_width=float(args.outer_min_width),
                    f_rel_tol=float(args.outer_proj_rel_tol),
                    f_abs_tol=float(args.outer_proj_abs_tol),
                    det2_cache_children=not bool(args.outer_proj_recompute_det2),
                )
            else:
                raise ValueError(f"{gauge} is only supported with --outer-mode midpoint")

            # Cache load/build
            if args.outer_cache_load is not None:
                print(f"[audit_phase] loading outer cache {args.outer_cache_load} ...")
                assert hasattr(outer_engine, "load_cache")
                outer_engine.load_cache(
                    args.outer_cache_load, strict=not bool(args.outer_cache_ignore_mismatch)
                )
                n_leaf = len(getattr(outer_engine, "t_intervals", None) or [])
                print(f"[audit_phase] loaded. leaves={n_leaf}")
            else:
                outer_engine.build()
                n_leaf = len(getattr(outer_engine, "t_intervals", None) or [])
                print(f"[audit_phase] built. leaves={n_leaf}")
        else:
            # midpoint outer
            if gauge == "outer_xi":
                outer_engine = OuterNormalizerEngineMidpoint(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                )
            elif gauge == "outer_zeta":
                outer_engine = OuterNormalizerEngineMidpointZeta(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                )
            elif gauge == "outer_zeta_log":
                outer_engine = OuterNormalizerEngineMidpointZetaLog(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                    unwrap=True,
                )
            else:
                outer_engine = OuterNormalizerEngineMidpointZetaProject(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                )

            if args.outer_cache_load is not None:
                print(f"[audit_phase] loading outer cache {args.outer_cache_load} ...")
                assert hasattr(outer_engine, "load_cache")
                outer_engine.load_cache(
                    args.outer_cache_load, strict=not bool(args.outer_cache_ignore_mismatch)
                )
                n_nodes = len(getattr(outer_engine, "t_nodes", None) or [])
                print(f"[audit_phase] loaded. nodes={n_nodes}")
            else:
                outer_engine.build()
                n_nodes = len(getattr(outer_engine, "t_nodes", None) or [])
                print(f"[audit_phase] built. nodes={n_nodes}")

        if args.outer_cache_save is not None:
            if not hasattr(outer_engine, "save_cache"):
                raise ValueError("--outer-cache-save is not supported for this outer engine")
            outer_engine.save_cache(args.outer_cache_save)
            print(f"[audit_phase] saved outer cache to {args.outer_cache_save}")

    # Audit loop.
    print()
    print("[audit_phase] gauge:", gauge, "| det2_mode:", det2_mode, "| object:", obj)
    print("[audit_phase] t-range:", f"[{t_min},{t_max}]", "step", dt)
    if budget is not None:
        print(f"[audit_phase] budget_rad: {budget:.12g}")
    print()

    results: List[Dict[str, Any]] = []
    for sigma in sigmas:
        window_len = 1.0 / float(sigma)
        t_vals: List[float] = []
        phases: List[float] = []
        min_abs = float("inf")
        zero_hits = 0

        t = t_min
        while t <= t_max + 1e-15:
            s = acb(float(sigma), float(t))
            Ja = _arith_eval_point_mid(
                s=s,
                primes=primes,
                gauge=gauge,
                outer=outer_engine,
                det2_mode=det2_mode,
                det2_P_cut=arith_P_cut,
                det2_k_max=int(args.arith_det2_kmax),
            )

            z = Ja
            if obj == "theta":
                denom = Ja * acb(2) + acb(1)
                if _denom_contains_zero(denom):
                    raise ValueError("Theta undefined at a sample point: 2J+1 encloses 0")
                z = cayley_theta_from_J(Ja)

            az = abs(z)
            if az.contains(0):
                zero_hits += 1
            min_abs = min(min_abs, max(0.0, arb_lower(az)))

            re = float(z.real)
            im = float(z.imag)
            phases.append(math.atan2(im, re))
            t_vals.append(float(t))
            t += dt

        phases_u = _unwrap_phases(phases)
        max_osc, witness = _sliding_window_max_osc(t_vals, phases_u, window_len=window_len)
        ok = None if budget is None else (max_osc <= budget)

        print(
            f"[audit_phase] sigma={sigma:.6g}  window_len=1/sigma={window_len:.6g}  "
            f"max_osc≈{max_osc:.6g}  min_abs≈{min_abs:.6g}  abs_contains0_hits={zero_hits}"
            + ("" if ok is None else f"  PASS? {ok}")
        )
        if witness is not None:
            print(
                f"             worst window t∈[{witness.get('t_start'):.6g},{witness.get('t_end'):.6g}]  "
                f"argmin@t={witness.get('t_argmin'):.6g}  argmax@t={witness.get('t_argmax'):.6g}"
            )

        results.append(
            {
                "sigma": float(sigma),
                "window_len": float(window_len),
                "t_min": float(t_min),
                "t_max": float(t_max),
                "t_step": float(dt),
                "n_points": int(len(t_vals)),
                "object": obj,
                "max_osc": float(max_osc),
                "min_abs_lower": (None if min_abs == float("inf") else float(min_abs)),
                "abs_contains0_hits": int(zero_hits),
                "budget_rad": budget,
                "pass": ok,
                "witness": witness,
            }
        )

    if args.audit_phase_out is not None:
        _write_json(
            str(args.audit_phase_out),
            {
                "kind": "audit_phase_artifact",
                "version": 1,
                "args": vars(args),
                "results": results,
            },
        )
        print(f"[audit_phase] wrote artifact to {args.audit_phase_out}")


# -------------------------
# Arithmetic Pick minor (diagnostic)
# -------------------------


def _s_from_disk_z(sigma0: float, z: complex) -> complex:
    """
    Inverse map for z_{sigma0}(s) = (s - sigma0) / (s + sigma0):
      s(z) = sigma0 * (1+z)/(1-z)
    """
    return sigma0 * (1.0 + z) / (1.0 - z)


def _s_from_disk_z_far_chart(sigma0: float, z: acb) -> acb:
    """
    Disk chart used in `Riemann-near-field-fully-close.tex` (Definition def:disk-map-far):

      s_{sigma0}(z) = sigma0 + (1+z)/(1-z).

    This is the inverse map for
      z_{sigma0}(s) = (s-(sigma0+1))/(s-(sigma0-1)),
    sending {Re(s) > sigma0} to the unit disk and s_{sigma0}(0)=sigma0+1.
    """
    sig = acb(arb(float(sigma0)))
    return sig + (acb(1) + z) / (acb(1) - z)


def _arith_eval_point_rig(
    *,
    s: acb,
    primes: List[int],
    gauge: str,
    outer: OuterNormalizerEngineLike | None,
    det2_mode: str,
    det2_P_cut: int,
    det2_k_max: int,
) -> acb:
    """
    Rigorous point evaluation (ball arithmetic) for the arithmetic gauge at an `acb` point `s`.

    This is the certified counterpart to `_arith_eval_point_mid`. It is designed for use in
    Pick-certificate / Taylor-coefficient computations where we need certified enclosures at
    exact points (zero-radius `acb` inputs, though intermediate operations still produce balls).
    """
    if det2_mode not in {"trunc", "enclosure"}:
        raise ValueError(f"unknown det2_mode: {det2_mode}")

    need_zeta = gauge in {"raw_zeta", "outer_zeta", "outer_zeta_log", "outer_zeta_proj", "canonical_zeta"}
    need_xi = gauge in {"raw_xi", "outer_xi"}
    need_outer = gauge in {"outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj", "canonical_zeta"}

    # det2
    if det2_mode == "trunc":
        # WARNING: trunc is not a certified enclosure for the prime tail. Kept for diagnostics.
        det2_val = det2_prime_trunc(s, primes)
    else:
        det2_val = det2_full_enclosure(s, P_cut=int(det2_P_cut), k_max=int(det2_k_max))

    zeta_val: acb | None = None
    if need_zeta:
        zeta_val = s.zeta()

    xi_val: acb | None = None
    if need_xi:
        xi_val = xi_completed(s)

    O_val: acb | None = None
    if need_outer:
        if outer is None:
            raise ValueError(f"gauge {gauge} requires an outer engine, but outer=None")
        # For some engines we have a dedicated box-evaluation routine that is much
        # tighter (and avoids exp(...) enclosures swallowing 0 on wide balls).
        if hasattr(outer, "O_box"):
            O_val = getattr(outer, "O_box")(s)
        else:
            O_val = outer.O(s)
        if abs(O_val).contains(0):
            raise ValueError("outer enclosure contains 0 at evaluation point; cannot divide safely")

    # J_arith
    if gauge == "raw_xi":
        assert xi_val is not None
        if abs(xi_val).contains(0):
            raise ValueError("xi enclosure contains 0 at evaluation point; cannot divide safely")
        Ja = det2_val / xi_val
    elif gauge == "raw_zeta":
        assert zeta_val is not None
        if abs(zeta_val).contains(0):
            raise ValueError("zeta enclosure contains 0 at evaluation point; cannot divide safely")
        Ja = det2_val / zeta_val * compensator_B(s)
    elif gauge == "outer_xi":
        assert xi_val is not None and O_val is not None
        if abs(xi_val).contains(0):
            raise ValueError("xi enclosure contains 0 at evaluation point; cannot divide safely")
        Ja = det2_val / (O_val * xi_val)
    elif gauge in {"outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
        assert zeta_val is not None and O_val is not None
        if abs(zeta_val).contains(0):
            raise ValueError("zeta enclosure contains 0 at evaluation point; cannot divide safely")
        Ja = det2_val / (O_val * zeta_val) * compensator_B(s)
    else:
        raise ValueError(f"unknown gauge: {gauge}")

    return Ja


def _theta_arith_point_rig(
    *,
    s: acb,
    primes: List[int],
    gauge: str,
    outer: OuterNormalizerEngineLike | None,
    det2_mode: str,
    det2_P_cut: int,
    det2_k_max: int,
) -> acb:
    """
    Certified evaluation of Θ(s) using a stable (X±Y) geometry.

    We avoid forming J(s)=det2/(Oζ)*B (or det2/ξ) directly since division on wide boxes can
    easily introduce catastrophic overestimation (or enclosure of 0). Instead we compute
      Θ = (X - Y)/(X + Y)
    where:
      - ζ-gauges (manuscript B(s)=s/(s-1)):
            Θ = ( 2 det2(s) * s  -  (s-1) O(s) ζ(s) ) / ( 2 det2(s) * s  +  (s-1) O(s) ζ(s) )
        i.e. X = 2 det2(s) * s and Y = (s-1) O(s) ζ(s) (or Y=(s-1)ζ(s) if no outer).
        This avoids ever evaluating B(s)=s/(s-1) on wide balls that might intersect s=1.
      - ξ-gauges: X = 2 det2(I-A)(s),         Y = O(s) * ξ(s)   (or Y=ξ(s) if no outer)
    """
    if det2_mode not in {"trunc", "enclosure"}:
        raise ValueError(f"unknown det2_mode: {det2_mode}")

    # det2
    if det2_mode == "trunc":
        det2_val = det2_prime_trunc(s, primes)
    else:
        det2_val = det2_full_enclosure(s, P_cut=int(det2_P_cut), k_max=int(det2_k_max))

    need_zeta = gauge in {"raw_zeta", "outer_zeta", "outer_zeta_log", "outer_zeta_proj", "canonical_zeta"}
    need_xi = gauge in {"raw_xi", "outer_xi"}
    need_outer = gauge in {"outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj", "canonical_zeta"}
    if not (need_zeta or need_xi):
        raise ValueError(f"unknown gauge: {gauge}")

    O_val: acb | None = None
    if need_outer:
        if outer is None:
            raise ValueError(f"gauge {gauge} requires an outer engine, but outer=None")
        O_val = outer.O(s)

    if need_zeta:
        zeta_val = s.zeta()
        X = det2_val * acb(2) * s
        Y0 = zeta_val if O_val is None else (O_val * zeta_val)
        Y = Y0 * (s - acb(1))
    else:
        xi_val = xi_completed(s)
        X = det2_val * acb(2)
        Y = xi_val if O_val is None else (O_val * xi_val)

    denom = X + Y
    if _denom_contains_zero(denom):
        raise ValueError("X+Y enclosure contains 0 at evaluation point; Theta undefined")
    return (X - Y) / denom


def _acb_from_polar(r: float, theta: arb) -> acb:
    """
    Return acb for r * exp(i*theta) with ball arithmetic.
    """
    z = acb(arb(0), theta).exp()  # exp(i theta)
    return acb(arb(float(r))) * z


def _max_abs_on_z_disk_via_s_box(
    *,
    sigma0: float,
    z_radius: float,
    primes: List[int],
    gauge: str,
    outer: OuterNormalizerEngineLike | None,
    det2_mode: str,
    det2_P_cut: int,
    det2_k_max: int,
    progress: bool = False,
    box_depth: int = 8,
    arc_n0: int = 32,
    arc_split_depth: int = 8,
    arc_refine_depth: int = 10,
    eval_budget: int = 50000,
) -> float:
    """
    Compute a certified (potentially very conservative) upper bound for
      sup_{|z|<=z_radius} |theta_sigma0(z)|
    by enclosing the image of the z-disk under s_{sigma0}(z) in an s-rectangle and
    evaluating theta on that single s-box.

    This is used only as a coefficient-aliasing bound in pick certification.
    """
    r = float(z_radius)
    if not (0.0 < r < 1.0):
        raise ValueError("z_radius must be in (0,1)")

    # For w(z)=(1+z)/(1-z), we have:
    #   Re w = (1-|z|^2)/|1-z|^2,  |Im w| = 2|Im z|/|1-z|^2.
    # Thus for |z|<=r:
    #   Re w ∈ [ (1-r)/(1+r), (1+r)/(1-r) ]
    #   |Im w| <= 2r/(1-r)^2.
    re_lo = (1.0 - r) / (1.0 + r)
    re_hi = (1.0 + r) / (1.0 - r)
    im_hi = (2.0 * r) / ((1.0 - r) * (1.0 - r))

    s_re_lo = float(sigma0) + re_lo
    s_re_hi = float(sigma0) + re_hi
    s_im_mid = 0.0
    s_im_rad = im_hi

    s_box = acb_from_box(
        re_mid=0.5 * (s_re_lo + s_re_hi),
        re_rad=0.5 * (s_re_hi - s_re_lo),
        im_mid=s_im_mid,
        im_rad=s_im_rad,
    )

    evals = 0

    def _refine_abs_bound(box: acb, depth: int) -> float:
        nonlocal evals
        evals += 1
        if evals > int(eval_budget):
            raise RuntimeError(
                f"M_bound certification budget exceeded ({eval_budget} evals). "
                "Try smaller --pick-rho-bound or lower --pick-N / --pick-coeff-count, "
                "or increase the eval_budget in _max_abs_on_z_disk_via_s_box."
            )
        try:
            th = _theta_arith_point_rig(
                s=box,
                primes=primes,
                gauge=gauge,
                outer=outer,
                det2_mode=det2_mode,
                det2_P_cut=det2_P_cut,
                det2_k_max=det2_k_max,
            )
            hi = _safe_abs_upper(th)
            if math.isfinite(hi):
                return hi
        except Exception:
            if depth <= 0:
                return float("inf")

            re_mid = float(box.real.mid())
            re_rad = float(box.real.rad())
            im_mid = float(box.imag.mid())
            im_rad = float(box.imag.rad())

            # Split along the larger radius.
            if re_rad >= im_rad:
                h = 0.5 * re_rad
                b1 = acb_from_box(re_mid - h, h, im_mid, im_rad)
                b2 = acb_from_box(re_mid + h, h, im_mid, im_rad)
            else:
                h = 0.5 * im_rad
                b1 = acb_from_box(re_mid, re_rad, im_mid - h, h)
                b2 = acb_from_box(re_mid, re_rad, im_mid + h, h)

            return max(_refine_abs_bound(b1, depth - 1), _refine_abs_bound(b2, depth - 1))

    # Try the single enclosing s-box first, but with a modest split depth: if this fails,
    # the circle-cover fallback below is typically much more efficient than exhaustively
    # splitting a large 2D box.
    hi0 = _refine_abs_bound(s_box, depth=int(box_depth))
    if math.isfinite(hi0):
        return hi0

    # Fallback: if evaluating on the single enclosing s-box still fails (typically because
    # outer.O(s_box) or the (X+Y) denominator overestimates and encloses 0), bound
    # sup_{|z|<=r} |theta(z)| by covering the *circle* |z|=r using arc enclosures.
    #
    # By maximum modulus, sup on the disk equals sup on the boundary circle.
    two_pi = 2.0 * math.pi

    def max_abs_on_arc(phi0: float, phi1: float, depth: int) -> float:
        mid = 0.5 * (phi0 + phi1)
        rad = 0.5 * (phi1 - phi0)
        thetaI = arb(mid, rad)
        z_ball = _acb_from_polar(r, thetaI)
        s_ball = _s_from_disk_z_far_chart(float(sigma0), z_ball)
        hi = _refine_abs_bound(s_ball, depth=int(arc_refine_depth))
        if math.isfinite(hi):
            return hi
        if depth <= 0:
            return float("inf")
        m = 0.5 * (phi0 + phi1)
        return max(max_abs_on_arc(phi0, m, depth - 1), max_abs_on_arc(m, phi1, depth - 1))

    # Start with a coarse arc cover and adaptively bisect where needed.
    n0 = int(arc_n0)
    arc_depth = int(arc_split_depth)  # at most n0*2^arc_depth arc pieces (worst case)
    hi = 0.0
    if progress:
        print(f"[pick_certify] M_bound: s-box failed; falling back to {n0} arcs (depth={arc_depth})")
    for j in range(n0):
        a = (two_pi * j) / n0
        b = (two_pi * (j + 1)) / n0
        hi = max(hi, max_abs_on_arc(a, b, depth=arc_depth))
        if not math.isfinite(hi):
            return float("inf")
        if progress and (j + 1) % 4 == 0:
            print(f"[pick_certify] M_bound: arcs {j+1}/{n0} (current hi={hi:.6g}, evals={evals})")
    return hi


def _pick_matrix_from_coeffs(a: List[acb], N: int) -> acb_mat:
    """
    Build the N×N arithmetic Pick minor from Taylor coefficients a_0..a_{N-1}:
      P_{ij} = δ_{ij} - Σ_{k=0}^{min(i,j)} a_{i-k} conj(a_{j-k})
    and enforce Hermitian symmetry by construction.
    """
    if len(a) < N:
        raise ValueError("need at least N coefficients")
    P = acb_mat(N, N)
    one = acb(1)
    zero = acb(0)
    for i in range(N):
        for j in range(i + 1):
            s = one if i == j else zero
            m = min(i, j)
            for k in range(m + 1):
                s -= a[i - k] * a[j - k].conjugate()
            P[i, j] = s
            if i != j:
                P[j, i] = s.conjugate()
    return P


def _chol_lower_hermitian_spd(H: acb_mat) -> acb_mat:
    """
    Cholesky factorization for Hermitian SPD matrix H:
      H = L * L^* with L lower-triangular.

    This is a ball-arithmetic implementation intended for certification:
    it will throw if a diagonal pivot cannot be certified as strictly positive real.
    """
    n = H.nrows()
    if H.ncols() != n:
        raise ValueError("matrix is not square")
    L = acb_mat(n, n)
    for i in range(n):
        for j in range(i + 1):
            s = H[i, j]
            for k in range(j):
                s -= L[i, k] * L[j, k].conjugate()
            if i == j:
                # Require imag to contain 0 and real to be strictly positive.
                if not s.imag.contains(0):
                    raise ValueError("Cholesky pivot has nonzero-imag enclosure")
                if arb_lower(s.real) <= 0.0:
                    raise ValueError("Cholesky pivot not certified positive")
                L[i, j] = acb(s.real.sqrt())
            else:
                L[i, j] = s / L[j, j]
    return L


def pick_certify(args: Any) -> None:
    """
    Rigorous arithmetic Pick minor certification:

    - Computes Taylor coefficients a_0..a_{N-1} of the disk pullback
        theta_sigma0(z) = Theta(s_sigma0(z))
      using a discrete Fourier method on a tiny circle |z|=rho, with an explicit aliasing bound.

    - Builds the N×N Pick minor P_N and certifies a spectral gap by bisection:
        find delta>0 such that P_N - delta I is certified Hermitian SPD.

    Outputs a JSON artifact to --pick-out.
    """
    sigma0 = float(args.pick_sigma0)
    N = int(args.pick_N)
    Ncoeff = int(args.pick_coeff_count) if getattr(args, "pick_coeff_count", None) is not None else N
    K = int(args.pick_K)
    rho = float(args.pick_rho)
    rho_bound = float(args.pick_rho_bound)
    if not (sigma0 > 0.0):
        raise ValueError("--pick-sigma0 must be positive")
    if N < 2:
        raise ValueError("--pick-N must be >= 2")
    if Ncoeff < N:
        raise ValueError("--pick-coeff-count must be >= --pick-N")
    if K <= N:
        raise ValueError("--pick-K must be > pick-N (avoid trivial aliasing)")
    if not (0.0 < rho < 1.0):
        raise ValueError("--pick-rho must be in (0,1)")
    if not (0.0 < rho_bound < 1.0):
        raise ValueError("--pick-rho-bound must be in (0,1)")
    if not (rho < rho_bound):
        raise ValueError("--pick-rho must be < --pick-rho-bound")

    gauge = str(args.arith_gauge)
    det2_mode = str(args.arith_det2_mode)
    det2_kmax = int(args.arith_det2_kmax)
    arith_P_cut = int(args.P if args.arith_P_cut is None else args.arith_P_cut)
    primes = [p for p in cached_primes_upto(arith_P_cut) if p >= 2]

    # Build outer (if needed). Reuse the machinery from theta_certify_rect.
    outer_engine: OuterNormalizerEngineLike | None = None
    if gauge in {"outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
        outer_mode = str(getattr(args, "outer_mode", "rigorous"))
        if outer_mode != "rigorous":
            raise ValueError("--pick-certify currently requires --outer-mode rigorous")
        if gauge == "outer_xi":
            outer_engine = OuterNormalizerEngine(
                sigma_ref=float(args.outer_sigma_ref),
                T=float(args.outer_T),
                n_intervals=int(args.outer_n),
                P_cut=int(args.outer_P_cut),
            )
            outer_engine.build()
        elif gauge == "outer_zeta":
            outer_engine = OuterNormalizerEngineZetaRigorous(
                sigma_ref=float(args.outer_sigma_ref),
                T=float(args.outer_T),
                n_intervals=int(args.outer_n),
                P_cut=int(args.outer_P_cut),
                k_max=int(args.arith_det2_kmax),
                progress=bool(args.progress),
                max_depth=int(args.outer_max_depth),
                min_width=float(args.outer_min_width),
            )
            if args.outer_cache_load is not None:
                outer_engine.load_cache(
                    str(args.outer_cache_load), strict=not bool(args.outer_cache_ignore_mismatch)
                )
            else:
                outer_engine.build()
            if args.outer_cache_save is not None:
                outer_engine.save_cache(str(args.outer_cache_save))
        elif gauge == "outer_zeta_proj":
            outer_engine = OuterNormalizerEngineZetaProjectRigorous(
                sigma_ref=float(args.outer_sigma_ref),
                T=float(args.outer_T),
                n_intervals=int(args.outer_n),
                P_cut=int(args.outer_P_cut),
                det2_mode=str(args.arith_det2_mode),
                k_max=int(args.arith_det2_kmax),
                progress=bool(args.progress),
                max_depth=int(args.outer_max_depth),
                min_width=float(args.outer_min_width),
                f_rel_tol=float(args.outer_proj_rel_tol),
                f_abs_tol=float(args.outer_proj_abs_tol),
                det2_cache_children=not bool(args.outer_proj_recompute_det2),
            )
            if args.outer_cache_load is not None:
                outer_engine.load_cache(
                    str(args.outer_cache_load), strict=not bool(args.outer_cache_ignore_mismatch)
                )
            else:
                outer_engine.build()
            if args.outer_cache_save is not None:
                outer_engine.save_cache(str(args.outer_cache_save))
        else:
            raise ValueError(f"--pick-certify does not support gauge={gauge!r} (need outer)")

    # Bound M on the z-disk |z|<=rho_bound (used only for aliasing control).
    #
    # This can be the slowest / most brittle part of the certification because it requires
    # box-evaluating theta on a large s-region (when rho_bound is close to 1).
    #
    # If --pick-M-bound is provided, we trust the caller to justify it externally and
    # we skip the automatic certification step.
    user_M = getattr(args, "pick_M_bound", None)
    if user_M is not None:
        M_bound = float(user_M)
        if not (M_bound > 0.0 and math.isfinite(M_bound)):
            raise ValueError("--pick-M-bound must be a finite positive number")
        M_bound_source = "user"
        if bool(getattr(args, "progress", False)):
            print(f"[pick_certify] using user M_bound={M_bound:.6g} on |z|<=rho_bound={rho_bound:g}")
    else:
        M_bound = _max_abs_on_z_disk_via_s_box(
            sigma0=sigma0,
            z_radius=rho_bound,
            primes=primes,
            gauge=gauge,
            outer=outer_engine,
            det2_mode=det2_mode,
            det2_P_cut=arith_P_cut,
            det2_k_max=det2_kmax,
            progress=bool(getattr(args, "progress", False)),
        )
        M_bound_source = "certified"
        if bool(getattr(args, "progress", False)):
            print(f"[pick_certify] M_bound <= {M_bound:.6g} on |z|<=rho_bound={rho_bound:g}")

    # Sample on |z|=rho at K equally spaced nodes and compute DFT coefficients.
    two_pi = arb(2) * arb.pi()
    phi_step = two_pi / arb(K)
    omega_step = acb(arb(0), -phi_step).exp()  # exp(-i*2pi/K)

    theta_vals: List[acb] = []
    t_samp0 = time.time()
    progress_every = max(1, int(K // 20))  # ~20 updates
    for k in range(K):
        if bool(getattr(args, "progress", False)) and (k % progress_every == 0):
            dt = time.time() - t_samp0
            print(f"[pick_certify] sampling theta on |z|=rho: {k}/{K} (elapsed {dt:.1f}s)")
        phi = arb(k) * phi_step
        z = _acb_from_polar(rho, phi)
        s = _s_from_disk_z_far_chart(sigma0, z)
        th = _theta_arith_point_rig(
            s=s,
            primes=primes,
            gauge=gauge,
            outer=outer_engine,
            det2_mode=det2_mode,
            det2_P_cut=arith_P_cut,
            det2_k_max=det2_kmax,
        )
        theta_vals.append(th)
    if bool(getattr(args, "progress", False)):
        dt = time.time() - t_samp0
        print(f"[pick_certify] sampling complete: {K} points (elapsed {dt:.1f}s)")

    # DFT: b_n = (1/K) Σ_k theta_k * exp(-i n phi_k) = Σ_{j>=0} a_{n+jK} rho^{n+jK}.
    invK = acb(arb(1) / arb(K))
    rho_arb = arb(rho)
    rho_bound_arb = arb(rho_bound)
    q = (rho_arb / rho_bound_arb) ** K  # (rho/rho_bound)^K
    # q/(1-q) in arb, safe even if tiny.
    q_over = q / (arb(1) - q)

    coeffs: List[acb] = []
    alias_err_hi: List[float] = []
    for n in range(Ncoeff):
        omega_n = acb(arb(0), -(arb(n) * phi_step)).exp()  # exp(-i n 2pi/K)
        pow_ = acb(1)
        ssum = acb(0)
        for k in range(K):
            ssum += theta_vals[k] * pow_
            pow_ *= omega_n
        b_n = invK * ssum
        # Divide out rho^n
        rho_n = rho_arb**n
        a_n = b_n / acb(rho_n)
        # Aliasing bound: |Σ_{j>=1} a_{n+jK} rho^{jK}| <= (M_bound / rho_bound^n) * q/(1-q)
        err = (arb(M_bound) / (rho_bound_arb**n)) * q_over
        # Inflate by a conservative square enclosure (adds ±err to real and imaginary parts).
        err_hi = arb_upper(err)
        a_n = a_n + acb(arb(0, err_hi), arb(0, err_hi))
        coeffs.append(a_n)
        alias_err_hi.append(err_hi)

    # Build Pick minor and certify a spectral gap by bisection.
    P = _pick_matrix_from_coeffs(coeffs, N)

    # Bisection on delta in [0, 1). (Pick minors are typically bounded by 1 in scale.)
    delta_lo = 0.0
    delta_hi = float(args.pick_delta_hi)
    if not (0.0 < delta_hi <= 1.0):
        raise ValueError("--pick-delta-hi must be in (0,1]")
    steps = int(args.pick_delta_steps)
    if steps <= 0:
        raise ValueError("--pick-delta-steps must be positive")

    def is_spd_for_delta(d: float) -> bool:
        H = acb_mat(N, N)
        for i in range(N):
            for j in range(N):
                H[i, j] = P[i, j]
        dd = acb(arb(float(d)))
        for i in range(N):
            H[i, i] -= dd
        try:
            _chol_lower_hermitian_spd(H)
            return True
        except Exception:
            return False

    spd_at_0 = is_spd_for_delta(0.0)

    for _ in range(steps):
        mid = 0.5 * (delta_lo + delta_hi)
        if is_spd_for_delta(mid):
            delta_lo = mid
        else:
            delta_hi = mid

    delta_cert = delta_lo

    # Tail diagnostics (partial; does NOT replace Lemma lem:pick-tail-perturbation).
    tail_weighted_l2_partial_hi = 0.0
    for n in range(N, Ncoeff):
        an_abs_hi = _safe_abs_upper(coeffs[n])
        tail_weighted_l2_partial_hi += float(n + 1) * (an_abs_hi * an_abs_hi)

    out_path = getattr(args, "pick_out", None)
    if out_path is None:
        raise ValueError("--pick-certify requires --pick-out <artifact.json>")

    _write_json(
        str(out_path),
        {
            "kind": "pick_certify_artifact",
            "version": 1,
            "args": vars(args),
            "pick": {
                "sigma0": float(sigma0),
                "N": int(N),
                "Ncoeff": int(Ncoeff),
                "K": int(K),
                "rho": float(rho),
                "rho_bound": float(rho_bound),
                "M_bound": float(M_bound),
                "M_bound_source": str(M_bound_source),
                "P_spd_at_0": bool(spd_at_0),
                "delta_cert": float(delta_cert),
                "alias_err_hi": alias_err_hi,
                "tail_weighted_l2_partial_hi": float(tail_weighted_l2_partial_hi),
            },
            "coeffs": {
                "a_re": [str(z.real) for z in coeffs],
                "a_im": [str(z.imag) for z in coeffs],
            },
        },
    )
    print(f"[pick_certify] wrote artifact: {out_path}")
    print(f"[pick_certify] certified delta >= {delta_cert:.6g} for Pick minor size N={N}")


def cbnf_certify(args: Any) -> None:
    """
    Phase-1 helper for the near-field hypothesis (CB_NF) from the manuscript.

    This does *not* attempt to derive a Carleson bound from analytic number theory yet.
    Instead it records a machine-checkable artifact that:
      - declares an assumed upper bound for C_box,NF^(zeta)(sigma0),
      - declares an assumed upper bound for the CR–Green window constant C(psi),
      - computes the implied minimum depth lower bound
            eta_min = L_rec^2 / (8 C(psi)^2 C_box,NF)
        and checks the inequality eta_min > sigma0-1/2.

    This is the correct “last-mile” numerical check once an analytic bound for
    C_box,NF^(zeta)(sigma0) is obtained (see `CB_NF_ATTACK_PLAN.md`).
    """
    out_path = getattr(args, "cbnf_out", None)
    if out_path is None:
        raise ValueError("--cbnf-certify requires --cbnf-out <path.json>")

    sigma0 = float(args.cbnf_sigma0)
    if not (0.5 < sigma0 < 1.0):
        raise ValueError("--cbnf-sigma0 must be in (1/2,1)")

    # Treat these as user-supplied *upper bounds* (assumptions) to be justified externally.
    C_box_nf_hi = float(args.cbnf_C_box_nf_upper)
    if not (C_box_nf_hi > 0.0 and math.isfinite(C_box_nf_hi)):
        raise ValueError("--cbnf-C-box-nf-upper must be a finite positive number")

    Cpsi_hi = float(args.cbnf_Cpsi_upper)
    if not (Cpsi_hi > 0.0 and math.isfinite(Cpsi_hi)):
        raise ValueError("--cbnf-Cpsi-upper must be a finite positive number")

    # L_rec := 2 arctan 2, computed in Arb.
    L_rec = arb(2) * arb(2).atan()

    # Lower bound for eta_min given only *upper* bounds on Cpsi and C_box_nf.
    denom_hi = arb(8) * (arb(Cpsi_hi) ** 2) * arb(C_box_nf_hi)
    eta_min = (L_rec**2) / denom_hi
    eta_min_lo = arb_lower(eta_min)

    eta_target = sigma0 - 0.5
    ok = bool(eta_min_lo > eta_target)

    out = {
        "kind": "cbnf_certify_artifact",
        "version": 1,
        "args": {
            "sigma0": sigma0,
            # Stored for traceability; not used by the phase-1 computation itself.
            "alpha": (None if getattr(args, "cbnf_alpha", None) is None else float(args.cbnf_alpha)),
            "C_box_nf_upper": C_box_nf_hi,
            "Cpsi_upper": Cpsi_hi,
        },
        "constants": {
            "L_rec_mid": float(L_rec.mid()),
            "L_rec_rad": float(L_rec.rad()),
        },
        "results": {
            "eta_target": eta_target,
            "eta_min_lower_assuming_upper_inputs": eta_min_lo,
            "ok": ok,
        },
        "notes": (
            "This artifact is Phase-1: it does not prove (CB_NF). It records the last-mile "
            "numerical implication once an analytic bound for C_box,NF^(zeta)(sigma0) is supplied."
        ),
    }
    _write_json(str(out_path), out)
    print(f"[cbnf_certify] wrote artifact: {out_path}")


def pick_demo(args: Any) -> None:
    """
    Exploratory arithmetic Pick-minor audit.

    WARNING: This is a *diagnostic* numeric routine. It computes approximate Taylor
    coefficients of the disk pullback theta(z) = Theta(s_{sigma0}(z)) near z=0
    from tiny-circle sampling, then forms a truncated Hankel/shift matrix and
    reports the smallest eigenvalue of the associated finite Pick minor.

    It is meant to answer: "does the far-field arithmetic Pick gap look plausible?"
    It is NOT a certified proof step (no rigorous remainder bounds).
    """
    sigma0 = float(args.pick_sigma0)
    if sigma0 <= 0:
        raise ValueError("--pick-sigma0 must be positive")

    N = int(args.pick_N)
    if N <= 1:
        raise ValueError("--pick-N must be >= 2")

    K = int(args.pick_K)
    if K < N:
        raise ValueError("--pick-K must be >= pick-N")
    if K <= 0:
        raise ValueError("--pick-K must be positive")

    rho = float(args.pick_rho)
    if not (0.0 < rho < 0.25):
        raise ValueError("--pick-rho must be in (0,0.25) for numerical stability")

    gauge = str(args.arith_gauge)
    det2_mode = str(args.arith_det2_mode)
    arith_P_cut = int(args.P if args.arith_P_cut is None else args.arith_P_cut)
    primes = [p for p in cached_primes_upto(arith_P_cut) if p >= 2]

    # Build/load outer engine if required (reuse the same machinery as audit_phase).
    outer_engine: OuterNormalizerEngineLike | None = None
    if gauge in {"outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
        if gauge == "outer_zeta_log" and str(args.outer_mode) != "midpoint":
            raise ValueError("--arith-gauge outer_zeta_log requires --outer-mode midpoint (diagnostic only)")

        print(
            f"[pick_demo] building O(s) from boundary line Re(s)={args.outer_sigma_ref} "
            f"with T={args.outer_T}, n={args.outer_n}, P_cut={args.outer_P_cut} ..."
        )
        if str(args.outer_mode) == "rigorous":
            if gauge == "outer_xi":
                outer_engine = OuterNormalizerEngine(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_intervals=args.outer_n,
                    P_cut=args.outer_P_cut,
                )
            elif gauge == "outer_zeta":
                outer_engine = OuterNormalizerEngineZetaRigorous(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_intervals=args.outer_n,
                    P_cut=args.outer_P_cut,
                    k_max=int(args.arith_det2_kmax),
                    progress=bool(args.progress),
                    max_depth=int(args.outer_max_depth),
                    min_width=float(args.outer_min_width),
                )
            elif gauge == "outer_zeta_proj":
                outer_engine = OuterNormalizerEngineZetaProjectRigorous(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_intervals=args.outer_n,
                    P_cut=args.outer_P_cut,
                    det2_mode=str(args.arith_det2_mode),
                    k_max=int(args.arith_det2_kmax),
                    progress=bool(args.progress),
                    max_depth=int(args.outer_max_depth),
                    min_width=float(args.outer_min_width),
                    f_rel_tol=float(args.outer_proj_rel_tol),
                    f_abs_tol=float(args.outer_proj_abs_tol),
                    det2_cache_children=not bool(args.outer_proj_recompute_det2),
                )
            else:
                raise ValueError(f"{gauge} is only supported with --outer-mode midpoint")

            if args.outer_cache_load is not None:
                print(f"[pick_demo] loading outer cache {args.outer_cache_load} ...")
                assert hasattr(outer_engine, "load_cache")
                outer_engine.load_cache(
                    args.outer_cache_load, strict=not bool(args.outer_cache_ignore_mismatch)
                )
                n_leaf = len(getattr(outer_engine, "t_intervals", None) or [])
                print(f"[pick_demo] loaded. leaves={n_leaf}")
            else:
                outer_engine.build()
                n_leaf = len(getattr(outer_engine, "t_intervals", None) or [])
                print(f"[pick_demo] built. leaves={n_leaf}")
        else:
            if gauge == "outer_xi":
                outer_engine = OuterNormalizerEngineMidpoint(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                )
            elif gauge == "outer_zeta":
                outer_engine = OuterNormalizerEngineMidpointZeta(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                )
            elif gauge == "outer_zeta_log":
                outer_engine = OuterNormalizerEngineMidpointZetaLog(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                    unwrap=True,
                )
            else:
                outer_engine = OuterNormalizerEngineMidpointZetaProject(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                )

            if args.outer_cache_load is not None:
                print(f"[pick_demo] loading outer cache {args.outer_cache_load} ...")
                assert hasattr(outer_engine, "load_cache")
                outer_engine.load_cache(
                    args.outer_cache_load, strict=not bool(args.outer_cache_ignore_mismatch)
                )
                n_nodes = len(getattr(outer_engine, "t_nodes", None) or [])
                print(f"[pick_demo] loaded. nodes={n_nodes}")
            else:
                outer_engine.build()
                n_nodes = len(getattr(outer_engine, "t_nodes", None) or [])
                print(f"[pick_demo] built. nodes={n_nodes}")

    # Sample theta(z) on a tiny circle and fit a truncated Taylor series.
    angles = np.linspace(0.0, 2.0 * math.pi, num=K, endpoint=False)
    z_pts = rho * np.exp(1j * angles)
    vals = np.zeros(K, dtype=np.complex128)

    for i in range(K):
        z = complex(z_pts[i])
        # Use the manuscript disk chart (Definition def:disk-map-far).
        s = sigma0 + (1.0 + z) / (1.0 - z)
        s_acb = acb(s.real, s.imag)
        Ja = _arith_eval_point_mid(
            s=s_acb,
            primes=primes,
            gauge=gauge,
            outer=outer_engine,
            det2_mode=det2_mode,
            det2_P_cut=arith_P_cut,
            det2_k_max=int(args.arith_det2_kmax),
        )
        th = cayley_theta_from_J(Ja)
        vals[i] = complex(float(th.real), float(th.imag))

    # Solve Vandermonde for coefficients a_0..a_{N-1} using the first N points.
    zN = z_pts[:N]
    VN = np.vander(zN, N, increasing=True)  # columns: z^0..z^{N-1}
    a = np.linalg.solve(VN, vals[:N])

    # Build truncated Hankel/shift matrix A_{ij} = a_{i+j} using a_0..a_{2N-2}.
    if N > len(a):
        raise ValueError("internal: coefficient vector too short")
    # Extend coefficients by a crude FFT-based estimator for indices up to 2N-2.
    # For diagnostics, use the same Vandermonde solution to evaluate higher terms as ~0.
    a_ext = np.zeros(2 * N - 1, dtype=np.complex128)
    a_ext[:N] = a
    # (Remaining entries left as 0; this is only a heuristic minor.)

    A = np.zeros((N, N), dtype=np.complex128)
    for i in range(N):
        for j in range(N):
            A[i, j] = a_ext[i + j]

    H = np.eye(N, dtype=np.complex128) - A.conj().T @ A
    eigs = np.linalg.eigvalsh(H.real)  # crude: ignore small imaginary noise
    lam_min = float(np.min(eigs))

    print()
    print("[pick_demo] WARNING: diagnostic only (non-rigorous).")
    print("[pick_demo] sigma0:", sigma0, "| gauge:", gauge, "| det2_mode:", det2_mode)
    print("[pick_demo] rho:", rho, "| K:", K, "| N:", N)
    print(f"[pick_demo] approx lambda_min(H_arith^(N)) ≈ {lam_min:.6g}")
    print("[pick_demo] a_0..a_5:")
    for k in range(min(6, len(a))):
        print(f"  a[{k}] ≈ {a[k].real:.6g} + {a[k].imag:.6g} i")


# -------------------------
# Main
# -------------------------


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--prec", type=int, default=200, help="Arb working precision in bits")
    ap.add_argument("--sigma0", type=float, default=0.6)
    ap.add_argument("--P", type=int, default=31)
    ap.add_argument("--n-mode", type=int, default=4)
    ap.add_argument(
        "--cert-weights-mode",
        type=str,
        default="printed",
        choices=["printed", "geom", "pointmass"],
        help="Certificate weights sequence: printed geometric weights, general geometric weights, or pointmass w1=1/2 (diagnostic).",
    )
    ap.add_argument(
        "--cert-weights-r",
        type=float,
        default=17.0 / 19.0,
        help="Geometric ratio r for --cert-weights-mode=geom (0<=r<1).",
    )
    ap.add_argument(
        "--cert-psi-scale",
        type=float,
        default=1.0,
        help="Diagnostic scale factor for ψ_cert (scales both ψ̂_cert and m_cert).",
    )
    ap.add_argument(
        "--cert-amp-shift",
        type=float,
        default=0.5,
        help="Use p^{-(sigma0 + cert_amp_shift)} in Γ amplitude (printed uses 0.5).",
    )
    ap.add_argument(
        "--cert-coeff-mode",
        type=str,
        default="printed",
        choices=["printed", "primepower_sqrt", "primepower_inv"],
        help="Certificate Γ coefficient model (diagnostic): printed vs prime-power-decay variants.",
    )
    ap.add_argument(
        "--cert-port",
        type=str,
        default="const",
        choices=["const", "eigmax"],
        help="Scalar certificate port choice (diagnostic). Use 'const' for the manuscript port; 'eigmax' uses an eigen-aligned port to change scaling.",
    )
    ap.add_argument(
        "--arith-P-cut",
        type=int,
        default=None,
        help="Prime cut for arithmetic det2 truncation/enclosure (defaults to --P).",
    )
    ap.add_argument(
        "--arith-det2-mode",
        type=str,
        default="trunc",
        choices=["trunc", "enclosure"],
        help="How to evaluate det2 on the arithmetic side: fast prime truncation vs a certified enclosure with prime-tail bound.",
    )
    ap.add_argument(
        "--arith-det2-kmax",
        type=int,
        default=40,
        help="k_max for det2 enclosure tail bound (only used when --arith-det2-mode=enclosure).",
    )
    ap.add_argument(
        "--arith-gauge",
        type=str,
        default="raw_zeta",
        choices=[
            "raw_xi",
            "raw_zeta",
            "outer_xi",
            "outer_zeta",
            "outer_zeta_log",
            "outer_zeta_proj",
            "canonical_zeta",
        ],
        help="Diagnostic arithmetic gauge for J_arith (outer-free / outer-normalized / canonical placeholder).",
    )
    ap.add_argument("--outer-sigma-ref", type=float, default=0.55)
    ap.add_argument("--outer-T", type=float, default=10.0)
    ap.add_argument("--outer-n", type=int, default=200)
    ap.add_argument("--outer-P-cut", type=int, default=20000)
    ap.add_argument(
        "--outer-max-depth",
        type=int,
        default=12,
        help="Max adaptive bisection depth for rigorous ζ-gauge outer build.",
    )
    ap.add_argument(
        "--outer-min-width",
        type=float,
        default=1e-3,
        help="Minimum boundary subinterval width before giving up (rigorous ζ-gauge outer build).",
    )
    ap.add_argument(
        "--outer-proj-rel-tol",
        type=float,
        default=0.0,
        help="Rigorous outer_zeta_proj only: bisect boundary intervals if the complex f(t) enclosure radius is too large relative to |mid| (0 disables).",
    )
    ap.add_argument(
        "--outer-proj-abs-tol",
        type=float,
        default=0.0,
        help="Rigorous outer_zeta_proj only: absolute tolerance added to the f(t) width test (0 disables).",
    )
    ap.add_argument(
        "--outer-proj-recompute-det2",
        action="store_true",
        help="Rigorous outer_zeta_proj only: recompute det2 enclosure on bisection children (slower, tighter).",
    )
    ap.add_argument(
        "--outer-mode",
        type=str,
        default="rigorous",
        choices=["rigorous", "midpoint"],
        help="How to build the normalizer O(s): fully interval-rigorous vs midpoint exploratory.",
    )
    ap.add_argument("--psihat-steps", type=int, default=4000)
    ap.add_argument("--rect-sigma-min", type=float, default=0.7)
    ap.add_argument("--rect-sigma-max", type=float, default=0.9)
    ap.add_argument("--rect-t-min", type=float, default=0.0)
    ap.add_argument("--rect-t-max", type=float, default=10.0)
    ap.add_argument("--cover-n-sigma", type=int, default=20)
    ap.add_argument("--cover-n-t", type=int, default=40)
    ap.add_argument(
        "--max-refines",
        type=int,
        default=6,
        help="Max rectangle-cover refinements (doubling n_sigma and n_t each step).",
    )
    ap.add_argument(
        "--eval-mode",
        type=str,
        default="rigorous",
        choices=["rigorous", "midpoint"],
        help="rigorous: evaluate on complex boxes; midpoint: evaluate only at box midpoints (non-rigorous, diagnostic).",
    )
    ap.add_argument(
        "--progress",
        action="store_true",
        help="Print progress updates during long loops (useful if a run looks 'stuck').",
    )
    ap.add_argument(
        "--outer-build-only",
        action="store_true",
        help="Build/load the outer normalizer only (no certificate build, no rectangle check).",
    )
    ap.add_argument(
        "--outer-cache-load",
        type=str,
        default=None,
        help="Load cached outer boundary data from this JSON file (rigorous outer_zeta / outer_zeta_proj, or midpoint zeta_log/zeta_proj).",
    )
    ap.add_argument(
        "--outer-cache-save",
        type=str,
        default=None,
        help="Save cached outer boundary data to this JSON file after building/loading (rigorous outer_zeta / outer_zeta_proj, or midpoint zeta_log/zeta_proj).",
    )
    ap.add_argument(
        "--outer-cache-ignore-mismatch",
        action="store_true",
        help="Allow loading an outer cache even if parameters (sigma_ref/T/P_cut/...) differ.",
    )
    ap.add_argument(
        "--scan-sigma",
        type=float,
        default=None,
        help="If set, run a pointwise scan (diagnostic) at s = scan_sigma + i t for t in [scan_t_min, scan_t_max].",
    )
    ap.add_argument("--scan-t-min", type=float, default=0.0)
    ap.add_argument("--scan-t-max", type=float, default=0.0)
    ap.add_argument("--scan-t-step", type=float, default=1.0)
    ap.add_argument(
        "--scan-only",
        action="store_true",
        help="If set together with --scan-sigma, exit after the scan (do not run the rectangle cover).",
    )
    ap.add_argument(
        "--scan-out",
        type=str,
        default=None,
        help="If set together with --scan-sigma, write scan results to this JSON file.",
    )
    ap.add_argument(
        "--debug-point-sigma",
        type=float,
        default=None,
        help="If set (together with --debug-point-t), print J_cert/J_arith at this point before running the rectangle cover.",
    )
    ap.add_argument(
        "--debug-point-t",
        type=float,
        default=None,
        help="If set (together with --debug-point-sigma), print J_cert/J_arith at this point before running the rectangle cover.",
    )
    ap.add_argument(
        "--artifact-out",
        type=str,
        default=None,
        help="If set, write a machine-checkable JSON artifact for the final rectangle cover run (margin/error/PASS and basic denominator diagnostics).",
    )
    ap.add_argument(
        "--pass-metric",
        type=str,
        default="J",
        choices=["J", "theta", "pseudohyp"],
        help=(
            "Which condition to treat as the PASS criterion on the final rectangle cover: "
            "'J' = sup|J_arith-J_cert| <= m_lo/4 (manuscript-style), "
            "'theta' = sup|Theta(J_arith)| < 1 (direct Schur check), "
            "'pseudohyp' = sup|phi| < 1 where phi is the pseudohyperbolic witness vs Theta_cert."
        ),
    )
    ap.add_argument(
        "--gauge-sweep",
        action="store_true",
        help="Run a structured gauge/parameter sweep (diagnostic) and exit.",
    )
    ap.add_argument(
        "--sweep-gauges",
        type=str,
        default="outer_zeta_log,outer_zeta_proj,outer_zeta,raw_zeta",
        help="Comma-separated list of gauges to sweep (e.g. 'outer_zeta_log,outer_zeta_proj').",
    )
    ap.add_argument(
        "--sweep-sigma-refs",
        type=str,
        default="0.501,0.505,0.51,0.52,0.55,0.58",
        help="Comma-separated sigma_ref values (>1/2) to sweep for outer gauges.",
    )
    ap.add_argument(
        "--sweep-Ts",
        type=str,
        default="20,50,100",
        help="Comma-separated T values to sweep for outer gauges.",
    )
    ap.add_argument(
        "--sweep-P-cuts",
        type=str,
        default="2000,5000",
        help="Comma-separated prime cuts to sweep (used for det2 truncation and outer boundary sampling).",
    )
    ap.add_argument(
        "--sweep-node-step",
        type=float,
        default=0.08,
        help="Boundary node spacing for midpoint outers (n_nodes ≈ 2T/step).",
    )
    ap.add_argument("--sweep-rect-sigma-min", type=float, default=0.6)
    ap.add_argument("--sweep-rect-sigma-max", type=float, default=0.7)
    ap.add_argument("--sweep-rect-t-min", type=float, default=0.0)
    ap.add_argument("--sweep-rect-t-max", type=float, default=20.0)
    ap.add_argument("--sweep-cover-n-sigma", type=int, default=8)
    ap.add_argument("--sweep-cover-n-t", type=int, default=32)
    ap.add_argument(
        "--sweep-det2-mode",
        type=str,
        default="trunc",
        choices=["trunc", "enclosure"],
        help="det2 mode used inside the sweep (diagnostic).",
    )
    ap.add_argument(
        "--sweep-max-cases",
        type=int,
        default=60,
        help="Stop the sweep after this many configurations.",
    )
    ap.add_argument(
        "--sweep-time-limit",
        type=float,
        default=0.0,
        help="Stop the sweep after this many seconds (0 disables).",
    )
    ap.add_argument(
        "--sweep-top",
        type=int,
        default=15,
        help="How many top results (smallest theta_hi) to print at the end.",
    )
    ap.add_argument(
        "--sweep-out",
        type=str,
        default=None,
        help="If set, write sweep results to this JSON path.",
    )
    ap.add_argument(
        "--theta-certify",
        action="store_true",
        help="Certify |Theta(J_arith)|<1 on the given rectangle via adaptive refinement (rigorous).",
    )
    ap.add_argument(
        "--theta-certify-cover",
        type=str,
        default=None,
        help=(
            "Run --theta-certify over a JSON list of rectangles and write a single bundle JSON to --theta-out. "
            "The cover file may be a list of rect objects or an object with key 'rects'."
        ),
    )
    ap.add_argument("--theta-init-n-sigma", type=int, default=10)
    ap.add_argument("--theta-init-n-t", type=int, default=50)
    ap.add_argument("--theta-min-sigma-width", type=float, default=5e-4)
    ap.add_argument("--theta-min-t-width", type=float, default=0.02)
    ap.add_argument("--theta-max-boxes", type=int, default=20000)
    ap.add_argument("--theta-time-limit", type=float, default=60.0, help="Seconds (0 disables).")
    ap.add_argument("--theta-progress-every", type=int, default=2000)
    ap.add_argument("--theta-out", type=str, default=None, help="If set, write theta-certify JSON artifact here.")
    ap.add_argument(
        "--theta-resume-in",
        type=str,
        default=None,
        help="Resume an interrupted --theta-certify run from a previous theta_out artifact.",
    )
    ap.add_argument(
        "--audit-phase",
        action="store_true",
        help=(
            "Audit phase oscillation of the selected arithmetic gauge on sliding t-windows "
            "(diagnostic). Uses midpoint-style evaluation and does NOT build the certificate."
        ),
    )
    ap.add_argument(
        "--audit-phase-sigmas",
        type=str,
        default=None,
        help=(
            "Comma-separated list of σ values to audit (evaluates at s=σ+it). "
            "Defaults to --scan-sigma if set, else [rect_sigma_min, mid, rect_sigma_max]."
        ),
    )
    ap.add_argument("--audit-phase-t-min", type=float, default=None, help="t_min for phase audit (defaults to rect_t_min).")
    ap.add_argument("--audit-phase-t-max", type=float, default=None, help="t_max for phase audit (defaults to rect_t_max).")
    ap.add_argument("--audit-phase-t-step", type=float, default=0.02, help="t step for phase audit sampling grid.")
    ap.add_argument(
        "--audit-phase-object",
        type=str,
        default="J",
        choices=["J", "theta"],
        help="Which phase to audit: arg(J_arith) or arg(Theta(J_arith)).",
    )
    ap.add_argument(
        "--audit-phase-budget-rad",
        type=float,
        default=None,
        help="Optional numeric budget (radians). If set, report PASS if max window oscillation stays ≤ budget.",
    )
    ap.add_argument(
        "--audit-phase-out",
        type=str,
        default=None,
        help="If set, write a machine-checkable JSON artifact for the phase audit to this path.",
    )
    ap.add_argument(
        "--pick-demo",
        action="store_true",
        help="Diagnostic: compute a crude arithmetic Pick minor near z=0 for the disk pullback theta(z)=Theta(s_{sigma0}(z)). NOT rigorous.",
    )
    ap.add_argument("--pick-certify", action="store_true", help="Rigorous: certify positivity of a finite arithmetic Pick minor and output a JSON artifact.")
    ap.add_argument("--pick-out", type=str, default=None, help="Output JSON path for --pick-certify.")
    ap.add_argument(
        "--pick-sigma0",
        type=float,
        default=0.6,
        help="sigma0 used in the far-half-plane disk map s(z)=sigma0+(1+z)/(1-z) (Definition def:disk-map-far).",
    )
    ap.add_argument("--pick-N", type=int, default=16, help="Size N of the crude Pick minor (diagnostic).")
    ap.add_argument(
        "--pick-coeff-count",
        type=int,
        default=None,
        help="How many Taylor coefficients to compute/output for pick-certify (>= pick-N). Defaults to pick-N.",
    )
    ap.add_argument("--pick-K", type=int, default=64, help="Number of sample points on the tiny circle (diagnostic).")
    ap.add_argument("--pick-rho", type=float, default=1e-3, help="Tiny circle radius rho for local Taylor fitting (diagnostic).")
    ap.add_argument(
        "--pick-rho-bound",
        type=float,
        default=0.2,
        help="Radius rho_bound>rho used to bound aliasing (choose so s(z) stays in Re(s)>1 for unconditional analyticity).",
    )
    ap.add_argument(
        "--pick-M-bound",
        type=float,
        default=None,
        help=(
            "If set, skip automatic certification of M_bound := sup_{|z|<=rho_bound} |theta(z)| and "
            "use this value instead (you must justify it externally)."
        ),
    )
    ap.add_argument(
        "--pick-delta-hi",
        type=float,
        default=0.99,
        help="Upper bracket for spectral-gap bisection (delta in [0, pick_delta_hi]).",
    )
    ap.add_argument(
        "--pick-delta-steps",
        type=int,
        default=40,
        help="Bisection steps for certifying a delta such that P_N - delta I is SPD.",
    )

    # Near-field (CB_NF) helper: Phase-1 “last-mile” inequality artifact.
    ap.add_argument("--cbnf-certify", action="store_true", help="Emit a Phase-1 (CB_NF) inequality artifact and exit.")
    ap.add_argument("--cbnf-out", type=str, default=None, help="Output JSON path for --cbnf-certify.")
    ap.add_argument("--cbnf-sigma0", type=float, default=0.6, help="sigma0 used in Lemma lem:energy-barrier.")
    ap.add_argument(
        "--cbnf-alpha",
        type=float,
        default=None,
        help="(Optional) cone aperture alpha used in the CB_NF definition; stored in JSON for traceability.",
    )
    ap.add_argument(
        "--cbnf-C-box-nf-upper",
        dest="cbnf_C_box_nf_upper",
        type=float,
        default=None,
        help="Assumed numeric upper bound for C_box,NF^(zeta)(sigma0). Required for --cbnf-certify.",
    )
    ap.add_argument(
        "--cbnf-Cpsi-upper",
        dest="cbnf_Cpsi_upper",
        type=float,
        default=None,
        help="Assumed numeric upper bound for the CR–Green window constant C(psi). Required for --cbnf-certify.",
    )
    args = ap.parse_args()

    flint.ctx.prec = int(args.prec)

    if args.gauge_sweep:
        run_gauge_sweep(args)
        return

    if args.theta_certify_cover is not None:
        theta_certify_cover(args)
        return
    if args.theta_certify:
        theta_certify_rect(args)
        return
    if args.audit_phase:
        audit_phase(args)
        return
    if args.pick_demo:
        pick_demo(args)
        return
    if args.pick_certify:
        pick_certify(args)
        return
    if getattr(args, "cbnf_certify", False):
        if args.cbnf_C_box_nf_upper is None or args.cbnf_Cpsi_upper is None:
            raise ValueError("--cbnf-certify requires --cbnf-C-box-nf-upper and --cbnf-Cpsi-upper")
        cbnf_certify(args)
        return

    # Guards for midpoint-only diagnostic normalizers.
    if args.arith_gauge == "outer_zeta_log":
        if args.outer_mode != "midpoint":
            raise ValueError("--arith-gauge outer_zeta_log requires --outer-mode midpoint")
        if args.eval_mode != "midpoint":
            raise ValueError("--arith-gauge outer_zeta_log requires --eval-mode midpoint (diagnostic only)")
    if args.arith_gauge == "outer_zeta_proj" and args.outer_mode == "midpoint":
        # Allow eval_mode=rigorous for the discrete midpoint zeta_proj normalizer *only*:
        # we can evaluate its finite-sum normalizer on s-boxes via O_box(s).
        if args.eval_mode not in {"midpoint", "rigorous"}:
            raise ValueError("--eval-mode must be 'midpoint' or 'rigorous'")

    # ------------------------------------------------------------
    # Outer-build-only mode (precompute/cache boundary data)
    # ------------------------------------------------------------
    if args.outer_build_only:
        if args.arith_gauge not in {"outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
            raise ValueError(
                "--outer-build-only requires --arith-gauge outer_xi, outer_zeta, outer_zeta_log, or outer_zeta_proj"
            )

        outer_engine: OuterNormalizerEngineLike
        if args.outer_mode == "rigorous":
            if args.arith_gauge == "outer_xi":
                outer_engine = OuterNormalizerEngine(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_intervals=args.outer_n,
                    P_cut=args.outer_P_cut,
                )
            elif args.arith_gauge == "outer_zeta":
                outer_engine = OuterNormalizerEngineZetaRigorous(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_intervals=args.outer_n,
                    P_cut=args.outer_P_cut,
                    k_max=int(args.arith_det2_kmax),
                    progress=bool(args.progress),
                    max_depth=int(args.outer_max_depth),
                    min_width=float(args.outer_min_width),
                )
            elif args.arith_gauge == "outer_zeta_proj":
                outer_engine = OuterNormalizerEngineZetaProjectRigorous(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_intervals=args.outer_n,
                    P_cut=args.outer_P_cut,
                    det2_mode=str(args.arith_det2_mode),
                    k_max=int(args.arith_det2_kmax),
                    progress=bool(args.progress),
                    max_depth=int(args.outer_max_depth),
                    min_width=float(args.outer_min_width),
                    f_rel_tol=float(args.outer_proj_rel_tol),
                    f_abs_tol=float(args.outer_proj_abs_tol),
                    det2_cache_children=not bool(args.outer_proj_recompute_det2),
                )
            else:
                raise ValueError(f"{args.arith_gauge} is only supported with --outer-mode midpoint")
        else:
            if args.arith_gauge == "outer_xi":
                outer_engine = OuterNormalizerEngineMidpoint(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                )
            elif args.arith_gauge == "outer_zeta":
                outer_engine = OuterNormalizerEngineMidpointZeta(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                )
            elif args.arith_gauge == "outer_zeta_log":
                outer_engine = OuterNormalizerEngineMidpointZetaLog(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                    unwrap=True,
                )
            else:
                outer_engine = OuterNormalizerEngineMidpointZetaProject(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                )

        if args.outer_cache_load is not None:
            if isinstance(outer_engine, OuterNormalizerEngineZetaRigorous):
                print(f"[outer] loading cache {args.outer_cache_load} ...")
                outer_engine.load_cache(
                    args.outer_cache_load, strict=not bool(args.outer_cache_ignore_mismatch)
                )
                print(f"[outer] loaded. leaves={len(outer_engine.t_intervals or [])}")
            elif isinstance(outer_engine, OuterNormalizerEngineZetaProjectRigorous):
                print(f"[outer] loading cache {args.outer_cache_load} ...")
                outer_engine.load_cache(
                    args.outer_cache_load, strict=not bool(args.outer_cache_ignore_mismatch)
                )
                print(f"[outer] loaded. leaves={len(outer_engine.t_intervals or [])}")
            elif isinstance(outer_engine, OuterNormalizerEngineMidpointZetaLog):
                print(f"[outer] loading cache {args.outer_cache_load} ...")
                outer_engine.load_cache(
                    args.outer_cache_load, strict=not bool(args.outer_cache_ignore_mismatch)
                )
                print(f"[outer] loaded. nodes={len(outer_engine.t_nodes or [])}")
            elif isinstance(outer_engine, OuterNormalizerEngineMidpointZetaProject):
                print(f"[outer] loading cache {args.outer_cache_load} ...")
                outer_engine.load_cache(
                    args.outer_cache_load, strict=not bool(args.outer_cache_ignore_mismatch)
                )
                print(f"[outer] loaded. nodes={len(outer_engine.t_nodes or [])}")
            else:
                raise ValueError(
                    "--outer-cache-load is only supported for rigorous outer_zeta/outer_zeta_proj or midpoint outer_zeta_log/outer_zeta_proj"
                )
        else:
            print(
                f"[outer] building O(s) from boundary line Re(s)={args.outer_sigma_ref} "
                f"with T={args.outer_T}, n={args.outer_n}, P_cut={args.outer_P_cut} ..."
            )
            outer_engine.build()
            if isinstance(outer_engine, OuterNormalizerEngineZetaRigorous):
                print(f"[outer] built. leaves={len(outer_engine.t_intervals or [])}")
            elif isinstance(outer_engine, OuterNormalizerEngineZetaProjectRigorous):
                print(f"[outer] built. leaves={len(outer_engine.t_intervals or [])}")
            elif isinstance(outer_engine, OuterNormalizerEngineMidpointZetaLog):
                print(f"[outer] built. nodes={len(outer_engine.t_nodes or [])}")
            elif isinstance(outer_engine, OuterNormalizerEngineMidpointZetaProject):
                print(f"[outer] built. nodes={len(outer_engine.t_nodes or [])}")
            else:
                print("[outer] built.")

        if args.outer_cache_save is not None:
            if isinstance(outer_engine, OuterNormalizerEngineZetaRigorous):
                outer_engine.save_cache(args.outer_cache_save)
                print(f"[outer] saved cache to {args.outer_cache_save}")
            elif isinstance(outer_engine, OuterNormalizerEngineZetaProjectRigorous):
                outer_engine.save_cache(args.outer_cache_save)
                print(f"[outer] saved cache to {args.outer_cache_save}")
            elif isinstance(outer_engine, OuterNormalizerEngineMidpointZetaLog):
                outer_engine.save_cache(args.outer_cache_save)
                print(f"[outer] saved cache to {args.outer_cache_save}")
            elif isinstance(outer_engine, OuterNormalizerEngineMidpointZetaProject):
                outer_engine.save_cache(args.outer_cache_save)
                print(f"[outer] saved cache to {args.outer_cache_save}")
            else:
                raise ValueError(
                    "--outer-cache-save is only supported for rigorous outer_zeta/outer_zeta_proj or midpoint outer_zeta_log/outer_zeta_proj"
                )

        return

    print("[psihat] building β-integral engine (rigorous) ...")
    psihat_base = PsiHatEngine.build(n_steps=args.psihat_steps)
    psihat: Any = (
        psihat_base
        if float(args.cert_psi_scale) == 1.0
        else ScaledPsiHatEngine(psihat_base, scale=float(args.cert_psi_scale))
    )
    print(f"[psihat] built. m_cert={psihat.m_cert}")

    print("[cert] building certificate matrices (rigorous) ...")
    if args.cert_weights_mode == "printed":
        weights = None
    elif args.cert_weights_mode == "geom":
        weights = weights_geometric_param(int(args.n_mode), float(args.cert_weights_r))
    else:
        weights = weights_pointmass(int(args.n_mode))

    cert = Certificate(
        sigma0=args.sigma0,
        P=args.P,
        n_mode=args.n_mode,
        psihat=psihat,
        weights=weights,
        amp_shift=float(args.cert_amp_shift),
        coeff_mode=str(args.cert_coeff_mode),
        default_port=str(args.cert_port),
    )
    cert.build()
    print("[cert] built.")

    arith_P_cut = int(args.P if args.arith_P_cut is None else args.arith_P_cut)
    primes = [p for p in cached_primes_upto(arith_P_cut) if p >= 2]

    outer_engine: OuterNormalizerEngineLike | None = None
    if args.arith_gauge in {"outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj"}:
        print(
            f"[outer] building O(s) from boundary line Re(s)={args.outer_sigma_ref} "
            f"with T={args.outer_T}, n={args.outer_n}, P_cut={args.outer_P_cut} ..."
        )
        if args.outer_mode == "rigorous":
            if args.arith_gauge == "outer_xi":
                outer_engine = OuterNormalizerEngine(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_intervals=args.outer_n,
                    P_cut=args.outer_P_cut,
                )
                outer_engine.build()
                print("[outer] built (rigorous interval integration; xi-gauge, coarse det2 bound).")
            elif args.arith_gauge == "outer_zeta":
                outer_engine = OuterNormalizerEngineZetaRigorous(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_intervals=args.outer_n,
                    P_cut=args.outer_P_cut,
                    k_max=int(args.arith_det2_kmax),
                    progress=bool(args.progress),
                    max_depth=int(args.outer_max_depth),
                    min_width=float(args.outer_min_width),
                )
                if args.outer_cache_load is not None:
                    print(f"[outer] loading cache {args.outer_cache_load} ...")
                    outer_engine.load_cache(
                        args.outer_cache_load, strict=not bool(args.outer_cache_ignore_mismatch)
                    )
                    print(
                        f"[outer] loaded (rigorous zeta-gauge). leaves={len(outer_engine.t_intervals or [])}"
                    )
                else:
                    outer_engine.build()
                    print(
                        f"[outer] built (rigorous interval integration; zeta-gauge det2 enclosure). "
                        f"leaves={len(outer_engine.t_intervals or [])}"
                    )

                if args.outer_cache_save is not None:
                    outer_engine.save_cache(args.outer_cache_save)
                    print(f"[outer] saved cache to {args.outer_cache_save}")
            elif args.arith_gauge == "outer_zeta_proj":
                outer_engine = OuterNormalizerEngineZetaProjectRigorous(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_intervals=args.outer_n,
                    P_cut=args.outer_P_cut,
                    det2_mode=str(args.arith_det2_mode),
                    k_max=int(args.arith_det2_kmax),
                    progress=bool(args.progress),
                    max_depth=int(args.outer_max_depth),
                    min_width=float(args.outer_min_width),
                    f_rel_tol=float(args.outer_proj_rel_tol),
                    f_abs_tol=float(args.outer_proj_abs_tol),
                    det2_cache_children=not bool(args.outer_proj_recompute_det2),
                )
                if args.outer_cache_load is not None:
                    print(f"[outer] loading cache {args.outer_cache_load} ...")
                    outer_engine.load_cache(
                        args.outer_cache_load, strict=not bool(args.outer_cache_ignore_mismatch)
                    )
                    print(
                        f"[outer] loaded (rigorous zeta_proj). leaves={len(outer_engine.t_intervals or [])}"
                    )
                else:
                    outer_engine.build()
                    print(
                        f"[outer] built (rigorous interval integration; zeta_proj). "
                        f"leaves={len(outer_engine.t_intervals or [])}"
                    )
                if args.outer_cache_save is not None:
                    outer_engine.save_cache(args.outer_cache_save)
                    print(f"[outer] saved cache to {args.outer_cache_save}")
            else:
                raise ValueError(f"{args.arith_gauge} is only supported with --outer-mode midpoint")
        else:
            if args.arith_gauge == "outer_xi":
                outer_mid: OuterNormalizerEngineLike = OuterNormalizerEngineMidpoint(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                )
                outer_mid.build()
                outer_engine = outer_mid
                print("[outer] built (midpoint exploratory; xi).")
            elif args.arith_gauge == "outer_zeta":
                outer_mid_z: OuterNormalizerEngineLike = OuterNormalizerEngineMidpointZeta(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                )
                outer_mid_z.build()
                outer_engine = outer_mid_z
                print("[outer] built (midpoint exploratory; zeta).")
            elif args.arith_gauge == "outer_zeta_log":
                outer_mid_log = OuterNormalizerEngineMidpointZetaLog(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                    unwrap=True,
                )
                if args.outer_cache_load is not None:
                    print(f"[outer] loading cache {args.outer_cache_load} ...")
                    outer_mid_log.load_cache(
                        args.outer_cache_load, strict=not bool(args.outer_cache_ignore_mismatch)
                    )
                    print(f"[outer] loaded (midpoint zeta_log). nodes={len(outer_mid_log.t_nodes or [])}")
                else:
                    outer_mid_log.build()
                    print(f"[outer] built (midpoint exploratory; zeta_log). nodes={len(outer_mid_log.t_nodes or [])}")
                outer_engine = outer_mid_log
                if args.outer_cache_save is not None:
                    outer_mid_log.save_cache(args.outer_cache_save)
                    print(f"[outer] saved cache to {args.outer_cache_save}")
            else:
                outer_mid_proj = OuterNormalizerEngineMidpointZetaProject(
                    sigma_ref=args.outer_sigma_ref,
                    T=args.outer_T,
                    n_nodes=max(2, args.outer_n),
                    P_cut=args.outer_P_cut,
                )
                if args.outer_cache_load is not None:
                    print(f"[outer] loading cache {args.outer_cache_load} ...")
                    outer_mid_proj.load_cache(
                        args.outer_cache_load, strict=not bool(args.outer_cache_ignore_mismatch)
                    )
                    print(
                        f"[outer] loaded (midpoint zeta_proj). nodes={len(outer_mid_proj.t_nodes or [])}"
                    )
                else:
                    outer_mid_proj.build()
                    print(
                        f"[outer] built (midpoint exploratory; zeta_proj). nodes={len(outer_mid_proj.t_nodes or [])}"
                    )
                outer_engine = outer_mid_proj
                if args.outer_cache_save is not None:
                    outer_mid_proj.save_cache(args.outer_cache_save)
                    print(f"[outer] saved cache to {args.outer_cache_save}")

    # ------------------------------------------------------------
    # Diagnostic scan (pointwise)
    # ------------------------------------------------------------
    scan_rows: List[Dict[str, Any]] = []
    if args.scan_sigma is not None:
        if args.scan_t_step <= 0:
            raise ValueError("--scan-t-step must be positive")
        sigma_scan = float(args.scan_sigma)
        t = float(args.scan_t_min)
        t_max = float(args.scan_t_max)
        print()
        print(f"[scan] sigma={sigma_scan} t∈[{t},{t_max}] step={args.scan_t_step}")
        while t <= t_max + 1e-15:
            s = acb(sigma_scan, t)
            Jc = cert.J_cert(s)
            Jc_mid = acb_midpoint(Jc)
            two_re = (Jc * acb(2)).real
            m_lo = arb_lower(two_re)
            m_hi = arb_upper(two_re)

            # Arithmetic side (diagnostic): evaluate at the point using midpoint outer if available.
            if args.arith_gauge == "raw_xi":
                Ja = J_arith_raw_xi(s, primes)
            elif args.arith_gauge == "raw_zeta":
                Ja = J_arith_raw_zeta(s, primes)
            elif args.arith_gauge == "outer_xi":
                if outer_engine is None:
                    raise ValueError("--scan-sigma with outer_xi requires an outer engine")
                O = (
                    getattr(outer_engine, "O_midpoint")(s)
                    if hasattr(outer_engine, "O_midpoint")
                    else acb_midpoint(outer_engine.O(s))
                )
                Ja = det2_prime_trunc(s, primes) / (O * xi_completed(s))
            elif args.arith_gauge == "outer_zeta":
                if outer_engine is None:
                    raise ValueError("--scan-sigma with outer_zeta requires an outer engine")
                O = (
                    getattr(outer_engine, "O_midpoint")(s)
                    if hasattr(outer_engine, "O_midpoint")
                    else acb_midpoint(outer_engine.O(s))
                )
                Ja = det2_prime_trunc(s, primes) / (O * acb_midpoint(s.zeta())) * compensator_B(s)
            elif args.arith_gauge == "outer_zeta_log":
                if outer_engine is None:
                    raise ValueError("--scan-sigma with outer_zeta_log requires an outer engine")
                O = (
                    getattr(outer_engine, "O_midpoint")(s)
                    if hasattr(outer_engine, "O_midpoint")
                    else acb_midpoint(outer_engine.O(s))
                )
                Ja = det2_prime_trunc(s, primes) / (O * acb_midpoint(s.zeta())) * compensator_B(s)
            elif args.arith_gauge == "outer_zeta_proj":
                if outer_engine is None:
                    raise ValueError("--scan-sigma with outer_zeta_proj requires an outer engine")
                O = (
                    getattr(outer_engine, "O_midpoint")(s)
                    if hasattr(outer_engine, "O_midpoint")
                    else acb_midpoint(outer_engine.O(s))
                )
                Ja = det2_prime_trunc(s, primes) / (O * acb_midpoint(s.zeta())) * compensator_B(s)
            else:
                raise ValueError(f"unknown arith gauge: {args.arith_gauge}")

            diff = abs(Ja - Jc_mid)
            # Arithmetic Schur/Herglotz diagnostics (midpoint).
            Ja_mid = acb_midpoint(Ja)
            twoJa = Ja_mid * acb(2)
            re2Ja = float(twoJa.real)
            theta = (twoJa - acb(1)) / (twoJa + acb(1))
            theta_abs = float(abs(theta))

            # Ratio against midpoint of margin (purely diagnostic).
            m_mid = float(two_re.mid())
            ratio = float("nan")
            if m_mid > 0:
                ratio = float(diff) / (m_mid / 4.0)

            print(
                f"[scan] t={t:.6g}  Re(2Jc)∈[{m_lo:.6g},{m_hi:.6g}]  "
                f"|Ja-Jc_mid|≈{float(diff):.6g}  ratio≈{ratio:.6g}  "
                f"Re(2Ja_mid)≈{re2Ja:.6g}  |Theta(Ja_mid)|≈{theta_abs:.6g}"
            )
            scan_rows.append(
                {
                    "sigma": sigma_scan,
                    "t": t,
                    "Re_2Jc_lo": m_lo,
                    "Re_2Jc_hi": m_hi,
                    "Jc_mid_re": float(Jc_mid.real),
                    "Jc_mid_im": float(Jc_mid.imag),
                    "Ja_re": float(Ja.real),
                    "Ja_im": float(Ja.imag),
                    "abs_Ja_minus_Jc_mid": float(diff),
                    "ratio_vs_m_mid_over4": ratio,
                    "Re_2Ja_mid": re2Ja,
                    "abs_Theta_Ja_mid": theta_abs,
                }
            )
            t += float(args.scan_t_step)

        if args.scan_out is not None:
            _write_json(args.scan_out, {"rows": scan_rows})
            print(f"[scan] wrote {len(scan_rows)} rows to {args.scan_out}")
        if args.scan_only:
            return

    # Optional: print a single diagnostic point value.
    if args.debug_point_sigma is not None and args.debug_point_t is not None:
        s_dbg = acb(args.debug_point_sigma, args.debug_point_t)
        Jc_dbg = cert.J_cert(s_dbg)
        print()
        print(f"[debug] s = {s_dbg}")
        print(f"[debug] J_cert(s) = {Jc_dbg}")
        print(f"[debug] Re(2 J_cert) = {(Jc_dbg * acb(2)).real}")
        if args.arith_gauge == "raw_xi":
            Ja_dbg = J_arith_raw_xi(s_dbg, primes)
        elif args.arith_gauge == "raw_zeta":
            Ja_dbg = J_arith_raw_zeta(s_dbg, primes)
        elif args.arith_gauge == "outer_xi":
            assert outer_engine is not None
            det2P = det2_prime_trunc(s_dbg, primes)
            if args.eval_mode == "midpoint" and hasattr(outer_engine, "O_midpoint"):
                O_dbg = getattr(outer_engine, "O_midpoint")(s_dbg)
            else:
                O_dbg = outer_engine.O(s_dbg)
                if args.eval_mode == "midpoint":
                    O_dbg = acb_midpoint(O_dbg)
            Ja_dbg = det2P / (O_dbg * xi_completed(s_dbg))
            print(f"[debug] O(s) = {O_dbg}")
        elif args.arith_gauge == "outer_zeta":
            assert outer_engine is not None
            if args.eval_mode == "midpoint" and hasattr(outer_engine, "O_midpoint"):
                O_dbg = getattr(outer_engine, "O_midpoint")(s_dbg)
            else:
                O_dbg = outer_engine.O(s_dbg)
                if args.eval_mode == "midpoint":
                    O_dbg = acb_midpoint(O_dbg)
            Ja_dbg = det2_prime_trunc(s_dbg, primes) / (O_dbg * acb_midpoint(s_dbg.zeta())) * compensator_B(s_dbg)
            print(f"[debug] O(s) = {O_dbg}")
        elif args.arith_gauge == "outer_zeta_log":
            assert outer_engine is not None
            if args.eval_mode == "midpoint" and hasattr(outer_engine, "O_midpoint"):
                O_dbg = getattr(outer_engine, "O_midpoint")(s_dbg)
            else:
                O_dbg = outer_engine.O(s_dbg)
                if args.eval_mode == "midpoint":
                    O_dbg = acb_midpoint(O_dbg)
            Ja_dbg = (
                det2_prime_trunc(s_dbg, primes)
                / (O_dbg * acb_midpoint(s_dbg.zeta()))
                * compensator_B(s_dbg)
            )
            print(f"[debug] O(s) = {O_dbg}")
        elif args.arith_gauge == "outer_zeta_proj":
            assert outer_engine is not None
            if args.eval_mode == "midpoint" and hasattr(outer_engine, "O_midpoint"):
                O_dbg = getattr(outer_engine, "O_midpoint")(s_dbg)
            else:
                O_dbg = outer_engine.O(s_dbg)
                if args.eval_mode == "midpoint":
                    O_dbg = acb_midpoint(O_dbg)
            Ja_dbg = (
                det2_prime_trunc(s_dbg, primes)
                / (O_dbg * acb_midpoint(s_dbg.zeta()))
                * compensator_B(s_dbg)
            )
            print(f"[debug] O(s) = {O_dbg}")
        else:
            raise ValueError(f"unknown gauge: {args.arith_gauge}")

        print(f"[debug] J_arith(s) = {Ja_dbg}")
        print(f"[debug] |J_arith - J_cert| = {abs(Ja_dbg - Jc_dbg)}")
        # Schur/Herglotz diagnostic for arithmetic J (midpoint).
        Ja_m = acb_midpoint(Ja_dbg)
        twoJa_m = Ja_m * acb(2)
        theta_m = (twoJa_m - acb(1)) / (twoJa_m + acb(1))
        print(f"[debug] Re(2 J_arith_mid) = {twoJa_m.real}")
        print(f"[debug] Theta(J_arith_mid) = {theta_m}")
        print(f"[debug] |Theta(J_arith_mid)| ≈ {float(abs(theta_m))}")

    R = Rect(
        sigma_min=args.rect_sigma_min,
        sigma_max=args.rect_sigma_max,
        t_min=args.rect_t_min,
        t_max=args.rect_t_max,
    )
    print(f"[rect] R = [{R.sigma_min},{R.sigma_max}] × [{R.t_min},{R.t_max}]")
    m_lo, m_hi, err_lo, err_hi, theta_hi, phi_hi, ns, nt = certify_attachment_refine(
        cert,
        primes,
        R,
        n_sigma=args.cover_n_sigma,
        n_t=args.cover_n_t,
        gauge=args.arith_gauge,
        outer=outer_engine,
        max_refines=max(0, int(args.max_refines)),
        eval_mode=str(args.eval_mode),
        progress=bool(args.progress),
        det2_mode=str(args.arith_det2_mode),
        det2_P_cut=arith_P_cut,
        det2_k_max=int(args.arith_det2_kmax),
    )
    print(
        f"[margin] m_R = inf_R Re(2 J_cert) ∈ [{m_lo:.6g}, {m_hi:.6g}] (cover {ns}×{nt})"
    )
    print(f"[error] sup_R |J_arith - J_cert| ∈ [{err_lo:.6g}, {err_hi:.6g}] ({args.arith_gauge})")
    print(f"[theta] sup_R |Theta(J_arith)| ≤ {theta_hi:.6g}")
    print(f"[pseudohyp] sup_R |phi(Theta_arith; Theta_cert)| ≤ {phi_hi:.6g}")

    passed = False
    if not (math.isnan(m_lo) or math.isnan(err_hi) or math.isnan(theta_hi) or math.isnan(phi_hi)):
        if args.pass_metric == "J":
            need = m_lo / 4
            print(f"[check] metric=J: need err_hi ≤ m_lo/4 = {need:.6g}")
            passed = err_hi <= need
        elif args.pass_metric == "theta":
            print("[check] metric=theta: need sup|Theta(J_arith)| < 1")
            passed = theta_hi < 1.0
        else:
            print("[check] metric=pseudohyp: need sup|phi| < 1 (with m_lo>0 to keep Theta_cert strictly inside disk)")
            passed = (m_lo > 0.0) and (phi_hi < 1.0)
        print(f"[check] PASS? {passed}")

    # Machine-checkable artifact output (single-rectangle run).
    if args.artifact_out is not None:
        denoms: Dict[str, Any] = {}
        try:
            boxes = cover_rect(R, n_sigma=ns, n_t=nt)
            min_abs_zeta = float("inf")
            min_abs_O = float("inf")
            zeta_contains0 = 0
            O_contains0 = 0

            need_zeta = args.arith_gauge in {"raw_zeta", "outer_zeta", "outer_zeta_log", "outer_zeta_proj"}
            need_outer = args.arith_gauge in {"outer_xi", "outer_zeta", "outer_zeta_log", "outer_zeta_proj"}

            for box in boxes:
                s_eval = box if args.eval_mode == "rigorous" else acb_midpoint(box)

                if need_zeta:
                    z = s_eval.zeta()
                    if args.eval_mode == "midpoint":
                        z = acb_midpoint(z)
                    az = abs(z)
                    if az.contains(0):
                        zeta_contains0 += 1
                    min_abs_zeta = min(min_abs_zeta, max(0.0, arb_lower(az)))

                if need_outer:
                    if outer_engine is None:
                        raise ValueError("artifact denominator scan expected an outer engine but it was None")
                    if args.eval_mode == "midpoint" and hasattr(outer_engine, "O_midpoint"):
                        O_val = getattr(outer_engine, "O_midpoint")(s_eval)
                    elif args.eval_mode == "rigorous" and hasattr(outer_engine, "O_box"):
                        O_val = getattr(outer_engine, "O_box")(s_eval)
                    else:
                        O_val = outer_engine.O(s_eval)
                        if args.eval_mode == "midpoint":
                            O_val = acb_midpoint(O_val)
                    aO = abs(O_val)
                    if aO.contains(0):
                        O_contains0 += 1
                    min_abs_O = min(min_abs_O, max(0.0, arb_lower(aO)))

            denoms = {
                "need_zeta": need_zeta,
                "need_outer": need_outer,
                "min_abs_zeta_lower": (None if min_abs_zeta == float("inf") else min_abs_zeta),
                "min_abs_O_lower": (None if min_abs_O == float("inf") else min_abs_O),
                "zeta_abs_contains0_boxes": zeta_contains0,
                "O_abs_contains0_boxes": O_contains0,
            }
        except Exception as e:
            denoms = {"error": str(e)}

        artifact = {
            "kind": "verify_attachment_arb_artifact",
            "version": 1,
            "args": vars(args),
            "rect": {
                "sigma_min": R.sigma_min,
                "sigma_max": R.sigma_max,
                "t_min": R.t_min,
                "t_max": R.t_max,
            },
            "cover": {"n_sigma": ns, "n_t": nt},
            "results": {
                "m_lo": m_lo,
                "m_hi": m_hi,
                "err_lo": err_lo,
                "err_hi": err_hi,
                "theta_hi": theta_hi,
                "phi_hi": phi_hi,
                "passed": passed,
            },
            "outer_engine": (None if outer_engine is None else type(outer_engine).__name__),
            "denominators": denoms,
        }
        _write_json(args.artifact_out, artifact)
        print(f"[artifact] wrote {args.artifact_out}")

    print()
    print(
        "TODO(arith): replace outer-free gauge with a computable far-field outer normalizer (Option 2 proper)."
    )


if __name__ == "__main__":
    main()



