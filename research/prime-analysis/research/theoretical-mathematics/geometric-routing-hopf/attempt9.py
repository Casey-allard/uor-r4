import math
from multiprocessing import Pool, cpu_count
from bitarray import bitarray
import psutil
import time
import pickle
import gc


def calculate_segment_size(M):
    """Determine segment size based on available memory and M."""
    total_memory = psutil.virtual_memory().available
    num_workers = cpu_count()
    # Aggressive reduction of max segment size
    max_segment_size = min(8 * 1024 * 1024, (total_memory // num_workers) // 64)  # 8MB cap per worker
    return min(max(10**5, max_segment_size), M // num_workers)


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


def save_primes_to_file(primes, filename="primes.txt"):
    """Append primes to a file."""
    with open(filename, "a") as f:
        for prime in primes:
            f.write(f"{prime}\n")


def save_checkpoint(last_processed_chunk, checkpoint_file="checkpoint.pkl"):
    """Save intermediate results."""
    with open(checkpoint_file, "wb") as f:
        pickle.dump(last_processed_chunk, f)


def load_checkpoint(checkpoint_file="checkpoint.pkl"):
    """Load intermediate results."""
    try:
        with open(checkpoint_file, "rb") as f:
            return pickle.load(f)
    except (FileNotFoundError, EOFError, pickle.UnpicklingError):
        print("Checkpoint file missing or corrupted. Starting fresh.")
        return None


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

    # Load checkpoint if available
    last_processed_chunk = load_checkpoint()
    all_primes = []
    if last_processed_chunk:
        print(f"Resuming from checkpoint: {last_processed_chunk} processed.")
        chunks = chunks[last_processed_chunk:]  # Skip already processed chunks

    # Use multiprocessing to process chunks
    worker_count = max(1, cpu_count() // 4)
    print(f"Using {worker_count} workers.")
    start_time = time.time()

    try:
        with Pool(processes=worker_count) as pool:
            for i, result in enumerate(pool.imap(process_chunk, chunks), 1):
                current_chunk = last_processed_chunk + i if last_processed_chunk else i
                print(f"Processed chunk {current_chunk}/{len(chunks)}")
                save_primes_to_file(result)  # Save chunk results immediately
                save_checkpoint(current_chunk)  # Save checkpoint for redundancy
                gc.collect()  # Force garbage collection
    except KeyboardInterrupt:
        print("Execution interrupted by user.")
    except Exception as e:
        print(f"Multiprocessing error: {e}")
    finally:
        print("Shutting down workers...")

    end_time = time.time()
    print(f"Total time: {end_time - start_time:.2f}s")
    return all_primes


if __name__ == "__main__":
    try:
        M = int(input("Enter the value of M: "))
        start_time = time.time()
        primes = compute_primes_segmented(M)
        end_time = time.time()

        if primes:
            print(f"Number of primes up to {M}: {len(primes)}")
            print(f"Last Prime Found: {primes[-1]}")
        else:
            print("No primes found or computation failed.")

        print(f"Time taken: {end_time - start_time:.2f} seconds")
        print(f"{math.ceil(math.log(M, 2))} bits required for Max {M}")
    except KeyboardInterrupt:
        print("\nExecution interrupted by user.")
    except ValueError:
        print("Invalid input. Please enter an integer.")
    except Exception as e:
        print(f"An error occurred: {e}")
