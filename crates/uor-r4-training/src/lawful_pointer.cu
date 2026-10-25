// Ahead-of-time CUDA kernel for LawfulPointerHead (lawful_pointer.rs).
//
// Pure integer arithmetic only: binary GCD on 128-bit prime products,
// bit-rank rarity weighting via __clzll, canonical token-id fold table
// lookup, and deterministic integer margin comparison. Zero floating-point
// operations, zero transcendental instructions, zero matrix products.

__device__ static int u128_is_zero(unsigned long long lo, unsigned long long hi) {
    return lo == 0ULL && hi == 0ULL;
}

__device__ static int u128_eq(
    unsigned long long a_lo, unsigned long long a_hi,
    unsigned long long b_lo, unsigned long long b_hi)
{
    return a_lo == b_lo && a_hi == b_hi;
}

__device__ static int u128_gt(
    unsigned long long a_lo, unsigned long long a_hi,
    unsigned long long b_lo, unsigned long long b_hi)
{
    return (a_hi > b_hi) || (a_hi == b_hi && a_lo > b_lo);
}

__device__ static void u128_shr1(unsigned long long* lo, unsigned long long* hi) {
    *lo = (*lo >> 1) | (*hi << 63);
    *hi = *hi >> 1;
}

__device__ static void u128_shl(
    unsigned long long* lo, unsigned long long* hi, unsigned int shift)
{
    if (shift == 0u) {
        return;
    }
    if (shift >= 128u) {
        *lo = 0ULL;
        *hi = 0ULL;
    } else if (shift >= 64u) {
        *hi = *lo << (shift - 64u);
        *lo = 0ULL;
    } else {
        *hi = (*hi << shift) | (*lo >> (64u - shift));
        *lo = *lo << shift;
    }
}

__device__ static void u128_sub(
    unsigned long long a_lo, unsigned long long a_hi,
    unsigned long long b_lo, unsigned long long b_hi,
    unsigned long long* out_lo, unsigned long long* out_hi)
{
    unsigned long long borrow = (a_lo < b_lo) ? 1ULL : 0ULL;
    *out_lo = a_lo - b_lo;
    *out_hi = a_hi - b_hi - borrow;
}

// Binary greatest common divisor (Stein's algorithm) on 128-bit unsigned integers.
// Uses only shifts, bitwise ops, comparisons, and subtraction (zero division).
__device__ static void gcd_u128_dev(
    unsigned long long a_lo, unsigned long long a_hi,
    unsigned long long b_lo, unsigned long long b_hi,
    unsigned long long* out_lo, unsigned long long* out_hi)
{
    if (u128_is_zero(a_lo, a_hi)) {
        *out_lo = b_lo;
        *out_hi = b_hi;
        return;
    }
    if (u128_is_zero(b_lo, b_hi)) {
        *out_lo = a_lo;
        *out_hi = a_hi;
        return;
    }
    unsigned int shift = 0u;
    while (((a_lo | b_lo) & 1ULL) == 0ULL) {
        u128_shr1(&a_lo, &a_hi);
        u128_shr1(&b_lo, &b_hi);
        ++shift;
    }
    while ((a_lo & 1ULL) == 0ULL) {
        u128_shr1(&a_lo, &a_hi);
    }
    while (!u128_is_zero(b_lo, b_hi)) {
        while ((b_lo & 1ULL) == 0ULL) {
            u128_shr1(&b_lo, &b_hi);
        }
        if (u128_gt(a_lo, a_hi, b_lo, b_hi)) {
            unsigned long long tmp_lo = a_lo;
            unsigned long long tmp_hi = a_hi;
            a_lo = b_lo;
            a_hi = b_hi;
            b_lo = tmp_lo;
            b_hi = tmp_hi;
        }
        u128_sub(b_lo, b_hi, a_lo, a_hi, &b_lo, &b_hi);
    }
    u128_shl(&a_lo, &a_hi, shift);
    *out_lo = a_lo;
    *out_hi = a_hi;
}

__device__ static int divides_u128_dev(
    unsigned long long divisor_lo, unsigned long long divisor_hi,
    unsigned long long cand_lo, unsigned long long cand_hi)
{
    if (u128_is_zero(divisor_lo, divisor_hi) || u128_is_zero(cand_lo, cand_hi)) {
        return 0;
    }
    unsigned long long g_lo = 0ULL;
    unsigned long long g_hi = 0ULL;
    gcd_u128_dev(divisor_lo, divisor_hi, cand_lo, cand_hi, &g_lo, &g_hi);
    return u128_eq(g_lo, g_hi, divisor_lo, divisor_hi);
}

__device__ static long long bit_rank_weight_q8_dev(
    unsigned long long d_lo, unsigned long long d_hi)
{
    if (d_hi == 0ULL && d_lo <= 1ULL) {
        return 0LL;
    }
    if (d_hi != 0ULL) {
        int lz = __clzll((long long)d_hi);
        return ((long long)(127 - lz)) << 8;
    }
    int lz = __clzll((long long)d_lo);
    return ((long long)(63 - lz)) << 8;
}

