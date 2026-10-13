import math
from multiprocessing import Pool, cpu_count
from bitarray import bitarray
import psutil
import time


def calculate_segment_size(M):
    """Determine segment size based on available memory and M."""
    total_memory = psutil.virtual_memory().available
    num_workers = cpu_count()
    max_segment_size = (total_memory // num_workers) // 8  # Approx memory per worker
    return min(max(10**6, max_segment_size), M // num_workers)


def sieve_segment(low, high, small_primes):
    """Sieve a segment of the range [low, high)."""
    segment_size = high - low
    is_prime_segment = bitarray(segment_size)
    is_prime_segment.setall(True)

    try:
        for prime in small_primes:
            start = max(prime * prime, ((low + prime - 1) // prime) * prime)
            for j in range(start, high, prime):
                is_prime_segment[j - low] = False
        primes = [low + i for i in range(segment_size) if is_prime_segment[i]]
    except Exception as e:
        print(f"Error processing segment {low}-{high}: {e}")
        primes = []  # Return empty list on failure

    return primes


def process_chunk(args):
    """Helper function to unpack arguments for multiprocessing."""
    return sieve_segment(*args)


def compute_primes_segmented(M):
    """Compute primes up to M using a segmented sieve with multiprocessing."""
    sqrt_limit = int(math.sqrt(M)) + 1
    sieve = bitarray(sqrt_limit)
    sieve.setall(True)

    # Regular sieve to find small primes up to sqrt(M)
    for i in range(2, sqrt_limit):
        if sieve[i]:
            for j in range(i * i, sqrt_limit, i):
                sieve[j] = False

    small_primes = [i for i in range(2, sqrt_limit) if sieve[i]]

    # Dynamic segment size calculation
    segment_size = calculate_segment_size(M)
    print(f"Using dynamic segment size: {segment_size}")

    # Create chunks
    chunks = [(low, min(low + segment_size, M), small_primes)
              for low in range(sqrt_limit, M, segment_size)]

    # Use multiprocessing to process chunks
    all_primes = small_primes
    start_time = time.time()
    with Pool(processes=cpu_count()) as pool:
        for i, result in enumerate(pool.imap(process_chunk, chunks), 1):
            print(f"Processed chunk {i}/{len(chunks)}")
            all_primes.extend(result)

    end_time = time.time()
    print(f"Total time: {end_time - start_time:.2f}s")
    return all_primes


if __name__ == "__main__":
    M = int(input("Enter the value of M: "))
    start_time = time.time()
    primes = compute_primes_segmented(M)
    end_time = time.time()

    print(f"Number of primes up to {M}: {len(primes)}")
    print(f"Last Prime Found: {primes[-1]}")
    print(f"Time taken: {end_time - start_time:.2f} seconds")
    print(f"{math.ceil(math.log(M, 2))} bits required for Max {M}")
