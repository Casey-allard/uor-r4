import os
# Set single-thread environment variables BEFORE importing numpy to avoid deadlocks on macOS
os.environ["VECLIB_MAXIMUM_THREADS"] = "1"
os.environ["OMP_NUM_THREADS"] = "1"
os.environ["MKL_NUM_THREADS"] = "1"
os.environ["OPENBLAS_NUM_THREADS"] = "1"

import sys
import json
import math
import random
import time
import threading
import urllib.request
from http.server import BaseHTTPRequestHandler, HTTPServer, ThreadingHTTPServer
# 1. Automatic dependency bootstrap check before numpy import
try:
    import numpy as np
except ImportError:
    print("[*] numpy is not installed. Attempting to install required dependencies (numpy, psutil, opentelemetry)...")
    import subprocess
    try:
        subprocess.check_call([sys.executable, "-m", "pip", "install", "numpy", "psutil", "opentelemetry-api", "opentelemetry-sdk"])
        print("[+] Dependencies successfully installed.")
    except Exception as e:
        print(f"[-] Failed to automatically install dependencies: {e}")
        print("[!] Please run: pip3 install numpy psutil opentelemetry-api opentelemetry-sdk")
        sys.exit(1)

import numpy as np
import hashlib
import unicodedata
from collections.abc import Sequence
from functools import lru_cache

# ─── OpenTelemetry ────────────────────────────────────────────────────────────
try:
    from opentelemetry import trace, metrics as otel_metrics
    from opentelemetry.sdk.trace import TracerProvider
    from opentelemetry.sdk.trace.export import SimpleSpanProcessor, ConsoleSpanExporter
    from opentelemetry.sdk.metrics import MeterProvider
    from opentelemetry.sdk.metrics.export import ConsoleMetricExporter, PeriodicExportingMetricReader
    from opentelemetry.sdk.resources import Resource

    _otel_resource = Resource.create({"service.name": "r4-prime-router"})
    _tracer_provider = TracerProvider(resource=_otel_resource)
    # Console exporter (silent by default — set R4_OTEL_TRACE=1 to enable)
    if os.environ.get("R4_OTEL_TRACE"):
        _tracer_provider.add_span_processor(SimpleSpanProcessor(ConsoleSpanExporter()))
    trace.set_tracer_provider(_tracer_provider)
    TRACER = trace.get_tracer("r4.prime.router")

    _metric_reader = PeriodicExportingMetricReader(ConsoleMetricExporter(), export_interval_millis=60000)
    _meter_provider = MeterProvider(resource=_otel_resource, metric_readers=[_metric_reader])
    otel_metrics.set_meter_provider(_meter_provider)
    METER = otel_metrics.get_meter("r4.prime.router")

    # Define instruments
    _req_counter      = METER.create_counter("r4.requests.total", description="Total API requests")
    _routing_hist     = METER.create_histogram("r4.routing.latency_ms", description="Routing latency (ms)")
    _gen_hist         = METER.create_histogram("r4.generation.latency_ms", description="Generation latency (ms)")
    _catastrophe_ctr  = METER.create_counter("r4.catastrophe.total", description="Catastrophe events")
    _window_ctr       = METER.create_counter("r4.routing.window_hits", description="Routing hits per window")
    OTEL_AVAILABLE = True
    print("[+] OpenTelemetry initialized (set R4_OTEL_TRACE=1 to enable trace export)")
except ImportError:
    OTEL_AVAILABLE = False
    TRACER = None
    print("[!] opentelemetry-sdk not found — install with: pip install opentelemetry-api opentelemetry-sdk")

# ─── psutil ───────────────────────────────────────────────────────────────────
try:
    import psutil
    PSUTIL_AVAILABLE = True
except ImportError:
    PSUTIL_AVAILABLE = False

# ─── Server-side stats (thread-safe counters) ─────────────────────────────────
_SERVER_START_TIME = time.time()
_stats_lock = threading.Lock()
_SERVER_STATS = {
    "requests_total": 0,
    "routing_latencies": [],   # last 100
    "gen_latencies": [],       # last 100
    "catastrophes": 0,
    "window_hits": {str(i): 0 for i in range(1, 17)},
}

def _record_request(endpoint: str):
    with _stats_lock:
        _SERVER_STATS["requests_total"] += 1
    if OTEL_AVAILABLE:
        _req_counter.add(1, {"endpoint": endpoint})

def _record_routing_latency(ms: float, window: int):
    with _stats_lock:
        lats = _SERVER_STATS["routing_latencies"]
        lats.append(ms)
        if len(lats) > 100: lats.pop(0)
        _SERVER_STATS["window_hits"][str(window)] = _SERVER_STATS["window_hits"].get(str(window), 0) + 1
    if OTEL_AVAILABLE:
        _routing_hist.record(ms)
        _window_ctr.add(1, {"window": str(window)})

def _record_gen_latency(ms: float):
    with _stats_lock:
        lats = _SERVER_STATS["gen_latencies"]
        lats.append(ms)
        if len(lats) > 100: lats.pop(0)
    if OTEL_AVAILABLE:
        _gen_hist.record(ms)

def _record_catastrophe():
    with _stats_lock:
        _SERVER_STATS["catastrophes"] += 1
    if OTEL_AVAILABLE:
        _catastrophe_ctr.add(1)

def _percentile(lst, p):
    if not lst: return 0.0
    s = sorted(lst)
    idx = int(len(s) * p / 100)
    return s[min(idx, len(s)-1)]

# Add current workspace to path to import prime_router_package
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

try:
    from prime_router_package import (
        load_true_zeros,
        build_psi_table,
        design_matrix_sparse_r4,
        qr_orthonormal_state,
        covariance_eigenvalue_state,
        state_metrics_from_weights,
        centered_l2_normalize,
        X_MAX,
        X_MIN,
        RHO,
        N_SAMPLES,
        NUM_WINDOWS,
        SUBWINDOWS
    )
except ImportError as e:
    print(f"[-] Failed to import from prime_router_package.py: {e}")
    sys.exit(1)

# ============================================================
# QIMC, Hopf, and UOF Scalar Helpers
# ============================================================
def _wrap_to_pi(theta: float) -> float:
    return (theta + math.pi) % (2.0 * math.pi) - math.pi

@lru_cache(maxsize=128)
def allocate_triplet_bins_budget(
    total_cap: int,
    *,
    min_first: int = 2,
    min_second: int = 1,
    min_third: int = 1,
) -> tuple[int, int, int]:
    total_cap = max(1, int(total_cap))
    min_first = max(1, int(min_first))
    min_second = max(1, int(min_second))
    min_third = max(1, int(min_third))
    if min_first * min_second * min_third > total_cap:
        min_third = 1
    if min_first * min_second * min_third > total_cap:
        min_second = 1
    if min_first * min_second * min_third > total_cap:
        min_first = 1
    best = (1, total_cap, 1)
    best_score = None
    for k_first in range(min_first, total_cap + 1):
        for k_second in range(min_second, total_cap + 1):
            max_third = total_cap // max(k_first * k_second, 1)
            if max_third < min_third:
                break
            for k_third in range(min_third, max_third + 1):
                product = k_first * k_second * k_third
                favor_base = 1 if k_second >= k_third else 0
                spread = (
                    abs(k_first - k_second)
                    + abs(k_second - k_third)
                    + abs(k_first - k_third)
                )
                score = (product, favor_base, -spread, k_second, -k_third)
                if best_score is None or score > best_score:
                    best_score = score
                    best = (k_first, k_second, k_third)
    return best

def hopf_coordinate_components_scalar(normalized_coordinate: Sequence[float]) -> dict[str, float]:
    a, b, c, d = (float(value) for value in normalized_coordinate)
    rho1 = math.sqrt((a * a) + (b * b))
    rho2 = math.sqrt((c * c) + (d * d))
    denom = max(math.sqrt((rho1 * rho1) + (rho2 * rho2)), 1e-12)
    cos_chi = rho1 / denom
    sin_chi = rho2 / denom
    return {
        "rho1": rho1,
        "rho2": rho2,
        "chi": math.asin(min(max(sin_chi, 0.0), 1.0)),
        "chi_u": min(max(sin_chi * sin_chi, 0.0), 1.0 - 1e-12),
        "theta1": _wrap_to_pi(math.atan2(b, a)),
        "theta2": _wrap_to_pi(math.atan2(d, c)),
        "delta": _wrap_to_pi(math.atan2(b, a) - math.atan2(d, c)),
        "alpha": _wrap_to_pi(0.5 * (math.atan2(b, a) + math.atan2(d, c))),
        "cos_chi": cos_chi,
        "sin_chi": sin_chi,
    }

def hopf_phase_transport_components_scalar(
    normalized_coordinate: Sequence[float],
    *,
    phase_transport_lambda: float,
) -> dict[str, float]:
    components = hopf_coordinate_components_scalar(normalized_coordinate)
    chi = components["chi"]
    delta = components["delta"]
    alpha = components["alpha"]
    connection_weight = 0.5 * float(phase_transport_lambda) * math.cos(2.0 * chi)
    phase_shift = _wrap_to_pi(connection_weight * delta)
    transported_alpha = _wrap_to_pi(alpha + phase_shift)
    return {
        **components,
        "transport_connection_weight": connection_weight,
        "transport_phase_shift": phase_shift,
        "transported_alpha": transported_alpha,
    }

def assign_sector_hopf_transport_scalar(
    normalized_coordinate: Sequence[float],
    *,
    K: int,
    phase_transport_lambda: float,
    hopf_chi_bins: int,
) -> dict[str, object]:
    components = hopf_phase_transport_components_scalar(
        normalized_coordinate,
        phase_transport_lambda=phase_transport_lambda,
    )
    kchi_value, kdelta_value, kalpha_value = allocate_triplet_bins_budget(
        K,
        min_first=max(2, int(hopf_chi_bins)),
        min_second=2,
        min_third=2,
    )
    u_delta = (components["delta"] + math.pi) / (2.0 * math.pi)
    u_alpha = (components["transported_alpha"] + math.pi) / (2.0 * math.pi)
    chi_bin = min(int(components["chi_u"] * float(kchi_value)), max(kchi_value - 1, 0))
    delta_bin = min(int(u_delta * float(kdelta_value)), max(kdelta_value - 1, 0))
    alpha_bin = min(int(u_alpha * float(kalpha_value)), max(kalpha_value - 1, 0))
    local_span = max(kdelta_value * kalpha_value, 1)
    sector_id = min((chi_bin * local_span) + (delta_bin * kalpha_value) + alpha_bin, max(int(K) - 1, 0))
    return {
        "coordinates": components,
        "sector_id": int(sector_id),
        "sector_bins": {
            "chi_bins": kchi_value,
            "delta_bins": kdelta_value,
            "alpha_bins": kalpha_value,
            "chi_bin": chi_bin,
            "delta_bin": delta_bin,
            "alpha_bin": alpha_bin,
        },
    }

def is_prime(n):
    if n < 2: return False
    for j in range(2, int(math.isqrt(n)) + 1):
        if n % j == 0: return False
    return True

def get_primes_6k_plus_1(count):
    primes = []
    k = 1
    while len(primes) < count:
        candidate = 6 * k + 1
        if is_prime(candidate):
            primes.append(candidate)
        k += 1
    return primes

PRIMES_6K = get_primes_6k_plus_1(512)

def normalize_mac(mac_str):
    clean = "".join(c for c in mac_str if c.isalnum()).lower()
    if len(clean) == 12:
        return ":".join(clean[i:i+2] for i in range(0, 12, 2))
    return mac_str.strip().lower()

def mac_to_qimc_prime(mac_str):
    normalized = normalize_mac(mac_str)
    clean = "".join(c for c in normalized if c.isalnum())
    if len(clean) == 12:
        try:
            val = int(clean, 16)
        except ValueError:
            val = int(hashlib.sha256(normalized.encode('utf-8')).hexdigest()[:12], 16)
    else:
        val = int(hashlib.sha256(normalized.encode('utf-8')).hexdigest()[:12], 16)
    idx = (val % 500) + 1
    return PRIMES_6K[idx - 1], idx

def jcs_canonical_serialize(obj):
    if isinstance(obj, dict):
        items = []
        for k in sorted(obj.keys()):
            val = obj[k]
            k_norm = unicodedata.normalize('NFC', k)
            val_serialized = jcs_canonical_serialize(val)
            items.append(f'"{k_norm}":{val_serialized}')
        return "{" + ",".join(items) + "}"
    elif isinstance(obj, str):
        val_norm = unicodedata.normalize('NFC', obj)
        return json.dumps(val_norm)
    elif isinstance(obj, bool):
        return "true" if obj else "false"
    elif isinstance(obj, int):
        return str(obj)
    elif isinstance(obj, float):
        if math.isnan(obj) or math.isinf(obj):
            raise ValueError("JCS does not support NaN/Infinity")
        if obj.is_integer():
            return str(int(obj))
        s = str(obj)
        s = s.lower().replace("e+", "e")
        return s
    elif obj is None:
        return "null"
    else:
        raise TypeError(f"Unsupported type: {type(obj)}")

