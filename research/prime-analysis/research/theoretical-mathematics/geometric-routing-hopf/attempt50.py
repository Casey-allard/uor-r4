import math
from concurrent.futures import ProcessPoolExecutor, as_completed
from bitarray import bitarray
import psutil
import time
import pickle
import gc
import os
import sys


def calculate_segment_size(M):
    """Determine segment size based on available memory."""
    total_memory = psutil.virtual_memory().available
    num_workers = psutil.cpu_count()
    segment_size = min(4 * 1024 * 1024, (total_memory // num_workers) // 128)  # 4MB cap
    return max(segment_size, 1 * 10**5)  # Minimum size


def sieve_segment(low, high, small_primes):
    """Sieve a segment of the range [low, high)."""
    segment_size = high - low
    is_prime_segment = bitarray(segment_size)
    is_prime_segment.setall(True)

    for prime in small_primes:
        start = max(prime * prime, ((low + prime - 1) // prime) * prime)
        for j in range(start, high, prime):
            is_prime_segment[j - low] = False

    return [low + i for i in range(segment_size) if is_prime_segment[i]]


def save_primes_to_file(primes, count, filename="primes.txt"):
    """Append primes to a file and keep track of count."""
    with open(filename, "a") as f:
        for prime in primes:
            f.write(f"{prime}\n")
    return count + len(primes)


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
        return None


def save_M(M, filename="M_value.txt"):
    """Save the value of M to a file."""
    with open(filename, "w") as f:
        f.write(str(M))


def load_M(filename="M_value.txt"):
    """Load the value of M from a file."""
    try:
        with open(filename, "r") as f:
            return int(f.read().strip())
    except (FileNotFoundError, ValueError):
        return None


def cleanup_processes():
    """Terminate all processes associated with the current script."""
    parent = psutil.Process(os.getpid())
    for child in parent.children(recursive=True):
        try:
            child.terminate()
            child.wait(timeout=5)
        except psutil.NoSuchProcess:
            pass
        except psutil.TimeoutExpired:
            child.kill()
    print("All child processes terminated.")


def compute_primes_segmented(M, max_chunks_before_restart=500):
    """Compute primes up to M using a segmented sieve with concurrent futures."""
    sqrt_limit = int(math.sqrt(M)) + 1
    sieve = bitarray(sqrt_limit)
    sieve.setall(True)

    for i in range(2, sqrt_limit):
        if sieve[i]:
            for j in range(i * i, sqrt_limit, i):
                sieve[j] = False

    small_primes = [i for i in range(2, sqrt_limit) if sieve[i]]

    segment_size = calculate_segment_size(M)
    print(f"Using segment size: {segment_size}")
    chunks = [(low, min(low + segment_size, M), small_primes)
              for low in range(sqrt_limit, M, segment_size)]

    total_chunks = len(chunks)
    print(f"Total chunks to process: {total_chunks}")

    last_processed_chunk = load_checkpoint()
    if last_processed_chunk is not None:
        print(f"Resuming from checkpoint: {last_processed_chunk}/{total_chunks}.")
        chunks = chunks[last_processed_chunk + 1:]
    else:
        print("No checkpoint found. Starting fresh.")

    if not chunks:
        print("All chunks have been processed. Exiting.")
        return

    start_time = time.time()
    global_start_time = start_time if last_processed_chunk is None else None
    prime_count = 0
    last_prime = None

    with ProcessPoolExecutor(max_workers=max(1, psutil.cpu_count() // 2)) as executor:
        futures = {executor.submit(sieve_segment, *chunk): chunk for chunk in chunks}
        for i, future in enumerate(as_completed(futures), 1):
            chunk_index = (last_processed_chunk or 0) + i
            try:
                result = future.result()
                last_prime = result[-1] if result else last_prime
                prime_count = save_primes_to_file(result, prime_count)  # Save and update count
                print(f"Processed chunk {chunk_index}/{total_chunks}")

                save_checkpoint(chunk_index)

                if chunk_index % max_chunks_before_restart == 0:
                    elapsed_time = time.time() - start_time
                    est_total_time = elapsed_time * total_chunks / chunk_index
                    print(f"Restarting script after processing {chunk_index} chunks...")
                    print(f"Estimated total time: {est_total_time:.2f} seconds.")
                    save_checkpoint(chunk_index)
                    save_M(M)
                    executor.shutdown(wait=False)
                    cleanup_processes()
                    os.execv(sys.executable, [sys.executable, sys.argv[0], "--resume"])

            except Exception as e:
                print(f"Error processing chunk {chunk_index}: {e}")

    total_time = time.time() - global_start_time if global_start_time else 0
    print(f"Total processing time: {total_time:.2f} seconds.")
    print(f"Number of primes found: {prime_count}")
    print(f"Largest prime found: {last_prime}")


if __name__ == "__main__":
    try:
        if "--resume" in sys.argv:
            # Auto-resume without prompt if "--resume" flag is present
            M = load_M()
            checkpoint = load_checkpoint()
        else:
            # Normal manual start
            checkpoint = load_checkpoint()
            if checkpoint is not None:
                response = input("A checkpoint was found. Would you like to resume? (y/n): ").strip().lower()
                if response != "y":
                    os.remove("checkpoint.pkl")
                    os.remove("primes.txt")
                    checkpoint = None

            if checkpoint is None:
                M = int(input("Enter the value of M: "))
                save_M(M)
            else:
                M = load_M()

        compute_primes_segmented(M)

    except KeyboardInterrupt:
        print("\nExecution interrupted by user.")
    except ValueError:
        print("Invalid input. Please enter an integer.")
    except Exception as e:
        print(f"An error occurred: {e}")
