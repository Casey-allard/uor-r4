//! The browser boundary: a host transport adapter, not the model.
//!
//! `holo_alloc`, `holo_free` and `holo_run` are the guest shape PrismPM's Core-Wasm guests export
//! (`core-wasm-v1` in hologram-live). Requests and responses are UTF-8 JSON; this adapter only decodes
//! the request into the generated records, calls the generated core, and encodes what came back. No
//! rule lives here.
//!
//! Operations, by module of the model:
//!   Workspace: preimage, restore, network, preview, head
//!   Inference: preimages, route, endpoint-ready, encode-completion, encode-role, encode-delta,
//!              encode-final, encode-error, encode-models, done, encode-openrouter-request
//!   Object:    expert-page, table-page, root-preimage, object-line, admit, page-action, pool-admit,
//!              fetch-source, prefetch-order, pack-rank, first-token-ready, promote, loader-start
//!   Publish:   publish-preimage, source-manifest, publish-decision, holo-path, and through PrismPM's
//!              vendored archive code: compose (a .holo v4 from the guest, the View bundle, the model
//!              document, the source manifest and the provenance; bytes travel as base64) and
//!              validate-application (PrismPM's strict Holo/1 profile check)
//!   Forge:     view
//! Errors: {"error":"…"}.

use crate::holo::archive::{compose_application, validate_application, ApplicationArchiveInput, ArchiveProvenance};
use crate::{
    admitPage, holoPath, publishDecision, publishPreimage, sourceManifest, warmup, done, encodeCompletion, encodeDelta, encodeError, encodeFinal, encodeModels, encodeOpenRouterRequest, encodeRole, endpointReady, expertPage, fetchSource, firstTokenReady, headOf, loaderStart, networkDecision, objEntry, packRank, packed, pageAction, poolAdmit, prefetchOrder, preimages, previewPath, promote, restoreDecision, rootPreimage, route, snapshotPreimage, tablePage, view, Admission, Capabilities, Completion, Decision, Entry, Grant, Manifest, Message, Obj, PageAction, Priority, Project, Ref, Request, Route, Section, Shard, Source, Staging, Start, Tier,
};
use hologram::space::{seal_private, PayloadCipher};
use serde_json::{json, Value};

/// hologram-space's private payload seam, with the concrete cipher its reference impl names:
/// ChaCha20-Poly1305 (spec 04 §Private). It never panics on ill-sized input.
struct ChaCha;
impl PayloadCipher for ChaCha {
    fn seal(&self, key: &[u8], nonce: &[u8], plaintext: &[u8]) -> Vec<u8> {
        use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, Key, KeyInit, Nonce};
        if key.len() != 32 || nonce.len() != 12 {
            return Vec::new();
        }
        ChaCha20Poly1305::new(Key::from_slice(key)).encrypt(Nonce::from_slice(nonce), plaintext).unwrap_or_default()
    }
    fn open(&self, key: &[u8], nonce: &[u8], ciphertext: &[u8]) -> Option<Vec<u8>> {
        use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, Key, KeyInit, Nonce};
        if key.len() != 32 || nonce.len() != 12 {
            return None;
        }
        ChaCha20Poly1305::new(Key::from_slice(key)).decrypt(Nonce::from_slice(nonce), ciphertext).ok()
    }
}

