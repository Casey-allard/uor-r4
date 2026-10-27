// Clean-room port of IntegerDiscreteModel::evaluate_prefix
// (crates/uor-r4-integer/src/discrete_d11.rs:1165-1256) and its reads:
// route_read (:856), flock_or_relation_read (:919-1146), address_distance and
// token_distance (:1147-1164). The row plan (build_row_plan, :518) comes from
// the host as a table. Integer only: no floating point, no squashing, no dense
// product. One call scores one row.
//
// The same source compiles twice: as host C++ (src/host_kernel.cpp, for the
// finite parity tests against IntegerDiscreteModel) and as CUDA device code
// (src/scorer.cu). The includer defines D11_HD (function qualifier) and
// D11_CONST (table storage) before including this header.
#ifndef D11_KERNEL_CORE_H
#define D11_KERNEL_CORE_H

#include <stdint.h>

#include "d11_tables.h"

#define D11_MAX_CONTEXT 243
#define D11_MAX_NODES 512
#define D11_MAX_FRONTIER 32
#define D11_MAX_BLOCKS 16
#define D11_MAX_STAGES 8
#define D11_MAX_KEPT 8
#define D11_EDGE_TYPES 5
#define D11_MAX_KEPT_CELLS 16
#define D11_IDENTITY 0

#define D11_READ_ROUTE 0
#define D11_READ_FLOCK 1
#define D11_READ_RELATION 2

// Status codes; 0 is success. The host maps them to the reference's errors.
#define D11_OK 0
#define D11_ERR_PREFIX_LENGTH 1
#define D11_ERR_PREFIX_TOKEN 2
#define D11_ERR_PLAN 3
#define D11_ERR_ORBIT 4
#define D11_ERR_ROUTE 5
#define D11_ERR_ZERO_TOKENS 6
#define D11_ERR_SHAPE 7
#define D11_ERR_TARGET_TOKEN 8

typedef struct {
    int32_t c0;
    int32_t c1;
    int32_t c2;
    int32_t atom;
    int32_t level;
    int32_t first;
    int32_t last;
} D11Node;

// The binder plan of one prefix length, precomputed by the host
// (kernel::row_plans, a port of build_row_plan, discrete_d11.rs:518-560): it is
// a function of the length only. Every field is a 32-bit integer.
typedef struct {
    int32_t node_count;
    int32_t frontier_len;
    int32_t frontier[D11_MAX_FRONTIER];
    D11Node nodes[D11_MAX_NODES];
} D11Plan;

typedef struct {
    int32_t vocabulary;
    int32_t blocks;
    int32_t context;
    int32_t kept_cells;
    int32_t stages;
    int32_t read_mode;
    int32_t read_kept;
    int32_t read_sweeps;
    const int32_t* table;   // vocabulary * blocks * 8
    const uint32_t* cells;  // vocabulary
    const uint32_t* routes; // blocks * stages
    const int32_t* gates;   // blocks * stages, then stages * 5 for Relation
    const D11Plan* plans;   // context records: prefix lengths 1..=context
    const uint32_t* trunk;  // optional 19-table trunk data (trunk[0] == 1 when active)
} D11Model;

typedef struct {
    int32_t status;
    int32_t admitted_count;
    int32_t top8_count;
    uint32_t prediction;
    uint32_t admitted[D11_MAX_KEPT_CELLS];
    uint32_t top8[8];
    int32_t read_vector[D11_MAX_BLOCKS * 8];
    uint32_t token_hinge;
    uint32_t address_hinge;
    uint32_t rank_hinge;
    uint32_t loss_ta;
    uint32_t loss_tr;
} D11RowOut;

typedef struct {
    int32_t frame[D11_MAX_NODES];
    int32_t payload[D11_MAX_NODES][8];
    int32_t prev[D11_MAX_NODES];
    int32_t rel[D11_MAX_NODES];
} D11Scratch;

D11_HD static uint32_t d11_abs_diff(int32_t a, int32_t b) {
    return a >= b ? (uint32_t)(a - b) : (uint32_t)(b - a);
}

D11_HD static const int32_t* d11_token_block(const D11Model* m, uint32_t token, int32_t block) {
    return m->table + ((int64_t)token * m->blocks + block) * 8;
}

// root_id (:431).
D11_HD static int32_t d11_root_id(const int32_t* x) {
    for (int32_t r = 0; r < 16; ++r) {
        int32_t same = 1;
        for (int32_t d = 0; d < 8; ++d) {
            if (x[d] != (int32_t)D11_ROOTS[r][d]) {
                same = 0;
            }
        }
        if (same) {
            return r;
        }
    }
    return -1;
}

