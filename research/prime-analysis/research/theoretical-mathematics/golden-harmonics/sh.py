import numpy as np
from sympy import primerange, isprime
from sympy.physics.quantum.spin import Ylm

def calculate_spherical_harmonics(fib_prime, numbers_range):
    results = []
    for number in numbers_range:
        # Assigning theoretical values to theta and phi
        theta = np.pi / 4 if isprime(number) else np.pi / 2  # Example assignment
        phi = np.pi / number  # Example assignment

        # Spherical harmonics calculation for each m in the range -l to l
        for m in range(-fib_prime, fib_prime + 1):
            ylm = Ylm(fib_prime, m, theta, phi).doit()
            results.append((number, m, ylm))

    return results

# Fibonacci primes and their ranges for analysis
fibonacci_primes = [2, 3, 5, 13]
ranges = {
    2: range(1, 3),
    3: range(1, 4),
    5: range(1, 6),
    13: range(1, 14)
}

# Calculating spherical harmonics for each Fibonacci prime
harmonics_results = {}
for fib_prime in fibonacci_primes:
    numbers_range = ranges[fib_prime]
    harmonics_results[fib_prime] = calculate_spherical_harmonics(fib_prime, numbers_range)

# Displaying results (simplified for illustration)
for fib_prime, results in harmonics_results.items():
    print(f"Fibonacci Prime: {fib_prime}")
    for res in results:
        print(f"Number: {res[0]}, m: {res[1]}, Ylm: {res[2]}")
    print("\n")


