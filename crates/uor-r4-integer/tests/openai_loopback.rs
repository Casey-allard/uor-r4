use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::Command;
use uor_r4_integer::loopback::{LoopbackConfig, LoopbackServer};
use uor_r4_integer::openai::{GenerationFamily, OpenAiServing, Utf8Buffer};
use uor_r4_integer::stack::{IntegerStackModel, StackChat, StackStop};
use uor_r4_tokenizer::dialogue::Message;

struct Builder {
    matrices: Vec<Value>,
    tables: Vec<Value>,
    data: Vec<u8>,
}

impl Builder {
    fn push(&mut self, bytes: &[u8]) -> Value {
        self.data.resize(self.data.len().div_ceil(64) * 64, 0);
        let span = json!({"offset": self.data.len(), "bytes": bytes.len()});
        self.data.extend_from_slice(bytes);
        span
    }
    fn matrix(&mut self, seed: &mut u64, name: &str, rows: usize, cols: usize) {
        let mut next = || {
            *seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (*seed >> 33) as u8
        };
        let nibbles: Vec<u8> = (0..rows * cols / 2).map(|_| next()).collect();
        let scales: Vec<u8> = (0..rows * cols / 32).map(|_| next() & 0x3f).collect();
        let (nibbles, scales) = (self.push(&nibbles), self.push(&scales));
        self.matrices.push(json!({"name": name, "rows": rows, "cols": cols, "exp_base": -9, "nibbles": nibbles, "scales": scales}));
    }
    fn table(&mut self, name: &str, kind: &str, len: usize, bytes: Vec<u8>) {
        let span = self.push(&bytes);
        self.tables
            .push(json!({"name": name, "kind": kind, "len": len, "span": span}));
    }
    fn i16s(&mut self, name: &str, values: &[i16]) {
        self.table(
            name,
            "i16",
            values.len(),
            values.iter().flat_map(|v| v.to_le_bytes()).collect(),
        );
    }
    fn i32s(&mut self, name: &str, values: &[i32]) {
        self.table(
            name,
            "i32",
            values.len(),
            values.iter().flat_map(|v| v.to_le_bytes()).collect(),
        );
    }
    fn u32s(&mut self, name: &str, values: &[u32]) {
        self.table(
            name,
            "u32",
            values.len(),
            values.iter().flat_map(|v| v.to_le_bytes()).collect(),
        );
    }
}

fn artifact(seed: u64) -> Vec<u8> {
    const VOCAB: usize = 40;
    const WIDTH: usize = 32;
    const CONTEXT: usize = 64;
    let mut rng = seed;
    let mut b = Builder {
        matrices: Vec::new(),
        tables: Vec::new(),
        data: Vec::new(),
    };
    let code = |m: u64, e: i16| ((e + 64) << 4) | (m & 15) as i16;
    b.matrix(&mut rng, "embed", VOCAB, WIDTH);
    b.matrix(&mut rng, "l0.rec_in", 2 * WIDTH, WIDTH);
    b.matrix(&mut rng, "l0.rec_gate", WIDTH / 4 + WIDTH, WIDTH);
    b.matrix(&mut rng, "l0.rec_out", WIDTH, WIDTH);
    let taps: Vec<i16> = (0..4 * WIDTH)
        .map(|i| {
            let c = code(rng, -2 - (i % 3) as i16);
            rng = rng
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            if (rng >> 33).is_multiple_of(2) {
                c
            } else {
                -c
            }
        })
        .collect();
    b.i16s("l0.conv_taps", &taps);
    let mut next = || {
        rng = rng
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (rng >> 33) as u32
    };
    let biases: Vec<i32> = (0..WIDTH / 4 + WIDTH)
        .map(|_| (next() % 60_000) as i32 - 30_000)
        .collect();
    b.i32s("l0.conv_bias", &biases[..WIDTH]);
    b.i32s("l0.gate_bias", &biases);
    b.i16s(
        "l0.decay_rate",
        &(0..WIDTH / 4)
            .map(|_| code(next() as u64, 1))
            .collect::<Vec<_>>(),
    );
    for part in ["query", "key", "value", "out"] {
        b.matrix(&mut rng, &format!("l1.{part}"), WIDTH, WIDTH);
    }
    b.matrix(&mut rng, "l1.null", 2, WIDTH);
    b.i32s("l1.null_bias", &[4_000, -9_000]);
    b.i32s(
        "l1.age",
        &(0..2 * CONTEXT)
            .map(|i| -((i as i32) << 12))
            .collect::<Vec<_>>(),
    );
    b.i16s("l1.beta", &[code(3, 0), code(9, -1)]);
    b.i32s("l1.offset", &[1 << 24, 3 << 23]);
    for l in 0..2 {
        b.matrix(&mut rng, &format!("l{l}.gate"), WIDTH, WIDTH);
        b.matrix(&mut rng, &format!("l{l}.up"), WIDTH, WIDTH);
        b.matrix(&mut rng, &format!("l{l}.down"), WIDTH, WIDTH);
    }
    b.matrix(&mut rng, "head", VOCAB, WIDTH);
    b.u32s(
        "exp",
        &(0..32 * 256 + 2)
            .map(|i: u32| (1u32 << 31) >> (i / 177).min(31))
            .collect::<Vec<_>>(),
    );
    let activation: Vec<i32> = (0..(2 << 12) + 1)
        .map(|i: i32| (i - 4096).max(0) << 8)
        .collect();
    b.i32s("silu", &activation);
    b.i32s("gelu", &activation);
    b.u32s(
        "arcosh",
        &(0..89089u32).map(|i| i << 10).collect::<Vec<_>>(),
    );
    let header = json!({
        "schema": "uor-r4.lut-stack/1",
        "shape": {"vocab": VOCAB, "width": WIDTH, "heads": 2, "mlp": WIDTH, "pattern": "ra", "read": "lorentz", "rotation": true, "context": CONTEXT},
        "group": 32,
        "numerics": {"rms_eps": {"mantissa": 2_814_749_767i64, "exp": -48}, "score_scale_q30": 268_435_456, "exp_step_log2": -8, "silu_step_log2": -8, "silu_range_log2": 4, "gelu_step_log2": -8, "gelu_range_log2": 4},
        "matrices": b.matrices, "tables": b.tables, "source": {"synthetic": true}
    });
    let json = serde_json::to_vec(&header).unwrap();
    let mut out = b"UORLUT01".to_vec();
    out.extend_from_slice(&(json.len() as u64).to_le_bytes());
    out.extend_from_slice(&json);
    out.resize(out.len().div_ceil(64) * 64, 0);
    out.extend_from_slice(&b.data);
    out
}

