/* tslint:disable */
/* eslint-disable */

export enum GeometryType {
    Spectral = 0,
    Vsa = 1,
}

/**
 * The unified router core coordinator.
 */
export class UorR4Router {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Computes live UOR resonance metrics for a given input text
     */
    calculate_resonance(text: string): any;
    clear_corpus(): void;
    /**
     * Compiles a raw string thought parameter down into its content-addressed math state
     */
    compile_thought(content: string): any;
    /**
     * Returns current connection drift
     */
    connection_drift(): number;
    /**
     * Evolves state vector using user prompt words and returns the new state
     */
    evolve_state(identity: string, text: string, gamma: number): Float64Array;
    /**
     * Reset the alignment back to native state ($0.00\%$ error) using ZKP 2i Sync-Handshake
     */
    execute_zkp_phase_reset(): string;
    /**
     * Exports the full router system database to JSON string
     */
    export_state(): string;
    /**
     * Decodes a response steered by the active brain state vector
     */
    generate_geometric_response(text: string, identity: string, max_tokens: number, temp: number, gravity: number, freq_penalty: number, gamma: number): any;
    /**
     * Returns the active stream list as a JS Array
     */
    get_active_streams(): any;
    get_angle_x(): number;
    get_angle_y(): number;
    /**
     * Retrieves the evolved brain state vector for a given identity
     */
    get_brain_state_wasm(identity: string): Float64Array;
    /**
     * Returns the active counts for the 64 experts
     */
    get_expert_counts(): Uint32Array;
    /**
     * Serves all points in the corpus index for the semantic map visualizer
     */
    get_semantic_map_points(): any;
    /**
     * Projects the active brain state vector into 2D coordinates for the map path tracing
     */
    get_sentence_projection_wasm(state_vector: Float64Array, win_idx: number): Float64Array;
    /**
     * Projects the active brain state vector into 4D coordinates
     */
    get_state_4d_projection_wasm(state_vector: Float64Array): Float64Array;
    get_store_epoch_root(): string;
    get_store_inclusion_proof(facet: string, path_str: string): any;
    /**
     * Dynamically computes the suggested token limit based on manifold routing metrics
     */
    get_suggested_token_limit(text: string, identity: string): number;
    /**
     * Returns the top N resonant sentences sorted by relevance
     */
    get_top_resonances(text: string, identity: string, top_n: number): any;
    /**
     * Returns the total number of indexed sentences in the corpus
     */
    get_total_indexed_sentences(): number;
    /**
     * Returns the number of words in the vocabulary index
     */
    get_vocab_size(): number;
    /**
     * Imports a JSON string and restores the router system database
     */
    import_state(json_str: string): void;
    /**
     * Indexes an entire block of text split into sentences
     */
    index_corpus(corpus_text: string, identity: string): number;
    index_default_corpus(): void;
    /**
     * Indexes a single sentence into the identity's scoped corpus
     */
    index_sentence(sentence: string, identity: string): void;
    /**
     * Injects a new thought stream, updates MoE activations, and returns the stream
     */
    inject_thought_stream(content: string): any;
    /**
     * Exposes read-only status of manifold alignment
     */
    is_aligned(): boolean;
    /**
     * Returns the kill switch threshold limit
     */
    kill_switch_threshold(): number;
    /**
     * Instantiates the R4 Router with perfect, error-free default states
     */
    constructor(threshold: number);
    /**
     * Resets the brain state vector for a specific identity
     */
    reset_brain(identity: string): void;
    /**
     * Resets the entire router system back to factory defaults
     */
    reset_to_defaults(): void;
    /**
     * Returns the routed window and detailed thermodynamic/Hopf metrics for a query
     */
    route_query_to_manifold(text: string, identity: string): any;
    /**
     * Runs the formal UOR coordinate reduction pipeline and returns both RoutingData and trace steps as a single JsValue
     */
    route_query_to_manifold_uor(text: string, identity: string): any;
    set_angle_x(val: number): void;
    set_angle_y(val: number): void;
    set_geometry_type(geom: string): void;
    /**
     * Progresses the connection drift state using delta-time ($dt$) increments.
     * Returns a log message string if a ZKP reset occurs, otherwise returns undefined.
     */
    update_drift_physics(dt: number, drift_rate: number): string | undefined;
    geometry_type: GeometryType;
}