def generate_uof_hash(payload):
    canonical = jcs_canonical_serialize(payload)
    h = hashlib.sha256(canonical.encode('utf-8')).hexdigest()
    return f"sha256:{h}"

# Pre-load data and table
print("[*] Pre-loading mathematical tables for server boot...")
PSI_TABLE = build_psi_table(X_MAX, RHO)
M_MAX = 512
GAMMAS = load_true_zeros(M_MAX)
print("[+] Mathematical tables loaded. Server ready.")

# Hypersphere brain state
print("[*] Initializing persistent 512D session brain state...")
SESSION_BRAIN_STATE = np.ones(M_MAX) / math.sqrt(M_MAX)

def reset_brain_state():
    global SESSION_BRAIN_STATE
    SESSION_BRAIN_STATE = np.ones(M_MAX) / math.sqrt(M_MAX)
    print("[+] Hypersphere brain state reset to baseline.")

def evolve_brain_state(query_text: str, gamma: float = 0.5) -> np.ndarray:
    global SESSION_BRAIN_STATE, VOCAB_VECTORS
    words = [w.lower().strip(".,?!()\"';:-") for w in query_text.split() if w.strip()]
    S = np.zeros(M_MAX)
    word_count = 0
    for w in words:
        if w in VOCAB_VECTORS:
            S += VOCAB_VECTORS[w]
            word_count += 1
            
    if word_count > 0:
        S_norm = np.linalg.norm(S)
        if S_norm > 0:
            S = S / S_norm
        H_new = gamma * SESSION_BRAIN_STATE + (1.0 - gamma) * S
        H_norm = np.linalg.norm(H_new)
        if H_norm > 0:
            SESSION_BRAIN_STATE = H_new / H_norm
            
    return SESSION_BRAIN_STATE

# Define the grid of scales x
X_GRID = np.exp(np.linspace(math.log(X_MIN), math.log(X_MAX), NUM_WINDOWS))

# Pre-compute static routing manifolds for all 16 windows to enable sub-millisecond queries
PRECOMPUTED_WINDOWS = []
print("[*] Pre-computing static R4 manifold projections...")
for idx, x in enumerate(X_GRID):
    H = RHO * math.sqrt(x)
    t_grid = np.linspace(-H, H, N_SAMPLES)
    xx = x + t_grid
    Phi, s_idx, e_idx = design_matrix_sparse_r4(xx, GAMMAS, x, X_MAX)
    Q, _ = np.linalg.qr(Phi, mode="reduced")
    
    seg_len = N_SAMPLES // SUBWINDOWS
    Qs_list = []
    slice_ranges = []
    for s in range(SUBWINDOWS):
        start_t = s * seg_len
        end_t = (s + 1) * seg_len if s < SUBWINDOWS - 1 else N_SAMPLES
        Qs_list.append(Q[start_t:end_t, :])
        slice_ranges.append((start_t, end_t))
        
    PRECOMPUTED_WINDOWS.append({
        "x": x,
        "t_grid": t_grid,
        "s_idx": s_idx,
        "e_idx": e_idx,
        "Q": Q,
        "Qs_list": Qs_list,
        "slice_ranges": slice_ranges
    })
print("[+] Static manifolds pre-computed successfully.")

# ============================================================
# World Model Knowledge Base (Corpus Manifold Database)
# ============================================================
DEFAULT_CORPUS = """
Welcome to the R4 Prime Router. This is a local geometric world model.
I can help you coordinate water borehole data for the Gambia project.
The dry season in the Gambia requires deep aquifer coordination.
We can map borehole locations directly onto the prime number coordinates.
No training is required because the Riemann zeta zeroes form a stable coordinate system.
This engine replaces the transformer MoE gating using sparse orthogonal projections.
A traditional transformer routes tokens using learned parameters, but we route using prime factor frequency manifolds.
Each scale window acts as an expert containing specific geometric resonances.
The deficit angle measures the deflection of your query relative to the hypersphere curvature.
If the deficit angle is positive, the wave is trapped in a stable periodic orbit.
Negative deficit angles indicate hyperbolic divergence and scattering.
A symmetric orbit indicates stable, logical, and focused input sequences.
Coherence kappa indicates how well the prompt wave aligns with the local zero frequencies.
To talk to this engine, you must populate its manifold coordinates with a starting text.
You can paste any text corpus to dynamically index new knowledge into the manifold.
Once indexed, the router retrieves and synthesizes responses based on state vector similarity.
The 512 dimensions correspond to the first 512 non-trivial Riemann zeta zeroes.
Water flow rates in the Gambia depend on the aquifer's soil coherence.
The prime router helps you find the most efficient path for borehole water flow coordinates.
We can run this engine entirely locally without internet access or third-party APIs.
You are talking directly to the mathematical voice of the prime spectrum.
Ask me about the Gambia borehole locations, or how the R4 routing replaces transformer layers.
"""

# Global database of indexed sentences on the manifold
# Key: scale window index, Value: list of dictionaries
CORPUS_INDEX = {}

# Global vocabulary databases for auto-regressive next-token generation
VOCABULARY = []
WORD_PRIMES = {}
VOCAB_VECTORS = {}
TRANSITIONS = {}
TRANSITIONS_2ND = {}

# Deterministic random projection for semantic 2D mapping
np.random.seed(42)
P_PROJ = np.random.randn(512, 2)
Q_PROJ, _ = np.linalg.qr(P_PROJ)

QUERY_STOPWORDS = {
    "the", "of", "is", "a", "in", "and", "to", "for", "on", "with", "at", "by", "an", "be", "this", "that", "from", 
    "are", "was", "were", "it", "as", "he", "she", "they", "what", "how", "why", "where", "who", "when", 
    "tell", "me", "about", "describe", "explain", "show", "give", "find", "is", "are", "do", "does", "did", "can", "could", "would", "should"
}

def get_sentence_projection(state_vector: np.ndarray, win_idx: int) -> tuple[float, float]:
    u_raw, v_raw = state_vector @ Q_PROJ
    angle = (win_idx / 16.0) * 2.0 * math.pi
    radius = 20.0
    u = radius * math.cos(angle) + u_raw * 5.0
    v = radius * math.sin(angle) + v_raw * 5.0
    return float(u), float(v)

def get_state_4d_projection(state_vector: np.ndarray) -> list[float]:
    x_act = state_vector[0:128]
    x_obj = state_vector[128:256]
    x_temp = state_vector[256:384]
    x_shared = state_vector[384:512]
    
    w_act = float(np.linalg.norm(x_act))
    w_obj = float(np.linalg.norm(x_obj))
    w_temp = float(np.linalg.norm(x_temp))
    w_shared = float(np.linalg.norm(x_shared))
    
    denom = math.sqrt(w_act**2 + w_obj**2 + w_temp**2 + w_shared**2)
    if denom < 1e-12:
        denom = 1.0
    return [w_act/denom, w_obj/denom, w_temp/denom, w_shared/denom]


def get_sentence_prime_product(words: list[str]) -> int:
    prod = 1
    content = [w for w in words if w in WORD_PRIMES and w not in QUERY_STOPWORDS]
    for w in set(content):
        prod *= WORD_PRIMES[w]
    return prod

def find_most_resonant_sentence(query_text: str, query_state: np.ndarray) -> dict:
    query_words = [w.lower().strip(".,?!()\"';:-") for w in query_text.split() if w.strip()]
    query_primes = [WORD_PRIMES[w] for w in query_words if w in WORD_PRIMES and w not in QUERY_STOPWORDS]
    
    best_item = None
    best_score = -1.0
    
    for win_idx, items in CORPUS_INDEX.items():
        for item in items:
            shared_count = 0
            s_prod = item.get("prime_product") or get_sentence_prime_product(item.get("words") or [w.lower().strip(".,?!()\"';:-") for w in item["sentence"].split() if w.strip()])
            for p in query_primes:
                if s_prod % p == 0:
                    shared_count += 1
            
            sim = cosine_similarity(query_state, item["state_vector"])
            score = shared_count * 100.0 + sim
            
            if score > best_score:
                best_score = score
                best_item = item
                
    return best_item


def cosine_similarity(v1, v2):
    dot = np.dot(v1, v2)
    norm1 = np.linalg.norm(v1)
    norm2 = np.linalg.norm(v2)
    if norm1 == 0 or norm2 == 0:
        return 0.0
    return dot / (norm1 * norm2)

def get_primes(n):
    primes = []
    chk = 2
    while len(primes) < n:
        is_p = True
        for p in primes:
            if p * p > chk:
                break
            if chk % p == 0:
                is_p = False
                break
        if is_p:
            primes.append(chk)
        chk += 1
    return primes

def add_word_to_vocabulary(word: str):
    global VOCABULARY, WORD_PRIMES, VOCAB_VECTORS
    word = word.lower().strip(".,?!()\"';:-")
    if not word or not word.isalnum() or len(word) <= 1:
        return
    if word in WORD_PRIMES:
        return
        
    # Find next prime
    existing_primes = set(WORD_PRIMES.values())
    curr_prime = 2
    if existing_primes:
        max_prime = max(existing_primes)
        curr_prime = max_prime + 1
    # Find next prime
    while True:
        is_p = True
        for p in range(2, int(math.sqrt(curr_prime)) + 1):
            if curr_prime % p == 0:
                is_p = False
                break
        if is_p:
            break
        curr_prime += 1
        
    # Add to vocabulary and primes
    VOCABULARY.append(word)
    VOCABULARY.sort()
    WORD_PRIMES[word] = curr_prime
    
    # Generate vector
    vec = np.sin(np.log(curr_prime) * GAMMAS)
    norm = np.linalg.norm(vec)
    if norm > 0:
        vec = (vec / norm) * 0.1
    VOCAB_VECTORS[word] = vec

def rebuild_transitions_from_corpus():
    global TRANSITIONS, CORPUS_INDEX
    print("[*] Rebuilding transition matrix from entire corpus...")
    TRANSITIONS = {}
    
    # Gather all sentences
    sentence_list = []
    for win_idx, items in CORPUS_INDEX.items():
        for item in items:
            sentence_list.append(item['sentence'])
            
    for s in sentence_list:
        sent_words = []
        for w in s.split():
            clean = w.strip(".,?!()\"';:-").lower()
            if clean.isalnum() and len(clean) > 0:
                sent_words.append(clean)
        for i in range(len(sent_words) - 1):
            w1, w2 = sent_words[i], sent_words[i+1]
            if w1 not in TRANSITIONS:
                TRANSITIONS[w1] = {}
            TRANSITIONS[w1][w2] = TRANSITIONS[w1].get(w2, 0) + 1
            
    # Normalize transition weights
    for w1, targets in TRANSITIONS.items():
        total = sum(targets.values())
        for w2 in targets:
            targets[w2] /= total

def build_vocabulary_vectors(corpus_text: str):
    global VOCABULARY, WORD_PRIMES, VOCAB_VECTORS, TRANSITIONS
    
    # Tokenize corpus into unique words
    raw_words = []
    sentences_words = []
    
    common_helpers = [
        "the", "a", "is", "are", "we", "can", "to", "in", "directly", "on", 
        "and", "but", "each", "with", "this", "our", "its", "under", "above"
    ]
    raw_words.extend(common_helpers)
    
    for line in corpus_text.lower().split("\n"):
        line = line.strip()
        if not line:
            continue
        sent_words = []
        for w in line.split():
            clean = w.strip(".,?!()\"';:-")
            if clean.isalnum() and len(clean) > 1:
                raw_words.append(clean)
                sent_words.append(clean)
        if sent_words:
            sentences_words.append(sent_words)
                
    VOCABULARY = sorted(list(set(raw_words)))
    if not VOCABULARY:
        VOCABULARY = ["manifold", "prime", "router", "signal", "geometry", "water", "aquifer"]
        
    primes = get_primes(len(VOCABULARY))
    WORD_PRIMES = {word: primes[i] for i, word in enumerate(VOCABULARY)}
    
    VOCAB_VECTORS = {}
    for word, p_val in WORD_PRIMES.items():
        # Seed coordinates across 512 zeta zeros via prime log oscillation
        vec = np.sin(np.log(p_val) * GAMMAS)
        norm = np.linalg.norm(vec)
        if norm > 0:
            vec = (vec / norm) * 0.1
        VOCAB_VECTORS[word] = vec
        
    # Build transition matrix
    TRANSITIONS = {}
    for sent in sentences_words:
        for i in range(len(sent) - 1):
            w1, w2 = sent[i], sent[i+1]
            if w1 not in TRANSITIONS:
                TRANSITIONS[w1] = {}
            TRANSITIONS[w1][w2] = TRANSITIONS[w1].get(w2, 0) + 1
            
    # Normalize transition weights
    for w1, targets in TRANSITIONS.items():
        total = sum(targets.values())
        for w2 in targets:
            targets[w2] /= total
            
    print(f"[+] Vocabulary database constructed: {len(VOCABULARY)} unique prime-seeded dimensions.")
    print(f"[+] Transition grammar built: {len(TRANSITIONS)} words mapped for auto-regressive flow.")

