// Native projection ABI v1. Static tables are compiled from retained FBS
// binders and actual SpiralCore BFS edges, bound by the artifact fingerprint.
// There is no learned input/output coefficient grid or all-input reduction.
#define SC_FBS_LEVELS 8
#define SC_FBS_ORBIT 16
#define SC_FBS_ABSENT 0xffffffffu

struct ScFbsTables {
    const uint* nodes;
    const uint* frontier;
    const uint* axes;
    const uint* compose;
    const uint* root_of_state;
    const uint* bfs_actions;
    const uint* branch_targets;
    const float* roots;
    uint stages;
    uint frontier_count;
    uint identity;
};

struct ScFbsRoute {
    uint atom;
    uint state;
    uint count;
    uint before[SC_FBS_LEVELS];
    uint choice[SC_FBS_LEVELS];
    uint goal[SC_FBS_LEVELS];
};

// One fixed 8D Euclidean pairing against a geometric root, not a learned map.
__device__ __forceinline__ float sc_fbs_distance(const float* q, const float* root) {
    float d = 0.0f;
    for (uint axis = 0; axis < 8; ++axis) {
        float delta = q[axis] - root[axis];
        d += delta * delta;
    }
    return d;
}

__device__ __forceinline__ uint sc_fbs_advance(ScFbsTables t, uint state, uint target) {
    uint from = t.root_of_state[state];
    uint action = t.bfs_actions[from * 16 + target];
    return t.compose[action * 64 + state];
}

// The optional forced choice is a lawful local counterfactual. Descendants are
// re-routed under that actual operator frame; no pooled/representative child is
// substituted for its complete retained descendants.
__device__ __forceinline__ ScFbsRoute sc_fbs_route(
    ScFbsTables t, const float* q, uint forced_stage, uint forced_choice
) {
    ScFbsRoute r;
    r.atom = SC_FBS_ABSENT;
    r.state = t.identity;
    r.count = 0;
    uint node = SC_FBS_ABSENT;
    for (uint stage = 0; stage < t.stages; ++stage) {
        uint root = t.root_of_state[r.state];
        uint count = stage == 0 ? t.frontier_count : 3;
        uint selected = 0;
        float best = 3.402823466e+38f;
        for (uint candidate = 0; candidate < count; ++candidate) {
            uint goal = stage == 0 ? candidate : t.branch_targets[root * 3 + candidate];
            float distance = sc_fbs_distance(q + stage * 8, t.roots + goal * 8);
            if (!isfinite(distance)) return r;
            if (distance < best) { best = distance; selected = candidate; }
        }
        if (stage == forced_stage) selected = forced_choice;
        uint goal = stage == 0 ? selected : t.branch_targets[root * 3 + selected];
        r.before[stage] = r.state;
        r.choice[stage] = selected;
        r.goal[stage] = goal;
        r.count = stage + 1;
        r.state = sc_fbs_advance(t, r.state, goal);
        node = stage == 0 ? t.frontier[selected] : t.nodes[node * 4 + selected];
        uint atom = t.nodes[node * 4 + 3];
        if (atom != SC_FBS_ABSENT) { r.atom = atom; return r; }
    }
    return r; // An unterminated route is rejected by the device error flag.
}

__device__ __forceinline__ float sc_fbs_coordinate(
    ScFbsTables t, const act_t* x, uint atom, uint state, uint axis
) {
    uint code = t.axes[state * 8 + axis];
    float value = act_to_f(x[atom * 8 + (code & 7)]);
    return code & 8 ? -value : value;
}

extern "C" __global__ void spiralcore_fbs_fwd(
    const float* queries, const act_t* input,
    const uint* nodes, const uint* frontier, const uint* axes,
    const uint* compose, const uint* root_of_state, const uint* bfs_actions,
    const uint* branch_targets, const float* roots,
    act_t* out, uint* error,
    uint rows, uint outputs, uint inputs, uint stages, uint frontier_count, uint identity
) {
    uint item = blockIdx.x * blockDim.x + threadIdx.x;
    if (item >= rows * outputs) return;
    uint row = item / outputs, output = item % outputs;
    ScFbsTables t = {nodes, frontier, axes, compose, root_of_state,
        bfs_actions, branch_targets, roots, stages, frontier_count, identity};
    const float* q = queries + (u64)output * stages * 8;
    for (uint i = 0; i < stages * 8; ++i) {
        if (!isfinite(q[i])) { atomicExch(error, 1u); return; }
    }
    ScFbsRoute r = sc_fbs_route(t, q, SC_FBS_ABSENT, 0);
    if (r.atom == SC_FBS_ABSENT || r.atom >= inputs) { atomicExch(error, 2u); return; }
    const act_t* x = input + (u64)row * inputs * 8;
    for (uint axis = 0; axis < 8; ++axis) {
        float value = sc_fbs_coordinate(t, x, r.atom, r.state, axis);
        if (!isfinite(value)) { atomicExch(error, 3u); return; }
        out[(u64)item * 8 + axis] = act_from_f(value);
    }
}