export function generate_r4g1_response(prompt: string, max_tokens: number): string | undefined;

export function init_wasm(): void;

export function vsa_encode_event(subj: string, act: string, time: string, loc: string, space: string): Uint8Array;

export function vsa_encode_graph_edge(src: string, rel: string, tgt: string, space: string): Uint8Array;

export function vsa_encode_statement(subj: string, pred: string, obj: string, space: string): Uint8Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly generate_r4g1_response: (a: number, b: number, c: number) => [number, number];
    readonly __wbg_get_uorr4router_geometry_type: (a: number) => number;
    readonly __wbg_set_uorr4router_geometry_type: (a: number, b: number) => void;
    readonly __wbg_uorr4router_free: (a: number, b: number) => void;
    readonly init_wasm: () => void;
    readonly uorr4router_calculate_resonance: (a: number, b: number, c: number) => any;
    readonly uorr4router_clear_corpus: (a: number) => void;
    readonly uorr4router_compile_thought: (a: number, b: number, c: number) => any;
    readonly uorr4router_connection_drift: (a: number) => number;
    readonly uorr4router_evolve_state: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly uorr4router_execute_zkp_phase_reset: (a: number) => [number, number];
    readonly uorr4router_export_state: (a: number) => [number, number];
    readonly uorr4router_generate_geometric_response: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => any;
    readonly uorr4router_get_active_streams: (a: number) => any;
    readonly uorr4router_get_angle_x: (a: number) => number;
    readonly uorr4router_get_angle_y: (a: number) => number;
    readonly uorr4router_get_brain_state_wasm: (a: number, b: number, c: number) => [number, number];
    readonly uorr4router_get_expert_counts: (a: number) => [number, number];
    readonly uorr4router_get_semantic_map_points: (a: number) => any;
    readonly uorr4router_get_sentence_projection_wasm: (a: number, b: number, c: number, d: number) => [number, number];
    readonly uorr4router_get_state_4d_projection_wasm: (a: number, b: number, c: number) => [number, number];
    readonly uorr4router_get_store_epoch_root: (a: number) => [number, number];
    readonly uorr4router_get_store_inclusion_proof: (a: number, b: number, c: number, d: number, e: number) => any;
    readonly uorr4router_get_suggested_token_limit: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly uorr4router_get_top_resonances: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
    readonly uorr4router_get_total_indexed_sentences: (a: number) => number;
    readonly uorr4router_get_vocab_size: (a: number) => number;
    readonly uorr4router_import_state: (a: number, b: number, c: number) => [number, number];
    readonly uorr4router_index_corpus: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly uorr4router_index_default_corpus: (a: number) => void;
    readonly uorr4router_index_sentence: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly uorr4router_inject_thought_stream: (a: number, b: number, c: number) => any;
    readonly uorr4router_is_aligned: (a: number) => number;
    readonly uorr4router_kill_switch_threshold: (a: number) => number;
    readonly uorr4router_new: (a: number) => number;
    readonly uorr4router_reset_brain: (a: number, b: number, c: number) => void;
    readonly uorr4router_reset_to_defaults: (a: number) => void;
    readonly uorr4router_route_query_to_manifold: (a: number, b: number, c: number, d: number, e: number) => any;
    readonly uorr4router_route_query_to_manifold_uor: (a: number, b: number, c: number, d: number, e: number) => any;
    readonly uorr4router_set_angle_x: (a: number, b: number) => void;
    readonly uorr4router_set_angle_y: (a: number, b: number) => void;
    readonly uorr4router_set_geometry_type: (a: number, b: number, c: number) => void;
    readonly uorr4router_update_drift_physics: (a: number, b: number, c: number) => [number, number];
    readonly vsa_encode_event: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number];
    readonly vsa_encode_graph_edge: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
    readonly vsa_encode_statement: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