__device__ static long long sat_add_i64_dev(long long a, long long b) {
    unsigned long long ua = (unsigned long long)a;
    unsigned long long ub = (unsigned long long)b;
    unsigned long long usum = ua + ub;
    long long sum = (long long)usum;
    if (a >= 0LL && b >= 0LL && sum < 0LL) {
        return 9223372036854775807LL;
    }
    if (a < 0LL && b < 0LL && sum >= 0LL) {
        return -9223372036854775807LL - 1LL;
    }
    return sum;
}

extern "C" __global__ void lawful_pointer_select_batch(
    unsigned int n_items,
    unsigned int admission_mode,
    unsigned int nlet_width,
    long long gate_threshold_margin,
    const unsigned int* fold_table,
    unsigned int fold_len,
    const unsigned int* window_tokens,
    const unsigned int* window_off,
    const unsigned int* window_len,
    const long long* base_margins,
    const unsigned int* base_off,
    const unsigned int* base_len,
    const long long* learned_gate_margin,
    const unsigned int* step_offset,
    const unsigned int* query_has_slot,
    const unsigned long long* query_slot_lo,
    const unsigned long long* query_slot_hi,
    const unsigned long long* query_prod_lo,
    const unsigned long long* query_prod_hi,
    const unsigned int* query_prec_tokens,
    const unsigned int* query_prec_off,
    const unsigned int* query_prec_len,
    const unsigned int* cand_off,
    const unsigned int* cand_count,
    const unsigned int* cand_span_start,
    const unsigned int* cand_span_end,
    const unsigned long long* cand_slot_lo,
    const unsigned long long* cand_slot_hi,
    const unsigned int* cand_prec_tokens,
    const unsigned int* cand_prec_off,
    const unsigned int* cand_prec_len,
    const long long* cand_rank_bonus,
    const unsigned int* oracle_mode,
    const unsigned int* oracle_start,
    const unsigned int* oracle_end,
    const long long* oracle_forced_gate,
    unsigned int* out_status,
    unsigned int* out_emitted_token,
    unsigned int* out_base_top_token,
    long long* out_base_top_margin,
    unsigned int* out_copied,
    unsigned int* out_has_span,
    unsigned int* out_span_start,
    unsigned int* out_span_end,
    unsigned int* out_has_copy_token,
    unsigned int* out_candidate_copy_token,
    unsigned int* out_has_route_score,
    long long* out_route_score,
    long long* out_effective_gate_margin)
{
    const unsigned int r = blockIdx.x * blockDim.x + threadIdx.x;
    if (r >= n_items) {
        return;
    }

    const unsigned int w_off = window_off[r];
    const unsigned int w_len = window_len[r];
    const unsigned int b_off = base_off[r];
    const unsigned int b_len = base_len[r];
    const unsigned int c_off = cand_off[r];
    const unsigned int c_cnt = cand_count[r];
    const unsigned int o_mode = oracle_mode[r];
    const unsigned int o_start = oracle_start[r];
    const unsigned int o_end = oracle_end[r];

    if (b_len == 0u) {
        out_status[r] = 1u;
        return;
    }
    if (o_mode == 1u || o_mode == 2u) {
        if (o_start >= o_end || o_end > w_len) {
            out_status[r] = 1u;
            return;
        }
    } else if (o_mode != 0u) {
        out_status[r] = 1u;
        return;
    }

    for (unsigned int i = 0u; i < c_cnt; ++i) {
        const unsigned int cs = cand_span_start[c_off + i];
        const unsigned int ce = cand_span_end[c_off + i];
        if (cs >= ce || ce > w_len) {
            out_status[r] = 1u;
            return;
        }
    }
    for (unsigned int i = 0u; i < w_len; ++i) {
        if (window_tokens[w_off + i] >= fold_len) {
            out_status[r] = 1u;
            return;
        }
    }

    unsigned int base_top_tok = 0u;
    long long base_top_m = base_margins[b_off];
    for (unsigned int idx = 1u; idx < b_len; ++idx) {
        long long m = base_margins[b_off + idx];
        if (m > base_top_m) {
            base_top_m = m;
            base_top_tok = idx;
        }
    }

    unsigned int has_sel = 0u;
    unsigned int sel_start = 0u;
    unsigned int sel_end = 0u;
    long long sel_score = 0LL;
    long long eff_gate = learned_gate_margin[r];

    if (o_mode == 0u) {
        for (unsigned int i = 0u; i < c_cnt; ++i) {
            const unsigned int ci = c_off + i;
            const unsigned int cs = cand_span_start[ci];
            const unsigned int ce = cand_span_end[ci];
            int admitted = 0;
            long long score = 0LL;

            if (admission_mode == 0u) {
                if (query_has_slot[r] != 0u) {
                    const unsigned long long qs_lo = query_slot_lo[r];
                    const unsigned long long qs_hi = query_slot_hi[r];
                    if (divides_u128_dev(qs_lo, qs_hi, cand_slot_lo[ci], cand_slot_hi[ci])) {
                        long long w = bit_rank_weight_q8_dev(qs_lo, qs_hi);
                        score = sat_add_i64_dev(w, cand_rank_bonus[ci]);
                        admitted = 1;
                    }
                }
            } else if (admission_mode == 1u) {
                unsigned long long g_lo = 0ULL;
                unsigned long long g_hi = 0ULL;
                gcd_u128_dev(
                    query_prod_lo[r], query_prod_hi[r],
                    cand_slot_lo[ci], cand_slot_hi[ci],
                    &g_lo, &g_hi);
                if (g_hi != 0ULL || g_lo > 1ULL) {
                    long long w = bit_rank_weight_q8_dev(g_lo, g_hi);
                    score = sat_add_i64_dev(w, cand_rank_bonus[ci]);
                    admitted = 1;
                }
            } else if (admission_mode == 2u) {
                const unsigned int cp_len = cand_prec_len[ci];
                const unsigned int qp_len = query_prec_len[r];
                if (nlet_width > 0u && cp_len >= nlet_width && qp_len >= nlet_width) {
                    const unsigned int cp_base = cand_prec_off[ci] + (cp_len - nlet_width);
                    const unsigned int qp_base = query_prec_off[r] + (qp_len - nlet_width);
                    int match_all = 1;
                    for (unsigned int k = 0u; k < nlet_width; ++k) {
                        const unsigned int c_tok = cand_prec_tokens[cp_base + k];
                        const unsigned int q_tok = query_prec_tokens[qp_base + k];
                        if (c_tok >= fold_len || q_tok >= fold_len) {
                            out_status[r] = 1u;
                            return;
                        }
                        if (fold_table[c_tok] != fold_table[q_tok]) {
                            match_all = 0;
                            break;
                        }
                    }
                    if (match_all) {
                        long long w = ((long long)nlet_width) << 8;
                        score = sat_add_i64_dev(w, cand_rank_bonus[ci]);
                        admitted = 1;
                    }
                }
            } else {
                out_status[r] = 1u;
                return;
            }

            if (admitted) {
                if (has_sel == 0u || score > sel_score || (score == sel_score && cs > sel_start)) {
                    has_sel = 1u;
                    sel_start = cs;
                    sel_end = ce;
                    sel_score = score;
                }
            }
        }
    } else {
        long long rank_bonus = 0LL;
        for (unsigned int rev = 0u; rev < c_cnt; ++rev) {
            const unsigned int ci = c_off + (c_cnt - 1u - rev);
            if (cand_span_start[ci] == o_start && cand_span_end[ci] == o_end) {
                rank_bonus = cand_rank_bonus[ci];
                break;
            }
        }
        long long w = 0LL;
        if (admission_mode == 0u) {
            if (query_has_slot[r] != 0u) {
                w = bit_rank_weight_q8_dev(query_slot_lo[r], query_slot_hi[r]);
            } else {
                w = bit_rank_weight_q8_dev(query_prod_lo[r], query_prod_hi[r]);
            }
        } else if (admission_mode == 1u) {
            w = bit_rank_weight_q8_dev(query_prod_lo[r], query_prod_hi[r]);
        } else if (admission_mode == 2u) {
            w = ((long long)nlet_width) << 8;
        } else {
            out_status[r] = 1u;
            return;
        }
        has_sel = 1u;
        sel_start = o_start;
        sel_end = o_end;
        sel_score = sat_add_i64_dev(w, rank_bonus);
        eff_gate = (o_mode == 1u) ? learned_gate_margin[r] : oracle_forced_gate[r];
    }

    out_status[r] = 0u;
    out_base_top_token[r] = base_top_tok;
    out_base_top_margin[r] = base_top_m;
    out_effective_gate_margin[r] = eff_gate;

    if (has_sel == 0u) {
        out_emitted_token[r] = base_top_tok;
        out_copied[r] = 0u;
        out_has_span[r] = 0u;
        out_span_start[r] = 0u;
        out_span_end[r] = 0u;
        out_has_copy_token[r] = 0u;
        out_candidate_copy_token[r] = 0u;
        out_has_route_score[r] = 0u;
        out_route_score[r] = 0LL;
        return;
    }

    out_has_span[r] = 1u;
    out_span_start[r] = sel_start;
    out_span_end[r] = sel_end;
    out_has_route_score[r] = 1u;
    out_route_score[r] = sel_score;

    const unsigned int span_len = sel_end - sel_start;
    const unsigned int step_off = step_offset[r];
    if (step_off >= span_len) {
        out_emitted_token[r] = base_top_tok;
        out_copied[r] = 0u;
        out_has_copy_token[r] = 0u;
        out_candidate_copy_token[r] = 0u;
        return;
    }

    const unsigned int copy_tok = window_tokens[w_off + sel_start + step_off];
    const long long total_copy_m = sat_add_i64_dev(sel_score, eff_gate);
    const unsigned int gate_open =
        (eff_gate >= gate_threshold_margin && total_copy_m > base_top_m) ? 1u : 0u;

    out_emitted_token[r] = gate_open ? copy_tok : base_top_tok;
    out_copied[r] = gate_open;
    out_has_copy_token[r] = 1u;
    out_candidate_copy_token[r] = copy_tok;
}