// action (:435).
D11_HD static void d11_action(int32_t a, const int32_t* x, int32_t* y) {
    int32_t base = a << 3;
    for (int32_t d = 0; d < 8; ++d) {
        int32_t axis = (int32_t)D11_AXES[base + d];
        int32_t v = x[axis & 7];
        y[d] = (axis & 8) != 0 ? -v : v;
    }
}

// compose (:446).
D11_HD static int32_t d11_compose(int32_t a, int32_t b) {
    return (int32_t)D11_COMPOSE[(a << 6) + b];
}

// root_cost (:450).
D11_HD static void d11_root_cost(const int32_t* x, int32_t root, uint32_t* distance, uint32_t* hamming) {
    uint32_t dist = 0;
    uint32_t ham = 0;
    for (int32_t d = 0; d < 8; ++d) {
        int32_t y = (int32_t)D11_ROOTS[root][d];
        dist += d11_abs_diff(x[d], y);
        ham += x[d] != y ? 1u : 0u;
    }
    *distance = dist;
    *hamming = ham;
}

// snap (:461): the nearest n-fold orbit root, by repeated addition.
D11_HD static int32_t d11_snap(const int32_t* x, int32_t n) {
    int32_t best = 0;
    uint32_t best_distance = 0xFFFFFFFFu;
    uint32_t best_hamming = 0xFFFFFFFFu;
    for (int32_t r = 0; r < 16; ++r) {
        uint32_t distance = 0;
        uint32_t hamming = 0;
        for (int32_t d = 0; d < 8; ++d) {
            int32_t y = 0;
            int32_t v = (int32_t)D11_ROOTS[r][d];
            for (int32_t k = 0; k < n; ++k) {
                y += v;
            }
            distance += d11_abs_diff(x[d], y);
            hamming += x[d] != y ? 1u : 0u;
        }
        if (distance < best_distance || (distance == best_distance && hamming < best_hamming)) {
            best = r;
            best_distance = distance;
            best_hamming = hamming;
        }
    }
    return best;
}

// route_read (:856-918), one block.
D11_HD static int32_t d11_route_block(
    const D11Model* m, const uint32_t* prefix, int32_t length, const D11Plan* p,
    int32_t block, int32_t* out) {
    const int32_t* query = d11_token_block(m, prefix[length - 1], block);
    int32_t node = 0;
    int32_t root = 0;
    int32_t total = D11_IDENTITY;
    for (int32_t stage = 0; stage < m->stages; ++stage) {
        int32_t frame = (int32_t)m->routes[block * m->stages + stage];
        int32_t q[8];
        d11_action(frame, query, q);
        int32_t count = stage == 0 ? p->frontier_len : 3;
        int32_t best = 0;
        uint32_t best_distance = 0xFFFFFFFFu;
        uint32_t best_hamming = 0xFFFFFFFFu;
        int32_t goal = 0;
        for (int32_t port = 0; port < count; ++port) {
            int32_t r = stage == 0 ? port : (int32_t)D11_BRANCH[root][port];
            if (r < 0 || r >= 16) {
                return D11_ERR_ROUTE;
            }
            uint32_t distance;
            uint32_t hamming;
            d11_root_cost(q, r, &distance, &hamming);
            if (distance < best_distance || (distance == best_distance && hamming < best_hamming)) {
                best = port;
                best_distance = distance;
                best_hamming = hamming;
                goal = r;
            }
        }
        total = d11_compose((int32_t)D11_BFS[(root << 4) + goal], total);
        root = goal;
        int32_t next_node;
        if (stage == 0) {
            next_node = p->frontier[best];
        } else {
            const D11Node* parent = &p->nodes[node];
            if (parent->atom >= 0) {
                return D11_ERR_ROUTE;
            }
            next_node = best == 0 ? parent->c0 : (best == 1 ? parent->c1 : parent->c2);
        }
        if (p->nodes[next_node].atom >= 0) {
            const int32_t* carrier = d11_token_block(m, prefix[p->nodes[next_node].atom], block);
            d11_action(total, carrier, out);
            return D11_OK;
        }
        node = next_node;
    }
    return D11_ERR_ROUTE;
}