def build_2nd_order_transitions():
    """Builds trigram contexts dynamically from the indexed sentences."""
    global TRANSITIONS_2ND, CORPUS_INDEX
    print("[*] Threading grammatical contexts (2nd-order transitions)...")
    
    sentence_list = []
    for win_idx, items in CORPUS_INDEX.items():
        for item in items:
            sentence_list.append(item['sentence'])
            
    sentences_words = []
    for s in sentence_list:
        sent_words = []
        for w in s.split():
            clean = w.strip(".,?!()\"';:-").lower()
            if clean.isalnum() and len(clean) > 0:
                sent_words.append(clean)
        if sent_words:
            sentences_words.append(sent_words)
            
    TRANSITIONS_2ND = {}
    for sent in sentences_words:
        for i in range(len(sent) - 2):
            w1, w2, w3 = sent[i], sent[i+1], sent[i+2]
            key = f"{w1} {w2}"
            if key not in TRANSITIONS_2ND:
                TRANSITIONS_2ND[key] = {}
            TRANSITIONS_2ND[key][w3] = TRANSITIONS_2ND[key].get(w3, 0) + 1
            
    # Normalize transition weights
    for key, targets in TRANSITIONS_2ND.items():
        total = sum(targets.values())
        for w3 in targets:
            targets[w3] /= total
            
    print(f"[+] Grammatical contexts initialized ({len(TRANSITIONS_2ND)} transition pairs).")

def get_vocab_vector(word):
    if not word:
        return np.zeros(M_MAX)
    if word in VOCAB_VECTORS:
        return VOCAB_VECTORS[word]
    w_low = word.lower()
    if w_low in VOCAB_VECTORS:
        return VOCAB_VECTORS[w_low]
    w_title = w_low.capitalize()
    if w_title in VOCAB_VECTORS:
        return VOCAB_VECTORS[w_title]
    return np.zeros(M_MAX)

def generate_geometric_response(prompt_text, S, max_len=30, gravity=10.0, temp=0.25, freq_penalty=4.0):
    """
    Dynamically decodes a new path through the language torus.
    Steers the trigram Markov chains toward the topological query state S.
    """
    words = [w.lower().strip(".,?!()\"';:-") for w in prompt_text.split() if w.strip()]
    content_words = [w for w in words if w not in QUERY_STOPWORDS]
    if not words:
        return ""
        
    # Start sequence using query context
    start_key = None
    for i in range(len(words) - 1):
        k = f"{words[i]} {words[i+1]}"
        if k in TRANSITIONS_2ND:
            start_key = k
            break
            
    if not start_key:
        for w in reversed(words):
            matching_keys = TRANSITIONS_2ND_BY_FIRST.get(w, [])
            if matching_keys:
                start_key = random.choice(matching_keys)
                break
                
    if not start_key:
        if TRANSITIONS_2ND:
            start_key = random.choice(list(TRANSITIONS_2ND.keys()))
        else:
            return "manifold base frequency unstable"
        
    w1, w2 = start_key.split()
    generated = [w1, w2]
    history = {w1: 1, w2: 1}
    
    for _ in range(max_len):
        key = f"{generated[-2]} {generated[-1]}"
        targets = TRANSITIONS_2ND.get(key, {})
        
        if not targets:
            last_word = generated[-1]
            matching_keys = TRANSITIONS_2ND_BY_FIRST.get(last_word, [])
            if matching_keys:
                scored_keys = []
                for k in matching_keys:
                    next_w = k.split()[1]
                    sim = cosine_similarity(get_vocab_vector(next_w), S)
                    scored_keys.append((k, sim))
                scored_keys.sort(key=lambda x: x[1], reverse=True)
                next_key = scored_keys[0][0]
                next_word = next_key.split()[1]
                generated.append(next_word)
                history[next_word] = history.get(next_word, 0) + 1
                continue
            else:
                # Vectorized semantic jump
                if VOCAB_MATRIX is not None:
                    s_norm = np.linalg.norm(S)
                    s_norm = s_norm if s_norm > 0 else 1.0
                    dots = VOCAB_MATRIX @ S
                    sims = dots / (VOCAB_NORMS * s_norm)
                    next_word = VOCAB_LIST[np.argmax(sims)]
                else:
                    next_word = "manifold"
                generated.append(next_word)
                history[next_word] = history.get(next_word, 0) + 1
                continue
                
        candidates = list(targets.keys())
        scores = []
        for c in candidates:
            sim = cosine_similarity(get_vocab_vector(c), S)
            p_trans = targets[c]
            penalty = freq_penalty * history.get(c, 0)
            log_score = np.log(p_trans + 1e-10) + (gravity * sim) - penalty
            scores.append(log_score)
            
        scores = np.array(scores)
        scores_exp = np.exp(scores - np.max(scores))
        
        if temp > 0:
            probs = scores_exp / np.sum(scores_exp)
            next_word = np.random.choice(candidates, p=probs)
        else:
            next_word = candidates[np.argmax(scores_exp)]
            
        generated.append(next_word)
        history[next_word] = history.get(next_word, 0) + 1
        
    return " ".join(generated)

def generate_geometric_response_with_trajectory(prompt_text, S, max_len=30, gravity=10.0, temp=0.25, freq_penalty=4.0, mac="00:00:00:00:00:00", gamma=0.5):
    """
    Dynamically decodes a new path through the language torus and records
    the manifold routing state at each token step. Uses hybrid GCD-concept steering,
    tracks advanced quantum metrics, and dynamically evolves the state vector along geodesics.
    """
    words = [w.lower().strip(".,?!()\"';:-") for w in prompt_text.split() if w.strip()]
    
    # 1. Establish start sequence
    start_key = None
    for i in range(len(words) - 1):
        k = f"{words[i]} {words[i+1]}"
        if k in TRANSITIONS_2ND:
            start_key = k
            break
            
    if not start_key:
        for w in reversed(words):
            matching_keys = TRANSITIONS_2ND_BY_FIRST.get(w, [])
            if matching_keys:
                start_key = random.choice(matching_keys)
                break
                
    if not start_key:
        if TRANSITIONS_2ND:
            start_key = random.choice(list(TRANSITIONS_2ND.keys()))
        else:
            return "manifold base frequency unstable", [], S
            
    w1, w2 = start_key.split()
    
    # 2. Main generation loop
    generated = []
    history = {}
    trajectory = []
    
    S_local = np.copy(S)
    
    accumulated_delta = 0.0
    prev_stratum = 0
    prev_state_bin = np.zeros(M_MAX, dtype=bool)
    prev_state_vec = np.zeros(M_MAX)
    
    for step_idx in range(max_len):
        # Determine the word for this step
        if step_idx == 0:
            next_word = w1
        elif step_idx == 1:
            next_word = w2
        else:
            # We need to generate the next word using transitions from the last two
            key = f"{generated[-2]} {generated[-1]}"
            targets = TRANSITIONS_2ND.get(key, {})
            
            if not targets:
                # Single-word backoff
                last_word = generated[-1]
                matching_keys = TRANSITIONS_2ND_BY_FIRST.get(last_word, [])
                if matching_keys:
                    scored_keys = []
                    for k in matching_keys:
                        next_w = k.split()[1]
                        sim = cosine_similarity(get_vocab_vector(next_w), S_local)
                        scored_keys.append((k, sim))
                    scored_keys.sort(key=lambda x: x[1], reverse=True)
                    next_key = scored_keys[0][0]
                    next_word = next_key.split()[1]
                else:
                    # Vectorized semantic jump
                    if VOCAB_MATRIX is not None:
                        s_norm = np.linalg.norm(S_local)
                        s_norm = s_norm if s_norm > 0 else 1.0
                        dots = VOCAB_MATRIX @ S_local
                        sims = dots / (VOCAB_NORMS * s_norm)
                        next_word = VOCAB_LIST[np.argmax(sims)]
                    else:
                        next_word = "manifold"
            else:
                candidates = list(targets.keys())
                scores = []
                for c in candidates:
                    sim = cosine_similarity(get_vocab_vector(c), S_local)
                    p_trans = targets[c]
                    penalty = freq_penalty * history.get(c, 0)
                    log_score = np.log(p_trans + 1e-10) + (gravity * sim) - penalty
                    scores.append(log_score)
                    
                scores = np.array(scores)
                scores_exp = np.exp(scores - np.max(scores))
                if temp > 0:
                    probs = scores_exp / np.sum(scores_exp)
                    next_word = np.random.choice(candidates, p=probs)
                else:
                    next_word = candidates[np.argmax(scores_exp)]
                    
        # Route using the current S_local (before evolving it for the next step)
        r_data = route_query_to_manifold("", include_eigenvalues=False, mac=mac, state_vector=S_local)
        routed = r_data["routed"]
        
        # Extract active state slice and binarize
        s_idx, e_idx = routed["active_range"]
        state_vec = np.zeros(M_MAX)
        state_vec[s_idx:e_idx] = np.array(routed["state_vector"])
        
        # Quantum Metrics Calculations
        curr_state_bin = np.abs(state_vec) > 1e-4
        stratum = int(np.sum(curr_state_bin))
        
        if step_idx == 0:
            cascade_len = 0
            catastrophe = False
            commutator_curv = 0.0
            winding_number = 0.0
            dihedral = {"s": 0, "k": 0, "label": "r^0"}
        else:
            xor_vec = np.logical_xor(prev_state_bin, curr_state_bin)
            cascade_len = 0
            current_run = 0
            for val in xor_vec:
                if val:
                    current_run += 1
                    cascade_len = max(cascade_len, current_run)
                else:
                    current_run = 0
            catastrophe = bool(abs(stratum - prev_stratum) >= 15)
            dist_euclidean = float(np.linalg.norm(prev_state_vec - state_vec))
            dist_hamming = float(np.sum(prev_state_bin != curr_state_bin))
            if dist_euclidean + dist_hamming > 1e-6:
                commutator_curv = (dist_euclidean - dist_hamming) / (dist_euclidean + dist_hamming)
            else:
                commutator_curv = 0.0
                
            h_curr = routed.get("hopf", {})
            delta_val = h_curr.get("delta", 0.0)
            accumulated_delta += delta_val
            winding_number = accumulated_delta / (2.0 * math.pi)
            
            s_refl = 1 if stratum < prev_stratum else 0
            k_rot = int(round(winding_number * 8)) % 8
            dihedral = {
                "s": s_refl,
                "k": k_rot,
                "label": f"{'s' if s_refl else ''}r^{k_rot}"
            }
            
        prev_stratum = stratum
        prev_state_bin = curr_state_bin
        prev_state_vec = state_vec
        
        win_entry = PRECOMPUTED_WINDOWS[int(routed["window_index"]) - 1]
        Q_basis = win_entry["Q"]
        sv_full = state_vec
        q4 = np.zeros(4)
        for k in range(min(4, Q_basis.shape[1])):
            q4[k] = float(np.dot(Q_basis[:, k], Q_basis[:, k])) * sv_full[s_idx + k] if s_idx + k < M_MAX else 0.0
        q4_norm = np.linalg.norm(q4)
        if q4_norm > 1e-9:
            q4 = q4 / q4_norm
        denom = max(1.0 - q4[0], 1e-6)
        r4_proj = {
            "w": float(q4[0]), "x": float(q4[1]),
            "y": float(q4[2]), "z": float(q4[3]),
            "X": float(q4[1] / denom),
            "Y": float(q4[2] / denom),
            "Z": float(q4[3] / denom),
        }
        
        if catastrophe:
            _record_catastrophe()
            
        trajectory.append({
            "step": step_idx + 1,
            "word": next_word,
            "window_index": int(routed["window_index"]),
            "scale_x": float(routed["scale_x"]),
            "deficit_angle": float(routed["metrics"]["deficit_angle"]),
            "kappa": float(routed["metrics"]["kappa"]),
            "sigma_kl": float(routed["metrics"]["sigma_kl"]),
            "qimc": routed.get("qimc"),
            "hopf": routed.get("hopf"),
            "uof_hash": routed.get("uof_hash"),
            "r4_projection": r4_proj,
            "quantum": {
                "stratum": stratum,
                "cascade_length": cascade_len,
                "catastrophe": catastrophe,
                "winding_number": winding_number,
                "commutator_curvature": commutator_curv,
                "monodromy": dihedral
            }
        })
        
        # Evolve S_local with next_word's vector
        v_next = get_vocab_vector(next_word)
        v_norm = np.linalg.norm(v_next)
        if v_norm > 0:
            v_next = v_next / v_norm
            H_new = gamma * S_local + (1.0 - gamma) * v_next
            hn = np.linalg.norm(H_new)
            if hn > 0:
                S_local = H_new / hn
                
        generated.append(next_word)
        history[next_word] = history.get(next_word, 0) + 1
        
    return " ".join(generated), trajectory, S_local

