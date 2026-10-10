# Exact addressed memory

**The idea (first principles):**
- A fact should be stored at an **address** and read back exactly, not smeared across weights or recovered by a learned similarity.
- The project addresses by **prime atoms**. Every token has a registered prime, and a relation between two atoms is the square-free **semiprime** `p_a·p_b`. By unique factorization, that product names the unordered pair exactly, so the key is the same whichever role a tagger gave each atom.
- The value is routed *through* the expert, not multiplied into it, so direction survives: "Alex's friend is Sam" does not answer "Sam's friend".
- The `Sieve` read policy is a divisibility (gcd-style) membership filter over the clause's atoms. These are the owner's 09-29 prime-router principles: primes as data, gcd sieve, semiprime experts, n-lets.
- A second, learned form is **product-key memory**. It is sparse slots, optionally addressed by a *fixed geometric codebook*: the 120 unit icosians (600-cell, H4) or the 240 E8 roots. With a fixed codebook only the query map and the values learn.
- Sources: `stack_prime_route.rs` docs, `stack_memory.rs` docs, ADR-0003 (`docs/adr/0003-fixed-zeta-prime-route-attention.md`), AGENTS.md.

**What it replaces and why that matters:**
- It replaces recall by attention, where the model must *find* a value by soft similarity over the whole context, and dense MLP lookup.
- Exact identity is a project invariant: a hash or prime identity is not a semantic distance, and occurrence/version identity must survive.
- It targets M1 durable memory (D19's lead goal) and D11 serving, where reads are integer table lookups. Its cost is `heads·top_k` rows of a large table per token (D5's sparse end state), not all dense weights.

**Where it lives in code:**
- `crates/uor-r4-training/src/stack_memory.rs`: `MemoryScore::{Dot,Lorentz}`, `Codebook::{H4,E8}`. Flags: `memory_layers=L1,L2 memory_sub_keys=256 memory_top_k=32` (`examples/geometric-stack.rs`).
- `stack_prime_route.rs`: `ReadPolicy::{Tagged,Sieve}`. It is an evaluation instrument for the relation store (`stack_aerm`, G v1).

**What has been tried, honestly:** The M1 #2029 line used a learned *pointer/copy* read, not an addressed store.
- #2127: the value's position is kept, but the attention never lands on it.
- #2128: the pointer attends the sentence **frame**, never the varying slot.
- #2133: the frame carries the copy mass (trace gate 166/166).
- #2161: a token-identity pointer cut wrong-value answers 17→12, but memory stayed **19/40**. That pointer line was archived at 3/3.
- Net: these were negatives for a soft pointer, not for exact addressing.
- The addressed-memory operator named in DeepSeek's pivot card was **not built**, because `DialogueSettings` has no memory fields. So the idea is **untested** on the M1 panel.
- Product-key layers are wired and round-trip-tested, but no scored M1 result was found on main.

**What success looks like:**
- Own metrics: exact recall of asserted facts at any distance, with version-correct answers (current vs previous value) and correct abstention after eviction.
- Milestone: M1 v4/v5 multi-turn memory well above the 19/40 pointer ceiling, with no BPB loss.

**What failure looks like:**
- With a correct write tagger and a trained-in read, the panel does not improve: failures move to *retrieval* (right address, unused value) rather than *tagging*.
- Or prime keys alias distinct facts in real dialogue (roles not disjoint), so exact identity buys nothing over learned keys.

**A fair test:**
- Add memory fields to `DialogueSettings` and train the store and read in from pretraining, or with a full fine-tune; no bolt-on.
- Use ≥20M params, 2–3 seeds, and the frozen v4/v5 panel with ≥80 rows; ±3/40 is the noise seen.
- Report tagger accuracy separately from read accuracy.
- Bar: ≥ +8/40 over 19/40, with BPB within 0.01.

**Open design questions:**
1. Ordered n-let keys (prime powers or positional exponents) for relations whose atoms are not role-disjoint.
2. A learned gate from the store's exact hit into the logits, versus copying the value token directly.
3. H4-codebook product keys as the *write* address for facts, with prime semiprimes as the exact check.
4. Eviction proofs (ring-slot overwrite) driving abstention.