// flock_or_relation_read (:919-1146), one block.
D11_HD static int32_t d11_flock_relation_block(
    const D11Model* m, const uint32_t* prefix, int32_t length, const D11Plan* p,
    D11Scratch* s, int32_t block, int32_t relation_active, int32_t* out) {
    int32_t blocks = m->blocks;
    int32_t stages = m->stages;
    int32_t n_nodes = p->node_count;
    int32_t kept = m->read_kept;
    int32_t f = block == 0 ? blocks - 1 : 0;
    uint32_t last_token = prefix[length - 1];
    const int32_t* query_frame = d11_token_block(m, last_token, f);
    int32_t u[8];
    const int32_t* own = d11_token_block(m, last_token, block);
    for (int32_t d = 0; d < 8; ++d) {
        u[d] = own[d];
    }
    int32_t fq = d11_root_id(query_frame);
    if (fq < 0) {
        return D11_ERR_ORBIT;
    }

    for (int32_t idx = 0; idx < n_nodes; ++idx) {
        const D11Node* node = &p->nodes[idx];
        if (node->atom >= 0) {
            uint32_t tok = prefix[node->atom];
            int32_t fr = d11_root_id(d11_token_block(m, tok, f));
            if (fr < 0) {
                return D11_ERR_ORBIT;
            }
            s->frame[idx] = fr;
            const int32_t* p = d11_token_block(m, tok, block);
            for (int32_t d = 0; d < 8; ++d) {
                s->payload[idx][d] = p[d];
            }
        } else {
            int32_t children[3] = {node->c0, node->c1, node->c2};
            int32_t c0 = children[0];
            s->frame[idx] = s->frame[c0];
            int32_t sum[8] = {0, 0, 0, 0, 0, 0, 0, 0};
            for (int32_t c = 0; c < 3; ++c) {
                int32_t child = children[c];
                int32_t act = (int32_t)D11_BFS[(s->frame[child] << 4) + s->frame[c0]];
                int32_t carried[8];
                d11_action(act, s->payload[child], carried);
                for (int32_t d = 0; d < 8; ++d) {
                    sum[d] += carried[d];
                }
            }
            int32_t r = d11_snap(sum, 3);
            for (int32_t d = 0; d < 8; ++d) {
                s->payload[idx][d] = (int32_t)D11_ROOTS[r][d];
            }
        }
    }

    for (int32_t i = 0; i < n_nodes; ++i) {
        s->rel[i] = 0;
    }
    if (relation_active) {
        for (int32_t i = 0; i < n_nodes; ++i) {
            uint32_t distance;
            uint32_t hamming;
            d11_root_cost(query_frame, s->frame[i], &distance, &hamming);
            s->prev[i] = 8 - (int32_t)distance;
        }
        const int32_t* relation_gates = m->gates + blocks * stages;
        for (int32_t sweep = 0; sweep < m->read_sweeps; ++sweep) {
            for (int32_t i = 0; i < n_nodes; ++i) {
                const D11Node* ni = &p->nodes[i];
                if (ni->last >= length) {
                    continue;
                }
                int32_t li = ni->level;
                for (int32_t e = 0; e < D11_EDGE_TYPES; ++e) {
                    int32_t gt = relation_gates[li * D11_EDGE_TYPES + e];
                    if (gt == 0) {
                        continue;
                    }
                    int32_t esum = 0;
                    if (e == 0) {
                        if (ni->atom < 0) {
                            int32_t children[3] = {ni->c0, ni->c1, ni->c2};
                            for (int32_t c = 0; c < 3; ++c) {
                                if (p->nodes[children[c]].last < length) {
                                    esum += s->prev[children[c]];
                                }
                            }
                        }
                        for (int32_t pi = i + 1; pi < n_nodes; ++pi) {
                            const D11Node* np = &p->nodes[pi];
                            if (np->atom < 0 && np->last < length
                                && (np->c0 == i || np->c1 == i || np->c2 == i)) {
                                esum += s->prev[pi];
                            }
                        }
                    } else if (e == 1) {
                        for (int32_t pi = i + 1; pi < n_nodes; ++pi) {
                            const D11Node* np = &p->nodes[pi];
                            if (np->atom >= 0) {
                                continue;
                            }
                            if (np->last >= length) {
                                continue;
                            }
                            int32_t c0 = np->c0;
                            int32_t c1 = np->c1;
                            int32_t c2 = np->c2;
                            if (i == c0 && p->nodes[c1].last < length) {
                                esum += s->prev[c1];
                            } else if (i == c1) {
                                if (p->nodes[c0].last < length) {
                                    esum += s->prev[c0];
                                }
                                if (p->nodes[c2].last < length) {
                                    esum += s->prev[c2];
                                }
                            } else if (i == c2 && p->nodes[c1].last < length) {
                                esum += s->prev[c1];
                            }
                        }
                    } else if (e == 2) {
                        for (int32_t j = 0; j < i; ++j) {
                            const D11Node* nj = &p->nodes[j];
                            if (nj->last < length && nj->level == li && nj->last + 1 == ni->first) {
                                esum += s->prev[j];
                            }
                        }
                    } else if (e == 3) {
                        if (li == 0) {
                            uint32_t tid = prefix[ni->last];
                            for (int32_t j = 0; j < i; ++j) {
                                const D11Node* nj = &p->nodes[j];
                                if (nj->last < length && nj->level == 0 && prefix[nj->last] == tid) {
                                    esum = s->prev[j];
                                }
                            }
                        }
                    } else {
                        for (int32_t j = 0; j < i; ++j) {
                            const D11Node* nj = &p->nodes[j];
                            if (nj->last < length && nj->level == li && s->frame[j] == s->frame[i]) {
                                esum = s->prev[j];
                            }
                        }
                    }
                    s->rel[i] += gt > 0 ? esum : -esum;
                }
            }
            for (int32_t i = 0; i < n_nodes; ++i) {
                s->prev[i] = s->rel[i];
            }
        }
    }

    int32_t terms = 1;
    for (int32_t l = 0; l < stages; ++l) {
        int32_t gate = m->gates[block * stages + l];
        if (gate == 0) {
            continue;
        }
        int32_t top[D11_MAX_KEPT];
        int32_t key_r[D11_MAX_KEPT];
        uint32_t key_d[D11_MAX_KEPT];
        uint32_t key_h[D11_MAX_KEPT];
        int32_t found = 0;
        for (int32_t node = 0; node < n_nodes; ++node) {
            const D11Node* nn = &p->nodes[node];
            if (nn->level != l || nn->last + 1 >= length) {
                continue;
            }
            uint32_t distance;
            uint32_t hamming;
            d11_root_cost(query_frame, s->frame[node], &distance, &hamming);
            int32_t rscore = s->rel[node];
            int32_t at = found;
            while (at > 0) {
                int32_t o = top[at - 1];
                int32_t better = rscore > key_r[at - 1]
                    || (rscore == key_r[at - 1]
                        && (distance < key_d[at - 1]
                            || (distance == key_d[at - 1]
                                && (hamming < key_h[at - 1]
                                    || (hamming == key_h[at - 1]
                                        && (nn->last > p->nodes[o].last
                                            || (nn->last == p->nodes[o].last && node < o)))))));
                if (!better) {
                    break;
                }
                if (at < kept) {
                    top[at] = top[at - 1];
                    key_r[at] = key_r[at - 1];
                    key_d[at] = key_d[at - 1];
                    key_h[at] = key_h[at - 1];
                }
                at -= 1;
            }
            if (at < kept) {
                top[at] = node;
                key_r[at] = rscore;
                key_d[at] = distance;
                key_h[at] = hamming;
                if (found < kept) {
                    found += 1;
                }
            }
        }
        if (found == 0) {
            continue;
        }
        int32_t sum[8] = {0, 0, 0, 0, 0, 0, 0, 0};
        for (int32_t k = 0; k < found; ++k) {
            int32_t node = top[k];
            int32_t act = (int32_t)D11_BFS[(s->frame[node] << 4) + fq];
            int32_t carried[8];
            d11_action(act, s->payload[node], carried);
            for (int32_t d = 0; d < 8; ++d) {
                sum[d] += carried[d];
            }
        }
        int32_t mroot = d11_snap(sum, found);
        for (int32_t d = 0; d < 8; ++d) {
            int32_t v = (int32_t)D11_ROOTS[mroot][d];
            u[d] += gate > 0 ? v : -v;
        }
        terms += 1;
    }
    int32_t out_root = d11_snap(u, terms);
    for (int32_t d = 0; d < 8; ++d) {
        out[d] = (int32_t)D11_ROOTS[out_root][d];
    }
    return D11_OK;
}