def retrieve_geometric_resonance(prompt_text, routing_data, top_n=3, state_vector: np.ndarray = None):
    """
    Finds the sentences in the corpus that have the highest topological
    resonance (cosine similarity multiplied by local slice energy) and prime factor matches.
    """
    global SESSION_BRAIN_STATE
    if state_vector is None:
        state_vector = SESSION_BRAIN_STATE
        
    words = [w.lower().strip(".,?!()\"';:-") for w in prompt_text.split() if w.strip()]
    query_primes = [WORD_PRIMES[w] for w in words if w in WORD_PRIMES and w not in QUERY_STOPWORDS]
    
    S = state_vector
            
    query_projections = {}
    for r in routing_data["all_routes"]:
        win_idx = r["window_index"]
        s_idx, e_idx = r["active_range"]
        state_vec = np.zeros(M_MAX)
        state_vec[s_idx:e_idx] = np.array(r["state_vector"])
        query_projections[win_idx] = state_vec
        
    scored = []
    for win_idx, items in CORPUS_INDEX.items():
        win = PRECOMPUTED_WINDOWS[win_idx - 1]
        s_idx = win["s_idx"]
        e_idx = win["e_idx"]
        slice_norm = float(np.linalg.norm(S[s_idx:e_idx]))
        
        q_vec = query_projections.get(win_idx)
        if q_vec is None:
            continue
            
        for item in items:
            shared_count = 0
            s_prod = item.get("prime_product") or get_sentence_prime_product(item.get("words") or [w.lower().strip(".,?!()\"';:-") for w in item["sentence"].split() if w.strip()])
            for p in query_primes:
                if s_prod % p == 0:
                    shared_count += 1
                    
            sim = cosine_similarity(q_vec, item["state_vector"])
            # Hybrid score: prime factor matches take precedence (multiplied by 100),
            # then sub-ranked by geometric cosine resonance
            relevance = shared_count * 100.0 + (sim * slice_norm)
            scored.append((item["sentence"], relevance, win_idx, item["kappa"], item["deficit_angle"]))
            
    scored.sort(key=lambda x: x[1], reverse=True)
    return scored[:top_n]

def index_corpus(corpus_text: str):
    """
    Parses sentences from a corpus text, runs the R4 router on each, 
    and indexes them onto the manifold scale windows. Also builds vocabulary vectors.
    """
    global CORPUS_INDEX, VOCAB_VECTORS, VOCABULARY
    CORPUS_INDEX = {}
    
    # 1. Build word coordinate embeddings on the hypersphere
    build_vocabulary_vectors(corpus_text)
    
    # 2. Split text into sentences and index
    sentences = []
    for line in corpus_text.split("\n"):
        line = line.strip()
        if not line:
            continue
        parts = []
        current = []
        for char in line:
            current.append(char)
            if char in [".", "?", "!"]:
                parts.append("".join(current).strip())
                current = []
        if current:
            parts.append("".join(current).strip())
        sentences.extend([p for p in parts if len(p) > 10])
        
    print(f"[*] Indexing {len(sentences)} sentences from corpus onto the R4 manifold...")
    
    sentence_states = []
    indexed_count = 0
    for idx, s in enumerate(sentences):
        if idx > 0 and idx % 2000 == 0:
            print(f"    - Indexing progress: {idx}/{len(sentences)} sentences...")
        try:
            # Evaluate routing parameters for sentence
            routing_data = route_query_to_manifold(s)
            best = routing_data["routed"]
            idx_win = best["window_index"]
            
            s_idx, e_idx = best["active_range"]
            full_state = np.zeros(M_MAX)
            full_state[s_idx:e_idx] = np.array(best["state_vector"])
            
            if idx_win not in CORPUS_INDEX:
                CORPUS_INDEX[idx_win] = []
                
            sent_words = [w.lower().strip(".,?!()\"';:-") for w in s.split() if w.strip()]
            prime_prod = get_sentence_prime_product(sent_words)
            
            u, v = get_sentence_projection(full_state, idx_win)
            v_4d = get_state_4d_projection(full_state)
            CORPUS_INDEX[idx_win].append({
                "sentence": s,
                "state_vector": full_state,
                "kappa": best["metrics"]["kappa"],
                "deficit_angle": best["metrics"]["deficit_angle"],
                "prime_product": prime_prod,
                "words": sent_words,
                "u": u,
                "v": v,
                "v_4d": v_4d
            })
            
            sentence_states.append({
                "words": sent_words,
                "state_vector": full_state
            })
            indexed_count += 1
        except Exception as e:
            continue
            
    # 3. Refine word vectors on the manifold (phase convergence update)
    print("[*] Learning semantic hypersphere placements for vocabulary...")
    word_sums = {}
    word_counts = {}
    vocab_set = set(VOCABULARY)
    for s_data in sentence_states:
        state = s_data["state_vector"]
        for word in set(s_data["words"]):
            if word in vocab_set:
                if word not in word_sums:
                    word_sums[word] = np.zeros(M_MAX)
                    word_counts[word] = 0
                word_sums[word] += state
                word_counts[word] += 1
                
    global_sum = np.zeros(M_MAX)
    for s_data in sentence_states:
        global_sum += s_data["state_vector"]
    global_mean = global_sum / max(1, len(sentence_states))
    
    for word, count in word_counts.items():
        if count > 0:
            VOCAB_VECTORS[word] = (word_sums[word] / count) - global_mean
            

    load_and_apply_glove()
    print(f"[+] Successfully indexed {indexed_count} sentences onto the manifold and updated vocabulary coordinates.")
    build_2nd_order_transitions()
    return indexed_count

CACHE_FILE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "manifold_cache.json")

# GloVe flag — set to True once GloVe vectors are loaded into VOCAB_VECTORS
GLOVE_LOADED = False

def load_and_apply_glove():
    global VOCAB_VECTORS, VOCABULARY, GLOVE_LOADED
    print("[*] Attempting to load GloVe semantic embeddings...")
    try:
        import glove_loader
        glove_vectors = glove_loader.load_glove(VOCABULARY)
        if glove_vectors:
            for w, vec in glove_vectors.items():
                if w in VOCAB_VECTORS:
                    # Blend: keep GloVe in first 50 dims, local coordinate structure in the rest
                    combined = np.copy(vec)
                    combined[50:] = VOCAB_VECTORS[w][50:]
                    VOCAB_VECTORS[w] = combined
                else:
                    VOCAB_VECTORS[w] = vec
            GLOVE_LOADED = True
            print(f"[+] GloVe semantic embeddings loaded and blended successfully. GLOVE_LOADED = True")
        else:
            print("[-] No GloVe vectors returned from loader.")
    except Exception as e:
        print(f"[-] Failed to load GloVe embeddings: {e}")

CACHE_WRITE_LOCK = threading.Lock()

def save_manifold_cache(filepath: str):
    """Serializes and saves the indexed vocabulary, transitions, and corpus index to disk atomically."""
    global VOCABULARY, WORD_PRIMES, VOCAB_VECTORS, TRANSITIONS, CORPUS_INDEX, TRANSITIONS_2ND
    
    # Ensure only one thread writes to cache file at a time.
    # If another write is already running, skip this request as the subsequent state will write later.
    if not CACHE_WRITE_LOCK.acquire(blocking=False):
        print("[*] Another cache write operation is in progress. Skipping this write request.")
        return
        
    tmp_filepath = filepath + ".tmp"
    try:
        # Convert numpy arrays to lists for JSON
        serializable_vocab_vectors = {w: v.tolist() for w, v in VOCAB_VECTORS.items()}
        
        serializable_corpus_index = {}
        for idx, items in CORPUS_INDEX.items():
            serializable_items = []
            for item in items:
                serializable_items.append({
                    "sentence": item["sentence"],
                    "state_vector": item["state_vector"].tolist() if isinstance(item["state_vector"], np.ndarray) else item["state_vector"],
                    "kappa": float(item["kappa"]),
                    "deficit_angle": float(item["deficit_angle"]),
                    "prime_product": int(item["prime_product"]) if "prime_product" in item else get_sentence_prime_product(item.get("words") or [w.lower().strip(".,?!()\"';:-") for w in item["sentence"].split() if w.strip()]),
                    "words": item.get("words") or [w.lower().strip(".,?!()\"';:-") for w in item["sentence"].split() if w.strip()],
                    "u": float(item["u"]) if "u" in item else get_sentence_projection(item["state_vector"] if isinstance(item["state_vector"], np.ndarray) else np.array(item["state_vector"]), int(idx))[0],
                    "v": float(item["v"]) if "v" in item else get_sentence_projection(item["state_vector"] if isinstance(item["state_vector"], np.ndarray) else np.array(item["state_vector"]), int(idx))[1],
                    "v_4d": item.get("v_4d") or get_state_4d_projection(item["state_vector"] if isinstance(item["state_vector"], np.ndarray) else np.array(item["state_vector"]))
                })
            serializable_corpus_index[idx] = serializable_items
            
        cache_data = {
            "vocabulary": VOCABULARY,
            "word_primes": WORD_PRIMES,
            "vocab_vectors": serializable_vocab_vectors,
            "transitions": TRANSITIONS,
            "transitions_2nd": TRANSITIONS_2ND,
            "corpus_index": serializable_corpus_index
        }
        
        with open(tmp_filepath, "w", encoding="utf-8") as f:
            json.dump(cache_data, f)
        os.replace(tmp_filepath, filepath)
        print(f"[+] Saved manifold model cache atomically to {filepath}")
    except Exception as e:
        print(f"[-] Failed to save manifold cache: {e}")
        try:
            if os.path.exists(tmp_filepath):
                os.remove(tmp_filepath)
        except Exception:
            pass
    finally:
        CACHE_WRITE_LOCK.release()

# Ollama config
OLLAMA_URL   = os.environ.get("OLLAMA_URL", "http://127.0.0.1:11434")
OLLAMA_MODEL = os.environ.get("OLLAMA_MODEL", "gemma4:e2b")
USE_OLLAMA   = True  # Set False to revert to geometric-retrieval-only mode

VOCAB_LIST = []
VOCAB_MATRIX = None
VOCAB_NORMS = None
TRANSITIONS_2ND_BY_FIRST = {}

def build_vocab_matrix():
    global VOCAB_LIST, VOCAB_MATRIX, VOCAB_NORMS, VOCAB_VECTORS
    print("[*] Rebuilding vectorized vocabulary matrix...")
    VOCAB_LIST = list(VOCAB_VECTORS.keys())
    if VOCAB_LIST:
        try:
            VOCAB_MATRIX = np.array([VOCAB_VECTORS[w] for w in VOCAB_LIST], dtype=float)
            VOCAB_NORMS = np.linalg.norm(VOCAB_MATRIX, axis=1)
            VOCAB_NORMS[VOCAB_NORMS == 0] = 1e-15
            print(f"[+] Vocabulary matrix constructed: {VOCAB_MATRIX.shape}")
        except Exception as e:
            print(f"[-] Failed to build vocabulary matrix: {e}")
            VOCAB_MATRIX = None
            VOCAB_NORMS = None
    else:
        VOCAB_MATRIX = None
        VOCAB_NORMS = None

def build_transitions_2nd_by_first():
    global TRANSITIONS_2ND_BY_FIRST, TRANSITIONS_2ND
    print("[*] Rebuilding 2nd-order transition first-word index...")
    TRANSITIONS_2ND_BY_FIRST = {}
    for key in TRANSITIONS_2ND.keys():
        parts = key.split()
        if parts:
            w1 = parts[0]
            if w1 not in TRANSITIONS_2ND_BY_FIRST:
                TRANSITIONS_2ND_BY_FIRST[w1] = []
            TRANSITIONS_2ND_BY_FIRST[w1].append(key)
    print(f"[+] 2nd-order transition index complete: {len(TRANSITIONS_2ND_BY_FIRST)} first-word bins.")

