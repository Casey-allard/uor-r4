#!/usr/bin/env python3
"""
Explore the prime polynomial sum:
S_{L,t0} = sum_{log p <= kappa/L} (log p / sqrt(p)) * exp(i * t0 * log p)

Goal: Determine if |S_{L,t0}| is uniformly bounded in t0.
"""

import numpy as np
from sympy import primerange
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

def get_primes_up_to_log(max_log):
    """Get primes p where log(p) <= max_log"""
    max_p = int(np.exp(max_log)) + 1
    return list(primerange(2, max_p + 1))

def compute_S(primes, t0):
    """Compute S_{L,t0} for given primes and t0"""
    log_p = np.log(primes)
    weights = log_p / np.sqrt(primes)
    phases = np.exp(1j * t0 * log_p)
    return np.sum(weights * phases)

def explore_prime_sum(L, kappa=2*np.pi, t_max=1000, n_samples=10000):
    """Explore |S_{L,t0}| over a range of t0 values"""
    max_log = kappa / L
    primes = np.array(get_primes_up_to_log(max_log))
    print(f"L = {L}, max_log = {max_log:.2f}, num_primes = {len(primes)}")
    print(f"Primes from 2 to {primes[-1] if len(primes) > 0 else 'N/A'}")
    
    # Compute diagonal energy (for reference)
    log_p = np.log(primes)
    weights = log_p / np.sqrt(primes)
    diagonal_energy = np.sum(weights**2)
    print(f"Diagonal energy (sum of |a_p|^2): {diagonal_energy:.4f}")
    print(f"Expected RMS by random phase: {np.sqrt(diagonal_energy):.4f}")
    
    # Sample |S| over many t0 values
    t0_values = np.linspace(0, t_max, n_samples)
    S_values = np.array([compute_S(primes, t0) for t0 in t0_values])
    S_abs = np.abs(S_values)
    
    print(f"\nStatistics over t0 in [0, {t_max}]:")
    print(f"  Max |S|:  {np.max(S_abs):.4f}")
    print(f"  Mean |S|: {np.mean(S_abs):.4f}")
    print(f"  Min |S|:  {np.min(S_abs):.4f}")
    print(f"  Std |S|:  {np.std(S_abs):.4f}")
    
    # Find where max occurs
    max_idx = np.argmax(S_abs)
    print(f"  Max at t0 = {t0_values[max_idx]:.4f}")
    
    return t0_values, S_abs, primes

print("=" * 60)
print("EXPLORING PRIME POLYNOMIAL BOUND")
print("=" * 60)

# Test for different L values
for L in [0.2, 0.1, 0.05]:
    print(f"\n{'='*60}")
    print(f"L = {L}")
    print("=" * 60)
    t0_vals, S_abs, primes = explore_prime_sum(L, t_max=10000, n_samples=50000)
    
    # Save plot
    plt.figure(figsize=(12, 4))
    plt.subplot(1, 2, 1)
    plt.plot(t0_vals, S_abs, linewidth=0.5)
    plt.xlabel('t0')
    plt.ylabel('|S_{L,t0}|')
    plt.title(f'L = {L}, {len(primes)} primes')
    plt.axhline(y=np.sqrt(np.sum((np.log(primes)/np.sqrt(primes))**2)), 
                color='r', linestyle='--', label='sqrt(diagonal)')
    plt.legend()
    
    plt.subplot(1, 2, 2)
    plt.hist(S_abs, bins=100, density=True)
    plt.xlabel('|S|')
    plt.ylabel('Density')
    plt.title(f'Distribution of |S|')
    
    plt.tight_layout()
    plt.savefig(f'/Users/jonathanwashburn/Projects/Riemann/prime_sum_L{L}.png', dpi=150)
    plt.close()

print("\n" + "=" * 60)
print("EXTENDING TO LARGER t0 RANGES")
print("=" * 60)

# Focus on L=0.1 and extend t0 range
L = 0.1
max_log = 2 * np.pi / L
primes = np.array(get_primes_up_to_log(max_log))

# Sample at larger t0
for t_max in [100000, 1000000]:
    print(f"\nt0 range [0, {t_max}]:")
    t0_samples = np.random.uniform(0, t_max, 10000)
    S_samples = np.array([np.abs(compute_S(primes, t0)) for t0 in t0_samples])
    print(f"  Max |S|:  {np.max(S_samples):.4f}")
    print(f"  Mean |S|: {np.mean(S_samples):.4f}")

print("\n" + "=" * 60)
print("KEY FINDING")
print("=" * 60)
print("""
If max|S_{L,t0}| appears bounded (doesn't grow with t0 range),
then the prime polynomial bound might hold!

This would close the gap unconditionally.
""")
