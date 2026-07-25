/* @ts-self-types="./uor_r4_wasm_router.d.ts" */

/**
 * @enum {0 | 1}
 */
export const GeometryType = Object.freeze({
    Spectral: 0, "0": "Spectral",
    Vsa: 1, "1": "Vsa",
});

/**
 * The unified router core coordinator.
 */
export class UorR4Router {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        UorR4RouterFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_uorr4router_free(ptr, 0);
    }
    /**
     * @returns {GeometryType}
     */
    get geometry_type() {
        const ret = wasm.__wbg_get_uorr4router_geometry_type(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {GeometryType} arg0
     */
    set geometry_type(arg0) {
        wasm.__wbg_set_uorr4router_geometry_type(this.__wbg_ptr, arg0);
    }
    /**
     * Computes live UOR resonance metrics for a given input text
     * @param {string} text
     * @returns {any}
     */
    calculate_resonance(text) {
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_calculate_resonance(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    clear_corpus() {
        wasm.uorr4router_clear_corpus(this.__wbg_ptr);
    }
    /**
     * Compiles a raw string thought parameter down into its content-addressed math state
     * @param {string} content
     * @returns {any}
     */
    compile_thought(content) {
        const ptr0 = passStringToWasm0(content, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_compile_thought(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Returns current connection drift
     * @returns {number}
     */
    connection_drift() {
        const ret = wasm.uorr4router_connection_drift(this.__wbg_ptr);
        return ret;
    }
    /**
     * Evolves state vector using user prompt words and returns the new state
     * @param {string} identity
     * @param {string} text
     * @param {number} gamma
     * @returns {Float64Array}
     */
    evolve_state(identity, text, gamma) {
        const ptr0 = passStringToWasm0(identity, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_evolve_state(this.__wbg_ptr, ptr0, len0, ptr1, len1, gamma);
        var v3 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v3;
    }
    /**
     * Reset the alignment back to native state ($0.00\%$ error) using ZKP 2i Sync-Handshake
     * @returns {string}
     */
    execute_zkp_phase_reset() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.uorr4router_execute_zkp_phase_reset(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Exports the full router system database to JSON string
     * @returns {string}
     */
    export_state() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.uorr4router_export_state(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Decodes a response steered by the active brain state vector
     * @param {string} text
     * @param {string} identity
     * @param {number} max_tokens
     * @param {number} temp
     * @param {number} gravity
     * @param {number} freq_penalty
     * @param {number} gamma
     * @returns {any}
     */
    generate_geometric_response(text, identity, max_tokens, temp, gravity, freq_penalty, gamma) {
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(identity, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_generate_geometric_response(this.__wbg_ptr, ptr0, len0, ptr1, len1, max_tokens, temp, gravity, freq_penalty, gamma);
        return ret;
    }
    /**
     * Returns the active stream list as a JS Array
     * @returns {any}
     */
    get_active_streams() {
        const ret = wasm.uorr4router_get_active_streams(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_angle_x() {
        const ret = wasm.uorr4router_get_angle_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_angle_y() {
        const ret = wasm.uorr4router_get_angle_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * Retrieves the evolved brain state vector for a given identity
     * @param {string} identity
     * @returns {Float64Array}
     */
    get_brain_state_wasm(identity) {
        const ptr0 = passStringToWasm0(identity, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_get_brain_state_wasm(this.__wbg_ptr, ptr0, len0);
        var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v2;
    }
    /**
     * Returns the active counts for the 64 experts
     * @returns {Uint32Array}
     */
    get_expert_counts() {
        const ret = wasm.uorr4router_get_expert_counts(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Serves all points in the corpus index for the semantic map visualizer
     * @returns {any}
     */
    get_semantic_map_points() {
        const ret = wasm.uorr4router_get_semantic_map_points(this.__wbg_ptr);
        return ret;
    }
    /**
     * Projects the active brain state vector into 2D coordinates for the map path tracing
     * @param {Float64Array} state_vector
     * @param {number} win_idx
     * @returns {Float64Array}
     */
    get_sentence_projection_wasm(state_vector, win_idx) {
        const ptr0 = passArrayF64ToWasm0(state_vector, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_get_sentence_projection_wasm(this.__wbg_ptr, ptr0, len0, win_idx);
        var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v2;
    }
    /**
     * Projects the active brain state vector into 4D coordinates
     * @param {Float64Array} state_vector
     * @returns {Float64Array}
     */
    get_state_4d_projection_wasm(state_vector) {
        const ptr0 = passArrayF64ToWasm0(state_vector, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_get_state_4d_projection_wasm(this.__wbg_ptr, ptr0, len0);
        var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v2;
    }
    /**
     * @returns {string}
     */
    get_store_epoch_root() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.uorr4router_get_store_epoch_root(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} facet
     * @param {string} path_str
     * @returns {any}
     */
    get_store_inclusion_proof(facet, path_str) {
        const ptr0 = passStringToWasm0(facet, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(path_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_get_store_inclusion_proof(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return ret;
    }
    /**
     * Dynamically computes the suggested token limit based on manifold routing metrics
     * @param {string} text
     * @param {string} identity
     * @returns {number}
     */
    get_suggested_token_limit(text, identity) {
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(identity, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_get_suggested_token_limit(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return ret >>> 0;
    }
    /**
     * Returns the top N resonant sentences sorted by relevance
     * @param {string} text
     * @param {string} identity
     * @param {number} top_n
     * @returns {any}
     */
    get_top_resonances(text, identity, top_n) {
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(identity, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_get_top_resonances(this.__wbg_ptr, ptr0, len0, ptr1, len1, top_n);
        return ret;
    }
    /**
     * Returns the total number of indexed sentences in the corpus
     * @returns {number}
     */
    get_total_indexed_sentences() {
        const ret = wasm.uorr4router_get_total_indexed_sentences(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the number of words in the vocabulary index
     * @returns {number}
     */
    get_vocab_size() {
        const ret = wasm.uorr4router_get_vocab_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Imports a JSON string and restores the router system database
     * @param {string} json_str
     */
    import_state(json_str) {
        const ptr0 = passStringToWasm0(json_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_import_state(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Indexes an entire block of text split into sentences
     * @param {string} corpus_text
     * @param {string} identity
     * @returns {number}
     */
    index_corpus(corpus_text, identity) {
        const ptr0 = passStringToWasm0(corpus_text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(identity, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_index_corpus(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return ret >>> 0;
    }
    index_default_corpus() {
        wasm.uorr4router_index_default_corpus(this.__wbg_ptr);
    }
    /**
     * Indexes a single sentence into the identity's scoped corpus
     * @param {string} sentence
     * @param {string} identity
     */
    index_sentence(sentence, identity) {
        const ptr0 = passStringToWasm0(sentence, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(identity, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        wasm.uorr4router_index_sentence(this.__wbg_ptr, ptr0, len0, ptr1, len1);
    }
    /**
     * Injects a new thought stream, updates MoE activations, and returns the stream
     * @param {string} content
     * @returns {any}
     */
    inject_thought_stream(content) {
        const ptr0 = passStringToWasm0(content, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_inject_thought_stream(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Exposes read-only status of manifold alignment
     * @returns {boolean}
     */
    is_aligned() {
        const ret = wasm.uorr4router_is_aligned(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Returns the kill switch threshold limit
     * @returns {number}
     */
    kill_switch_threshold() {
        const ret = wasm.uorr4router_kill_switch_threshold(this.__wbg_ptr);
        return ret;
    }
    /**
     * Instantiates the R4 Router with perfect, error-free default states
     * @param {number} threshold
     */
    constructor(threshold) {
        const ret = wasm.uorr4router_new(threshold);
        this.__wbg_ptr = ret;
        UorR4RouterFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Resets the brain state vector for a specific identity
     * @param {string} identity
     */
    reset_brain(identity) {
        const ptr0 = passStringToWasm0(identity, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.uorr4router_reset_brain(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Resets the entire router system back to factory defaults
     */
    reset_to_defaults() {
        wasm.uorr4router_reset_to_defaults(this.__wbg_ptr);
    }
    /**
     * Returns the routed window and detailed thermodynamic/Hopf metrics for a query
     * @param {string} text
     * @param {string} identity
     * @returns {any}
     */
    route_query_to_manifold(text, identity) {
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(identity, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_route_query_to_manifold(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return ret;
    }
    /**
     * Runs the formal UOR coordinate reduction pipeline and returns both RoutingData and trace steps as a single JsValue
     * @param {string} text
     * @param {string} identity
     * @returns {any}
     */
    route_query_to_manifold_uor(text, identity) {
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(identity, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.uorr4router_route_query_to_manifold_uor(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return ret;
    }
    /**
     * @param {number} val
     */
    set_angle_x(val) {
        wasm.uorr4router_set_angle_x(this.__wbg_ptr, val);
    }
    /**
     * @param {number} val
     */
    set_angle_y(val) {
        wasm.uorr4router_set_angle_y(this.__wbg_ptr, val);
    }
    /**
     * @param {string} geom
     */
    set_geometry_type(geom) {
        const ptr0 = passStringToWasm0(geom, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.uorr4router_set_geometry_type(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Progresses the connection drift state using delta-time ($dt$) increments.
     * Returns a log message string if a ZKP reset occurs, otherwise returns undefined.
     * @param {number} dt
     * @param {number} drift_rate
     * @returns {string | undefined}
     */
    update_drift_physics(dt, drift_rate) {
        const ret = wasm.uorr4router_update_drift_physics(this.__wbg_ptr, dt, drift_rate);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
}
if (Symbol.dispose) UorR4Router.prototype[Symbol.dispose] = UorR4Router.prototype.free;

/**
 * @param {string} prompt
 * @param {number} max_tokens
 * @returns {string | undefined}
 */
export function generate_r4g1_response(prompt, max_tokens) {
    const ptr0 = passStringToWasm0(prompt, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.generate_r4g1_response(ptr0, len0, max_tokens);
    let v2;
    if (ret[0] !== 0) {
        v2 = getStringFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    }
    return v2;
}

export function init_wasm() {
    wasm.init_wasm();
}

/**
 * @param {string} subj
 * @param {string} act
 * @param {string} time
 * @param {string} loc
 * @param {string} space
 * @returns {Uint8Array}
 */
export function vsa_encode_event(subj, act, time, loc, space) {
    const ptr0 = passStringToWasm0(subj, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(act, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passStringToWasm0(time, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passStringToWasm0(loc, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len3 = WASM_VECTOR_LEN;
    const ptr4 = passStringToWasm0(space, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len4 = WASM_VECTOR_LEN;
    const ret = wasm.vsa_encode_event(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, ptr4, len4);
    var v6 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v6;
}

/**
 * @param {string} src
 * @param {string} rel
 * @param {string} tgt
 * @param {string} space
 * @returns {Uint8Array}
 */
export function vsa_encode_graph_edge(src, rel, tgt, space) {
    const ptr0 = passStringToWasm0(src, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(rel, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passStringToWasm0(tgt, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passStringToWasm0(space, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len3 = WASM_VECTOR_LEN;
    const ret = wasm.vsa_encode_graph_edge(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
    var v5 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v5;
}

/**
 * @param {string} subj
 * @param {string} pred
 * @param {string} obj
 * @param {string} space
 * @returns {Uint8Array}
 */
export function vsa_encode_statement(subj, pred, obj, space) {
    const ptr0 = passStringToWasm0(subj, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(pred, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passStringToWasm0(obj, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passStringToWasm0(space, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len3 = WASM_VECTOR_LEN;
    const ret = wasm.vsa_encode_statement(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
    var v5 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v5;
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_92b29b0548f8b746: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg___wbindgen_is_string_ea5e6cc2e4141dfe: function(arg0) {
            const ret = typeof(arg0) === 'string';
            return ret;
        },
        __wbg___wbindgen_throw_344f42d3211c4765: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_32b398fb48b6d94a: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_7796ffc7ed656783: function() {
            const ret = new Map();
            return ret;
        },
        __wbg_new_da52cf8fe3429cb2: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_set_575dd786d51585f8: function(arg0, arg1, arg2) {
            const ret = arg0.set(arg1, arg2);
            return ret;
        },
        __wbg_set_6be42768c690e380: function(arg0, arg1, arg2) {
            arg0[arg1] = arg2;
        },
        __wbg_set_8a16b38e4805b298: function(arg0, arg1, arg2) {
            arg0[arg1 >>> 0] = arg2;
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbindgen_cast_0000000000000001: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0) {
            // Cast intrinsic for `I64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000004: function(arg0) {
            // Cast intrinsic for `U64 -> Externref`.
            const ret = BigInt.asUintN(64, arg0);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./uor_r4_wasm_router_bg.js": import0,
    };
}

const UorR4RouterFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_uorr4router_free(ptr, 1));

function getArrayF64FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat64ArrayMemory0().subarray(ptr / 8, ptr / 8 + len);
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat64ArrayMemory0 = null;
function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passArrayF64ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 8, 8) >>> 0;
    getFloat64ArrayMemory0().set(arg, ptr / 8);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat64ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('uor_r4_wasm_router_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