fn request(value: &Value) -> Result<Request, String> {
    let messages = value["messages"]
        .as_array()
        .ok_or("messages must be an array")?
        .iter()
        .map(|m| Message {
            role: m["role"].as_str().unwrap_or("user").to_owned(),
            content: match &m["content"] {
                Value::String(text) => text.clone(),
                Value::Array(parts) => parts
                    .iter()
                    .filter_map(|part| part["text"].as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
                _ => String::new(),
            },
        })
        .collect();
    let temperature = match &value["temperature"] {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        // The wire spelling is the adapter's job: what the daemon's f32 Display prints.
        other => other
            .as_f64()
            .map(|f| format!("{}", f as f32))
            .ok_or("temperature must be a number or a decimal string")?,
    };
    Ok(Request {
        model: value["model"].as_str().unwrap_or("").to_owned(),
        messages,
        maxTokens: value["max_tokens"].as_u64(),
        seed: value["seed"].as_u64(),
        temperature,
    })
}

fn completion(value: &Value) -> Completion {
    let text = |key: &str| value[key].as_str().unwrap_or("").to_owned();
    Completion {
        id: text("id"),
        created: match &value["created"] {
            Value::Number(n) => n.to_string(),
            other => other.as_str().unwrap_or("0").to_owned(),
        },
        model: text("model"),
        text: text("text"),
        fingerprint: text("fingerprint"),
        receipt: text("receipt"),
    }
}

fn u(value: &Value) -> u64 {
    value.as_u64().unwrap_or(0)
}

fn text(value: &Value) -> String {
    value.as_str().unwrap_or("").to_owned()
}

fn manifest(value: &Value) -> Manifest {
    Manifest {
        spec: text(&value["spec"]),
        repo: text(&value["repo"]),
        revision: text(&value["revision"]),
        experts: u(&value["experts"]),
        tableRows: u(&value["table_rows"]),
        shards: value["shards"].as_array().map(|shards| shards.iter().map(|sh| Shard {
            label: text(&sh["name"]),
            bytes: u(&sh["bytes"]),
            sha256: text(&sh["sha256"]),
            kappa: text(&sh["kappa"]),
            objects: text(&sh["objects"]),
        }).collect()).unwrap_or_default(),
    }
}

fn project(value: &Value) -> Project {
    Project {
        label: text(&value["label"]),
        entries: value["entries"]
            .as_array()
            .map(|entries| {
                entries
                    .iter()
                    .map(|e| Entry { path: text(&e["path"]), kappa: text(&e["kappa"]), bytes: e["bytes"].as_u64().unwrap_or(0) })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn decision(d: Decision) -> Value {
    json!({ "decision": match d { Decision::Accept => "Accept", Decision::Refuse => "Refuse" } })
}

// Bytes cross the JSON boundary as standard base64 (RFC 4648, padded); a transport spelling only.
const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
fn b64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16) | (chunk.get(1).map(|b| u32::from(*b) << 8).unwrap_or(0)) | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    out
}
fn b64_decode(text: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let mut acc: u32 = 0;
    let mut bits = 0;
    for c in text.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\n' | b'\r' => continue,
            _ => return Err("bytes are not base64".to_owned()),
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Ok(out)
}
fn bytes_of(value: &Value) -> Result<Vec<u8>, String> {
    b64_decode(value.as_str().unwrap_or(""))
}

fn run(input: &[u8]) -> Value {
    let value: Value = match serde_json::from_slice(input) {
        Ok(value) => value,
        Err(error) => return json!({ "error": format!("request is not JSON: {error}") }),
    };
    match value["op"].as_str().unwrap_or("") {
        "preimages" => match request(&value["request"]) {
            Ok(request) => {
                let out = preimages(&request);
                json!({
                    "prompt": String::from_utf8_lossy(&out.prompt),
                    "params": String::from_utf8_lossy(&out.params),
                })
            }
            Err(error) => json!({ "error": error }),
        },
        "preimage" => json!({ "bytes": snapshotPreimage(&project(&value["project"]), text(&value["parent"])) }),
        "restore" => decision(restoreDecision(text(&value["derived"]), text(&value["expected"]))),
        "network" => {
            let caps = Capabilities {
                endpoints: value["endpoints"]
                    .as_array()
                    .map(|l| l.iter().map(|e| Grant { endpoint: text(e) }).collect())
                    .unwrap_or_default(),
            };
            decision(networkDecision(&caps, text(&value["origin"])))
        }
        "preview" => json!({ "path": previewPath(text(&value["kappa"])) }),
        "head" => {
            let refs: Vec<Ref> = value["refs"].as_array().map(|l| l.iter().map(|r| Ref { branch: text(&r["branch"]), kappa: text(&r["kappa"]) }).collect()).unwrap_or_default();
            json!({ "kappa": headOf(&refs, text(&value["branch"])) })
        }
        "view" => view_json(),
        // Publish: the words and bytes the model owns, then PrismPM's own composer and validator.
        "publish-preimage" => json!({ "bytes": publishPreimage(text(&value["application"]), text(&value["version"]), text(&value["closure"]), text(&value["attestation"])) }),
        "source-manifest" => json!({ "bytes": sourceManifest(text(&value["leanManifest"]), text(&value["coverage"]), text(&value["kernel"]), text(&value["modelId"]), text(&value["semanticId"]), text(&value["sourceId"])) }),
        "publish-decision" => decision(publishDecision(text(&value["closure"]), text(&value["built"]), value["attestation"].as_str().map(str::to_owned))),
        "holo-path" => json!({ "path": holoPath(text(&value["kappa"])) }),
        "compose" => {
            let provenance: ArchiveProvenance = match serde_json::from_value(value["provenance"].clone()) {
                Ok(p) => p,
                Err(error) => return json!({ "error": format!("provenance: {error}") }),
            };
            let input = match (bytes_of(&value["guest"]), bytes_of(&value["view"]), bytes_of(&value["model"]), bytes_of(&value["source"])) {
                (Ok(guest_wasm), Ok(view_bundle), Ok(model_document), Ok(source_manifest)) => ApplicationArchiveInput { application_name: text(&value["application"]), guest_wasm, view_bundle, model_document, source_manifest, provenance },
                _ => return json!({ "error": "guest, view, model and source must be base64" }),
            };
            match compose_application(&input) {
                Ok(holo) => json!({
                    "bytes": b64_encode(&holo.bytes),
                    "byteLength": holo.bytes.len(),
                    "identities": holo.identities,
                    "directory": String::from_utf8_lossy(&holo.directory),
                    "provenance": String::from_utf8_lossy(&holo.prism_extension),
                    "applicationManifest": b64_encode(&holo.application_manifest),
                    "capabilityRequest": b64_encode(&holo.capability_request),
                }),
                Err(error) => json!({ "error": format!("{}: {}", error.code, error.message) }),
            }
        }
        "validate-application" => match bytes_of(&value["bytes"]) {
            Ok(bytes) => match validate_application(&bytes) {
                Ok(()) => json!({ "valid": true, "byteLength": bytes.len() }),
                Err(error) => json!({ "error": format!("{}: {}", error.code, error.message) }),
            },
            Err(error) => json!({ "error": error }),
        },
        // The wire: every response byte comes from the generated encoders.
        "encode-completion" => json!({ "bytes": encodeCompletion(&completion(&value["completion"])) }),
        "encode-role" => json!({ "bytes": encodeRole(&completion(&value["completion"])) }),
        "encode-delta" => json!({ "bytes": encodeDelta(&completion(&value["completion"]), value["delta"].as_str().unwrap_or("").to_owned()) }),
        "encode-final" => json!({ "bytes": encodeFinal(&completion(&value["completion"])) }),
        "encode-error" => json!({ "bytes": encodeError(value["message"].as_str().unwrap_or("").to_owned(), value["type"].as_str().unwrap_or("server_error").to_owned()) }),
        "encode-models" => json!({ "bytes": encodeModels(&value["ids"].as_array().map(|ids| ids.iter().filter_map(|i| i.as_str().map(str::to_owned)).collect::<Vec<_>>()).unwrap_or_default()) }),
        "done" => json!({ "bytes": done() }),
        // The κ object: page ranges, the root preimage and the admission rule come from the model.
        // The page arithmetic is checked: an overflow is a refusal, never a wrapped address.
        "expert-page" => match expertPage(u(&value["start"]), u(&value["length"]), u(&value["experts"]), u(&value["expert"])) {
            Ok(r) => json!({ "start": r.start, "end": r.stop }),
            Err(e) => json!({ "error": format!("{e:?}") }),
        },
        "table-page" => match tablePage(u(&value["start"]), u(&value["end"]), u(&value["rowBytes"]), u(&value["rows"]), u(&value["index"])) {
            Ok(r) => json!({ "start": r.start, "end": r.stop }),
            Err(e) => json!({ "error": format!("{e:?}") }),
        },
        "root-preimage" => json!({ "bytes": rootPreimage(&manifest(&value["manifest"])) }),
        "object-line" => json!({ "bytes": objEntry(&Obj { kind: text(&value["kind"]), label: text(&value["name"]), kappa: text(&value["kappa"]), bytes: u(&value["bytes"]) }) }),
        "admit" => json!({ "admit": admitPage(&value["listed"].as_array().map(|l| l.iter().filter_map(|k| k.as_str().map(str::to_owned)).collect::<Vec<_>>()).unwrap_or_default(), value["kappa"].as_str().unwrap_or(""), value["derived"].as_str().unwrap_or("")) }),
        // The pool and stage tables: what happens to a routed page, how the pool admits, where a page comes from, what is prefetched.
        "page-action" => {
            let staging = if value["staging"].as_str() == Some("staged-replace") { Staging::StagedReplace } else { Staging::TrueRouting };
            json!({ "action": match pageAction(value["resident"].as_bool().unwrap_or(false), staging) { PageAction::Bind => "Bind", PageAction::Fetch => "Fetch", PageAction::Drop => "Drop" } })
        }
        "pool-admit" => json!({ "admission": match poolAdmit(value["present"].as_bool().unwrap_or(false), value["spaceLeft"].as_bool().unwrap_or(false)) { Admission::Touch => "Touch", Admission::Insert => "Insert", Admission::EvictThenInsert => "EvictThenInsert" } }),
        "fetch-source" => json!({ "source": match fetchSource(value["onDevice"].as_bool().unwrap_or(false), value["onMirror"].as_bool().unwrap_or(false), value["peerFaster"].as_bool().unwrap_or(false)) { Source::Device => "Device", Source::Peer => "Peer", Source::Mirror => "Mirror", Source::Nowhere => "Nowhere" } }),
        "prefetch-order" => json!({ "priority": match prefetchOrder(value["predicted"].as_bool().unwrap_or(false), value["popular"].as_bool().unwrap_or(false)) { Priority::First => "First", Priority::Fill => "Fill", Priority::Skip => "Skip" } }),
        // Pack, Ladder, Loader: the archive's order, which model answers, how a visit starts.
        "pack-rank" => {
            let section = match value["section"].as_str().unwrap_or("") { "header" => Section::Header, "tokenizer" => Section::Tokenizer, "spine" => Section::Spine, "expert" => Section::Expert, _ => Section::Table };
            json!({ "rank": packRank(section.clone()), "packed": packed(section) })
        }
        "first-token-ready" => json!({ "ready": firstTokenReady(value["spinePresent"].as_bool().unwrap_or(false), value["promptPagesPresent"].as_bool().unwrap_or(false)) }),
        "promote" => {
            let current = if value["current"].as_str() == Some("large") { Tier::Large } else { Tier::Small };
            json!({ "tier": match promote(current, value["largeResident"].as_bool().unwrap_or(false), value["largeFast"].as_bool().unwrap_or(false)) { Tier::Large => "Large", Tier::Small => "Small" } })
        }
        "loader-start" => json!({ "start": match loaderStart(value["shellOnDevice"].as_bool().unwrap_or(false), value["snapshotOnDevice"].as_bool().unwrap_or(false)) { Start::Cold => "Cold", Start::Warm => "Warm", Start::Resume => "Resume" } }),
        // Who answers: the route table in the model, every row a theorem.
        "warmup" => json!({ "warmup": warmup(value["localReady"].as_bool().unwrap_or(false), value["keyPresent"].as_bool().unwrap_or(false), value["online"].as_bool().unwrap_or(false)) }),
        "route" => {
            let r = route(
                value["hit"].as_bool().unwrap_or(false),
                value["gpuReady"].as_bool().unwrap_or(false),
            );
            json!({ "route": match r { Route::Serve => "Serve", Route::Local => "Local", Route::NoGpu => "NoGpu" } })
        }
        // The endpoint control: shown only when the model says a request could be answered.
        "endpoint-ready" => json!({ "ready": endpointReady(
            value["resident"].as_bool().unwrap_or(false),
            value["keyPresent"].as_bool().unwrap_or(false),
            value["online"].as_bool().unwrap_or(false),
        ) }),
        // The bytes OpenRouter receives: only what the model spells, never a key.
        "encode-openrouter-request" => match request(&value["request"]) {
            Ok(request) => json!({ "bytes": encodeOpenRouterRequest(value["model"].as_str().unwrap_or("").to_owned(), &request, value["stream"].as_bool().unwrap_or(false)) }),
            Err(message) => json!({ "error": message }),
        },
        other => json!({ "error": format!("unknown op {other:?}") }),
    }
}

/// The View record as JSON, field by field, so the shell and the projector read the same words.
pub fn view_json() -> Value {
    let v = view();
    json!({
        "headline": v.headline,
        "lede": v.lede,
        "promptPlaceholder": v.promptPlaceholder,
        "sendLabel": v.sendLabel,
        "buildingLabel": v.buildingLabel,
        "previewLabel": v.previewLabel,
        "snapshotLabel": v.snapshotLabel,
        "rollbackLabel": v.rollbackLabel,
        "refusedLabel": v.refusedLabel,
        "offlineLabel": v.offlineLabel,
        "publishLabel": v.publishLabel,
        "publishingLabel": v.publishingLabel,
        "publishedLabel": v.publishedLabel,
        "publishRefusedLabel": v.publishRefusedLabel,
        "holoLabel": v.holoLabel,
        "downloadLabel": v.downloadLabel,
        "openLabel": v.openLabel,
        "runsLabel": v.runsLabel,
        "holoTitle": v.holoTitle,
        "holoLede": v.holoLede,
        "pickLabel": v.pickLabel,
        "verifyingLabel": v.verifyingLabel,
        "loadingLabel": v.loadingLabel,
        "ladderLabel": v.ladderLabel,
        "seedModelLabel": v.seedModelLabel,
        "largerLoadingLabel": v.largerLoadingLabel,
        "promotedLabel": v.promotedLabel,
        "servedLabel": v.servedLabel,
        "sealedLabel": v.sealedLabel,
        "rederiveLabel": v.rederiveLabel,
        "identicalLabel": v.identicalLabel,
        "noGpuLabel": v.noGpuLabel,
        "modelLabel": v.modelLabel,
        "appearanceLabel": v.appearanceLabel,
        "darkLabel": v.darkLabel,
        "lightLabel": v.lightLabel,
        "immersiveLabel": v.immersiveLabel,
        "wallpapers": v.wallpapers.iter().map(|w| json!({ "file": w.file, "name": w.label, "by": w.author, "byUrl": w.authorUrl })).collect::<Vec<_>>(),
        "localLabel": v.localLabel,
        "connectLabel": v.connectLabel,
        "connectedLabel": v.connectedLabel,
        "listeningLabel": v.listeningLabel,
        "notConnectedLabel": v.notConnectedLabel,
        "runLabel": v.runLabel,
        "verifyLabel": v.verifyLabel,
        "baseUrlLabel": v.baseUrlLabel,
        "anyKeyLabel": v.anyKeyLabel,
        "modelIdLabel": v.modelIdLabel,
        "testLabel": v.testLabel,
        "stayOpenLabel": v.stayOpenLabel,
        "askLabel": v.askLabel,
        "secondTabLabel": v.secondTabLabel,
        "copyLabel": v.copyLabel,
        "copiedLabel": v.copiedLabel,
        "macLabel": v.macLabel,
        "windowsLabel": v.windowsLabel,
    })
}

/// Allocate `len` bytes the host writes a request into.
#[no_mangle]
pub extern "C" fn holo_alloc(len: i32) -> i32 {
    let mut buffer = Vec::<u8>::with_capacity(len.max(0) as usize);
    let pointer = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    pointer as i32
}

/// Release a buffer `holo_alloc` or `holo_run` handed out.
#[no_mangle]
pub extern "C" fn holo_free(pointer: i32, len: i32) {
    if pointer != 0 && len > 0 {
        // SAFETY: only pointers this module allocated with the same length are passed back.
        unsafe { drop(Vec::from_raw_parts(pointer as *mut u8, len as usize, len as usize)) };
    }
}

/// The `len` bytes the host wrote at a pointer from `holo_alloc`.
fn input<'a>(pointer: i32, len: i32) -> &'a [u8] {
    // SAFETY: the host wrote `len` bytes at a pointer from `holo_alloc`.
    unsafe { std::slice::from_raw_parts(pointer as *const u8, len.max(0) as usize) }
}

/// `(pointer << 32) | len` of bytes the host must free, or -1 for none.
fn hand_out(output: Option<Vec<u8>>) -> i64 {
    let Some(output) = output else { return -1 };
    let out_len = output.len() as i64;
    let mut output = output.into_boxed_slice();
    let out_pointer = output.as_mut_ptr() as i64;
    std::mem::forget(output);
    (out_pointer << 32) | out_len
}

/// Run one request; the response is UTF-8 JSON.
#[no_mangle]
pub extern "C" fn holo_run(pointer: i32, len: i32) -> i64 {
    hand_out(Some(run(input(pointer, len)).to_string().into_bytes()))
}

/// Seal a record of the visitor's work (FA-93): the input is the 32-byte key, then the plaintext; the output
/// is nonce ‖ ciphertext ‖ tag, its nonce convergent (seal_private). Bytes in and out, with no JSON or base64
/// between, since every record the workspace stores and reads passes here.
#[no_mangle]
pub extern "C" fn holo_seal(pointer: i32, len: i32) -> i64 {
    let input = input(pointer, len);
    hand_out((input.len() >= 32).then(|| seal_private(&ChaCha, &input[..32], &input[32..])).map(|(n, s)| [&n[..], &s].concat()))
}

/// Open one: the input is the key, then nonce ‖ ciphertext ‖ tag; the output is the plaintext, or none when it
/// does not authenticate.
#[no_mangle]
pub extern "C" fn holo_open(pointer: i32, len: i32) -> i64 {
    let input = input(pointer, len);
    hand_out((input.len() >= 44).then(|| ChaCha.open(&input[..32], &input[32..44], &input[44..])).flatten())
}
