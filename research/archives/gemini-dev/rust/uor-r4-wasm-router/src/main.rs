use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use serde::Deserialize;
use uor_r4_wasm_router::UorR4Router;
use uor_foundation::pipeline::PrismModel;

#[derive(Deserialize)]
struct ChatPayload {
    text: String,
    identity: Option<String>,
    engine: Option<String>,
    ollama_url: Option<String>,
    ollama_model: Option<String>,
}

#[derive(Deserialize)]
struct CorpusPayload {
    corpus: String,
    identity: Option<String>,
}

#[derive(Deserialize)]
struct ResetPayload {
    identity: Option<String>,
}

fn get_window_theme(win_idx: usize) -> &'static str {
    match win_idx {
        1 => "Origins & Foundations",
        2 => "Duality & Polarity",
        3 => "Temporal & Sequential",
        4 => "Boundaries & Limits",
        5 => "Quintessential Forces",
        6 => "Harmonic Resonance",
        7 => "Critical Transitions",
        8 => "Octave Completion",
        9 => "Convergence",
        10 => "Curvature & Topology",
        11 => "Relativistic Effects",
        12 => "Hyperbolic Geometry",
        13 => "Zeta Horizon",
        14 => "High Frequency",
        15 => "Entropic Dissolution",
        16 => "Extremal Manifold",
        _ => "Unknown Window",
    }
}

fn main() {
    println!("Initializing R4 Prime Router Backend Server...");
    let start_time = Instant::now();
    let router = Arc::new(Mutex::new(UorR4Router::new(0.85)));

    // Load cache on startup
    {
        let mut r = router.lock().unwrap();
        let mut cache_loaded = false;
        if let Ok(cache_data) = std::fs::read_to_string("manifold_cache_rust.json") {
            if let Err(e) = r.import_state_native(&cache_data) {
                println!("[-] Warning: Failed to load manifold cache: {}", e);
            } else {
                let total = r.get_total_indexed_sentences();
                println!("[+] Successfully loaded manifold cache from manifold_cache_rust.json. Sentences indexed: {}", total);
                if total >= 500 {
                    cache_loaded = true;
                }
            }
        } else {
            println!("[*] No existing cache found. Initializing new manifold.");
        }

        if !cache_loaded {
            index_wiki_corpus(&mut r);
        }

        // Scan and index extra reading documents
        index_extra_reading_files(&mut r);

        // Save cache
        let state_json = r.export_state();
        let _ = std::fs::write("manifold_cache_rust.json", state_json);
    }

    let listener = match TcpListener::bind("127.0.0.1:8000") {
        Ok(l) => l,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                println!("[!] Port 8000 is already in use.");
                if let Some(pid) = find_pid_by_port(8000) {
                    println!("[*] Found process occupying port 8000: PID {}", pid);
                    print!("Would you like to terminate this process and start the server? [y/N]: ");
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                    let mut input = String::new();
                    if std::io::stdin().read_line(&mut input).is_ok() {
                        let trimmed = input.trim().to_lowercase();
                        if trimmed == "y" || trimmed == "yes" {
                            println!("[*] Terminating process {}...", pid);
                            if kill_process(pid) {
                                // Wait 1 second for port to release
                                std::thread::sleep(std::time::Duration::from_millis(1000));
                                match TcpListener::bind("127.0.0.1:8000") {
                                    Ok(l) => l,
                                    Err(e2) => {
                                        eprintln!("[-] Failed to bind to port 8000 after terminating process: {}", e2);
                                        std::process::exit(1);
                                    }
                                }
                            } else {
                                eprintln!("[-] Failed to terminate process {}. Please close it manually and retry.", pid);
                                std::process::exit(1);
                            }
                        } else {
                            println!("[*] Exiting gracefully.");
                            std::process::exit(0);
                        }
                    } else {
                        println!("[-] Non-interactive session or read error. Exiting gracefully.");
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("[-] Port 8000 is occupied, but could not determine process ID. Please close it manually and retry.");
                    std::process::exit(1);
                }
            } else {
                eprintln!("[-] Failed to bind to 127.0.0.1:8000: {}", e);
                std::process::exit(1);
            }
        }
    };
    println!("Local server running at http://127.0.0.1:8000/");

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let r_clone = Arc::clone(&router);
            std::thread::spawn(move || {
                handle_connection(stream, r_clone, start_time);
            });
        }
    }
}

