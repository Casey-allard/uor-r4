import sympy as sp
import time

# Function to apply the equation and check for a prime
def check_prime_equation(mersenne_exponent, other_prime):
    P = 2**mersenne_exponent - 1
    Q = other_prime
    K = (P + Q) // 2
    is_prime = sp.isprime(K)
    return P, Q, K, is_prime

# Select a very large Mersenne prime exponent
mersenne_exponent = 82589933  # Using 2^82589933 - 1

# Initialize variables to track the found prime
prime_found = False
found_prime_info = None

# Iterate over a wider range of primes for Q
start_time = time.time()
for i in range(1, 1000000):  # Adjust the range as needed
    other_prime = sp.nextprime(2**20 + i)
    P, Q, K, is_prime = check_prime_equation(mersenne_exponent, other_prime)
    if is_prime:
        prime_found = True
        found_prime_info = (P, Q, K, is_prime)
        break
end_time = time.time()

# Output the results
if prime_found:
    print(f"New large prime found: K = {found_prime_info[2]}")
    print(f"Time taken: {end_time - start_time} seconds")
else:
    print("No new prime found in the given range.")
    print(f"Time taken: {end_time - start_time} seconds")