// Input adjoint: the exact inverse signed action, scattered only to the selected
// retained atom. Query pullback: a declared geometric secant surrogate, NOT the
// derivative of discontinuous hard selection. For each lawful sibling it pairs
// dL/dy with the complete counterfactual carrier displacement, then resolves that
// loss contrast along the fixed root edge. No probabilities, exponentials or
// learned coefficient grid occur. Batch sharing is summed with device atomics.
extern "C" __global__ void spiralcore_fbs_bwd(
    const float* queries, const act_t* input, const act_t* grad,
    const uint* nodes, const uint* frontier, const uint* axes,
    const uint* compose, const uint* root_of_state, const uint* bfs_actions,
    const uint* branch_targets, const float* roots,
    float* d_queries, float* d_input, uint* error,
    uint rows, uint outputs, uint inputs, uint stages, uint frontier_count, uint identity
) {
    uint item = blockIdx.x * blockDim.x + threadIdx.x;
    if (item >= rows * outputs) return;
    uint row = item / outputs, output = item % outputs;
    ScFbsTables t = {nodes, frontier, axes, compose, root_of_state,
        bfs_actions, branch_targets, roots, stages, frontier_count, identity};
    const float* q = queries + (u64)output * stages * 8;
    for (uint i = 0; i < stages * 8; ++i) {
        if (!isfinite(q[i])) { atomicExch(error, 1u); return; }
    }
    ScFbsRoute selected = sc_fbs_route(t, q, SC_FBS_ABSENT, 0);
    if (selected.atom == SC_FBS_ABSENT || selected.atom >= inputs) { atomicExch(error, 2u); return; }
    const act_t* x = input + (u64)row * inputs * 8;
    float dy[8], y[8];
    for (uint axis = 0; axis < 8; ++axis) {
        dy[axis] = act_to_f(grad[(u64)item * 8 + axis]);
        y[axis] = sc_fbs_coordinate(t, x, selected.atom, selected.state, axis);
        if (!isfinite(dy[axis]) || !isfinite(y[axis])) { atomicExch(error, 3u); return; }
        uint code = axes[selected.state * 8 + axis];
        float value = code & 8 ? -dy[axis] : dy[axis];
        float prior = atomicAdd(d_input + (u64)row * inputs * 8 + selected.atom * 8 + (code & 7), value);
        if (!isfinite(prior + value)) { atomicExch(error, 4u); return; }
    }
    for (uint stage = 0; stage < selected.count; ++stage) {
        uint count = stage == 0 ? frontier_count : 3;
        uint root = root_of_state[selected.before[stage]];
        uint old_goal = selected.goal[stage];
        for (uint sibling = 0; sibling < count; ++sibling) {
            if (sibling == selected.choice[stage]) continue;
            ScFbsRoute alternative = sc_fbs_route(t, q, stage, sibling);
            if (alternative.atom == SC_FBS_ABSENT || alternative.atom >= inputs) {
                atomicExch(error, 2u); return;
            }
            uint goal = stage == 0 ? sibling : branch_targets[root * 3 + sibling];
            float contrast = 0.0f, squared_edge = 0.0f;
            for (uint axis = 0; axis < 8; ++axis) {
                float other = sc_fbs_coordinate(t, x, alternative.atom, alternative.state, axis);
                if (!isfinite(other)) { atomicExch(error, 3u); return; }
                contrast += dy[axis] * (other - y[axis]);
                float edge = roots[goal * 8 + axis] - roots[old_goal * 8 + axis];
                squared_edge += edge * edge;
            }
            if (squared_edge <= 0.0f || !isfinite(contrast)) { atomicExch(error, 4u); return; }
            float directional = contrast / squared_edge;
            for (uint axis = 0; axis < 8; ++axis) {
                float edge = roots[goal * 8 + axis] - roots[old_goal * 8 + axis];
                float value = directional * edge;
                float prior = atomicAdd(d_queries + (u64)output * stages * 8 + stage * 8 + axis, value);
                if (!isfinite(prior + value)) { atomicExch(error, 4u); return; }
            }
        }
    }
}
