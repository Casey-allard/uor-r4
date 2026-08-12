import math
from concurrent.futures import ProcessPoolExecutor, as_completed
from bitarray import bitarray
import psutil
import time
import pickle
import gc


def calculate_segment_size(M):
    """Determine segment size based on available memory."""
    total_memory = psutil.virtual_memory().available
    num_workers = psutil.cpu_count()
    segment_size = min(16 * 1024 * 1024, (total_memory // num_workers) // 64)
    return max(segment_size, 5 * 10**5)


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


def log_memory_usage():
    """Log memory and swap usage for debugging."""
    mem = psutil.virtual_memory()
    swap = psutil.swap_memory()
    print(f"Memory usage: {mem.used / (1024**3):.2f} GB, Swap usage: {swap.used / (1024**3):.2f} GB")


def compute_primes_segmented(M):
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
        if last_processed_chunk >= total_chunks:
            print("Checkpoint exceeds the current range. Resetting checkpoint.")
            last_processed_chunk = None
            save_checkpoint(None)
        else:
            print(f"Resuming from checkpoint: {last_processed_chunk} processed.")
            chunks = chunks[last_processed_chunk:]

    buffer = []
    start_time = time.time()

    # Use ProcessPoolExecutor for parallel processing
    with ProcessPoolExecutor(max_workers=max(1, psutil.cpu_count() // 2)) as executor:
        futures = {executor.submit(sieve_segment, *chunk): chunk for chunk in chunks}
        for i, future in enumerate(as_completed(futures), 1):
            chunk_index = (last_processed_chunk or 0) + i
            try:
                result = future.result()
                buffer.extend(result)

                print(f"Processed chunk {chunk_index}/{total_chunks}")

                if len(buffer) >= 50000:
                    save_primes_to_file(buffer)
                    buffer.clear()

                if chunk_index % 100 == 0:
                    log_memory_usage()

                save_checkpoint(chunk_index)
            except Exception as e:
                print(f"Error processing chunk {futures[future]}: {e}")

    if buffer:
        save_primes_to_file(buffer)

    end_time = time.time()
    print(f"Total time: {end_time - start_time:.2f}s")


if __name__ == "__main__":
    try:
        M = int(input("Enter the value of M: "))
        start_time = time.time()
        compute_primes_segmented(M)
        end_time = time.time()

        # Post-processing: Count primes and find the highest prime
        primes_count = 0
        highest_prime = None
        try:
            with open("primes.txt", "r") as f:
                for line in f:
                    primes_count += 1
                    highest_prime = int(line.strip())
        except FileNotFoundError:
            print("Primes file not found. No primes were saved.")

        if primes_count > 0:
            print(f"Number of primes up to {M}: {primes_count}")
            print(f"Last Prime Found: {highest_prime}")
        else:
            print("No primes were found.")

        print(f"Time taken: {end_time - start_time:.2f} seconds")
        print(f"{math.ceil(math.log(M, 2))} bits required for Max {M}")
    except KeyboardInterrupt:
        print("\nExecution interrupted by user.")
    except ValueError:
        print("Invalid input. Please enter an integer.")
    except Exception as e:
        print(f"An error occurred: {e}")