fn tokenizer_json() -> Vec<u8> {
    let mut table = ['\0'; 256];
    let mut extra = 0u32;
    for byte in 0u32..256 {
        let printable = (0x21..=0x7e).contains(&byte)
            || (0xa1..=0xac).contains(&byte)
            || (0xae..=0xff).contains(&byte);
        table[byte as usize] = if printable {
            char::from_u32(byte).unwrap()
        } else {
            extra += 1;
            char::from_u32(255 + extra).unwrap()
        };
    }
    let mut bytes: Vec<u8> = b"User: Assistant: System: Memory: b.\nHilmotwae".to_vec();
    bytes.sort_unstable();
    bytes.dedup();
    for byte in 0u8..=255 {
        if bytes.len() == 37 {
            break;
        }
        if !bytes.contains(&byte) {
            bytes.push(byte);
        }
    }
    bytes.sort_unstable();
    bytes.truncate(37);
    let mut vocab = serde_json::Map::new();
    for (id, surface) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
        vocab.insert((*surface).to_owned(), json!(id));
    }
    for (index, byte) in bytes.iter().enumerate() {
        vocab.insert(table[*byte as usize].to_string(), json!(index + 3));
    }
    let added: Vec<Value> = ["<|bos|>", "<|eos|>", "<|unk|>"]
        .iter()
        .enumerate()
        .map(|(id, surface)| json!({"id": id, "content": surface}))
        .collect();
    serde_json::to_vec(&json!({"pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false}, "added_tokens": added, "model": {"type": "BPE", "vocab": vocab, "merges": []}})).unwrap()
}

fn serving() -> OpenAiServing {
    let model = IntegerStackModel::parse(&artifact(5)).unwrap();
    OpenAiServing::new(model, tokenizer_json(), 2).unwrap()
}

fn config() -> LoopbackConfig {
    let mut config = LoopbackConfig::new(serving()).unwrap();
    config.port = 0;
    config.token = "test-token".into();
    config
}

fn config_with_long_generation() -> LoopbackConfig {
    for seed in 1..128 {
        let model = IntegerStackModel::parse(&artifact(seed)).unwrap();
        let serving = OpenAiServing::new(model, tokenizer_json(), 2).unwrap();
        let outcome = serving
            .generate(
                &[Message {
                    role: "user",
                    content: "Hi",
                }],
                40,
                |_, _| Ok(()),
            )
            .unwrap();
        if outcome.reply.ids.len() >= 8 {
            let mut config = LoopbackConfig::new(serving).unwrap();
            config.port = 0;
            config.token = "test-token".into();
            return config;
        }
    }
    panic!("no synthetic artifact produced at least eight tokens");
}

fn config_with_leading_space_generation() -> LoopbackConfig {
    for seed in 1..512 {
        let model = IntegerStackModel::parse(&artifact(seed)).unwrap();
        let serving = OpenAiServing::new(model, tokenizer_json(), 2).unwrap();
        let outcome = serving
            .generate(
                &[Message {
                    role: "user",
                    content: "Hi",
                }],
                2,
                |_, _| Ok(()),
            )
            .unwrap();
        let Some(&first) = outcome.reply.ids.first() else {
            continue;
        };
        let chat = StackChat::from_tokenizer_json(serving.model(), &tokenizer_json(), 2).unwrap();
        if chat.tokenizer().decode_bytes(&[first]) == b" " {
            assert!(!outcome.reply.text.starts_with(' '));
            let mut config = LoopbackConfig::new(serving).unwrap();
            config.port = 0;
            config.token = "test-token".into();
            return config;
        }
    }
    panic!("no synthetic artifact produced a leading-space reply");
}

#[test]
fn sse_applies_v2_reply_text_to_leading_space() {
    let server = LoopbackServer::spawn(config_with_leading_space_generation()).unwrap();
    let (_, _, raw) = request(
        server.port(),
        "POST",
        "/v1/chat/completions",
        &chat_request(true).to_string(),
        "test-token",
        "",
    );
    let (_, _, batch) = request(
        server.port(),
        "POST",
        "/v1/chat/completions",
        &chat_request(false).to_string(),
        "test-token",
        "",
    );
    let batch: Value = serde_json::from_str(&batch).unwrap();
    let batch_text = batch["choices"][0]["message"]["content"].as_str().unwrap();
    assert!(!batch_text.starts_with(' '));
    let frames = sse_json(&raw);
    let text = frames
        .iter()
        .filter_map(|frame| frame["choices"][0]["delta"]["content"].as_str())
        .collect::<String>();
    assert_eq!(text, batch_text);
    server.shutdown().unwrap();
}

fn request(
    port: u16,
    method: &str,
    target: &str,
    body: &str,
    token: &str,
    extra: &str,
) -> (u16, String, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let auth = if token.is_empty() {
        String::new()
    } else {
        format!("Authorization: Bearer {token}\r\n")
    };
    let content_type = if body.is_empty() {
        String::new()
    } else {
        "Content-Type: application/json\r\n".into()
    };
    let wire = format!("{method} {target} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{auth}{content_type}Content-Length: {}\r\nConnection: close\r\n{extra}\r\n{body}", body.len());
    stream.write_all(wire.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let status = response.split_whitespace().nth(1).unwrap().parse().unwrap();
    let split = response.find("\r\n\r\n").unwrap();
    (
        status,
        response[..split].to_string(),
        response[split + 4..].to_string(),
    )
}

fn post_json(port: u16, target: &str, value: Value) -> (u16, Value) {
    let (status, _, body) = request(port, "POST", target, &value.to_string(), "test-token", "");
    (status, serde_json::from_str(&body).unwrap())
}

fn chat_request(stream: bool) -> Value {
    json!({"model": "uor-chat", "messages": [{"role": "user", "content": "Hi"}], "max_tokens": 2, "stream": stream, "stream_options": {"include_usage": stream}})
}

fn assert_usage(value: &Value) {
    let usage = &value["usage"];
    let total =
        usage["prompt_tokens"].as_u64().unwrap() + usage["completion_tokens"].as_u64().unwrap();
    assert_eq!(usage["total_tokens"].as_u64().unwrap(), total);
    let metrics = &value["uor"];
    assert!(metrics["elapsed_ns"].is_u64());
    assert!(metrics["first_token_ns"].is_u64());
    assert!(
        metrics["tokens_per_second_milli"].is_u64() || metrics["tokens_per_second_milli"].is_null()
    );
    assert_eq!(metrics["completion_tokens"], usage["completion_tokens"]);
    assert_eq!(
        metrics["completion_ids"].as_array().unwrap().len(),
        usage["completion_tokens"].as_u64().unwrap() as usize
    );
}

#[test]
fn models_health_and_json_generation_routes() {
    let server = LoopbackServer::spawn(config()).unwrap();
    let (status, _, body) = request(server.port(), "GET", "/healthz", "", "test-token", "");
    assert_eq!(status, 200);
    assert_eq!(serde_json::from_str::<Value>(&body).unwrap()["ready"], true);
    let (_, _, models) = request(server.port(), "GET", "/v1/models", "", "test-token", "");
    let models: Value = serde_json::from_str(&models).unwrap();
    assert_eq!(models["object"], "list");
    assert_eq!(models["data"][0]["id"], "uor-chat");
    let (status, _, detail) = request(
        server.port(),
        "GET",
        "/v1/models/uor-chat",
        "",
        "test-token",
        "",
    );
    assert_eq!(status, 200);
    assert_eq!(
        serde_json::from_str::<Value>(&detail).unwrap()["id"],
        "uor-chat"
    );
    let (_, _, capabilities) = request(
        server.port(),
        "GET",
        "/v1/capabilities",
        "",
        "test-token",
        "",
    );
    let capabilities: Value = serde_json::from_str(&capabilities).unwrap();
    assert_eq!(capabilities["vision"], false);
    assert_eq!(capabilities["inflight_limit"], 1);
    assert_eq!(capabilities["context"], 64);

    let (status, chat) = post_json(server.port(), "/v1/chat/completions", chat_request(false));
    assert_eq!(status, 200, "{chat}");
    assert_eq!(chat["object"], "chat.completion");
    assert_eq!(chat["choices"][0]["message"]["role"], "assistant");
    assert!(!chat["choices"][0]["message"]["content"]
        .as_str()
        .unwrap()
        .is_empty());
    assert_usage(&chat);

    let history = json!({"model": "uor-chat", "messages": [{"role": "user", "content": "Hi"}, {"role": "assistant", "content": chat["choices"][0]["message"]["content"]}, {"role": "user", "content": "Again"}], "max_tokens": 2});
    let (status, second) = post_json(server.port(), "/v1/chat/completions", history);
    assert_eq!(status, 200, "{second}");
    assert!(
        second["usage"]["prompt_tokens"].as_u64().unwrap()
            > chat["usage"]["prompt_tokens"].as_u64().unwrap()
    );
    assert_usage(&second);

    let (status, legacy) = post_json(
        server.port(),
        "/v1/completions",
        json!({"model": "uor-chat", "prompt": "Hi", "max_tokens": 2}),
    );
    assert_eq!(status, 200, "{legacy}");
    assert_eq!(legacy["object"], "text_completion");
    assert_usage(&legacy);

    let (status, response) = post_json(
        server.port(),
        "/v1/responses",
        json!({"model": "uor-chat", "input": "Hi", "max_output_tokens": 2}),
    );
    assert_eq!(status, 200, "{response}");
    assert_eq!(response["object"], "response");
    assert_eq!(
        response["usage"]["input_tokens"],
        response["usage"]["prompt_tokens"]
    );
    assert_eq!(
        response["usage"]["output_tokens"],
        response["usage"]["completion_tokens"]
    );
    assert_usage(&response);
    server.shutdown().unwrap();
}

fn sse_json(body: &str) -> Vec<Value> {
    body.split("\r\n\r\n")
        .filter_map(|block| {
            block
                .lines()
                .find_map(|line| line.strip_prefix("data: "))
                .filter(|data| *data != "[DONE]")
                .map(|data| serde_json::from_str(data).unwrap())
        })
        .collect()
}

#[test]
fn sse_matches_json_and_supports_all_generation_families() {
    let server = LoopbackServer::spawn(config()).unwrap();
    let (_, _, raw) = request(
        server.port(),
        "POST",
        "/v1/chat/completions",
        &chat_request(true).to_string(),
        "test-token",
        "",
    );
    let (_, _, batch) = request(
        server.port(),
        "POST",
        "/v1/chat/completions",
        &chat_request(false).to_string(),
        "test-token",
        "",
    );
    let batch: Value = serde_json::from_str(&batch).unwrap();
    let frames = sse_json(&raw);
    let text = frames
        .iter()
        .filter_map(|frame| frame["choices"][0]["delta"]["content"].as_str())
        .collect::<String>();
    assert_eq!(
        text,
        batch["choices"][0]["message"]["content"].as_str().unwrap()
    );
    let ids = frames
        .iter()
        .filter_map(|frame| frame["uor"]["token_id"].as_u64())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        batch["uor"]["completion_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(Value::as_u64)
            .collect::<Option<Vec<_>>>()
            .unwrap()
    );
    let usage = frames
        .iter()
        .find(|frame| frame["choices"].as_array().unwrap().is_empty())
        .unwrap();
    assert_usage(&usage);
    assert!(raw.contains("data: [DONE]"));
    assert_eq!(
        frames
            .iter()
            .map(|frame| frame["id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        1,
        "one completion ID covers every chunk"
    );
    assert_eq!(
        frames
            .iter()
            .map(|frame| frame["created"].as_u64().unwrap())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        1,
        "one created timestamp covers every chunk"
    );

    let (_, _, legacy) = request(server.port(), "POST", "/v1/completions", &json!({"model": "uor-chat", "prompt": "Hi", "max_tokens": 2, "stream": true, "stream_options": {"include_usage": true}}).to_string(), "test-token", "");
    assert!(legacy.contains("text_completion"));
    assert!(legacy.contains("data: [DONE]"));
    let legacy_frames = sse_json(&legacy);
    let legacy_usage = legacy_frames
        .iter()
        .find(|frame| frame["choices"].as_array().unwrap().is_empty())
        .unwrap();
    assert_usage(&legacy_usage);
    assert_eq!(
        legacy_frames
            .iter()
            .map(|frame| frame["id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        1
    );
    let (_, _, response) = request(server.port(), "POST", "/v1/responses", &json!({"model": "uor-chat", "input": "Hi", "max_output_tokens": 2, "stream": true, "stream_options": {"include_usage": true}}).to_string(), "test-token", "");
    assert!(response.contains("event: response.output_text.delta"));
    assert!(response.contains("event: response.completed"));
    assert!(response.contains("data: [DONE]"));
    let response_frames = sse_json(&response);
    let created = response_frames
        .iter()
        .find(|frame| frame["type"] == "response.created")
        .unwrap();
    let completed = response_frames
        .iter()
        .find(|frame| frame["type"] == "response.completed")
        .unwrap();
    assert_usage(&completed["response"]);
    assert_eq!(
        created["response"]["id"], completed["response"]["id"],
        "Responses stream keeps one response ID"
    );
    let item_ids = response_frames
        .iter()
        .filter(|frame| frame["type"] == "response.output_text.delta")
        .map(|frame| frame["item_id"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(item_ids.len(), 1, "Responses deltas share one item ID");
    assert_eq!(
        item_ids.into_iter().next().unwrap(),
        completed["response"]["output"][0]["id"].as_str().unwrap()
    );
    server.shutdown().unwrap();
}

#[test]
fn truthful_validation_and_unsupported_family_errors() {
    let server = LoopbackServer::spawn(config()).unwrap();
    for (target, body, expected) in [
        ("/v1/chat/completions", "{".into(), 400),
        ("/v1/chat/completions", json!({"model": "uor-chat", "messages": [], "logprobs": true}).to_string(), 400),
        ("/v1/chat/completions", json!({"model": "uor-chat", "messages": [{"role": "user", "content": "Hi"}], "temperature": 0.5}).to_string(), 400),
        ("/v1/chat/completions", json!({"model": "uor-chat", "messages": [{"role": "user", "content": "Hi"}], "n": 2}).to_string(), 400),
        ("/v1/chat/completions", json!({"model": "uor-chat", "messages": [{"role": "user", "content": "Hi"}], "max_tokens": 2, "max_completion_tokens": 2}).to_string(), 400),
        ("/v1/chat/completions", json!({"model": "uor-chat", "messages": [{"role": "tool", "content": "Hi"}]}).to_string(), 400),
        ("/v1/chat/completions", json!({"model": "uor-chat", "messages": [{"role": "user", "content": "<|eos|>"}]}).to_string(), 400),
        ("/v1/chat/completions", json!({"model": "wrong", "messages": [{"role": "user", "content": "Hi"}]}).to_string(), 404),
        ("/v1/chat/completions", json!({"model": "uor-chat", "messages": [{"role": "user", "content": "H".repeat(10_000)}]}).to_string(), 413),
        ("/v1/chat/completions", json!({"model": "uor-chat", "messages": [{"role": "user", "content": [{"type": "image_url", "image_url": {"url": "https://example.invalid/x.png"}}]}]}).to_string(), 501),
        ("/v1/embeddings", json!({"model": "uor-chat", "input": "Hi"}).to_string(), 501),
        ("/v1/unknown", json!({}).to_string(), 501),
    ] {
        let (status, _, body) = request(server.port(), "POST", target, &body, "test-token", "");
        assert_eq!(status, expected, "{target}: {body}");
        let error: Value = serde_json::from_str(&body).unwrap();
        assert!(error["error"]["message"].is_string(), "{error}");
        assert!(error["error"]["type"].is_string());
    }
    let (status, _, body) = request(
        server.port(),
        "GET",
        "/v1/models/wrong",
        "",
        "test-token",
        "",
    );
    assert_eq!(status, 404);
    assert!(body.contains("\"error\""));
    let (status, _, _) = request(
        server.port(),
        "POST",
        "/v1/chat/completions",
        "{}",
        "test-token",
        "Content-Type: text/plain\r\n",
    );
    assert_eq!(status, 400);
    server.shutdown().unwrap();
}

#[test]
fn loopback_admission_busy_and_disconnect_are_explicit() {
    let server = LoopbackServer::spawn(config()).unwrap();
    let (status, _, _) = request(server.port(), "GET", "/v1/models", "", "", "");
    assert_eq!(status, 401);
    let (status, _, _) = request(
        server.port(),
        "GET",
        "/v1/models",
        "",
        "test-token",
        "Origin: https://evil.invalid\r\n",
    );
    assert_eq!(status, 403);
    let (status, _, _) = request(
        server.port(),
        "GET",
        "/v1/models",
        "",
        "test-token",
        "X-UOR-Studio-Token: wrong\r\n",
    );
    assert_eq!(status, 401);
    let port = server.port();
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let wire = "GET /v1/models HTTP/1.1\r\nHost: evil.invalid\r\nAuthorization: Bearer test-token\r\nConnection: close\r\n\r\n";
    stream.write_all(wire.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 400"));
    server.shutdown().unwrap();

    let serving = serving();
    let permit = serving.try_start_generation().unwrap();
    let error = serving.try_start_generation().unwrap_err();
    assert_eq!(error.status, 429);
    drop(permit);
    assert!(serving.try_start_generation().is_ok());
    let error = serving
        .generate(
            &[Message {
                role: "user",
                content: "Hi",
            }],
            2,
            |_, _| {
                Err(uor_r4_integer::openai::OpenAiError::new(
                    400,
                    "disconnected",
                ))
            },
        )
        .unwrap_err();
    assert_eq!(error.status, 400);
    assert_eq!(error.message, "disconnected");

    let mut bounded = config();
    bounded.max_body_bytes = 64;
    let bounded = LoopbackServer::spawn(bounded).unwrap();
    let (status, _, _) = request(
        bounded.port(),
        "POST",
        "/v1/chat/completions",
        &chat_request(false).to_string(),
        "test-token",
        "",
    );
    assert_eq!(status, 413);
    bounded.shutdown().unwrap();

    let mut allowed = config();
    allowed.origin = Some("https://studio.test".into());
    let allowed = LoopbackServer::spawn(allowed).unwrap();
    let (status, headers, _) = request(
        allowed.port(),
        "GET",
        "/v1/models",
        "",
        "test-token",
        "Origin: https://studio.test\r\n",
    );
    assert_eq!(status, 200);
    assert!(headers.contains("Access-Control-Allow-Origin: https://studio.test"));
    allowed.shutdown().unwrap();

    let mut invalid = config();
    invalid.origin = Some("https://studio.test\r\nX-Injected: true".into());
    assert!(LoopbackServer::spawn(invalid).is_err());
}

#[test]
fn studio_bootstrap_is_ephemeral_and_static_paths_are_bounded() {
    let root = std::env::temp_dir().join(format!("uor-openai-loopback-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::write(
        root.join("index.html"),
        "<html><head></head><body>Studio</body></html>",
    )
    .unwrap();
    std::fs::write(root.join("assets/app.js"), "console.log('ok');").unwrap();
    let outside = root.with_extension("secret");
    std::fs::write(&outside, "not served").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, root.join("assets/secret.txt")).unwrap();
    let mut config = config();
    config.studio_root = Some(root.clone());
    let server = LoopbackServer::spawn(config).unwrap();
    let (status, _, index) = request(server.port(), "GET", "/", "", "", "");
    assert_eq!(status, 200);
    assert!(index.contains("window.__UOR_NATIVE_BOOTSTRAP__"));
    assert!(index.contains("test-token"));
    assert!(index.contains("\"/v1\""));
    let (status, _, _) = request(server.port(), "GET", "/assets/app.js", "", "", "");
    assert_eq!(status, 200);
    for path in ["/../Cargo.toml", "/assets/secret.txt"] {
        let (status, _, _) = request(server.port(), "GET", path, "", "", "");
        assert_eq!(status, 404, "{path}");
    }
    server.shutdown().unwrap();
    std::fs::remove_dir_all(root).unwrap();
    std::fs::remove_file(outside).unwrap();
}

#[test]
fn studio_bootstrap_cannot_close_inline_script() {
    let root = std::env::temp_dir().join(format!("uor-openai-script-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("index.html"),
        "<html><head></head><body>Studio</body></html>",
    )
    .unwrap();
    let mut config = config();
    config.studio_root = Some(root.clone());
    config.token = "</script><script>alert(1)</script>".into();
    let server = LoopbackServer::spawn(config).unwrap();
    let (status, _, index) = request(server.port(), "GET", "/", "", "", "");
    assert_eq!(status, 200);
    assert!(index.contains("\\u003c/script>"), "{index}");
    assert!(!index.contains("</script><script>alert(1)</script>"));
    assert_eq!(index.matches("<script>").count(), 1);
    server.shutdown().unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn stop_mapping_text_parts_and_exact_prompt_counts() {
    let server = LoopbackServer::spawn(config()).unwrap();
    let (status, limited) = post_json(
        server.port(),
        "/v1/chat/completions",
        json!({"model": "uor-chat", "messages": [{"role": "user", "content": [{"type": "text", "text": "Hi"}]}], "max_completion_tokens": 1}),
    );
    assert_eq!(status, 200, "{limited}");
    assert_eq!(limited["choices"][0]["finish_reason"], "length");
    assert_eq!(limited["uor"]["stop"], "max_new_tokens");
    let model = IntegerStackModel::parse(&artifact(5)).unwrap();
    let chat = StackChat::from_tokenizer_json(&model, &tokenizer_json(), 2).unwrap();
    let encoded = chat
        .protocol()
        .bind(chat.tokenizer())
        .unwrap()
        .encode_assistant_prefix(&[Message {
            role: "user",
            content: "Hi",
        }]);
    assert_eq!(
        limited["usage"]["prompt_tokens"].as_u64().unwrap() as usize,
        encoded.tokens.len()
    );

    let (status, eos) = post_json(server.port(), "/v1/chat/completions", chat_request(false));
    assert_eq!(status, 200);
    assert_eq!(eos["choices"][0]["finish_reason"], "stop");
    assert_eq!(eos["uor"]["stop"], "eos");
    server.shutdown().unwrap();

    let mut cycle = None;
    for seed in 1..24 {
        let model = IntegerStackModel::parse(&artifact(seed)).unwrap();
        let serving = OpenAiServing::new(model, tokenizer_json(), 2).unwrap();
        let outcome = serving
            .generate(
                &[Message {
                    role: "user",
                    content: "Hi",
                }],
                4,
                |_, _| Ok(()),
            )
            .unwrap();
        if matches!(outcome.reply.stop, StackStop::ShortCycle { .. }) {
            cycle = Some((serving, outcome));
            break;
        }
    }
    let (serving, outcome) = cycle.expect("a synthetic short-cycle fixture");
    let response = serving.response_json(GenerationFamily::Chat, &outcome);
    assert_eq!(response["choices"][0]["finish_reason"], "stop");
    assert!(response["uor"]["stop"]
        .as_str()
        .unwrap()
        .starts_with("short_cycle"));
}

#[test]
fn http_busy_preflight_disconnect_and_header_bounds() {
    let mut config = config_with_long_generation();
    config.origin = Some("https://studio.test".into());
    let server = LoopbackServer::spawn(config).unwrap();
    let body = json!({"model": "uor-chat", "messages": [{"role": "user", "content": "Hi"}], "max_tokens": 40, "stream": true, "stream_options": {"include_usage": true}}).to_string();
    let wire = |port: u16| {
        format!("POST /v1/chat/completions HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer test-token\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
    };
    let mut first = TcpStream::connect(("127.0.0.1", server.port())).unwrap();
    first.write_all(wire(server.port()).as_bytes()).unwrap();
    let mut role = [0u8; 1];
    let read = first.read(&mut role).unwrap();
    assert!(read > 0);
    let (status, _, second) = request(
        server.port(),
        "POST",
        "/v1/chat/completions",
        &body,
        "test-token",
        "",
    );
    assert_eq!(
        status, 429,
        "a second stream is rejected before any 200/SSE bytes: {second}"
    );
    let mut remainder = String::new();
    first.read_to_string(&mut remainder).unwrap();
    assert!(remainder.contains("data: [DONE]"));
    assert!(!remainder.contains("HTTP/1.1 429"));

    let mut disconnect = TcpStream::connect(("127.0.0.1", server.port())).unwrap();
    disconnect
        .write_all(wire(server.port()).as_bytes())
        .unwrap();
    disconnect.read(&mut role).unwrap();
    disconnect.shutdown(std::net::Shutdown::Both).unwrap();
    let generation_request = json!({"model": "uor-chat", "messages": [{"role": "user", "content": "Hi"}], "max_tokens": 1});
    let mut next = post_json(
        server.port(),
        "/v1/chat/completions",
        generation_request.clone(),
    );
    for _ in 0..20 {
        if next.0 == 200 {
            break;
        }
        assert_eq!(next.0, 429, "{next:?}");
        std::thread::sleep(std::time::Duration::from_millis(5));
        next = post_json(
            server.port(),
            "/v1/chat/completions",
            generation_request.clone(),
        );
    }
    assert_eq!(
        next.0, 200,
        "disconnect releases the generation permit for a subsequent generation: {next:?}"
    );

    let (status, headers, _) = request(
        server.port(),
        "OPTIONS",
        "/v1/chat/completions",
        "",
        "",
        "Origin: https://studio.test\r\n",
    );
    assert_eq!(status, 204, "CORS preflight needs no API credential");
    assert!(headers.contains("Access-Control-Allow-Origin: https://studio.test"));

    let mut oversized = TcpStream::connect(("127.0.0.1", server.port())).unwrap();
    let header = format!(
        "GET /v1/models HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Oversize: {}\r\n",
        server.port(),
        "a".repeat(40_000)
    );
    oversized.write_all(header.as_bytes()).unwrap();
    oversized.shutdown(std::net::Shutdown::Write).unwrap();
    let mut response = String::new();
    let _ = oversized.read_to_string(&mut response);
    assert!(response.starts_with("HTTP/1.1 400"), "{response}");
    server.shutdown().unwrap();
}

#[test]
fn cli_serve_loads_the_same_artifact_without_logging_the_token() {
    let root = std::env::temp_dir().join(format!("uor-openai-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let model_path = root.join("model.lut");
    let tokenizer_path = root.join("tokenizer.json");
    let studio_path = root.join("studio");
    std::fs::create_dir(&studio_path).unwrap();
    std::fs::write(&model_path, artifact(5)).unwrap();
    std::fs::write(&tokenizer_path, tokenizer_json()).unwrap();
    std::fs::write(
        studio_path.join("index.html"),
        "<html><head></head><body>Studio</body></html>",
    )
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_uor-chat"))
        .arg("--stack")
        .arg(&model_path)
        .arg("--tokenizer")
        .arg(&tokenizer_path)
        .arg("--serve")
        .arg("127.0.0.1:0")
        .arg("--studio-root")
        .arg(&studio_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    assert!(line.contains("OpenAI loopback listening"), "{line}");
    let port = line
        .split("127.0.0.1:")
        .nth(1)
        .and_then(|tail| {
            tail.split(|character: char| !character.is_ascii_digit())
                .next()
        })
        .and_then(|digits| digits.parse::<u16>().ok())
        .unwrap();
    let (status, _, _) = request(port, "GET", "/v1/models", "", "test-token", "");
    assert_eq!(status, 401);
    let (_, _, index) = request(port, "GET", "/", "", "", "");
    let token = index
        .split("\"token\":\"")
        .nth(1)
        .and_then(|tail| tail.split('"').next())
        .unwrap();
    assert!(!token.is_empty());
    assert!(!line.contains(token));
    let (status, _, models) = request(port, "GET", "/v1/models", "", token, "");
    assert_eq!(status, 200);
    let models: Value = serde_json::from_str(&models).unwrap();
    let model = IntegerStackModel::parse(&artifact(5)).unwrap();
    assert_eq!(
        models["data"][0]["artifact_sha256"],
        model.artifact_sha256()
    );
    child.kill().unwrap();
    child.wait().unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn cli_token_stdin_supports_headless_loopback() {
    let root = std::env::temp_dir().join(format!("uor-openai-cli-stdin-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let model_path = root.join("model.lut");
    let tokenizer_path = root.join("tokenizer.json");
    std::fs::write(&model_path, artifact(5)).unwrap();
    std::fs::write(&tokenizer_path, tokenizer_json()).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_uor-chat"))
        .arg("--stack")
        .arg(&model_path)
        .arg("--tokenizer")
        .arg(&tokenizer_path)
        .arg("--serve")
        .arg("127.0.0.1:0")
        .arg("--token-stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let token = "h".repeat(256);
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{token}\n").as_bytes())
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    assert!(line.contains("OpenAI loopback listening"), "{line}");
    let port = line
        .split("127.0.0.1:")
        .nth(1)
        .and_then(|tail| {
            tail.split(|character: char| !character.is_ascii_digit())
                .next()
        })
        .and_then(|digits| digits.parse::<u16>().ok())
        .unwrap();
    let (status, _, _) = request(port, "GET", "/v1/models", "", &token, "");
    assert_eq!(status, 200);
    assert!(!line.contains(&token));
    child.kill().unwrap();
    child.wait().unwrap();
    let refused = Command::new(env!("CARGO_BIN_EXE_uor-chat"))
        .arg("--stack")
        .arg(&model_path)
        .arg("--tokenizer")
        .arg(&tokenizer_path)
        .arg("--serve")
        .arg("127.0.0.1:0")
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("--token-stdin"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn utf8_streaming_never_emits_partial_or_invalid_text() {
    let mut buffer = Utf8Buffer::default();
    assert_eq!(buffer.push(&[0xf0, 0x9f]).unwrap(), "");
    assert_eq!(buffer.push(&[0x98, 0x80]).unwrap(), "😀");
    buffer.finish().unwrap();
    buffer.push(&[0xf0, 0x9f]).unwrap();
    assert_eq!(buffer.finish().unwrap_err().status, 500);
    let mut invalid = Utf8Buffer::default();
    assert_eq!(invalid.push(&[0xff]).unwrap_err().status, 500);
}
