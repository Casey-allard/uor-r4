// The registered arms of docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md
// that change the projection itself. They reuse the pinned route, advance and coordinate
// helpers of src/spiralcore_fbs.cu unchanged; only what the arm changes is new here.
//   arm 1, Fbs_IdentityTransport_Control: the same base-3 tree selection sigma(o), but the
//          transport operator is fixed to the identity, y_o = x_sigma(o).
//   arm 2, Fbs_2Atom_Mix: two independent routed query paths per output block,
//          y_o = (A_1 x_sigma1(o) + A_2 x_sigma2(o)) / sqrt(2), a constant factor.
// The frozen-query arm needs no kernel (the standard backward with zero query credit).
// No probabilities, exponentials, squashing or learned coefficient grid occur.
#define SC_ARM_IDENTITY 1u
#define SC_ARM_TWO 2u
#define SC_ARM_INV_SQRT2 0.70710678118654752440f

__device__ __forceinline__ uint sc_arm_paths(uint arm) { return arm == SC_ARM_TWO ? 2u : 1u; }

__device__ __forceinline__ float sc_arm_scale(uint arm) {
    return arm == SC_ARM_TWO ? SC_ARM_INV_SQRT2 : 1.0f;
}

// The transported coordinate of the routed atom; the identity arm reads the atom as is.
__device__ __forceinline__ float sc_arm_coordinate(
    ScFbsTables t, const act_t* x, uint atom, uint state, uint axis, uint arm
) {
    if (arm == SC_ARM_IDENTITY) return act_to_f(x[atom * 8 + axis]);
    return sc_fbs_coordinate(t, x, atom, state, axis);
}

extern "C" __global__ void spiralcore_fbs_arm_fwd(
    const float* queries, const act_t* input,
    const uint* nodes, const uint* frontier, const uint* axes,
    const uint* compose, const uint* root_of_state, const uint* bfs_actions,
    const uint* branch_targets, const float* roots,
    act_t* out, uint* error,
    uint rows, uint outputs, uint inputs, uint stages, uint frontier_count, uint identity,
    uint arm
) {
    uint item = blockIdx.x * blockDim.x + threadIdx.x;
    if (item >= rows * outputs) return;
    if (arm != SC_ARM_IDENTITY && arm != SC_ARM_TWO) { atomicExch(error, 5u); return; }
    uint row = item / outputs, output = item % outputs;
    ScFbsTables t = {nodes, frontier, axes, compose, root_of_state,
        bfs_actions, branch_targets, roots, stages, frontier_count, identity};
    uint paths = sc_arm_paths(arm);
    const float* q = queries + (u64)output * paths * stages * 8;
    for (uint i = 0; i < paths * stages * 8; ++i) {
        if (!isfinite(q[i])) { atomicExch(error, 1u); return; }
    }
    const act_t* x = input + (u64)row * inputs * 8;
    float y[8] = {0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    for (uint path = 0; path < paths; ++path) {
        ScFbsRoute r = sc_fbs_route(t, q + path * stages * 8, SC_FBS_ABSENT, 0);
        if (r.atom == SC_FBS_ABSENT || r.atom >= inputs) { atomicExch(error, 2u); return; }
        for (uint axis = 0; axis < 8; ++axis) {
            y[axis] += sc_arm_coordinate(t, x, r.atom, r.state, axis, arm);
        }
    }
    float scale = sc_arm_scale(arm);
    for (uint axis = 0; axis < 8; ++axis) {
        float value = y[axis] * scale;
        if (!isfinite(value)) { atomicExch(error, 3u); return; }
        out[(u64)item * 8 + axis] = act_from_f(value);
    }
}

// Per path, the input adjoint is the exact inverse of that path's action (the identity in
// arm 1), scaled by the arm's constant; the query pullback is the same declared root-edge
// secant surrogate as spiralcore_fbs_bwd, paired with this path's share of dL/dy while the
// other path is held fixed. Batch sharing is summed with device atomics.
extern "C" __global__ void spiralcore_fbs_arm_bwd(
    const float* queries, const act_t* input, const act_t* grad,
    const uint* nodes, const uint* frontier, const uint* axes,
    const uint* compose, const uint* root_of_state, const uint* bfs_actions,
    const uint* branch_targets, const float* roots,
    float* d_queries, float* d_input, uint* error,
    uint rows, uint outputs, uint inputs, uint stages, uint frontier_count, uint identity,
    uint arm
) {
    uint item = blockIdx.x * blockDim.x + threadIdx.x;
    if (item >= rows * outputs) return;
    if (arm != SC_ARM_IDENTITY && arm != SC_ARM_TWO) { atomicExch(error, 5u); return; }
    uint row = item / outputs, output = item % outputs;
    ScFbsTables t = {nodes, frontier, axes, compose, root_of_state,
        bfs_actions, branch_targets, roots, stages, frontier_count, identity};
    uint paths = sc_arm_paths(arm);
    float scale = sc_arm_scale(arm);
    const float* q_all = queries + (u64)output * paths * stages * 8;
    for (uint i = 0; i < paths * stages * 8; ++i) {
        if (!isfinite(q_all[i])) { atomicExch(error, 1u); return; }
    }
    const act_t* x = input + (u64)row * inputs * 8;
    float dy[8];
    for (uint axis = 0; axis < 8; ++axis) {
        dy[axis] = act_to_f(grad[(u64)item * 8 + axis]) * scale;
        if (!isfinite(dy[axis])) { atomicExch(error, 3u); return; }
    }
    for (uint path = 0; path < paths; ++path) {
        const float* q = q_all + path * stages * 8;
        float* dq = d_queries + (u64)output * paths * stages * 8 + path * stages * 8;
        ScFbsRoute selected = sc_fbs_route(t, q, SC_FBS_ABSENT, 0);
        if (selected.atom == SC_FBS_ABSENT || selected.atom >= inputs) {
            atomicExch(error, 2u); return;
        }
        float y[8];
        for (uint axis = 0; axis < 8; ++axis) {
            y[axis] = sc_arm_coordinate(t, x, selected.atom, selected.state, axis, arm);
            if (!isfinite(y[axis])) { atomicExch(error, 3u); return; }
            uint target = axis;
            float value = dy[axis];
            if (arm != SC_ARM_IDENTITY) {
                uint code = axes[selected.state * 8 + axis];
                target = code & 7;
                if (code & 8) value = -value;
            }
            float prior = atomicAdd(d_input + (u64)row * inputs * 8 + selected.atom * 8 + target, value);
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
                    float other = sc_arm_coordinate(t, x, alternative.atom, alternative.state, axis, arm);
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
                    float prior = atomicAdd(dq + stage * 8 + axis, value);
                    if (!isfinite(prior + value)) { atomicExch(error, 4u); return; }
                }
            }
        }
    }
}