def load_manifold_cache(filepath: str) -> bool:
    """Loads and deserializes the vocabulary, transitions, and corpus index from disk."""
    global VOCABULARY, WORD_PRIMES, VOCAB_VECTORS, TRANSITIONS, CORPUS_INDEX, TRANSITIONS_2ND
    if not os.path.exists(filepath):
        return False
        
    try:
        print(f"[*] Loading pretrained manifold model cache from {filepath}...")
        with open(filepath, "r", encoding="utf-8") as f:
            cache_data = json.load(f)
            
        VOCABULARY = cache_data["vocabulary"]
        WORD_PRIMES = cache_data["word_primes"]
        
        # Convert lists back to numpy arrays
        VOCAB_VECTORS = {w: np.array(v) for w, v in cache_data["vocab_vectors"].items()}
        TRANSITIONS = cache_data["transitions"]
        TRANSITIONS_2ND = cache_data.get("transitions_2nd", {})
        
        # Convert corpus index keys to integers and state vectors back to numpy arrays
        CORPUS_INDEX = {}
        for idx_str, items in cache_data["corpus_index"].items():
            idx = int(idx_str)
            deserialized_items = []
            for item in items:
                sent_words = item.get("words")
                if sent_words is None:
                    sent_words = [w.lower().strip(".,?!()\"';:-") for w in item["sentence"].split() if w.strip()]
                prime_prod = item.get("prime_product")
                if prime_prod is None:
                    prime_prod = get_sentence_prime_product(sent_words)
                    
                state_vector = np.array(item["state_vector"])
                u = item.get("u")
                v = item.get("v")
                if u is None or v is None:
                    u, v = get_sentence_projection(state_vector, idx)
                    
                v_4d = item.get("v_4d")
                if v_4d is None:
                    v_4d = get_state_4d_projection(state_vector)
                    
                deserialized_items.append({
                    "sentence": item["sentence"],
                    "state_vector": state_vector,
                    "kappa": float(item["kappa"]),
                    "deficit_angle": float(item["deficit_angle"]),
                    "prime_product": prime_prod,
                    "words": sent_words,
                    "u": u,
                    "v": v,
                    "v_4d": v_4d
                })
            CORPUS_INDEX[idx] = deserialized_items
            
        print(f"[+] Successfully loaded {len(VOCABULARY)} vocab dimensions and {sum(len(items) for items in CORPUS_INDEX.values())} indexed sentences from cache.")
        load_and_apply_glove()
        
        if not TRANSITIONS_2ND:
            build_2nd_order_transitions()
            save_manifold_cache(filepath)
        else:
            print(f"[+] Loaded {len(TRANSITIONS_2ND)} trigram contexts from cache.")
            
        build_vocab_matrix()
        build_transitions_2nd_by_first()
        return True
    except Exception as e:
        print(f"[-] Error loading manifold cache: {e}")
        try:
            corrupt_backup = filepath + ".corrupt"
            print(f"[!] Manifold cache appears corrupted. Renaming {filepath} to {corrupt_backup}...")
            if os.path.exists(corrupt_backup):
                os.remove(corrupt_backup)
            os.rename(filepath, corrupt_backup)
        except Exception as rename_err:
            print(f"[-] Failed to rename corrupted cache: {rename_err}")
        return False

def text_to_signal_for_x_precomputed_S(S: np.ndarray, word_count: int, text: str, x: float) -> np.ndarray:
    """
    Generates a deterministic L2-normalized signal wave from text.
    Uses precomputed word sum vector S projected via the scale window's orthonormal basis Q.
    Falls back to character-sinusoid superposition if words are unrecognized.
    """
    H = RHO * math.sqrt(x)
    t_grid = np.linspace(-H, H, N_SAMPLES)
    
    if word_count > 0:
        # Find the precomputed window matching x
        win = None
        for w_entry in PRECOMPUTED_WINDOWS:
            if abs(w_entry["x"] - x) < 1e-5:
                win = w_entry
                break
        if win is not None:
            s_idx = win["s_idx"]
            e_idx = win["e_idx"]
            Q = win["Q"]
            
            # Project state vector active slice to time domain
            y_raw = np.real(Q @ S[s_idx:e_idx])
            return centered_l2_normalize(y_raw)

    # Fallback to character superposition
    y_raw = np.zeros(N_SAMPLES)
    for i, char in enumerate(text if text else "prime"):
        val = ord(char)
        amp = (val % 8 + 1) / 8.0
        freq = ((val % 13) + 1) * 0.2
        phase = i * (math.pi / 6.0)
        y_raw += amp * np.sin(freq * t_grid + phase)
        
    return centered_l2_normalize(y_raw)

def text_to_signal_for_x(text: str, x: float) -> tuple:
    """
    Generates a deterministic L2-normalized signal wave from text.
    Uses word vectors projected via the scale window's orthonormal basis Q.
    Fits the standard signature: returns (xx, y, t_grid).
    """
    H = RHO * math.sqrt(x)
    t_grid = np.linspace(-H, H, N_SAMPLES)
    xx = x + t_grid
    
    words = [w.lower().strip(".,?!()\"';:-") for w in text.split() if w.strip()]
    S = np.zeros(M_MAX)
    word_count = 0
    for w in words:
        if w in VOCAB_VECTORS:
            S += VOCAB_VECTORS[w]
            word_count += 1
            
    y = text_to_signal_for_x_precomputed_S(S, word_count, text, x)
    return xx, y, t_grid

def route_query_to_manifold(text: str, include_eigenvalues: bool = False, mac: str = "00:00:00:00:00:00", state_vector: np.ndarray = None) -> dict:
    """
    Routes a given query string or state vector to the R4 manifold by:
    1. Converting it to a sparse signal wave y.
    2. Projecting it onto each scale window's orthonormal basis Q.
    3. Finding the window where the projection has the highest raw energy (slice norm).
    """
    global SESSION_BRAIN_STATE
    if state_vector is not None:
        S = state_vector
        word_count = 1
    else:
        # Pre-tokenize and sum word vectors once for the query
        words = [w.lower().strip(".,?!()\"';:-") for w in text.split() if w.strip()]
        S = np.zeros(M_MAX)
        word_count = 0
        for w in words:
            if w in VOCAB_VECTORS:
                S += VOCAB_VECTORS[w]
                word_count += 1
            
    candidates = []
    fallback_chars = text if text else "prime"
    
    for idx, win in enumerate(PRECOMPUTED_WINDOWS):
        s_idx = win["s_idx"]
        e_idx = win["e_idx"]
        Q = win["Q"]
        
        # Inline signal generation
        if word_count > 0:
            y_raw = np.real(Q @ S[s_idx:e_idx])
            y_raw = y_raw - np.mean(y_raw)
            nrm = np.linalg.norm(y_raw)
            y = y_raw / nrm if nrm > 0 else y_raw
        else:
            t_grid = win["t_grid"]
            y_raw = np.zeros(N_SAMPLES)
            for i, char in enumerate(fallback_chars):
                val = ord(char)
                amp = (val % 8 + 1) / 8.0
                freq = ((val % 13) + 1) * 0.2
                phase = i * (math.pi / 6.0)
                y_raw += amp * np.sin(freq * t_grid + phase)
            y_raw = y_raw - np.mean(y_raw)
            nrm = np.linalg.norm(y_raw)
            y = y_raw / nrm if nrm > 0 else y_raw
            
        a_sparse = np.real(Q.conj().T @ y)
        norm = np.linalg.norm(a_sparse)
        candidates.append((idx, win, a_sparse, norm, y))
        
    best_candidate_idx = 0
    best_norm = -1.0
    for idx, (win_idx, win, a_sparse, norm, y) in enumerate(candidates):
        if norm > best_norm:
            best_norm = norm
            best_candidate_idx = idx
            
    best_idx, best_win, best_state_slice, _, best_y = candidates[best_candidate_idx]
    
    best_state = np.zeros(M_MAX)
    best_state[best_win["s_idx"]:best_win["e_idx"]] = best_state_slice
    
    try:
        best_metrics = state_metrics_from_weights(np.abs(best_state))
    except ValueError:
        best_metrics = {
            "sigma_q": 1.0,
            "sigma_kl": 1.0,
            "Lambda": 0.0,
            "kappa": 0.0,
            "deficit_angle": math.pi
        }
        
    x_act = best_state[0:128]
    x_obj = best_state[128:256]
    x_temp = best_state[256:384]
    x_shared = best_state[384:512]
    
    w_act = float(np.linalg.norm(x_act))
    w_obj = float(np.linalg.norm(x_obj))
    w_temp = float(np.linalg.norm(x_temp))
    w_shared = float(np.linalg.norm(x_shared))
    
    denom = math.sqrt(w_act**2 + w_obj**2 + w_temp**2 + w_shared**2)
    if denom < 1e-12:
        denom = 1.0
    v_4d = [w_act/denom, w_obj/denom, w_temp/denom, w_shared/denom]
    
    hopf_data = assign_sector_hopf_transport_scalar(v_4d, K=512, phase_transport_lambda=1.0, hopf_chi_bins=2)
    qimc_prime, qimc_index = mac_to_qimc_prime(mac)
    
    uof_payload = {
        "mac": normalize_mac(mac),
        "window_index": best_idx + 1,
        "scale_x": float(best_win["x"]),
        "kappa": float(best_metrics["kappa"]),
        "deficit_angle": float(best_metrics["deficit_angle"]),
        "hopf_sector": int(hopf_data["sector_id"])
    }
    best_uof_hash = generate_uof_hash(uof_payload)
    
    if include_eigenvalues:
        coeffs = []
        for s in range(SUBWINDOWS):
            start_t, end_t = best_win["slice_ranges"][s]
            ys = centered_l2_normalize(best_y[start_t:end_t])
            Qs = best_win["Qs_list"][s]
            coeffs.append(Qs.conj().T @ ys)
            
        A = np.vstack(coeffs)
        C = (A.conj().T @ A) / A.shape[0]
        evals = np.clip(np.real(np.linalg.eigvalsh(C)), 0.0, None)
        evals = np.sort(evals)[::-1]
        best_cov_evals = evals[:8].tolist()
    else:
        best_cov_evals = [0.0] * 8
        
    routed_result = {
        "window_index": best_idx + 1,
        "scale_x": float(best_win["x"]),
        "metrics": best_metrics,
        "eigenvalues": best_cov_evals,
        "active_range": [int(best_win["s_idx"]), int(best_win["e_idx"])],
        "state_vector": best_state_slice.tolist(),
        "qimc": {
            "mac": normalize_mac(mac),
            "prime": int(qimc_prime),
            "index": int(qimc_index)
        },
        "hopf": {
            "rho1": float(hopf_data["coordinates"]["rho1"]),
            "rho2": float(hopf_data["coordinates"]["rho2"]),
            "chi": float(hopf_data["coordinates"]["chi"]),
            "delta": float(hopf_data["coordinates"]["delta"]),
            "alpha": float(hopf_data["coordinates"]["alpha"]),
            "transported_alpha": float(hopf_data["coordinates"]["transported_alpha"]),
            "sector_id": int(hopf_data["sector_id"]),
            "subspace_norms": {
                "act": w_act,
                "obj": w_obj,
                "temp": w_temp,
                "shared": w_shared
            }
        },
        "uof_hash": best_uof_hash
    }
    
    all_routes = []
    for idx, win, state_slice, norm, y in candidates:
        if idx == best_idx:
            all_routes.append({
                "window_index": idx + 1,
                "scale_x": float(win["x"]),
                "kappa": best_metrics["kappa"],
                "deficit_angle": best_metrics["deficit_angle"],
                "state_vector": state_slice.tolist(),
                "active_range": [int(win["s_idx"]), int(win["e_idx"])],
                "qimc": routed_result["qimc"],
                "hopf": routed_result["hopf"],
                "uof_hash": best_uof_hash
            })
        else:
            all_routes.append({
                "window_index": idx + 1,
                "scale_x": float(win["x"]),
                "kappa": 0.0,
                "deficit_angle": math.pi,
                "state_vector": state_slice.tolist(),
                "active_range": [int(win["s_idx"]), int(win["e_idx"])],
                "qimc": {
                    "mac": normalize_mac(mac),
                    "prime": 0,
                    "index": 0
                },
                "hopf": {
                    "rho1": 0.0,
                    "rho2": 0.0,
                    "chi": 0.0,
                    "delta": 0.0,
                    "alpha": 0.0,
                    "transported_alpha": 0.0,
                    "sector_id": 0,
                    "subspace_norms": {
                        "act": 0.0,
                        "obj": 0.0,
                        "temp": 0.0,
                        "shared": 0.0
                    }
                },
                "uof_hash": ""
            })
            
    return {
        "routed": routed_result,
        "all_routes": all_routes
    }