// address_distance (:1147).
D11_HD static uint32_t d11_address_distance(const D11Model* m, const int32_t* q, uint32_t cell) {
    uint32_t a;
    uint32_t b;
    uint32_t hamming;
    d11_root_cost(q, (int32_t)(cell >> 4), &a, &hamming);
    d11_root_cost(q + (m->blocks - 1) * 8, (int32_t)(cell & 15u), &b, &hamming);
    return a + b;
}

// token_distance (:1153).
D11_HD static uint32_t d11_token_distance(const D11Model* m, const int32_t* q, uint32_t token) {
    uint32_t sum = 0;
    for (int32_t b = 0; b < m->blocks; ++b) {
        const int32_t* v = d11_token_block(m, token, b);
        for (int32_t d = 0; d < 8; ++d) {
            sum += d11_abs_diff(q[b * 8 + d], v[d]);
        }
    }
    return sum;
}

D11_HD static int32_t d11_cl06_inverse(int32_t a) {
    return d11_compose(d11_compose(a, a), a);
}

D11_HD static uint32_t d11_trunk_sparse_get(const uint32_t* trunk, int32_t vocab,
                                            int32_t table_idx, uint32_t key, uint32_t tok) {
    int32_t meta = 70 + 2 * vocab + 2 * table_idx;
    uint32_t base = trunk[meta];
    uint32_t n = trunk[meta + 1];
    const uint32_t* triples = trunk + base;
    uint32_t lo = 0u;
    uint32_t hi = n;
    while (lo < hi) {
        uint32_t mid = lo + ((hi - lo) >> 1);
        uint32_t k_mid = triples[3u * mid];
        uint32_t t_mid = triples[3u * mid + 1u];
        if (k_mid < key || (k_mid == key && t_mid < tok)) {
            lo = mid + 1u;
        } else if (k_mid > key || t_mid > tok) {
            hi = mid;
        } else {
            return triples[3u * mid + 2u];
        }
    }
    return 0u;
}

