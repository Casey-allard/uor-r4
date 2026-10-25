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
} D11Model;

typedef struct {
    int32_t status;
    int32_t admitted_count;
    int32_t top8_count;
    uint32_t prediction;
    uint32_t admitted[D11_MAX_KEPT_CELLS];
    uint32_t top8[8];
    int32_t read_vector[D11_MAX_BLOCKS * 8];
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

// evaluate_prefix (:1165-1256).
D11_HD static void d11_evaluate_row(
    const D11Model* m, const uint32_t* prefix, int32_t length, D11Scratch* s, D11RowOut* out) {
    out->status = D11_OK;
    out->admitted_count = 0;
    out->top8_count = 0;
    out->prediction = 0;
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
}

#endif