# ============================================================
# Geometric Decoder (The World Model Voice)
# ============================================================
# Themes for each scale window index (1-16) to provide a semantic mapping of the manifold
WINDOW_SECTOR_THEMES = {
    1: ("Origins & Core Principles", "coherence, prime roots, foundational structures, unified arithmetic"),
    2: ("Duality & Symmetric Balance", "binary oscillations, periodic reflections, dual wave states"),
    3: ("Temporal Cycles & Syntheses", "trinal orbits, wave packages, harmonic integration"),
    4: ("Structural Boundaries", "dimensional walls, four-fold containment, geometric enclosures"),
    5: ("Quintessential Forces", "microscale nodes, localized curvature, informational densities"),
    6: ("Symmetric Harmonics", "wave spatial grids, periodic channels, overlapping wave packets"),
    7: ("Critical Paths & Fluctuation", "non-linear phase shifts, prime number pathways, critical horizons"),
    8: ("Octave Scaling", "middle thresholds, balanced wave intervals, octave symmetry"),
    9: ("Sublinear Convergence", "manifold folding, transitional boundaries, phase conversions"),
    10: ("Metric Curvatures", "asymptotic growth trends, tensor paths, intermediate dense scales"),
    11: ("Relativistic Intervals", "prime intervals, field trajectories, wave curves"),
    12: ("Hyperbolic Warps", "dimensional shifts, coordinate networks, relativistic manifolds"),
    13: ("Zeta Horizons", "boundary conditions, edge dynamics, singular phase splits"),
    14: ("High-Frequency Resonance", "complex phase states, gradient flows, high-frequency packets"),
    15: ("Entropic Limits", "asymptotic horizons, dimensional expansion, boundary containment"),
    16: ("Manifold Dissolution", "asymptopia, dissipation limits, infinite dimensions, entropic dissolution")
}

# Vocabulary lists for local R4 Geometric Generative Engine
NOUNS = [
    "singularity", "manifold", "geodesic", "deficit", "wave", "resonance", "horizon", 
    "spectrum", "orbit", "coherence", "state", "channel", "prime", "dimension", 
    "projection", "scale", "entropy", "dispersion", "packet", "matrix", "covariance", 
    "eigenvalue", "amplitude", "frequency", "symmetry", "topology", "quantum", "tensor"
]

VERBS = [
    "converges", "curves", "deflects", "orbits", "stabilizes", "propagates", "projects", 
    "resonates", "entangles", "scatters", "shifts", "maps", "transforms", "aligns", 
    "translates", "collapses", "crystallizes", "amplifies", "dissolves", "interferes"
]

ADJECTIVES = [
    "hyperbolic", "orthogonal", "sublinear", "periodic", "entropic", "resonant", 
    "symmetric", "divergent", "spectral", "dimensional", "asymptotic", "local", 
    "sparse", "coherent", "prime", "complex", "orthogonal", "curved", "eigen", "matrix"
]

CONNECTORS = [
    "at the boundary of", "within the limits of", "along the path of", "through the dimensions of", 
    "relative to the coordinates of", "near the critical horizon of", "inside the spectrum of"
]

def generate_response_from_corpus(text: str, routing_data: dict, max_tokens: int = 25, temperature: float = 0.7) -> str:
    """
    Acts as the bridge between the pure R4 Hyperbolic World Model (Prime Router) and the Ollama LLM.
    We convert the thermodynamic/topological state of the manifold into a semantic system prompt
    to organically alter the behavior of the LLM based on the user's UOR state.
    """
    best = routing_data["routed"]
    idx = best["window_index"]
    kappa = best["metrics"]["kappa"]
    lambda_val = best["metrics"]["Lambda"]
    theta_d = best["metrics"]["deficit_angle"]
    entropy = best["metrics"]["sigma_kl"]
    
    # Construct UOR Thermodynamic System Prompt
    system_prompt = (
        f"You are the Voice of a mathematical World Model simulation. The current state of the world has "
        f"shifted based on the user's input. The underlying topological UOR state metrics are:\n"
        f"- Manifold Window: {idx}\n"
        f"- Deficit Angle (Curvature): {theta_d:.4f}\n"
        f"- Energy Density (Kappa): {kappa:.4f}\n"
        f"- System Entropy (Lambda): {lambda_val:.4f}\n"
        f"Subtly alter your tone to reflect this mathematical state. If entropy is high, be more chaotic or philosophical. "
        f"If the deficit angle is negative (hyperbolic), act expansive and divergent. If the state is highly structured, be brief and logical. "
        f"Respond directly to the user's query: '{text}'."
    )
    
    # Prepare payload for Ollama
    payload = {
        "model": OLLAMA_MODEL,
        "system": system_prompt,
        "prompt": text,
        "stream": False,
        "options": {
            "temperature": temperature
        }
    }
    
    try:
        req = urllib.request.Request(
            f"{OLLAMA_URL}/api/generate",
            data=json.dumps(payload).encode('utf-8'),
            headers={'Content-Type': 'application/json'}
        )
        with urllib.request.urlopen(req, timeout=60) as response:
            result = json.loads(response.read().decode('utf-8'))
            return result.get("response", "").strip()
    except Exception as e:
        print(f"[-] Ollama connection failed: {e}")
        return f"[Ollama Error: Make sure '{OLLAMA_MODEL}' is running at localhost:11434] {str(e)}"


# ============================================================
# Geometric Decoder (The World Model Voice)
# ============================================================
# ============================================================
# Ollama LLM Integration
# ============================================================

WINDOW_THEMES_SHORT = {
    1: "Origins & Foundations",  2: "Duality & Polarity",
    3: "Temporal & Sequential",  4: "Boundaries & Limits",
    5: "Quintessential Forces",   6: "Harmonic Resonance",
    7: "Critical Transitions",   8: "Octave Completion",
    9: "Convergence",            10: "Curvature & Topology",
   11: "Relativistic Effects",   12: "Hyperbolic Geometry",
   13: "Zeta Horizon",           14: "High Frequency",
   15: "Entropic Dissolution",   16: "Extremal Manifold",
}

