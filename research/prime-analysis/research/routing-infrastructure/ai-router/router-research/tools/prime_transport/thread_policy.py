"""
CPU thread policy for prime transport router.

Replaces the old hard-coded `torch.set_num_threads(1)`.

POLICY (updated):
  - Default: min(8, os.cpu_count()) — use available cores by default.
  - 1 thread is ONLY used when explicitly requested via PRIME_THREADS=1
    or override=1.
  - Can always be overridden via PRIME_THREADS env var or explicit arg.
  - Resolved thread count is printed at startup.

Change from previous version:
  - Removed FLOP-based auto-detection that silently fell back to 1 thread
    at small D/batch sizes. The old default produced 1 thread for all
    typical experiment configs (D=32, B=256/512) even without any explicit
    request. New default is min(8, cpu_count) unconditionally.
"""
import os
import torch


def select_threads(batch_size: int = 0, d_in: int = 0, d_hidden: int = 0,
                   override: int | None = None, verbose: bool = True) -> int:
    """Select and apply thread count.

    Priority:
      1. PRIME_THREADS env var (if set)
      2. explicit override arg
      3. default: min(8, os.cpu_count())

    The old FLOP-crossover heuristic that defaulted to 1 thread has been
    removed. Use override=1 or PRIME_THREADS=1 if a single thread is needed.

    Returns the chosen thread count.
    """
    # Check env override first
    env_val = os.environ.get("PRIME_THREADS")
    if env_val is not None:
        n = int(env_val)
        torch.set_num_threads(n)
        if verbose:
            print(f"  [thread_policy] {n} thread(s) — from PRIME_THREADS env var")
        return n

    # Explicit override
    if override is not None:
        torch.set_num_threads(override)
        if verbose:
            print(f"  [thread_policy] {override} thread(s) — explicit override")
        return override

    # Default: min(8, cpu_count)
    cpu_count = os.cpu_count() or 1
    n = min(8, cpu_count)
    torch.set_num_threads(n)
    if verbose:
        print(f"  [thread_policy] {n} thread(s) — default min(8, cpu_count={cpu_count})")
    return n