D11_HD static void d11_trunk_sparse_key_range(const uint32_t* trunk, int32_t vocab,
                                              int32_t table_idx, uint32_t key,
                                              uint32_t* out_lo, uint32_t* out_hi) {
    int32_t meta = 70 + 2 * vocab + 2 * table_idx;
    uint32_t base = trunk[meta];
    uint32_t n = trunk[meta + 1];
    const uint32_t* triples = trunk + base;
    uint32_t lo = 0u;
    uint32_t hi = n;
    while (lo < hi) {
        uint32_t mid = lo + ((hi - lo) >> 1);
        if (triples[3u * mid] < key) {
            lo = mid + 1u;
        } else {
            hi = mid;
        }
    }
    uint32_t start = lo;
    hi = n;
    while (lo < hi) {
        uint32_t mid = lo + ((hi - lo) >> 1);
        if (triples[3u * mid] <= key) {
            lo = mid + 1u;
        } else {
            hi = mid;
        }
    }
    *out_lo = start;
    *out_hi = lo;
}

D11_HD static void d11_trunk_consider_candidate(const uint32_t* trunk, int32_t vocab,
                                                uint32_t q_key, uint32_t r_key, uint32_t c_key,
                                                int32_t has_ptr, uint32_t ptr_tok, uint32_t tok,
                                                uint64_t* best_s, uint32_t* best_u,
                                                uint32_t* best_tok) {
    if (tok >= (uint32_t)vocab) {
        return;
    }
    uint32_t qr_key = (q_key << 16) | r_key;
    uint32_t qc_key = (q_key << 16) | c_key;
    uint32_t rc_key = (r_key << 16) | c_key;
    uint64_t c_q = (uint64_t)d11_trunk_sparse_get(trunk, vocab, 7, q_key, tok);
    uint64_t c_qr = (uint64_t)d11_trunk_sparse_get(trunk, vocab, 8, qr_key, tok);
    uint64_t c_r = (uint64_t)d11_trunk_sparse_get(trunk, vocab, 0, r_key, tok);
    uint64_t c_qc = (uint64_t)d11_trunk_sparse_get(trunk, vocab, 9, qc_key, tok);
    uint64_t c_rr = (uint64_t)d11_trunk_sparse_get(trunk, vocab, 2, rc_key, tok);
    uint64_t ptr_boost = (has_ptr && ptr_tok == tok) ? (1ULL << 8) : 0ULL;
    uint64_t s = (c_qr << 6) + (c_qc << 5) + (c_rr << 3) + (c_r << 1) + c_q + ptr_boost;
    uint32_t u = trunk[70 + (int32_t)tok];
    if (s > *best_s || (s == *best_s && s > 0ULL && u > *best_u)
        || (s == *best_s && s > 0ULL && u == *best_u && tok < *best_tok)) {
        *best_s = s;
        *best_u = u;
        *best_tok = tok;
    }
}