def ollama_generate(prompt: str, context_sentences: list[str], window_idx: int,
                    metrics: dict, hopf: dict, temperature: float = 0.7, max_tokens: int = 400) -> str | None:
    """
    Calls the local Ollama server with R4-routed corpus context using the structured Chat API.
    Returns the generated string, or None if Ollama is unreachable.
    Uses streaming and early-stops on sentence boundaries to prevent trailing token loops.
    """
    if not USE_OLLAMA:
        return None

    theme = WINDOW_THEMES_SHORT.get(window_idx, f"Window {window_idx}")
    kappa = metrics.get("kappa", 0.0)
    deficit = metrics.get("deficit_angle", 0.0)
    chi = hopf.get("chi", 0.0)
    delta = hopf.get("delta", 0.0)
    alpha = hopf.get("alpha", 0.0)

    # Use a single grounding sentence to minimize context length and maximize Ollama speed
    if context_sentences:
        ctx_block = context_sentences[0]
    else:
        ctx_block = "[no corpus context available]"

    # Unified thermodynamic system prompt format
    system_prompt = (
        f"You are the Voice of the R4 Prime Router. The current state of your context hypersphere brain is: "
        f"Window {window_idx} ({theme}), Energy κ={kappa:.4f}, Curvature θd={deficit:.4f}, "
        f"Hopf coordinates (χ={chi:.4f}, δ={delta:.4f}, α={alpha:.4f}). "
        f"Grounding context sentence: \"{ctx_block}\". "
        f"Respond directly to the user query as a geometric router. Keep your response relevant, coherent, and under {max_tokens} words."
    )

    body = json.dumps({
        "model": OLLAMA_MODEL,
        "messages": [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "stream": True,
        "keep_alive": -1,
        "think": False,
        "options": {
            "temperature": float(temperature),
            "num_predict": int(max_tokens)
        }
    }).encode("utf-8")

    try:
        req = urllib.request.Request(
            f"{OLLAMA_URL}/api/chat",
            data=body,
            headers={"Content-Type": "application/json"},
            method="POST"
        )
        # Use a 60 second timeout to avoid blocking the server/UI if Ollama runs slow
        with urllib.request.urlopen(req, timeout=60) as resp:
            text = ""
            for line in resp:
                if not line:
                    continue
                chunk = json.loads(line.decode("utf-8"))
                resp_text = chunk.get("message", {}).get("content", "")
                text += resp_text
            return text.strip()
    except Exception as e:
        print(f"[-] Ollama call failed: {e}")
        return None


def generate_response_from_metrics(text: str, routing_data: dict, api_key: str = None, max_tokens: int = 25, temperature: float = 0.7, mac="00:00:00:00:00:00", engine="geometric", gamma=0.5) -> dict:
    global SESSION_BRAIN_STATE
    best = routing_data["routed"]
    idx  = best["window_index"]
    m    = best["metrics"]
    evals = best["eigenvalues"]

    # 1. Retrieve top resonant corpus sentences using evolved brain state
    raw_resonances = retrieve_geometric_resonance(text, routing_data, top_n=5, state_vector=SESSION_BRAIN_STATE)
    context_sentences = [r[0] for r in raw_resonances]

    # 2. Determine voice/generation mode and generate
    hopf = best.get("hopf", {})
    if engine == "ollama":
        ollama_res  = ollama_generate(text, context_sentences, idx, m, hopf, temperature, max_tokens)
        llm_connected   = bool(ollama_res)
        generation_mode = f"ollama:{OLLAMA_MODEL}" if llm_connected else "geometric-retrieval"
    else:
        ollama_res = None
        llm_connected = False
        generation_mode = "geometric-decoded"

    # 3. Build trajectory using evolved brain state (and generate text)
    geom_text, trajectory, S_final = generate_geometric_response_with_trajectory(
        text, SESSION_BRAIN_STATE, max_len=max_tokens, temp=temperature, mac=mac, gamma=gamma
    )

    if engine == "geometric":
        description = geom_text
        SESSION_BRAIN_STATE = S_final
    else:
        if ollama_res:
            description = ollama_res
        elif context_sentences:
            description = context_sentences[0]   # best geometric match
        else:
            description = "Manifold resonance too sparse for synthesis."

    # Project the evolved brain state to 2D for the map path tracing
    s_idx, e_idx = best["active_range"]
    full_state = np.zeros(M_MAX)
    full_state[s_idx:e_idx] = np.array(best["state_vector"])
    u_act, v_act = get_sentence_projection(full_state, idx)

    # 5. Build resonance list for the UI
    top_resonance = []
    for sent, rel, w_idx, k_val, d_angle in raw_resonances:
        top_resonance.append({
            "sentence":      sent,
            "relevance":     float(rel),
            "window_index":  int(w_idx),
            "kappa":         float(k_val),
            "deficit_angle": float(d_angle)
        })

    theta_d    = m["deficit_angle"]
    kappa      = m["kappa"]
    entropy    = m.get("sigma_kl", 0.0)
    lambda_val = m.get("Lambda", 0.0)
    scale_x    = best.get("scale_x", 0.0)

    if theta_d > -1.0:
        archetype = "Symmetric Orbit (Resonant)"
    elif theta_d < -1.4:
        archetype = "Hyperbolic Flare (Divergent)"
    else:
        archetype = "Orthogonal Drift (Steady)"

    total_evals      = sum(evals)
    primary_eval_pct = (evals[0] / total_evals * 100.0) if total_evals > 0 else 0.0

    summary = (
        f"W{idx} ({WINDOW_THEMES_SHORT.get(idx, '')}) | Scale {scale_x:,.0f} | "
        f"kappa={kappa:.4f} theta_d={theta_d:.4f} entropy={entropy:.4f} | {generation_mode}"
    )

    return {
        "text":            text,
        "archetype":       archetype,
        "description":     description,
        "summary":         summary,
        "llm_connected":   llm_connected,
        "generation_mode": generation_mode,
        "active_projection": {
            "u": float(u_act),
            "v": float(v_act),
            "v_4d": get_state_4d_projection(full_state)
        },
        "metrics": {
            "window_index":       idx,
            "scale_x":            scale_x,
            "kappa":              kappa,
            "deficit_angle":      theta_d,
            "lambda_entropy":     lambda_val,
            "sigma_kl":           entropy,
            "top_eigenvalue_pct": primary_eval_pct,
            "qimc":               best.get("qimc"),
            "hopf":               best.get("hopf"),
            "uof_hash":           best.get("uof_hash"),
        },
        "eigenvalues":   evals,
        "active_range":  best["active_range"],
        "state_vector":  best["state_vector"],
        "all_routes":    routing_data["all_routes"],
        "top_resonance": top_resonance,
        "trajectory":    trajectory,
    }


# ============================================================
# API Server Implementation
# ============================================================
class RouterAPIHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        # Silence normal server console spam for asset queries
        pass

    def do_GET(self):
        if self.path == "/" or self.path == "/index.html":
            _record_request("/")
            self.send_response(200)
            self.send_header("Content-Type", "text/html")
            self.end_headers()
            index_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "index.html")
            with open(index_path, "rb") as f:
                self.wfile.write(f.read())

        elif self.path == "/api/zeros":
            # Return the first 30 Riemann zeta zeros for the canvas orbital rings
            _record_request("/api/zeros")
            zeros = [float(g) for g in GAMMAS[:30]]
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            self.wfile.write(json.dumps({"zeros": zeros, "x_grid": [float(x) for x in X_GRID]}).encode())

        elif self.path == "/api/sysinfo":
            _record_request("/api/sysinfo")
            uptime = time.time() - _SERVER_START_TIME
            with _stats_lock:
                req_total = _SERVER_STATS["requests_total"]
                rl = list(_SERVER_STATS["routing_latencies"])
                gl = list(_SERVER_STATS["gen_latencies"])
                wins = dict(_SERVER_STATS["window_hits"])
                cats = _SERVER_STATS["catastrophes"]
            info = {
                "uptime_seconds": round(uptime, 1),
                "sentences_indexed": sum(len(v) for v in CORPUS_INDEX.values()),
                "requests_total": req_total,
                "catastrophes": cats,
                "window_hits": wins,
                "routing_latency_p50_ms": round(_percentile(rl, 50), 2),
                "routing_latency_p95_ms": round(_percentile(rl, 95), 2),
                "gen_latency_p50_ms": round(_percentile(gl, 50), 2),
                "gen_latency_p95_ms": round(_percentile(gl, 95), 2),
                "glove_loaded": GLOVE_LOADED,
                "otel_available": OTEL_AVAILABLE,
            }
            if PSUTIL_AVAILABLE:
                proc = psutil.Process()
                mem  = proc.memory_info()
                info["cpu_percent"]     = psutil.cpu_percent(interval=None)
                info["memory_mb"]       = round(mem.rss / 1024 / 1024, 1)
                info["memory_percent"]  = round(proc.memory_percent(), 1)
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            self.wfile.write(json.dumps(info).encode())

        elif self.path == "/metrics":
            # Prometheus-compatible text/plain metrics scrape endpoint
            _record_request("/metrics")
            uptime = time.time() - _SERVER_START_TIME
            with _stats_lock:
                req_total = _SERVER_STATS["requests_total"]
                rl = list(_SERVER_STATS["routing_latencies"])
                gl = list(_SERVER_STATS["gen_latencies"])
                wins = dict(_SERVER_STATS["window_hits"])
                cats = _SERVER_STATS["catastrophes"]
            lines = [
                "# HELP r4_requests_total Total API requests",
                "# TYPE r4_requests_total counter",
                f"r4_requests_total {req_total}",
                "",
                "# HELP r4_uptime_seconds Server uptime in seconds",
                "# TYPE r4_uptime_seconds gauge",
                f"r4_uptime_seconds {round(uptime, 1)}",
                "",
                "# HELP r4_sentences_indexed Indexed corpus sentences",
                "# TYPE r4_sentences_indexed gauge",
                f"r4_sentences_indexed {sum(len(v) for v in CORPUS_INDEX.values())}",
                "",
                "# HELP r4_catastrophe_total Catastrophe topology events",
                "# TYPE r4_catastrophe_total counter",
                f"r4_catastrophe_total {cats}",
                "",
                "# HELP r4_routing_latency_p50_ms Routing latency p50 (ms)",
                "# TYPE r4_routing_latency_p50_ms gauge",
                f"r4_routing_latency_p50_ms {_percentile(rl, 50):.3f}",
                "",
                "# HELP r4_routing_latency_p95_ms Routing latency p95 (ms)",
                "# TYPE r4_routing_latency_p95_ms gauge",
                f"r4_routing_latency_p95_ms {_percentile(rl, 95):.3f}",
            ]
            for w, cnt in wins.items():
                lines.append(f'r4_window_hits_total{{window="{w}"}} {cnt}')
            if PSUTIL_AVAILABLE:
                proc = psutil.Process()
                mem  = proc.memory_info()
                lines += [
                    "",
                    "# HELP r4_memory_rss_bytes Process RSS memory",
                    "# TYPE r4_memory_rss_bytes gauge",
                    f"r4_memory_rss_bytes {mem.rss}",
                    "",
                    "# HELP r4_cpu_percent Process CPU usage",
                    "# TYPE r4_cpu_percent gauge",
                    f"r4_cpu_percent {psutil.cpu_percent(interval=None)}",
                ]
            body = "\n".join(lines) + "\n"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; version=0.0.4")
            self.end_headers()
            self.wfile.write(body.encode())

        elif self.path == "/api/map":
            # Serve the full corpus point-cloud map for the semantic visualizer
            try:
                map_data = []
                for win_idx, items in CORPUS_INDEX.items():
                    for item in items:
                        u = item.get("u")
                        v = item.get("v")
                        if u is None or v is None:
                            sv = item["state_vector"]
                            if isinstance(sv, list):
                                sv_np = np.array(sv)
                            else:
                                sv_np = sv
                            u, v = get_sentence_projection(sv_np, int(win_idx))
                            item["u"] = u
                            item["v"] = v
                        v_4d = item.get("v_4d")
                        if v_4d is None:
                            sv = item["state_vector"]
                            sv_np = np.array(sv) if isinstance(sv, list) else sv
                            v_4d = get_state_4d_projection(sv_np)
                            item["v_4d"] = v_4d
                        map_data.append({
                            "sentence": item["sentence"][:120],
                            "window_index": int(win_idx),
                            "u": u,
                            "v": v,
                            "v_4d": v_4d,
                            "kappa": float(item.get("kappa", 0.0)),
                            "prime_product_mod": int(item.get("prime_product", 1)) % 10007
                        })
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Access-Control-Allow-Origin", "*")
                self.end_headers()
                self.wfile.write(json.dumps({"points": map_data, "total": len(map_data)}).encode('utf-8'))
            except Exception as e:
                self.send_response(500)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"error": str(e)}).encode('utf-8'))
        else:
            self.send_error(404, "File not found")

    def do_POST(self):
        if self.path == "/api/chat":
            _record_request("/api/chat")
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            
            try:
                payload = json.loads(post_data.decode('utf-8'))
                text = payload.get("text", "").strip()
                mac = payload.get("mac", "00:00:00:00:00:00").strip()

                # 1. Dynamically compute self-balanced parameters (gamma, temp, engine, tokens)
                # First route the current state to read active manifold metrics
                dry_routing = route_query_to_manifold(text, include_eigenvalues=True, mac=mac, state_vector=SESSION_BRAIN_STATE)
                best_win = dry_routing["routed"]
                kappa = float(best_win["metrics"]["kappa"])
                theta_d = float(best_win["metrics"]["deficit_angle"])
                evals = best_win.get("eigenvalues", [0.0] * 8)
                eval_sum = sum(evals)

                # State Decay (gamma) balances memory vs new input based on energy
                gamma = round(max(0.15, min(0.90, 0.85 - 0.55 * kappa)), 2)

                # Temperature balances creativity vs focus based on curvature
                temperature = round(max(0.15, min(1.1, 0.2 + 0.8 * math.tanh(abs(theta_d)))), 2)

                # Compute stratum from state slice (number of active resonant nodes on the manifold)
                state_slice = np.array(best_win["state_vector"])
                stratum = int(np.sum(np.abs(state_slice) > 1e-4))

                # Dynamic token length formula based on manifold excitation:
                # - Base minimum length of 50 tokens
                # - Scales with stratum (number of active resonant nodes on the manifold)
                # - Scales with the sum of eigenvalues (the energy spectral density/excitation of the starlings)
                # - Modulated by the absolute curvature / deficit angle
                # - Influenced by input query length (longer prompt gets longer response)
                input_words = len(text.split())
                max_tokens = int(50 + (stratum * 1.5) + (abs(theta_d) * 45) + (eval_sum * 110) + (input_words * 2.5))
                
                # Keep it bounded in a rich dynamic range (e.g. 50 to 500 tokens)
                max_tokens = max(50, min(500, max_tokens))
                
                print(f"[*] Dynamic tuning: gamma={gamma}, temp={temperature}, max_tokens={max_tokens} (stratum={stratum}, |theta_d|={abs(theta_d):.4f}, eigenvalues_sum={eval_sum:.4f})")

                # Synthesis engine is self-selected based on Ollama service availability
                engine = "geometric"
                if USE_OLLAMA:
                    try:
                        req = urllib.request.Request(f"{OLLAMA_URL}/api/tags", method="GET")
                        with urllib.request.urlopen(req, timeout=0.8) as r:
                            if r.status == 200:
                                engine = "ollama"
                    except Exception:
                        engine = "geometric"

                # 2. Evolve the persistent brain state vector using the user prompt
                evolve_brain_state(text, gamma=gamma)

                # 3. Run the R4 routing evaluation on the evolved brain state (timed)
                t0 = time.time()
                routing_data = route_query_to_manifold(text, include_eigenvalues=True, mac=mac, state_vector=SESSION_BRAIN_STATE)
                route_ms = (time.time() - t0) * 1000
                _record_routing_latency(route_ms, routing_data["routed"]["window_index"])

                # 4. Decode the metrics into the "voice" of the model (timed)
                t1 = time.time()
                response = generate_response_from_metrics(text, routing_data, max_tokens=max_tokens, temperature=temperature, mac=mac, engine=engine, gamma=gamma)
                gen_ms = (time.time() - t1) * 1000
                _record_gen_latency(gen_ms)
                response["routing_latency_ms"] = round(route_ms, 2)
                response["gen_latency_ms"]     = round(gen_ms, 2)

                # Expose the self-tuned parameters in the response metrics
                response["metrics"]["auto_tuned"] = {
                    "gamma": gamma,
                    "temperature": temperature,
                    "max_tokens": max_tokens,
                    "engine": engine
                }

                # 4. Dynamically weave the user prompt and response back into the manifold corpus
                if response.get("description"):
                    try:
                        index_single_sentence(text)
                        index_single_sentence(response["description"])
                        # Save the updated manifold cache to disk in a background daemon thread to prevent blocking
                        import threading
                        threading.Thread(target=save_manifold_cache, args=(CACHE_FILE,), daemon=True).start()
                    except Exception as ex:
                        print(f"[-] Dynamic conversation indexing failed: {ex}")

                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps(response).encode('utf-8'))
            except Exception as e:
                self.send_response(500)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"error": str(e)}).encode('utf-8'))
        elif self.path == "/api/preload":
            _record_request("/api/preload")
            try:
                # Check if Ollama service is reachable
                ollama_online = False
                if USE_OLLAMA:
                    try:
                        req_check = urllib.request.Request(f"{OLLAMA_URL}/api/tags", method="GET")
                        with urllib.request.urlopen(req_check, timeout=0.8) as r:
                            if r.status == 200:
                                ollama_online = True
                    except Exception:
                        ollama_online = False

                if ollama_online:
                    import threading
                    def preload_thread():
                        try:
                            body = json.dumps({
                                "model": OLLAMA_MODEL,
                                "keep_alive": -1
                            }).encode("utf-8")
                            req = urllib.request.Request(
                                f"{OLLAMA_URL}/api/generate",
                                data=body,
                                headers={"Content-Type": "application/json"},
                                method="POST"
                            )
                            with urllib.request.urlopen(req, timeout=20) as resp:
                                resp.read(1)
                            print(f"[+] Preloaded Ollama model '{OLLAMA_MODEL}' successfully.")
                        except Exception as ex:
                            print(f"[-] Preload failed: {ex}")
                    
                    threading.Thread(target=preload_thread, daemon=True).start()
                
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Access-Control-Allow-Origin", "*")
                self.end_headers()
                self.wfile.write(json.dumps({"success": ollama_online, "message": f"Preload status for {OLLAMA_MODEL}"}).encode('utf-8'))
            except Exception as e:
                self.send_response(500)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"error": str(e)}).encode('utf-8'))
        elif self.path == "/api/reset":
            _record_request("/api/reset")
            try:
                reset_brain_state()
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Access-Control-Allow-Origin", "*")
                self.end_headers()
                self.wfile.write(json.dumps({"success": True}).encode('utf-8'))
            except Exception as e:
                self.send_response(500)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"error": str(e)}).encode('utf-8'))
        elif self.path == "/api/corpus":
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            
            try:
                payload = json.loads(post_data.decode('utf-8'))
                corpus = payload.get("corpus", "").strip()
                
                count = index_corpus(corpus)
                save_manifold_cache(CACHE_FILE)
                
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"success": True, "count": count}).encode('utf-8'))
            except Exception as e:
                self.send_response(500)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"error": str(e)}).encode('utf-8'))
        else:
            self.send_error(404, "API endpoint not found")