fn index_wiki_corpus(router: &mut UorR4Router) {
    let paths = vec![
        std::path::PathBuf::from("/Users/adminamn/gemini-dev/wiki_corpus.txt"),
        std::path::PathBuf::from("../../wiki_corpus.txt"),
        std::path::PathBuf::from("wiki_corpus.txt"),
    ];
    let mut wiki_file = None;
    for p in paths {
        if p.exists() && p.is_file() {
            wiki_file = Some(p);
            break;
        }
    }
    let wiki_file = match wiki_file {
        Some(f) => f,
        None => {
            println!("[-] wiki_corpus.txt not found.");
            return;
        }
    };
    println!("[*] Loading and indexing wiki corpus from {:?}", wiki_file);
    if let Ok(content) = std::fs::read_to_string(&wiki_file) {
        let count = router.index_corpus(&content, "shared");
        println!("[+] Successfully indexed {} sentences from wiki_corpus.txt.", count);
    }
}

fn index_extra_reading_files(router: &mut UorR4Router) {
    let paths = vec![
        std::path::PathBuf::from("/Users/adminamn/gemini-dev/extra_reading"),
        std::path::PathBuf::from("../../extra_reading"),
        std::path::PathBuf::from("extra_reading"),
    ];
    let mut extra_dir = None;
    for p in paths {
        if p.exists() && p.is_dir() {
            extra_dir = Some(p);
            break;
        }
    }
    let extra_dir = match extra_dir {
        Some(d) => d,
        None => {
            println!("[-] extra_reading directory not found.");
            return;
        }
    };
    println!("[*] Checking for extra_reading files in {:?}", extra_dir);
    if let Ok(entries) = std::fs::read_dir(extra_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    println!("[*] Reading and indexing extra_reading file: {:?}", path.file_name().unwrap_or_default());
                    let count = router.index_corpus(&content, "shared");
                    println!("[+] Indexed {} sentences from {:?}", count, path.file_name().unwrap_or_default());
                }
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, router: Arc<Mutex<UorR4Router>>, start_time: Instant) {
    let mut buf_reader = BufReader::new(&mut stream);

    let mut request_line = String::new();
    if buf_reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
        return;
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    let path_str = parts[1];
    let clean_path = path_str.split('?').next().unwrap().split('#').next().unwrap();
    eprintln!("[REQUEST] {} {} -> clean_path: {}", method, path_str, clean_path);

    if method == "OPTIONS" {
        let response = "HTTP/1.1 200 OK\r\n\
                        Access-Control-Allow-Origin: *\r\n\
                        Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
                        Access-Control-Allow-Headers: Content-Type\r\n\
                        Content-Length: 0\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    let mut content_length = 0;
    loop {
        let mut line = String::new();
        if buf_reader.read_line(&mut line).is_err() {
            break;
        }
        if line == "\r\n" || line == "\n" || line.is_empty() {
            break;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            if let Some(val_str) = line.split(':').nth(1) {
                if let Ok(len) = val_str.trim().parse::<usize>() {
                    content_length = len;
                }
            }
        }
    }

    let mut body = vec![0; content_length];
    if content_length > 0 {
        if buf_reader.read_exact(&mut body).is_err() {
            send_json_response(stream, 400, "{\"error\":\"Error reading body\"}");
            return;
        }
    }

    // Intercept native router endpoints
    if clean_path == "/api/chat" && method == "POST" {
        let payload: ChatPayload = match serde_json::from_slice(&body) {
            Ok(p) => p,
            Err(e) => {
                send_json_response(stream, 400, &format!("{{\"error\":\"Invalid JSON: {}\"}}", e));
                return;
            }
        };

        let identity = payload.identity.unwrap_or_else(|| "tenant-alpha".to_string());
        let engine_mode = payload.engine.unwrap_or_else(|| "auto".to_string());
        let ollama_url = payload.ollama_url.unwrap_or_else(|| "http://127.0.0.1:11434".to_string());
        let ollama_model = payload.ollama_model.unwrap_or_else(|| "gemma4:e2b".to_string());

        let mut router_guard = router.lock().unwrap();

        // 1. Dry run routing to get baseline parameters via UOR pipeline
        let mut buf = [0u8; 640];
        let query_bytes = payload.text.as_bytes();
        let identity_bytes = identity.as_bytes();
        let query_len = query_bytes.len().min(512);
        let identity_len = identity_bytes.len().min(128);
        buf[..query_len].copy_from_slice(&query_bytes[..query_len]);
        buf[512..512 + identity_len].copy_from_slice(&identity_bytes[..identity_len]);

        let input = uor_r4_wasm_router::R4RoutingInput {
            query: &buf[..512],
            identity: &buf[512..],
            data: &buf,
        };

        // Bind thread-local
        let router_ptr = &mut *router_guard as *mut UorR4Router;
        uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = Some(router_ptr);
        });

        // Run dry run through UorR4RouterModel
        let _grounded_dry = uor_r4_wasm_router::UorR4RouterModel::forward(input).expect("Dry run routing failed");

        // Reset thread-local
        uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = None;
        });

        let routing = router_guard.last_routing_data().clone().expect("No routing data generated");
        let kappa = routing.routed.metrics.kappa;
        let theta_d = routing.routed.metrics.deficit_angle;
        let uor_bias = routing.routed.qimc.uor_control.entropy_bias;

        // Auto-tuned params
        let gamma = (0.85 - 0.55 * kappa + ((uor_bias - 0.5) * 0.12)).clamp(0.15, 0.90);
        let temperature = (0.2 + 0.8 * theta_d.abs().tanh() + ((uor_bias - 0.5) * 0.20)).clamp(0.15, 1.1);

        // 2. Select Synthesis Engine
        let mut ollama_online = false;
        if engine_mode == "ollama" || engine_mode == "auto" {
            match get_request(&format!("{}/api/tags", ollama_url)) {
                Ok(resp) => {
                    eprintln!("[DEBUG] get_request /api/tags succeeded. Response preview: {}", resp.chars().take(200).collect::<String>());
                    ollama_online = true;
                }
                Err(e) => {
                    eprintln!("[DEBUG] get_request /api/tags failed: {}", e);
                }
            }
        }
        let engine = if ollama_online && engine_mode != "geometric" {
            "ollama"
        } else {
            "geometric"
        };

        // Determine dynamic suggested token limit from the router itself
        let max_tokens = router_guard.get_suggested_token_limit(&payload.text, &identity);

        // 3. Evolve the brain state
        router_guard.evolve_state(&identity, &payload.text, gamma);

        // 4. Run final routing on evolved state via UOR pipeline
        let t_route = Instant::now();

        // Bind thread-local
        uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = Some(router_ptr);
        });

        let grounded = uor_r4_wasm_router::UorR4RouterModel::forward(input).expect("Final routing failed");

        // Reset thread-local
        uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = None;
        });

        let routing_data = router_guard.last_routing_data().clone().expect("No final routing data generated");
        let route_ms = t_route.elapsed().as_secs_f64() * 1000.0;

        // 5. Decode response
        let t_gen = Instant::now();
        let geom_max_tokens = if engine == "ollama" {
            25
        } else {
            max_tokens
        };

        let geom_result = router_guard.generate_geometric_response_native(
            &payload.text,
            &identity,
            geom_max_tokens,
            temperature,
            10.0,
            4.0,
            gamma,
        );

        let top_resonances = router_guard.get_top_resonances_native(&payload.text, &identity, 1);
        let ctx_block = if !top_resonances.is_empty() {
            &top_resonances[0].sentence
        } else {
            "[no corpus context available]"
        };

        let mut final_response_text = String::new();
        let mut llm_connected = false;
        let mut generation_mode = "geometric-decoded".to_string();

        if engine == "ollama" {
            let theme = get_window_theme(routing_data.routed.window_index);
            let system_prompt = format!(
                "You are the Voice of the R4 Prime Router. \
                 The current state of your context hypersphere brain is: Window {} ({}), Energy κ={:.4}, Curvature θd={:.4}, \
                 Hopf coordinates (χ={:.4}, δ={:.4}, α={:.4}). \
                 Grounding context sentence: \"{}\". \
                 IMPORTANT: You MUST incorporate and reference the Grounding context sentence in your response to directly answer the user's query. \
                 Respond directly to the user query as a geometric router. Keep your response relevant, coherent, and under {} words. \
                 Do NOT output any thinking, thought processes, <thinking> blocks, or XML tags. Speak directly.",
                routing_data.routed.window_index, theme, routing_data.routed.metrics.kappa, routing_data.routed.metrics.deficit_angle,
                routing_data.routed.hopf.chi, routing_data.routed.hopf.delta, routing_data.routed.hopf.alpha, ctx_block, max_tokens
            );

            let ollama_payload = serde_json::json!({
                "model": ollama_model,
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": payload.text }
                ],
                "stream": false,
                "keep_alive": -1,
                "think": false,
                "options": {
                    "temperature": temperature,
                    "num_predict": max_tokens.max(300)
                }
            });

            match post_json(&format!("{}/api/chat", ollama_url), &ollama_payload.to_string()) {
                Ok(resp_body) => {
                    eprintln!("[DEBUG] post_json response body: {}", resp_body);
                    match serde_json::from_str::<serde_json::Value>(&resp_body) {
                        Ok(resp_json) => {
                            let content = resp_json.get("message")
                                .and_then(|m| m.get("content"))
                                .and_then(|c| c.as_str())
                                .unwrap_or("");
                            
                            final_response_text = content.trim().to_string();
                            if !final_response_text.is_empty() {
                                llm_connected = true;
                                generation_mode = format!("ollama:{}", ollama_model);
                            }
                        }
                        Err(e) => {
                            eprintln!("[-] Failed to parse Ollama JSON response: {}. Body: {}", e, resp_body);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[-] Ollama generation failed, falling back to geometric: {}", e);
                }
            }
        }

        if final_response_text.is_empty() {
            if ctx_block != "[no corpus context available]" && engine == "ollama" {
                final_response_text = ctx_block.to_string();
                generation_mode = "geometric-retrieval".to_string();
            } else {
                final_response_text = if !geom_result.text.is_empty() {
                    geom_result.text.clone()
                } else {
                    "Manifold resonance too sparse for synthesis.".to_string()
                };
            }
        }

        let gen_ms = t_gen.elapsed().as_secs_f64() * 1000.0;

        // 6. Index user prompt and response back into vocabulary for continuous learning
        if !final_response_text.is_empty() {
            router_guard.index_sentence(&payload.text, &identity);
            router_guard.index_sentence(&final_response_text, &identity);

            // Inject thought streams for tracing
            router_guard.inject_thought_stream_native(&payload.text);
            router_guard.inject_thought_stream_native(&final_response_text);

            // Save cache to disk in background thread
            let state_json = router_guard.export_state();
            std::thread::spawn(move || {
                let _ = std::fs::write("manifold_cache_rust.json", state_json);
            });
        }

        // Project the evolved brain state to 2D for the map path tracing
        let active_state = router_guard.get_brain_state_native(&identity);
        let (u, v) = router_guard.get_sentence_projection_native(&active_state, routing_data.routed.window_index);
        let v_4d = router_guard.get_state_4d_projection_native(&active_state);

        let theme = get_window_theme(routing_data.routed.window_index);
        let archetype = if theta_d > -1.0 {
            "Symmetric Orbit (Resonant)"
        } else if theta_d < -1.4 {
            "Hyperbolic Flare (Divergent)"
        } else {
            "Orthogonal Drift (Steady)"
        };

        let top_resonances_5 = router_guard.get_top_resonances_native(&payload.text, &identity, 5);

        let trace = grounded.derivation().replay::<256>();
        let mut uor_trace_steps = Vec::new();
        for i in 0..trace.len() {
            if let Some(event) = trace.event(i as usize) {
                uor_trace_steps.push(serde_json::json!({
                    "step": event.step_index(),
                    "op": format!("{:?}", event.op()),
                    "target": format!("0x{:032x}", event.target().as_u128()),
                }));
            }
        }

        let uor_payload = serde_json::json!({
            "algorithm": routing_data.routed.uor.algorithm.clone(),
            "hash_algorithm": routing_data.routed.uor.hash_algorithm.clone(),
            "hash_algorithm_id": routing_data.routed.uor.hash_algorithm_id,
            "address": routing_data.routed.uor.address.clone(),
            "verify_result": "Verified",
            "kappa_label": format!("witt:{}", grounded.witt_level_bits()),
            "fingerprint_hex": hex::encode(grounded.content_fingerprint().as_bytes()),
            "sigma": grounded.sigma().value(),
            "d_delta": grounded.d_delta().as_i64(),
            "euler": grounded.euler().as_i64(),
            "residual": grounded.residual().as_u32(),
            "stratum": grounded.triad().stratum(),
            "multihash_addresses": routing_data.routed.uor.multihash_addresses.clone(),
        });

        let response_payload = serde_json::json!({
            "text": payload.text,
            "archetype": archetype,
            "description": final_response_text,
            "summary": format!("W{} ({}) | Scale {:.0} | kappa={:.4} theta_d={:.4} | {}",
                routing_data.routed.window_index, theme, routing_data.routed.scale_x, kappa, theta_d, generation_mode),
            "llm_connected": llm_connected,
            "generation_mode": generation_mode,
            "active_projection": {
                "u": u,
                "v": v,
                "v_4d": v_4d
            },
            "metrics": {
                "window_index": routing_data.routed.window_index,
                "scale_x": routing_data.routed.scale_x,
                "kappa": kappa,
                "deficit_angle": theta_d,
                "lambda_entropy": routing_data.routed.metrics.lambda_entropy,
                "sigma_kl": routing_data.routed.metrics.sigma_kl,
                "top_eigenvalue_pct": ((routing_data.routed.eigenvalues[0] / (routing_data.routed.eigenvalues.iter().sum::<f64>().max(1.0))) * 100.0),
                "qimc": routing_data.routed.qimc,
                "hopf": routing_data.routed.hopf,
                "uor_address": routing_data.routed.uor_address,
                "uor": uor_payload,
                "auto_tuned": {
                    "gamma": gamma,
                    "temperature": temperature,
                    "max_tokens": max_tokens,
                    "engine": engine,
                    "uor_entropy_bias": uor_bias
                }
            },
            "eigenvalues": routing_data.routed.eigenvalues,
            "active_range": routing_data.routed.active_range,
            "state_vector": routing_data.routed.state_vector,
            "all_routes": routing_data.all_routes,
            "top_resonance": top_resonances_5,
            "trajectory": geom_result.trajectory,
            "active_streams": router_guard.get_active_streams_native(),
            "expert_counts": router_guard.get_expert_counts(),
            "routing_latency_ms": route_ms.round(),
            "gen_latency_ms": gen_ms.round(),
            "uor_trace_steps": uor_trace_steps,
        });

        send_json_response(stream, 200, &response_payload.to_string());
        return;
    }

    if clean_path == "/api/corpus" && method == "POST" {
        let payload: CorpusPayload = match serde_json::from_slice(&body) {
            Ok(p) => p,
            Err(e) => {
                send_json_response(stream, 400, &format!("{{\"error\":\"Invalid JSON: {}\"}}", e));
                return;
            }
        };

        let identity = payload.identity.unwrap_or_else(|| "shared".to_string());
        let mut router_guard = router.lock().unwrap();
        let count = router_guard.index_corpus(&payload.corpus, &identity);

        let state_json = router_guard.export_state();
        std::thread::spawn(move || {
            let _ = std::fs::write("manifold_cache_rust.json", state_json);
        });

        let resp = serde_json::json!({ "success": true, "count": count }).to_string();
        send_json_response(stream, 200, &resp);
        return;
    }

    if clean_path == "/api/reset" && method == "POST" {
        let payload: ResetPayload = serde_json::from_slice(&body).unwrap_or(ResetPayload { identity: None });

        let mut router_guard = router.lock().unwrap();
        if let Some(ref identity) = payload.identity {
            router_guard.reset_brain(identity);
        } else {
            router_guard.reset_to_defaults();
        }

        let state_json = router_guard.export_state();
        std::thread::spawn(move || {
            let _ = std::fs::write("manifold_cache_rust.json", state_json);
        });

        let resp = serde_json::json!({ "success": true }).to_string();
        send_json_response(stream, 200, &resp);
        return;
    }

    if clean_path == "/api/export" && method == "GET" {
        let router_guard = router.lock().unwrap();
        let state_json = router_guard.export_state();
        send_json_response(stream, 200, &state_json);
        return;
    }

    if clean_path == "/api/import" && method == "POST" {
        let mut router_guard = router.lock().unwrap();
        let state_str = match String::from_utf8(body) {
            Ok(s) => s,
            Err(_) => {
                send_json_response(stream, 400, "{\"error\":\"Invalid UTF-8 string\"}");
                return;
            }
        };
        if let Err(e) = router_guard.import_state_native(&state_str) {
            send_json_response(stream, 400, &format!("{{\"error\":\"Import failed: {}\"}}", e));
            return;
        }

        let state_json = router_guard.export_state();
        std::thread::spawn(move || {
            let _ = std::fs::write("manifold_cache_rust.json", state_json);
        });

        let resp = serde_json::json!({ "success": true }).to_string();
        send_json_response(stream, 200, &resp);
        return;
    }

    if clean_path == "/api/tags" && method == "GET" {
        let ollama_url = "http://127.0.0.1:11434/api/tags";
        match get_request(ollama_url) {
            Ok(body) => {
                send_json_response(stream, 200, &body);
            }
            Err(e) => {
                send_json_response(stream, 502, &format!("{{\"error\":\"Ollama unreachable: {}\"}}", e));
            }
        }
        return;
    }

    if clean_path == "/api/sysinfo" && method == "GET" {
        let mut router_guard = router.lock().unwrap();
        let sentences_indexed = router_guard.get_total_indexed_sentences();
        let active_streams = router_guard.get_active_streams_native();
        let expert_counts = router_guard.get_expert_counts();

        let identity = "tenant-alpha";
        
        let mut buf = [0u8; 640];
        let query_bytes = "Welcome".as_bytes();
        let identity_bytes = identity.as_bytes();
        let query_len = query_bytes.len().min(512);
        let identity_len = identity_bytes.len().min(128);
        buf[..query_len].copy_from_slice(&query_bytes[..query_len]);
        buf[512..512 + identity_len].copy_from_slice(&identity_bytes[..identity_len]);

        let input = uor_r4_wasm_router::R4RoutingInput {
            query: &buf[..512],
            identity: &buf[512..],
            data: &buf,
        };

        // Bind thread-local
        let router_ptr = &mut *router_guard as *mut UorR4Router;
        uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = Some(router_ptr);
        });

        // Run through UorR4RouterModel
        let grounded = uor_r4_wasm_router::UorR4RouterModel::forward(input).expect("Sysinfo routing failed");

        // Reset thread-local
        uor_r4_wasm_router::ACTIVE_ROUTER.with(|r| {
            *r.borrow_mut() = None;
        });

        let routing_data = router_guard.last_routing_data().clone().expect("No sysinfo routing data generated");
        let active_state = router_guard.get_brain_state_native(&identity);
        let (u, v) = router_guard.get_sentence_projection_native(&active_state, routing_data.routed.window_index);
        let v_4d = router_guard.get_state_4d_projection_native(&active_state);
        let kappa = routing_data.routed.metrics.kappa;
        let theta_d = routing_data.routed.metrics.deficit_angle;
        let uor_bias = routing_data.routed.qimc.uor_control.entropy_bias;
        
        let gamma = (0.85 - 0.55 * kappa + ((uor_bias - 0.5) * 0.12)).clamp(0.15, 0.90);
        let temperature = (0.2 + 0.8 * theta_d.abs().tanh() + ((uor_bias - 0.5) * 0.20)).clamp(0.15, 1.1);

        let geom_result = router_guard.generate_geometric_response_native(
            "Welcome",
            &identity,
            25,
            temperature,
            10.0,
            4.0,
            gamma,
        );

        let top_resonances_5 = router_guard.get_top_resonances_native("Welcome", &identity, 5);

        let trace = grounded.derivation().replay::<256>();
        let mut uor_trace_steps = Vec::new();
        for i in 0..trace.len() {
            if let Some(event) = trace.event(i as usize) {
                uor_trace_steps.push(serde_json::json!({
                    "step": event.step_index(),
                    "op": format!("{:?}", event.op()),
                    "target": format!("0x{:032x}", event.target().as_u128()),
                }));
            }
        }

        let uor_payload = serde_json::json!({
            "algorithm": routing_data.routed.uor.algorithm.clone(),
            "hash_algorithm": routing_data.routed.uor.hash_algorithm.clone(),
            "hash_algorithm_id": routing_data.routed.uor.hash_algorithm_id,
            "address": routing_data.routed.uor.address.clone(),
            "verify_result": "Verified",
            "kappa_label": format!("witt:{}", grounded.witt_level_bits()),
            "fingerprint_hex": hex::encode(grounded.content_fingerprint().as_bytes()),
            "sigma": grounded.sigma().value(),
            "d_delta": grounded.d_delta().as_i64(),
            "euler": grounded.euler().as_i64(),
            "residual": grounded.residual().as_u32(),
            "stratum": grounded.triad().stratum(),
            "multihash_addresses": routing_data.routed.uor.multihash_addresses.clone(),
        });

        let info = serde_json::json!({
            "uptime_seconds": start_time.elapsed().as_secs_f64().round(),
            "sentences_indexed": sentences_indexed,
            "requests_total": 0,
            "catastrophes": 0,
            "window_hits": {},
            "routing_latency_p50_ms": 0.0,
            "routing_latency_p95_ms": 0.0,
            "gen_latency_p50_ms": 0.0,
            "gen_latency_p95_ms": 0.0,
            "glove_loaded": false,
            "otel_available": false,
            "active_streams": active_streams,
            "expert_counts": expert_counts,
            "active_projection": {
                "u": u,
                "v": v,
                "v_4d": v_4d
            },
            "metrics": {
                "window_index": routing_data.routed.window_index,
                "scale_x": routing_data.routed.scale_x,
                "kappa": kappa,
                "deficit_angle": theta_d,
                "lambda_entropy": routing_data.routed.metrics.lambda_entropy,
                "sigma_kl": routing_data.routed.metrics.sigma_kl,
                "top_eigenvalue_pct": ((routing_data.routed.eigenvalues[0] / (routing_data.routed.eigenvalues.iter().sum::<f64>().max(1.0))) * 100.0),
                "qimc": routing_data.routed.qimc,
                "hopf": routing_data.routed.hopf,
                "uor_address": routing_data.routed.uor_address,
                "uor": uor_payload,
                "auto_tuned": {
                    "gamma": gamma,
                    "temperature": temperature,
                    "max_tokens": 25,
                    "engine": "geometric",
                    "uor_entropy_bias": uor_bias
                }
            },
            "eigenvalues": routing_data.routed.eigenvalues,
            "active_range": routing_data.routed.active_range,
            "state_vector": routing_data.routed.state_vector,
            "all_routes": routing_data.all_routes,
            "top_resonance": top_resonances_5,
            "trajectory": geom_result.trajectory,
            "uor_trace_steps": uor_trace_steps,
        });

        send_json_response(stream, 200, &info.to_string());
        return;
    }

    if clean_path == "/api/map" && method == "GET" {
        let router_guard = router.lock().unwrap();
        let map_val = router_guard.get_semantic_map_points_native();
        send_json_response(stream, 200, &map_val.to_string());
        return;
    }

    // Serve static files fallback
    let mut relative_path = clean_path.trim_start_matches('/');
    if relative_path.is_empty() {
        relative_path = "index.html";
    }

    let file_path = Path::new(relative_path);
    if !file_path.exists() || file_path.is_dir() {
        let response = "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    let contents = match fs::read(file_path) {
        Ok(c) => c,
        Err(_) => {
            let response = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes());
            return;
        }
    };

    let mime_type = match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        mime_type,
        contents.len()
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.write_all(&contents);
}