// DiscreteTrunkTables::apply_joint_pointer_gate (discrete_d11.rs:627-662).
D11_HD static int32_t d11_trunk_apply_joint_pointer_gate(
    const D11Model* m, const uint32_t* prefix, int32_t length, int32_t* q) {
    if (m->blocks != 4) {
        return D11_ERR_SHAPE;
    }
    int32_t read_roots[4];
    for (int32_t b = 0; b < 4; ++b) {
        read_roots[b] = d11_root_id(q + b * 8);
        if (read_roots[b] < 0) {
            return D11_ERR_ORBIT;
        }
    }
    uint32_t r_key = ((uint32_t)read_roots[0] & 0xFu)
        | (((uint32_t)read_roots[1] & 0xFu) << 4)
        | (((uint32_t)read_roots[2] & 0xFu) << 8)
        | (((uint32_t)read_roots[3] & 0xFu) << 12);

    const uint32_t* step_candidates = m->trunk + 1;
    const uint32_t* norm_candidates = m->trunk + 65;
    const uint32_t* packed_roots = m->trunk + 70 + m->vocabulary;

    int32_t last_pos = length - 1;
    int32_t prev_pos = last_pos > 0 ? last_pos - 1 : 0;
    int32_t mid_pos = last_pos / 2;
    uint32_t c_key = 0u;
    for (int32_t b = 0; b < 4; ++b) {
        int32_t acc = D11_IDENTITY;
        int32_t f_prev = D11_IDENTITY;
        int32_t f_mid = D11_IDENTITY;
        for (int32_t pos = 0; pos < length; ++pos) {
            uint32_t tok = prefix[pos];
            int32_t r = (int32_t)((packed_roots[tok] >> (4 * b)) & 0xFu);
            int32_t step = (int32_t)step_candidates[(b << 4) + r];
            acc = d11_compose(step, acc);
            if (pos == prev_pos) {
                f_prev = acc;
            }
            if (pos == mid_pos) {
                f_mid = acc;
            }
        }
        int32_t f_last = acc;
        int32_t h_prev = d11_compose(f_last, d11_cl06_inverse(f_prev));
        int32_t h_mid = d11_compose(f_last, d11_cl06_inverse(f_mid));

        int32_t r_start = (int32_t)((packed_roots[prefix[0]] >> (4 * b)) & 0xFu);
        int32_t r_last = (int32_t)((packed_roots[prefix[last_pos]] >> (4 * b)) & 0xFu);
        int32_t r_prev = (int32_t)((packed_roots[prefix[prev_pos]] >> (4 * b)) & 0xFu);
        int32_t r_mid = (int32_t)((packed_roots[prefix[mid_pos]] >> (4 * b)) & 0xFu);

        int32_t q_coords[8];
        int32_t prev_coords[8];
        int32_t mid_coords[8];
        int32_t start_coords[8];
        for (int32_t d = 0; d < 8; ++d) {
            q_coords[d] = (int32_t)D11_ROOTS[r_last][d];
            prev_coords[d] = (int32_t)D11_ROOTS[r_prev][d];
            mid_coords[d] = (int32_t)D11_ROOTS[r_mid][d];
            start_coords[d] = (int32_t)D11_ROOTS[r_start][d];
        }
        int32_t prev_rot[8];
        int32_t mid_rot[8];
        int32_t full_rot[8];
        d11_action(h_prev, prev_coords, prev_rot);
        d11_action(h_mid, mid_coords, mid_rot);
        d11_action(f_last, start_coords, full_rot);

        int32_t sum_block[8];
        for (int32_t d = 0; d < 8; ++d) {
            sum_block[d] = q_coords[d] + prev_rot[d] + mid_rot[d] + full_rot[d];
        }
        int32_t snapped_root = d11_snap(sum_block, 4);
        int32_t snapped_coords[8];
        for (int32_t d = 0; d < 8; ++d) {
            snapped_coords[d] = (int32_t)D11_ROOTS[snapped_root][d];
        }
        int32_t align_frame = (int32_t)norm_candidates[b];
        int32_t normalized_block[8];
        d11_action(align_frame, snapped_coords, normalized_block);
        int32_t norm_root = d11_root_id(normalized_block);
        if (norm_root < 0) {
            return D11_ERR_ORBIT;
        }
        c_key |= ((uint32_t)norm_root & 0xFu) << (4 * b);
    }

    int32_t has_ptr = 0;
    uint32_t ptr_tok = 0u;
    if (length >= 2) {
        uint32_t q_code = packed_roots[prefix[last_pos]];
        for (int32_t i = 0; i < last_pos; ++i) {
            if (packed_roots[prefix[i]] == q_code) {
                has_ptr = 1;
                ptr_tok = prefix[i + 1];
            }
        }
    }

    uint32_t q_key = prefix[last_pos];
    uint32_t best_tok = m->trunk[69];
    uint64_t best_s = 0ULL;
    uint32_t best_u = 0u;

    int32_t cand_tables[3] = {7, 0, 1};
    uint32_t cand_keys[3] = {q_key, r_key, c_key};
    for (int32_t ti = 0; ti < 3; ++ti) {
        int32_t table_idx = cand_tables[ti];
        uint32_t key = cand_keys[ti];
        uint32_t lo = 0u;
        uint32_t hi = 0u;
        d11_trunk_sparse_key_range(m->trunk, m->vocabulary, table_idx, key, &lo, &hi);
        uint32_t base = m->trunk[70 + 2 * m->vocabulary + 2 * table_idx];
        for (uint32_t idx = lo; idx < hi; ++idx) {
            uint32_t tok = m->trunk[base + 3u * idx + 1u];
            d11_trunk_consider_candidate(m->trunk, m->vocabulary, q_key, r_key, c_key,
                                         has_ptr, ptr_tok, tok, &best_s, &best_u, &best_tok);
        }
    }
    if (has_ptr) {
        d11_trunk_consider_candidate(m->trunk, m->vocabulary, q_key, r_key, c_key,
                                     has_ptr, ptr_tok, ptr_tok, &best_s, &best_u, &best_tok);
    }
    if (best_tok >= (uint32_t)m->vocabulary) {
        return D11_ERR_SHAPE;
    }
    uint32_t target_packed = packed_roots[best_tok];
    for (int32_t b = 0; b < 4; ++b) {
        int32_t src = read_roots[b];
        int32_t dst = (int32_t)((target_packed >> (4 * b)) & 0xFu);
        int32_t frame = (int32_t)D11_BFS[(src << 4) + dst];
        int32_t rotated[8];
        d11_action(frame, q + b * 8, rotated);
        int32_t snapped_root = d11_snap(rotated, 1);
        for (int32_t d = 0; d < 8; ++d) {
            q[b * 8 + d] = (int32_t)D11_ROOTS[snapped_root][d];
        }
    }
    return D11_OK;
}

