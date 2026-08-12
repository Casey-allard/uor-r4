
import numpy as np
import matplotlib.pyplot as plt
from scipy.ndimage import uniform_filter, laplace
import random
import os

# Parameters
dim = 100
mod_base = 210

def is_prime(n):
    if n <= 1:
        return False
    if n <= 3:
        return True
    if n % 2 == 0 or n % 3 == 0:
        return False
    i = 5
    while i * i <= n:
        if n % i == 0 or n % (i + 2) == 0:
            return False
        i += 6
    return True

def generate_modular_matrix(dim, mod_base):
    return np.fromfunction(lambda i, j: (i * dim + j + 1) % mod_base, (dim, dim), dtype=int)

def prime_mask(matrix):
    return np.vectorize(is_prime)(matrix)

def calculate_entropy_field(matrix):
    prime_binary = prime_mask(matrix).astype(int)
    local_density = uniform_filter(prime_binary.astype(float), size=3)
    entropy_field = 1.0 - local_density
    return entropy_field

def simulate_rattling(matrix, entropy_field, steps=5000, alpha=3.0):
    position = [dim // 2, dim // 2]
    path = [tuple(position)]
    for _ in range(steps):
        i, j = position
        neighbors = [(i + di, j + dj) for di in [-1, 0, 1] for dj in [-1, 0, 1]
                     if not (di == 0 and dj == 0) and 0 <= i + di < dim and 0 <= j + dj < dim]
        weights = np.array([entropy_field[ni, nj] ** alpha for ni, nj in neighbors])
        weights /= weights.sum()
        chosen = random.choices(neighbors, weights=weights)[0]
        position = list(chosen)
        path.append(tuple(position))
    return path

def compute_entropy_curvature(entropy_field):
    return laplace(entropy_field)

def identify_attractors(curvature_field, threshold=-0.02):
    return np.argwhere(curvature_field < threshold)

def twin_prime_mask(matrix):
    primes = matrix[prime_mask(matrix)]
    twin_set = set(p for p in primes if is_prime(p + 2))
    return np.vectorize(lambda x: x in twin_set)(matrix)

def harmonic_phi_mask(matrix, phi=1.61803398875, tolerance=0.03):
    mod_phi = np.mod(matrix / phi, 1.0)
    return (mod_phi < tolerance) | (mod_phi > (1 - tolerance))

def predict_prime_zones(entropy_field, phi_mask, weight_entropy=0.6, weight_phi=0.4):
    phi_float_mask = phi_mask.astype(float)
    score = (weight_entropy * entropy_field) + (weight_phi * phi_float_mask)
    return score

# Run full simulation
matrix = generate_modular_matrix(dim, mod_base)
entropy = calculate_entropy_field(matrix)
path = simulate_rattling(matrix, entropy, steps=10000, alpha=2.5)
curvature = compute_entropy_curvature(entropy)
attractors = identify_attractors(curvature)
twin_mask = twin_prime_mask(matrix)
phi_mask = harmonic_phi_mask(matrix)
prediction = predict_prime_zones(entropy, phi_mask)

# Save visualization
output_path = "modular_prime_rattling_analysis.png"
fig, axes = plt.subplots(1, 7, figsize=(35, 5))
titles = [
    "Modular Matrix", "Entropy Field", "Rattling Agent Path", "Entropy Curvature",
    "Twin Prime Zones", "φ-Aligned Zones", "Predicted Prime Zones"
]
data = [
    matrix, entropy,
    np.histogram2d(*zip(*path), bins=dim, range=[[0, dim], [0, dim]])[0],
    curvature, twin_mask, phi_mask, prediction
]
cmaps = ['viridis', 'plasma', 'inferno', 'coolwarm', 'cividis', 'spring', 'YlGnBu']

for ax, title, d, cmap in zip(axes, titles, data, cmaps):
    ax.set_title(title)
    im = ax.imshow(d, cmap=cmap)
    fig.colorbar(im, ax=ax)

fig.tight_layout()
plt.savefig(output_path)
plt.close()
print(f"Saved visualization to: {output_path}")