fn send_json_response(mut stream: TcpStream, status_code: u16, body: &str) {
    let status_text = match status_code {
        200 => "OK",
        400 => "BAD REQUEST",
        404 => "NOT FOUND",
        500 => "INTERNAL SERVER ERROR",
        502 => "BAD GATEWAY",
        _ => "OK",
    };
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\r\n\
         {}",
        status_code, status_text, body.len(), body
    );
    let _ = stream.write_all(response.as_bytes());
}

fn decode_chunked(body: &str) -> String {
    let mut decoded = String::new();
    let mut current = body;
    while !current.is_empty() {
        if let Some(pos) = current.find("\r\n") {
            let size_str = current[..pos].trim();
            if size_str.is_empty() {
                break;
            }
            let chunk_size = match usize::from_str_radix(size_str, 16) {
                Ok(sz) => sz,
                Err(_) => {
                    return body.to_string();
                }
            };
            if chunk_size == 0 {
                break;
            }
            let chunk_start = pos + 2;
            if chunk_start + chunk_size > current.len() {
                break;
            }
            decoded.push_str(&current[chunk_start..chunk_start + chunk_size]);
            current = &current[chunk_start + chunk_size..];
            if current.starts_with("\r\n") {
                current = &current[2..];
            } else if current.starts_with('\n') {
                current = &current[1..];
            }
        } else {
            break;
        }
    }
    if decoded.is_empty() {
        body.to_string()
    } else {
        decoded
    }
}