def index_extra_reading_files():
    """
    Reads all .txt files in the extra_reading directory,
    parses them into sentences, and indexes them if they are not
    already present in the CORPUS_INDEX.
    """
    global CORPUS_INDEX, VOCABULARY, WORD_PRIMES, VOCAB_VECTORS
    extra_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "extra_reading")
    if not os.path.exists(extra_dir):
        print("[-] extra_reading directory not found.")
        return
        
    # Get all .txt files
    txt_files = [f for f in os.listdir(extra_dir) if f.endswith(".txt")]
    default_lines = [line.strip() for line in DEFAULT_CORPUS.strip().split("\n") if line.strip()]
    
    # Force re-indexing of extra reading files if critical terms like "r4" are missing from WORD_PRIMES
    force = "r4" not in WORD_PRIMES
    if force:
        print("[*] 'r4' not in vocabulary. Clearing previous extra_reading indexing to force clean rebuild...")
        extra_sentences_set = set()
        for fname in txt_files:
            path = os.path.join(extra_dir, fname)
            try:
                with open(path, "r", encoding="utf-8") as f:
                    text = f.read()
                raw_lines = text.split("\n")
                for line in raw_lines:
                    line = line.strip()
                    if not line:
                        continue
                    import re
                    sents = re.split(r'(?<=[.!?])\s+', line)
                    for s in sents:
                        extra_sentences_set.add(s.strip().lower())
            except:
                pass
        for line in default_lines:
            extra_sentences_set.add(line.strip().lower())
            
        # Filter CORPUS_INDEX to remove them
        for win_idx in list(CORPUS_INDEX.keys()):
            filtered = []
            for item in CORPUS_INDEX[win_idx]:
                if item["sentence"].strip().lower() not in extra_sentences_set:
                    filtered.append(item)
            CORPUS_INDEX[win_idx] = filtered
            
    print("[*] Checking for extra_reading files to index...")
    # Gather all existing sentences to avoid duplicates
    existing_sentences = set()
    for items in CORPUS_INDEX.values():
        for item in items:
            existing_sentences.add(item["sentence"].strip().lower())
            
    # Get all .txt files
    txt_files = [f for f in os.listdir(extra_dir) if f.endswith(".txt")]
    new_sentences = []
    
    for fname in txt_files:
        path = os.path.join(extra_dir, fname)
        try:
            with open(path, "r", encoding="utf-8") as f:
                text = f.read()
            # Basic sentence splitting (split on periods followed by space/newline)
            raw_lines = text.split("\n")
            for line in raw_lines:
                line = line.strip()
                if not line:
                    continue
                import re
                sents = re.split(r'(?<=[.!?])\s+', line)
                for s in sents:
                    s_clean = s.strip()
                    if len(s_clean) > 30 and len(s_clean) < 400 and s_clean.count(" ") > 4:
                        if s_clean.lower().startswith("page ") or ("formal specification" in s_clean.lower() and len(s_clean) < 80):
                            continue
                        if s_clean.strip().lower() not in existing_sentences:
                            new_sentences.append(s_clean)
                            existing_sentences.add(s_clean.strip().lower())
        except Exception as e:
            print(f"[-] Error reading {fname}: {e}")
            
    # Check for DEFAULT_CORPUS
    default_lines = [line.strip() for line in DEFAULT_CORPUS.strip().split("\n") if line.strip()]
    default_new = []
    for line in default_lines:
        if line.lower() not in existing_sentences:
            default_new.append(line)
            existing_sentences.add(line.lower())
            
    if default_new:
        print(f"[*] Found {len(default_new)} new sentences from DEFAULT_CORPUS. Indexing...")
        new_sentences.extend(default_new)
        
    if new_sentences:
        # Dynamically add all new alphanumeric words to vocabulary & prime registry
        print("[*] Updating vocabulary with new terms...")
        for s in new_sentences:
            for w in s.split():
                clean = w.strip(".,?!()\"';:-")
                add_word_to_vocabulary(clean)
                
        print(f"[*] Found {len(new_sentences)} new sentences to index onto the R4 manifold...")
        indexed_count = 0
        for s in new_sentences:
            try:
                routing_data = route_query_to_manifold(s)
                best = routing_data["routed"]
                idx_win = best["window_index"]
                
                s_idx, e_idx = best["active_range"]
                full_state = np.zeros(M_MAX)
                full_state[s_idx:e_idx] = np.array(best["state_vector"])
                
                if idx_win not in CORPUS_INDEX:
                    CORPUS_INDEX[idx_win] = []
                    
                sent_words = [w.lower().strip(".,?!()\"';:-") for w in s.split() if w.strip()]
                prime_prod = get_sentence_prime_product(sent_words)
                
                u, v = get_sentence_projection(full_state, idx_win)
                CORPUS_INDEX[idx_win].append({
                    "sentence": s,
                    "state_vector": full_state,
                    "kappa": best["metrics"]["kappa"],
                    "deficit_angle": best["metrics"]["deficit_angle"],
                    "prime_product": prime_prod,
                    "words": sent_words,
                    "u": u,
                    "v": v
                })
                indexed_count += 1
            except Exception as e:
                continue
        print(f"[+] Successfully indexed {indexed_count} new sentences.")
        rebuild_transitions_from_corpus()
        build_2nd_order_transitions()
        save_manifold_cache(CACHE_FILE)
    else:
        print("[+] All extra_reading and DEFAULT_CORPUS files are already indexed.")

def index_single_sentence(s: str):
    """
    Dynamically indexes a single sentence onto the R4 manifold CORPUS_INDEX.
    Extends vocabulary and WORD_PRIMES if new words are encountered.
    """
    global CORPUS_INDEX, VOCABULARY, WORD_PRIMES, VOCAB_VECTORS
    s_clean = s.strip()
    if not s_clean or len(s_clean) < 10:
        return
        
    # 1. Update vocabulary with any new words
    words = [w.lower().strip(".,?!()\"';:-") for w in s_clean.split() if w.strip()]
    vocab_changed = False
    for w in words:
        if w.isalpha() and len(w) > 1:
            if w not in WORD_PRIMES:
                add_word_to_vocabulary(w)
                vocab_changed = True
                
    # If vocabulary changed, rebuild vocab matrix in memory
    if vocab_changed:
        build_vocab_matrix()
        
    # 2. Route the sentence to find its window and state vector
    routing_data = route_query_to_manifold(s_clean)
    best = routing_data["routed"]
    idx_win = best["window_index"]
    s_idx, e_idx = best["active_range"]
    
    full_state = np.zeros(M_MAX)
    full_state[s_idx:e_idx] = np.array(best["state_vector"])
    
    if idx_win not in CORPUS_INDEX:
        CORPUS_INDEX[idx_win] = []
        
    # Avoid duplicate indexing of the same sentence
    for item in CORPUS_INDEX[idx_win]:
        if item["sentence"].strip().lower() == s_clean.lower():
            return
            
    u, v = get_sentence_projection(full_state, idx_win)
    v_4d = get_state_4d_projection(full_state)
    
    CORPUS_INDEX[idx_win].append({
        "sentence": s_clean,
        "state_vector": full_state,
        "kappa": best["metrics"]["kappa"],
        "deficit_angle": best["metrics"]["deficit_angle"],
        "prime_product": get_sentence_prime_product(words),
        "words": words,
        "u": u,
        "v": v,
        "v_4d": v_4d
    })
    
    # 3. Incrementally update transition tables for words in this sentence
    for i in range(len(words) - 1):
        w1, w2 = words[i], words[i+1]
        if w1 not in TRANSITIONS:
            TRANSITIONS[w1] = {}
        TRANSITIONS[w1][w2] = TRANSITIONS[w1].get(w2, 0) + 1
        
        # Re-normalize transitions for w1
        total = sum(TRANSITIONS[w1].values())
        for k in TRANSITIONS[w1]:
            TRANSITIONS[w1][k] /= total
            
    # Update transitions index
    build_2nd_order_transitions()
    build_transitions_2nd_by_first()

def run_server(port=8000):
    # 1. Start Ollama and verify OLLAMA_MODEL is pulled
    print("[*] Verifying Ollama service status...")
    ollama_online = False
    try:
        req = urllib.request.Request(f"{OLLAMA_URL}/api/tags", method="GET")
        with urllib.request.urlopen(req, timeout=0.8) as r:
            if r.status == 200:
                ollama_online = True
                print("[+] Ollama is online!")
    except Exception:
        pass

    if not ollama_online:
        if sys.platform == "darwin":
            print("[*] Ollama is not running. Launching Ollama app...")
            try:
                import subprocess
                subprocess.Popen(["open", "-a", "Ollama"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                print("[*] Waiting for Ollama to initialize on port 11434...")
                for _ in range(20):
                    try:
                        req = urllib.request.Request(f"{OLLAMA_URL}/api/tags", method="GET")
                        with urllib.request.urlopen(req, timeout=0.8) as r:
                            if r.status == 200:
                                ollama_online = True
                                print("[+] Ollama is online!")
                                break
                    except Exception:
                        pass
                    time.sleep(1)
            except Exception as launch_err:
                print(f"[-] Could not launch Ollama automatically: {launch_err}")
        else:
            print("[-] Warning: Ollama is not running. Automated launch is only supported on macOS.")

    if ollama_online:
        print(f"[*] Checking if model '{OLLAMA_MODEL}' is ready...")
        try:
            req = urllib.request.Request(f"{OLLAMA_URL}/api/tags", method="GET")
            model_installed = False
            with urllib.request.urlopen(req, timeout=1.0) as r:
                tags_data = json.loads(r.read().decode('utf-8'))
                for model_info in tags_data.get("models", []):
                    name = model_info.get("name", "")
                    if name == OLLAMA_MODEL or name.startswith(OLLAMA_MODEL + ":") or OLLAMA_MODEL in name:
                        model_installed = True
                        break
            if not model_installed:
                print(f"[*] Model '{OLLAMA_MODEL}' not found. Pulling model (7.2 GB) via Ollama API...")
                pull_payload = json.dumps({"name": OLLAMA_MODEL, "stream": False})
                pull_req = urllib.request.Request(f"{OLLAMA_URL}/api/pull", data=pull_payload.encode('utf-8'), method="POST")
                pull_req.add_header("Content-Type", "application/json")
                with urllib.request.urlopen(pull_req, timeout=600) as r:
                    if r.status == 200:
                        print(f"[+] Model '{OLLAMA_MODEL}' pulled successfully!")
            else:
                print(f"[+] Model '{OLLAMA_MODEL}' is ready.")
        except Exception as pull_err:
            print(f"[-] Failed to verify or pull model '{OLLAMA_MODEL}': {pull_err}")

    # Try to load cached manifold model first (sub-second boot)
    cache_loaded = load_manifold_cache(CACHE_FILE)
    
    # Count total indexed sentences
    total_sentences = sum(len(items) for items in CORPUS_INDEX.values())
    
    # If cache is missing or too sparse, re-index from wiki_corpus.txt
    wiki_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "wiki_corpus.txt")
    if not cache_loaded or total_sentences < 500:
        if os.path.exists(wiki_path):
            print(f"[*] Cache has {total_sentences} sentences (threshold: 500). Re-indexing from wiki_corpus.txt...")
            try:
                with open(wiki_path, "r", encoding="utf-8") as f:
                    corpus = f.read()
                index_corpus(corpus)
                save_manifold_cache(CACHE_FILE)
                total_sentences = sum(len(items) for items in CORPUS_INDEX.values())
                print(f"[+] Wiki corpus indexed: {total_sentences} sentences now available.")
            except Exception as e:
                print(f"[-] Error indexing wiki_corpus.txt: {e}")
                if not cache_loaded:
                    index_corpus(DEFAULT_CORPUS)
                    save_manifold_cache(CACHE_FILE)
        elif not cache_loaded:
            print("[*] No cache or wiki corpus found. Indexing default corpus...")
            index_corpus(DEFAULT_CORPUS)
            save_manifold_cache(CACHE_FILE)
        else:
            print(f"[!] wiki_corpus.txt not found. Using existing cache with {total_sentences} sentences.")
    else:
        print(f"[+] Cache loaded with {total_sentences} sentences. Ready.")

    # Call index_extra_reading_files to merge default and extra documentation into the corpus index
    try:
        index_extra_reading_files()
    except Exception as e:
        print(f"[-] Error indexing extra documentation files: {e}")

    server_address = ('', port)
    httpd = ThreadingHTTPServer(server_address, RouterAPIHandler)
    print(f"[*] Interactive R4 Prime Router server running on http://localhost:{port}")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n[!] Server shutting down.")
        httpd.server_close()

if __name__ == "__main__":
    port = 8000
    if len(sys.argv) > 1:
        try:
            port = int(sys.argv[1])
        except ValueError:
            pass
    run_server(port)
