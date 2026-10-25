// The prime-sieve frame compiler's decision (sieve_frame_compiler.rs).
//
// One thread per turn. A signature is a squarefree prime product held as its
// prime-factor bitset, so "the frame divides the turn" (gcd equals the frame)
// is "no frame bit outside the turn's bits". A surviving frame must then align
// exactly: prefix words, a value of 1..max_value_words words, suffix words
// (or exactly the frame's words, for a query). The turn compiles only if every
// aligned frame names one expert semiprime and one value span.
//
// Integer compare, AND and bounded loops only: no float, no score, no
// matrix product. Status: 0 none, 1 compiled, 2 disagree.

extern "C" __global__ void sieve_frame_compile(
    const unsigned long long* turn_sig, const unsigned int* turn_tokens,
    const unsigned int* turn_off, const unsigned int* turn_len, unsigned int n_turns,
    const unsigned long long* frame_sig, const unsigned int* frame_tokens,
    const unsigned int* frame_off, const unsigned int* frame_prefix,
    const unsigned int* frame_suffix, const unsigned int* frame_slot,
    const unsigned long long* frame_expert, unsigned int n_frames,
    unsigned int sig_words, unsigned int max_value_words,
    unsigned int* out_status, unsigned int* out_frame, unsigned int* out_hole_start,
    unsigned int* out_hole_len, unsigned int* out_survivors)
{
    const unsigned int t = blockIdx.x * blockDim.x + threadIdx.x;
    if (t >= n_turns) {
        return;
    }
    const unsigned long long* ts = turn_sig + (unsigned long long)t * sig_words;
    const unsigned int* tok = turn_tokens + turn_off[t];
    const unsigned int len = turn_len[t];
    unsigned int survivors = 0;
    unsigned int disagree = 0;
    unsigned int first = 0;
    unsigned int hole_start = 0;
    unsigned int hole_len = 0;
    unsigned long long expert = 0;
    for (unsigned int f = 0; f < n_frames; ++f) {
        const unsigned long long* fs = frame_sig + (unsigned long long)f * sig_words;
        int divides = 1;
        for (unsigned int w = 0; w < sig_words; ++w) {
            if (fs[w] & ~ts[w]) {
                divides = 0;
                break;
            }
        }
        if (!divides) {
            continue;
        }
        const unsigned int p = frame_prefix[f];
        const unsigned int s = frame_suffix[f];
        const unsigned int* ft = frame_tokens + frame_off[f];
        unsigned int hs = 0;
        unsigned int hl = 0;
        if (frame_slot[f]) {
            if (len < p + s + 1) {
                continue;
            }
            hl = len - p - s;
            if (hl > max_value_words) {
                continue;
            }
            hs = p;
        } else if (len != p + s) {
            continue;
        }
        int aligned = 1;
        for (unsigned int i = 0; i < p && aligned; ++i) {
            aligned = tok[i] == ft[i];
        }
        for (unsigned int i = 0; i < s && aligned; ++i) {
            aligned = tok[len - s + i] == ft[p + i];
        }
        if (!aligned) {
            continue;
        }
        if (survivors == 0) {
            first = f;
            expert = frame_expert[f];
            hole_start = hs;
            hole_len = hl;
        } else if (frame_expert[f] != expert || hs != hole_start || hl != hole_len) {
            disagree = 1;
        }
        ++survivors;
    }
    out_survivors[t] = survivors;
    out_frame[t] = first;
    out_hole_start[t] = hole_start;
    out_hole_len[t] = hole_len;
    out_status[t] = survivors == 0 ? 0u : (disagree ? 2u : 1u);
}