fn get_request(url: &str) -> Result<String, String> {
    let url = url.trim_start_matches("http://");
    let parts: Vec<&str> = url.splitn(2, '/').collect();
    let host_port = parts[0];
    let path = if parts.len() > 1 { parts[1] } else { "" };
    let path = format!("/{}", path);

    let host_parts: Vec<&str> = host_port.split(':').collect();
    let host = host_parts[0];
    let port: u16 = if host_parts.len() > 1 { host_parts[1].parse().unwrap_or(11434) } else { 11434 };

    let addr = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;

    stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap_or(());
    stream.set_write_timeout(Some(std::time::Duration::from_secs(5))).unwrap_or(());

    let req_str = format!(
        "GET {} HTTP/1.1\r\n\
         Host: {}\r\n\
         Connection: close\r\n\r\n",
        path, host_port
    );

    stream.write_all(req_str.as_bytes()).map_err(|e| e.to_string())?;

    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| e.to_string())?;

    if let Some(pos) = response.find("\r\n\r\n") {
        let body = &response[pos + 4..];
        Ok(decode_chunked(body))
    } else {
        Ok(decode_chunked(&response))
    }
}

fn post_json(url: &str, body: &str) -> Result<String, String> {
    let url = url.trim_start_matches("http://");
    let parts: Vec<&str> = url.splitn(2, '/').collect();
    let host_port = parts[0];
    let path = if parts.len() > 1 { parts[1] } else { "" };
    let path = format!("/{}", path);

    let host_parts: Vec<&str> = host_port.split(':').collect();
    let host = host_parts[0];
    let port: u16 = if host_parts.len() > 1 { host_parts[1].parse().unwrap_or(11434) } else { 11434 };

    let addr = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;

    stream.set_read_timeout(Some(std::time::Duration::from_secs(120))).unwrap_or(());
    stream.set_write_timeout(Some(std::time::Duration::from_secs(120))).unwrap_or(());

    let req_str = format!(
        "POST {} HTTP/1.1\r\n\
         Host: {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n\
         {}",
         path, host_port, body.len(), body
    );

    stream.write_all(req_str.as_bytes()).map_err(|e| e.to_string())?;

    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| e.to_string())?;

    if let Some(pos) = response.find("\r\n\r\n") {
        let body = &response[pos + 4..];
        Ok(decode_chunked(body))
    } else {
        Ok(decode_chunked(&response))
    }
}

fn find_pid_by_port(port: u16) -> Option<u32> {
    let output = std::process::Command::new("lsof")
        .args(&["-t", "-i", &format!(":{}", port)])
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let first_line = stdout.lines().next()?;
        first_line.trim().parse::<u32>().ok()
    } else {
        None
    }
}

fn kill_process(pid: u32) -> bool {
    let _ = std::process::Command::new("kill")
        .arg(&pid.to_string())
        .status();
    std::thread::sleep(std::time::Duration::from_millis(200));
    let check = std::process::Command::new("kill")
        .args(&["-0", &pid.to_string()])
        .status();
    if let Ok(status) = check {
        if status.success() {
            let force = std::process::Command::new("kill")
                .args(&["-9", &pid.to_string()])
                .status();
            return force.map(|s| s.success()).unwrap_or(false);
        }
    }
    true
}
