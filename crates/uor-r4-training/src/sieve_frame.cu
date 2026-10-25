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

// The marker-gated decision (sieve_marker_gate.rs), one thread per turn.
//
// A reading strips at most max_markers discourse markers from the front of
// the turn (exact word sequences found in the training frames) and runs the
// same sieve and alignment on the rest. Markers never precede a query frame.
// An update marker reads the frame as a correction: its expert becomes
// frame_expert_update. Every surviving reading, over every strip, must name
// one expert and one value span; otherwise the turn is unresolved. Every
// word outside the value is explained by a marker or the frame, or the
// reading does not survive. Act: 0 assert, 1 update, 2 query.

#define SIEVE_MAX_SIG_WORDS 8

__device__ static void sieve_marked_reading(
    const unsigned int* tok, unsigned int len, unsigned int offset, unsigned int update,
    const unsigned long long* frame_sig, const unsigned int* frame_tokens,
    const unsigned int* frame_off, const unsigned int* frame_prefix,
    const unsigned int* frame_suffix, const unsigned int* frame_slot,
    const unsigned int* frame_act, const unsigned long long* frame_expert,
    const unsigned long long* frame_expert_update, unsigned int n_frames,
    unsigned int sig_words, unsigned int max_value_words,
    unsigned int* survivors, unsigned int* disagree, unsigned int* first,
    unsigned int* act, unsigned long long* expert, unsigned int* hole_start,
    unsigned int* hole_len)
{
    const unsigned int* sub = tok + offset;
    const unsigned int sub_len = len - offset;
    unsigned long long sub_sig[SIEVE_MAX_SIG_WORDS];
    for (unsigned int w = 0; w < sig_words; ++w) {
        sub_sig[w] = 0;
    }
    for (unsigned int i = 0; i < sub_len; ++i) {
        const unsigned int id = sub[i];
        if (id != 0) {
            const unsigned int k = id - 1;
            sub_sig[k / 64] |= 1ull << (k % 64);
        }
    }
    for (unsigned int f = 0; f < n_frames; ++f) {
        if (offset > 0 && !frame_slot[f]) {
            continue;
        }
        const unsigned long long* fs = frame_sig + (unsigned long long)f * sig_words;
        int divides = 1;
        for (unsigned int w = 0; w < sig_words; ++w) {
            if (fs[w] & ~sub_sig[w]) {
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
            if (sub_len < p + s + 1) {
                continue;
            }
            hl = sub_len - p - s;
            if (hl > max_value_words) {
                continue;
            }
            hs = offset + p;
        } else if (sub_len != p + s) {
            continue;
        }
        int aligned = 1;
        for (unsigned int i = 0; i < p && aligned; ++i) {
            aligned = sub[i] == ft[i];
        }
        for (unsigned int i = 0; i < s && aligned; ++i) {
            aligned = sub[sub_len - s + i] == ft[p + i];
        }
        if (!aligned) {
            continue;
        }
        const unsigned long long e =
            (update && frame_slot[f]) ? frame_expert_update[f] : frame_expert[f];
        const unsigned int a = frame_slot[f] ? (update ? 1u : frame_act[f]) : 2u;
        if (*survivors == 0) {
            *first = f;
            *expert = e;
            *act = a;
            *hole_start = hs;
            *hole_len = hl;
        } else if (hs == *hole_start && hl == *hole_len) {
            if (e != *expert) {
                *disagree |= 1u;
            }
        } else if (frame_slot[f] && *act != 2u && hs >= *hole_start &&
                   (hs + hl) <= (*hole_start + *hole_len) && hl < *hole_len) {
            *first = f;
            *expert = e;
            *act = a;
            *hole_start = hs;
            *hole_len = hl;
            *disagree &= ~1u;
        } else if (frame_slot[f] && *act != 2u && *hole_start >= hs &&
                   (*hole_start + *hole_len) <= (hs + hl) && *hole_len < hl) {
            // Current reading explains a strict superset of turn words; keep it.
        } else {
            *disagree |= 2u;
        }
        *survivors += 1;
    }
}

extern "C" __global__ void sieve_frame_compile_marked(
    const unsigned int* turn_tokens, const unsigned int* turn_off,
    const unsigned int* turn_len, unsigned int n_turns,
    const unsigned long long* frame_sig, const unsigned int* frame_tokens,
    const unsigned int* frame_off, const unsigned int* frame_prefix,
    const unsigned int* frame_suffix, const unsigned int* frame_slot,
    const unsigned int* frame_act, const unsigned long long* frame_expert,
    const unsigned long long* frame_expert_update, unsigned int n_frames,
    unsigned int sig_words, unsigned int max_value_words,
    const unsigned int* marker_tokens, const unsigned int* marker_off,
    const unsigned int* marker_len, const unsigned int* marker_update,
    unsigned int n_markers, unsigned int max_markers,
    unsigned int* out_status, unsigned int* out_frame, unsigned int* out_act,
    unsigned int* out_hole_start, unsigned int* out_hole_len, unsigned int* out_survivors)
{
    const unsigned int t = blockIdx.x * blockDim.x + threadIdx.x;
    if (t >= n_turns) {
        return;
    }
    if (sig_words > SIEVE_MAX_SIG_WORDS) {
        out_status[t] = 0u;
        out_frame[t] = 0u;
        out_act[t] = 0u;
        out_hole_start[t] = 0u;
        out_hole_len[t] = 0u;
        out_survivors[t] = 0u;
        return;
    }
    const unsigned int* tok = turn_tokens + turn_off[t];
    const unsigned int len = turn_len[t];
    unsigned int survivors = 0;
    unsigned int disagree = 0;
    unsigned int first = 0;
    unsigned int act = 0;
    unsigned int hole_start = 0;
    unsigned int hole_len = 0;
    unsigned long long expert = 0;
    // Reading 0: no marker, which is the sieve compiler's own reading.
    sieve_marked_reading(tok, len, 0u, 0u, frame_sig, frame_tokens, frame_off, frame_prefix,
                         frame_suffix, frame_slot, frame_act, frame_expert, frame_expert_update,
                         n_frames, sig_words, max_value_words, &survivors, &disagree, &first,
                         &act, &expert, &hole_start, &hole_len);
    for (unsigned int m1 = 0; max_markers >= 1 && m1 < n_markers; ++m1) {
        const unsigned int l1 = marker_len[m1];
        if (l1 == 0 || l1 >= len) {
            continue;
        }
        int at1 = 1;
        for (unsigned int i = 0; i < l1 && at1; ++i) {
            at1 = tok[i] == marker_tokens[marker_off[m1] + i];
        }
        if (!at1) {
            continue;
        }
        const unsigned int u1 = marker_update[m1];
        sieve_marked_reading(tok, len, l1, u1, frame_sig, frame_tokens, frame_off, frame_prefix,
                             frame_suffix, frame_slot, frame_act, frame_expert,
                             frame_expert_update, n_frames, sig_words, max_value_words,
                             &survivors, &disagree, &first, &act, &expert, &hole_start,
                             &hole_len);
        for (unsigned int m2 = 0; max_markers >= 2 && m2 < n_markers; ++m2) {
            const unsigned int l2 = marker_len[m2];
            if (l2 == 0 || l1 + l2 >= len) {
                continue;
            }
            int at2 = 1;
            for (unsigned int i = 0; i < l2 && at2; ++i) {
                at2 = tok[l1 + i] == marker_tokens[marker_off[m2] + i];
            }
            if (at2) {
                sieve_marked_reading(tok, len, l1 + l2, u1 | marker_update[m2], frame_sig,
                                     frame_tokens, frame_off, frame_prefix, frame_suffix,
                                     frame_slot, frame_act, frame_expert, frame_expert_update,
                                     n_frames, sig_words, max_value_words, &survivors,
                                     &disagree, &first, &act, &expert, &hole_start, &hole_len);
            }
        }
    }
    out_survivors[t] = survivors;
    out_frame[t] = first;
    out_act[t] = act;
    out_hole_start[t] = hole_start;
    out_hole_len[t] = hole_len;
    out_status[t] = survivors == 0 ? 0u : (disagree ? 2u : 1u);
}