// evaluate_prefix (:1165-1256) and observe_target (:1278-1378).
D11_HD static void d11_evaluate_row(
    const D11Model* m, const uint32_t* prefix, int32_t length, uint32_t target, D11Scratch* s,
    D11RowOut* out) {
    out->status = D11_OK;
    out->admitted_count = 0;
    out->top8_count = 0;
    out->prediction = 0;
    out->token_hinge = 0;
    out->address_hinge = 0;
    out->rank_hinge = 0;
    out->loss_ta = 0;
    out->loss_tr = 0;
    if (m->blocks < 1 || m->blocks > D11_MAX_BLOCKS || m->stages < 1 || m->stages > D11_MAX_STAGES
        || m->kept_cells < 1 || m->kept_cells > D11_MAX_KEPT_CELLS || m->read_kept > D11_MAX_KEPT) {
        out->status = D11_ERR_SHAPE;
        return;
    }
    if (length < 1 || length > m->context) {
        out->status = D11_ERR_PREFIX_LENGTH;
        return;
    }
    for (int32_t i = 0; i < length; ++i) {
        if (prefix[i] >= (uint32_t)m->vocabulary) {
            out->status = D11_ERR_PREFIX_TOKEN;
            return;
        }
    }
    if (target >= (uint32_t)m->vocabulary) {
        out->status = D11_ERR_TARGET_TOKEN;
        return;
    }
    const D11Plan* p = &m->plans[length - 1];
    if (p->node_count < length || p->node_count > D11_MAX_NODES || p->frontier_len < 1
        || p->frontier_len > D11_MAX_FRONTIER) {
        out->status = D11_ERR_PLAN;
        return;
    }
    int32_t status = D11_OK;
    for (int32_t b = 0; b < m->blocks; ++b) {
        int32_t* q = out->read_vector + b * 8;
        if (m->read_mode == D11_READ_ROUTE) {
            status = d11_route_block(m, prefix, length, p, b, q);
        } else {
            status = d11_flock_relation_block(m, prefix, length, p, s, b,
                                              m->read_mode == D11_READ_RELATION, q);
        }
        if (status != D11_OK) {
            out->status = status;
            return;
        }
    }
    if (m->trunk != 0 && m->trunk[0] != 0u) {
        status = d11_trunk_apply_joint_pointer_gate(m, prefix, length, out->read_vector);
        if (status != D11_OK) {
            out->status = status;
            return;
        }
    }
    const int32_t* q = out->read_vector;

    uint8_t occupied[256];
    uint8_t used[256];
    for (int32_t c = 0; c < 256; ++c) {
        occupied[c] = 0;
        used[c] = 0;
    }
    for (int32_t t = 0; t < m->vocabulary; ++t) {
        occupied[m->cells[t] & 255u] = 1;
    }
    for (int32_t k = 0; k < m->kept_cells; ++k) {
        int32_t best_cell = -1;
        uint32_t best_cost = 0xFFFFFFFFu;
        for (int32_t c = 0; c < 256; ++c) {
            if (occupied[c] && !used[c]) {
                uint32_t d = d11_address_distance(m, q, (uint32_t)c);
                if (d < best_cost || (d == best_cost && best_cell < 0)) {
                    best_cell = c;
                    best_cost = d;
                }
            }
        }
        if (best_cell < 0) {
            break;
        }
        used[best_cell] = 1;
        out->admitted[out->admitted_count++] = (uint32_t)best_cell;
    }

    uint32_t ranked[8];
    uint32_t distance[8];
    for (int32_t k = 0; k < 8; ++k) {
        ranked[k] = 0xFFFFFFFFu;
        distance[k] = 0xFFFFFFFFu;
    }
    int32_t found = 0;
    for (int32_t token = 0; token < m->vocabulary; ++token) {
        uint32_t cell = m->cells[token];
        int32_t admitted = 0;
        for (int32_t k = 0; k < out->admitted_count; ++k) {
            if (out->admitted[k] == cell) {
                admitted = 1;
            }
        }
        if (!admitted) {
            continue;
        }
        uint32_t d = d11_token_distance(m, q, (uint32_t)token);
        uint32_t tok = (uint32_t)token;
        int32_t at = found;
        while (at > 0) {
            int32_t prev = at - 1;
            if (!(d < distance[prev] || (d == distance[prev] && tok < ranked[prev]))) {
                break;
            }
            if (at < 8) {
                ranked[at] = ranked[prev];
                distance[at] = distance[prev];
            }
            at -= 1;
        }
        if (at < 8) {
            ranked[at] = tok;
            distance[at] = d;
            if (found < 8) {
                found += 1;
            }
        }
    }
    if (found == 0) {
        out->status = D11_ERR_ZERO_TOKENS;
        return;
    }
    out->top8_count = found;
    for (int32_t k = 0; k < found; ++k) {
        out->top8[k] = ranked[k];
    }
    out->prediction = ranked[0];

    uint32_t target_cell = m->cells[target];
    uint32_t rival_distance = 0xFFFFFFFFu;
    uint32_t rival_address = 0xFFFFFFFFu;
    for (int32_t k = 0; k < out->admitted_count; ++k) {
        uint32_t cell = out->admitted[k];
        uint32_t cost = d11_address_distance(m, q, cell);
        if (cell != target_cell && cost < rival_address) {
            rival_address = cost;
        }
        for (int32_t token = 0; token < m->vocabulary; ++token) {
            if (m->cells[token] == cell && (uint32_t)token != target) {
                uint32_t d = d11_token_distance(m, q, (uint32_t)token);
                if (d < rival_distance) {
                    rival_distance = d;
                }
            }
        }
    }
    uint32_t width = (uint32_t)(m->blocks << 3);
    uint32_t td = d11_token_distance(m, q, target);
    uint32_t ad = d11_address_distance(m, q, target_cell);
    uint32_t token_hinge = (rival_distance == 0xFFFFFFFFu || width + td <= rival_distance)
        ? 0u
        : (width + td - rival_distance);
    uint32_t address_hinge = (rival_address == 0xFFFFFFFFu || 8u + ad <= rival_address)
        ? 0u
        : (8u + ad - rival_address);
    uint32_t rank = 1u;
    for (int32_t c = 0; c < 256; ++c) {
        if (occupied[c]) {
            uint32_t d = d11_address_distance(m, q, (uint32_t)c);
            if (d < ad || (d == ad && (uint32_t)c < target_cell)) {
                rank += 1u;
            }
        }
    }
    uint32_t rank_hinge =
        rank > (uint32_t)m->kept_cells ? (rank - (uint32_t)m->kept_cells) : 0u;
    uint32_t weighted_token = token_hinge << 3;
    uint32_t weighted_address = 0u;
    uint32_t weighted_rank = 0u;
    for (uint32_t w = 0u; w < width; ++w) {
        weighted_address += address_hinge;
        weighted_rank += rank_hinge;
    }
    out->token_hinge = token_hinge;
    out->address_hinge = address_hinge;
    out->rank_hinge = rank_hinge;
    out->loss_ta = weighted_token + weighted_address;
    out->loss_tr = weighted_token + weighted_rank;
}

#endif
