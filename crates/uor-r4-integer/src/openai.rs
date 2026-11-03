use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};

use crate::stack::{
    stack_schema_for, IntegerStackModel, StackChat, StackChatError, StackReply, StackStop,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationFamily {
    Chat,
    Completion,
    Responses,
}

#[derive(Clone, Debug)]
pub struct OpenAiError {
    pub status: u16,
    pub message: String,
    pub error_type: &'static str,
    pub param: Option<String>,
    pub code: Option<String>,
}

impl OpenAiError {
    pub fn new(status: u16, message: impl Into<String>) -> Self {
        let error_type = match status {
            429 => "rate_limit_error",
            500..=599 => "server_error",
            _ => "invalid_request_error",
        };
        Self {
            status,
            message: message.into(),
            error_type,
            param: None,
            code: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_param(mut self, param: impl Into<String>) -> Self {
        self.param = Some(param.into());
        self
    }

    pub fn to_json(&self) -> Value {
        json!({"error": {"message": self.message, "type": self.error_type, "param": self.param, "code": self.code}})
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(501, message).with_code("not_implemented")
    }
}

impl std::fmt::Display for OpenAiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}
impl std::error::Error for OpenAiError {}

#[derive(Clone, Debug, Serialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct UorMetrics {
    pub elapsed_ns: u64,
    pub first_token_ns: Option<u64>,
    pub completion_tokens: usize,
    pub completion_ids: Vec<u32>,
    pub tokens_per_second_milli: Option<u64>,
    pub stop: String,
}

#[derive(Clone, Debug)]
pub struct GenerationOutcome {
    pub reply: StackReply,
    pub usage: Usage,
    pub uor: UorMetrics,
}

#[derive(Debug)]
pub struct GenerationPermit<'a> {
    busy: &'a AtomicBool,
}
impl Drop for GenerationPermit<'_> {
    fn drop(&mut self) {
        self.busy.store(false, Ordering::Release);
    }
}

pub struct OpenAiServing {
    model: IntegerStackModel,
    tokenizer_json: Vec<u8>,
    protocol_version: u8,
    busy: AtomicBool,
}

#[derive(Clone, Debug)]
pub struct OwnedMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct GenerationRequest {
    pub messages: Vec<OwnedMessage>,
    pub max_tokens: usize,
    pub stream: bool,
    pub include_usage: bool,
}

impl GenerationRequest {
    pub fn borrowed_messages(&self) -> Vec<Message<'_>> {
        self.messages
            .iter()
            .map(|message| Message {
                role: &message.role,
                content: &message.content,
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct StreamIdentity {
    pub id: String,
    pub created: u64,
    pub item_id: Option<String>,
}

#[derive(Debug, Default)]
pub struct Utf8Buffer {
    bytes: Vec<u8>,
}
impl Utf8Buffer {
    pub fn push(&mut self, bytes: &[u8]) -> Result<String, OpenAiError> {
        self.bytes.extend_from_slice(bytes);
        match std::str::from_utf8(&self.bytes) {
            Ok(text) => {
                let text = text.to_owned();
                self.bytes.clear();
                Ok(text)
            }
            Err(error) if error.error_len().is_none() => Ok(String::new()),
            Err(error) => Err(OpenAiError::new(
                500,
                format!("tokenizer emitted invalid UTF-8: {error}"),
            )),
        }
    }
    pub fn finish(&mut self) -> Result<(), OpenAiError> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err(OpenAiError::new(
                500,
                "tokenizer ended inside a UTF-8 code point",
            ))
        }
    }
}

impl OpenAiServing {
    pub fn new(
        model: IntegerStackModel,
        tokenizer_json: Vec<u8>,
        protocol_version: u8,
    ) -> Result<Self, StackChatError> {
        let _ = StackChat::from_tokenizer_json(&model, &tokenizer_json, protocol_version)?;
        Ok(Self {
            model,
            tokenizer_json,
            protocol_version,
            busy: AtomicBool::new(false),
        })
    }

    pub fn model_id(&self) -> &'static str {
        "uor-chat"
    }

    pub fn model(&self) -> &IntegerStackModel {
        &self.model
    }

    pub fn protocol(&self) -> Result<DialogueProtocol, OpenAiError> {
        let chat = StackChat::from_tokenizer_json(
            &self.model,
            &self.tokenizer_json,
            self.protocol_version,
        )
        .map_err(chat_error)?;
        Ok(chat.protocol().clone())
    }

    pub fn try_start_generation(&self) -> Result<GenerationPermit<'_>, OpenAiError> {
        self.busy
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| {
                OpenAiError::new(429, "one generation is already in flight").with_code("busy")
            })?;
        Ok(GenerationPermit { busy: &self.busy })
    }

    pub fn health_json(&self) -> Value {
        json!({"ready": true, "model": self.model_id(), "artifact_sha256": self.model.artifact_sha256(), "inflight_limit": 1})
    }

    pub fn model_card(&self) -> Value {
        json!({"id": self.model_id(), "object": "model", "created": unix_seconds(), "owned_by": "uor-r4", "artifact_sha256": self.model.artifact_sha256(), "artifact_schema": stack_schema_for(self.model.shape())})
    }

    pub fn models_json(&self) -> Value {
        json!({"object": "list", "data": [self.model_card()]})
    }

    pub fn capabilities_json(&self) -> Value {
        json!({
            "model": self.model_id(),
            "artifact_sha256": self.model.artifact_sha256(),
            "artifact_schema": stack_schema_for(self.model.shape()),
            "context": self.model.shape().context,
            "decoding": "greedy",
            "inflight_limit": 1,
            "vision": false,
            "body_limits": {"headers_bytes": 32_768, "default_body_bytes": 1_048_576},
            "routes": [
                {"path": "/healthz", "methods": ["GET"], "streaming": false},
                {"path": "/v1/models", "methods": ["GET"], "streaming": false},
                {"path": "/v1/models/{id}", "methods": ["GET"], "streaming": false},
                {"path": "/v1/chat/completions", "methods": ["POST"], "streaming": true},
                {"path": "/v1/completions", "methods": ["POST"], "streaming": true},
                {"path": "/v1/responses", "methods": ["POST"], "streaming": true},
                {"path": "/v1/capabilities", "methods": ["GET"], "streaming": false}
            ],
            "accepted": ["text strings", "text content parts", "max_tokens", "max_completion_tokens", "max_output_tokens (Responses)", "stream", "stream_options.include_usage", "temperature: 0", "n: 1"],
            "unsupported_families": ["embeddings", "files", "uploads", "batches", "fine_tuning", "assistants", "threads", "moderations", "audio", "images", "realtime", "vector_stores"]
        })
    }

    pub fn validate_request(
        &self,
        family: GenerationFamily,
        input: &Value,
    ) -> Result<GenerationRequest, OpenAiError> {
        let raw: RawRequest = serde_json::from_value(input.clone())
            .map_err(|error| OpenAiError::new(400, format!("invalid OpenAI request: {error}")))?;
        if raw.model.as_deref() != Some(self.model_id()) {
            return Err(OpenAiError::new(
                404,
                format!("model {:?} does not exist", raw.model.unwrap_or_default()),
            )
            .with_code("model_not_found")
            .with_param("model"));
        }
        if raw.n.unwrap_or(1) != 1 {
            return Err(OpenAiError::new(400, "only one choice is supported").with_param("n"));
        }
        if raw.temperature.unwrap_or(0.0) != 0.0 {
            return Err(OpenAiError::new(
                400,
                "this artifact serves greedy decoding only; temperature must be 0",
            )
            .with_param("temperature"));
        }
        let stream = raw.stream.unwrap_or(false);
        let include_usage = raw
            .stream_options
            .as_ref()
            .and_then(|options| options.include_usage)
            .unwrap_or(false);
        if include_usage && !stream {
            return Err(
                OpenAiError::new(400, "stream_options.include_usage requires stream=true")
                    .with_param("stream_options"),
            );
        }
        let mut maxima = Vec::new();
        for (name, value) in [
            ("max_tokens", raw.max_tokens),
            ("max_completion_tokens", raw.max_completion_tokens),
            ("max_output_tokens", raw.max_output_tokens),
        ] {
            if let Some(value) = value {
                if family == GenerationFamily::Responses || name != "max_output_tokens" {
                    maxima.push((name, value));
                } else {
                    return Err(OpenAiError::new(
                        400,
                        "max_output_tokens is accepted only by /v1/responses",
                    )
                    .with_param(name));
                }
            }
        }
        if maxima.len() > 1 {
            return Err(
                OpenAiError::new(400, "pass only one generation-token limit")
                    .with_param("max_tokens"),
            );
        }
        let max_tokens = maxima.first().map(|(_, value)| *value).unwrap_or(16);
        if max_tokens == 0 {
            return Err(
                OpenAiError::new(400, "the generation-token limit must be positive")
                    .with_param("max_tokens"),
            );
        }
        match family {
            GenerationFamily::Chat
                if raw.prompt.is_some() || raw.input.is_some() || raw.instructions.is_some() =>
            {
                return Err(OpenAiError::new(
                    400,
                    "prompt, input, and instructions are not chat-completion fields",
                ));
            }
            GenerationFamily::Completion
                if raw.input.is_some()
                    || raw.instructions.is_some()
                    || !raw.messages.is_empty() =>
            {
                return Err(OpenAiError::new(
                    400,
                    "input, instructions, and messages are not legacy-completion fields",
                ));
            }
            GenerationFamily::Responses if raw.prompt.is_some() || !raw.messages.is_empty() => {
                return Err(OpenAiError::new(
                    400,
                    "prompt and messages are not Responses fields; use input",
                ));
            }
            _ => {}
        }
        let mut inputs = raw.messages;
        if family == GenerationFamily::Completion {
            let prompt = raw
                .prompt
                .ok_or_else(|| OpenAiError::new(400, "prompt is required").with_param("prompt"))?;
            inputs = vec![InputMessage {
                role: "user".into(),
                content: prompt,
            }];
        }
        if family == GenerationFamily::Responses {
            let input = raw
                .input
                .ok_or_else(|| OpenAiError::new(400, "input is required").with_param("input"))?;
            inputs = match input {
                Value::String(text) => vec![InputMessage {
                    role: "user".into(),
                    content: Value::String(text),
                }],
                Value::Array(messages) => {
                    serde_json::from_value(Value::Array(messages)).map_err(|error| {
                        OpenAiError::new(400, format!("invalid Responses input: {error}"))
                    })?
                }
                _ => {
                    return Err(OpenAiError::new(
                        400,
                        "Responses input must be a string or message array",
                    )
                    .with_param("input"))
                }
            };
            if let Some(instructions) = raw.instructions {
                inputs.insert(
                    0,
                    InputMessage {
                        role: "system".into(),
                        content: Value::String(instructions),
                    },
                );
            }
        }
        if inputs.is_empty() {
            return Err(
                OpenAiError::new(400, "at least one message is required").with_param("messages")
            );
        }
        let mut messages = Vec::with_capacity(inputs.len());
        for input in inputs {
            if !matches!(input.role.as_str(), "system" | "user" | "assistant") {
                return Err(
                    OpenAiError::new(400, format!("unsupported role {:?}", input.role))
                        .with_param("messages"),
                );
            }
            let content = text_content(&input.content)?;
            if input.role != "assistant" && content.trim().is_empty() {
                return Err(
                    OpenAiError::new(400, "user and system content must not be blank")
                        .with_param("messages"),
                );
            }
            messages.push(OwnedMessage {
                role: input.role,
                content,
            });
        }
        if messages.last().map(|message| message.role.as_str()) != Some("user") {
            return Err(
                OpenAiError::new(400, "the final message must be a user turn")
                    .with_param("messages"),
            );
        }
        let request = GenerationRequest {
            messages,
            max_tokens,
            stream,
            include_usage,
        };
        self.validate_dialogue(&request.borrowed_messages(), max_tokens)?;
        Ok(request)
    }

    fn validate_dialogue(
        &self,
        messages: &[Message<'_>],
        max_tokens: usize,
    ) -> Result<(), OpenAiError> {
        let chat = StackChat::from_tokenizer_json(
            &self.model,
            &self.tokenizer_json,
            self.protocol_version,
        )
        .map_err(chat_error)?;
        let encoded = chat
            .protocol()
            .bind(chat.tokenizer())
            .map_err(|error| OpenAiError::new(400, format!("dialogue protocol: {error}")))?
            .encode_assistant_prefix(messages);
        if encoded.special_token_occurrences != 0 || encoded.skipped_unknown_role != 0 {
            return Err(OpenAiError::new(
                400,
                "messages contain an unknown role or literal special-token injection",
            )
            .with_param("messages"));
        }
        let needed = encoded
            .tokens
            .len()
            .checked_add(max_tokens)
            .ok_or_else(|| {
                OpenAiError::new(413, "the turn overflows the artifact context")
                    .with_code("context_overflow")
            })?;
        if needed > self.model.shape().context {
            return Err(OpenAiError::new(
                413,
                format!(
                    "the turn needs {needed} positions of a {}-position context",
                    self.model.shape().context
                ),
            )
            .with_code("context_overflow"));
        }
        Ok(())
    }

    pub fn generate<F>(
        &self,
        messages: &[Message<'_>],
        max_tokens: usize,
        emit: F,
    ) -> Result<GenerationOutcome, OpenAiError>
    where
        F: FnMut(u32, &[u8]) -> Result<(), OpenAiError>,
    {
        let _permit = self.try_start_generation()?;
        self.generate_with_permit(messages, max_tokens, emit)
    }

    pub(crate) fn generate_with_permit<F>(
        &self,
        messages: &[Message<'_>],
        max_tokens: usize,
        mut emit: F,
    ) -> Result<GenerationOutcome, OpenAiError>
    where
        F: FnMut(u32, &[u8]) -> Result<(), OpenAiError>,
    {
        let started = Instant::now();
        let mut first_token_ns = None;
        let mut emitted_error = None;
        let mut chat = StackChat::from_tokenizer_json(
            &self.model,
            &self.tokenizer_json,
            self.protocol_version,
        )
        .map_err(chat_error)?;
        let bos = chat.protocol().bos_id;
        chat.seed(&[bos]).map_err(chat_error)?;
        let encoded = chat
            .protocol()
            .bind(chat.tokenizer())
            .map_err(|error| OpenAiError::new(400, format!("dialogue protocol: {error}")))?
            .encode_assistant_prefix(messages);
        if encoded.special_token_occurrences != 0 || encoded.skipped_unknown_role != 0 {
            return Err(OpenAiError::new(
                400,
                "messages contain an unknown role or literal special-token injection",
            )
            .with_param("messages"));
        }
        let prompt_tokens = encoded.tokens.len();
        let reply = chat.reply_history_with(&encoded.tokens, max_tokens, |id, bytes| {
            if first_token_ns.is_none() {
                first_token_ns = Some(nanos(started.elapsed()));
            }
            emit(id, bytes).map_err(|error| {
                emitted_error = Some(error);
                StackChatError::Protocol("OpenAI emission aborted".into())
            })
        });
        if let Some(error) = emitted_error {
            return Err(error);
        }
        let reply = reply.map_err(chat_error)?;
        let elapsed_ns = nanos(started.elapsed());
        let completion_tokens = reply.ids.len();
        let tokens_per_second_milli = if elapsed_ns == 0 {
            None
        } else {
            u64::try_from(
                (completion_tokens as u128) * 1_000_000_000_000u128 / (elapsed_ns as u128),
            )
            .ok()
        };
        let usage = Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        };
        let uor = UorMetrics {
            elapsed_ns,
            first_token_ns,
            completion_tokens,
            completion_ids: reply.ids.clone(),
            tokens_per_second_milli,
            stop: reply.stop.name().into(),
        };
        Ok(GenerationOutcome { reply, usage, uor })
    }

    pub fn response_json(&self, family: GenerationFamily, outcome: &GenerationOutcome) -> Value {
        let created = unix_seconds();
        let usage = json!(outcome.usage);
        let metrics = json!(outcome.uor);
        let finish = finish_reason(outcome.reply.stop);
        match family {
            GenerationFamily::Chat => json!({
                "id": request_id("chatcmpl"), "object": "chat.completion", "created": created, "model": self.model_id(),
                "choices": [{"index": 0, "message": {"role": "assistant", "content": outcome.reply.text}, "finish_reason": finish}],
                "usage": usage, "uor": metrics
            }),
            GenerationFamily::Completion => json!({
                "id": request_id("cmpl"), "object": "text_completion", "created": created, "model": self.model_id(),
                "choices": [{"index": 0, "text": outcome.reply.text, "finish_reason": finish}],
                "usage": usage, "uor": metrics
            }),
            GenerationFamily::Responses => {
                self.responses_json_with_identity(outcome, &self.stream_identity(family))
            }
        }
    }

    pub fn stream_identity(&self, family: GenerationFamily) -> StreamIdentity {
        StreamIdentity {
            id: request_id(match family {
                GenerationFamily::Chat => "chatcmpl",
                GenerationFamily::Completion => "cmpl",
                GenerationFamily::Responses => "resp",
            }),
            created: unix_seconds(),
            item_id: (family == GenerationFamily::Responses).then(|| request_id("msg")),
        }
    }

    fn responses_json_with_identity(
        &self,
        outcome: &GenerationOutcome,
        identity: &StreamIdentity,
    ) -> Value {
        json!({
            "id": identity.id, "object": "response", "created_at": identity.created, "model": self.model_id(), "status": "completed",
            "output": [{"id": identity.item_id, "type": "message", "status": "completed", "role": "assistant", "content": [{"type": "output_text", "text": outcome.reply.text}]}],
            "usage": {"prompt_tokens": outcome.usage.prompt_tokens, "completion_tokens": outcome.usage.completion_tokens, "total_tokens": outcome.usage.total_tokens, "input_tokens": outcome.usage.prompt_tokens, "output_tokens": outcome.usage.completion_tokens},
            "uor": outcome.uor
        })
    }

    pub fn stream_role_chunk(&self, family: GenerationFamily, identity: &StreamIdentity) -> Value {
        match family {
            GenerationFamily::Chat => {
                json!({"id": identity.id, "object": "chat.completion.chunk", "created": identity.created, "model": self.model_id(), "choices": [{"index": 0, "delta": {"role": "assistant", "content": ""}, "finish_reason": null}]})
            }
            GenerationFamily::Completion => {
                json!({"id": identity.id, "object": "text_completion", "created": identity.created, "model": self.model_id(), "choices": [{"index": 0, "text": "", "finish_reason": null}]})
            }
            GenerationFamily::Responses => {
                json!({"type": "response.created", "response": {"id": identity.id, "object": "response", "created_at": identity.created, "model": self.model_id(), "status": "in_progress"}})
            }
        }
    }

    pub fn stream_token_chunk(
        &self,
        family: GenerationFamily,
        identity: &StreamIdentity,
        id: u32,
        text: &str,
    ) -> Value {
        let token = json!({"token_id": id});
        match family {
            GenerationFamily::Chat => {
                json!({"id": identity.id, "object": "chat.completion.chunk", "created": identity.created, "model": self.model_id(), "choices": [{"index": 0, "delta": {"content": text}, "finish_reason": null}], "uor": token})
            }
            GenerationFamily::Completion => {
                json!({"id": identity.id, "object": "text_completion", "created": identity.created, "model": self.model_id(), "choices": [{"index": 0, "text": text, "finish_reason": null}], "uor": token})
            }
            GenerationFamily::Responses => {
                json!({"type": "response.output_text.delta", "item_id": identity.item_id, "output_index": 0, "content_index": 0, "delta": text, "uor": token})
            }
        }
    }

    pub fn stream_finish_chunk(
        &self,
        family: GenerationFamily,
        identity: &StreamIdentity,
        outcome: &GenerationOutcome,
    ) -> Value {
        let finish = finish_reason(outcome.reply.stop);
        match family {
            GenerationFamily::Chat => {
                json!({"id": identity.id, "object": "chat.completion.chunk", "created": identity.created, "model": self.model_id(), "choices": [{"index": 0, "delta": {}, "finish_reason": finish}], "uor": {"stop": outcome.uor.stop}})
            }
            GenerationFamily::Completion => {
                json!({"id": identity.id, "object": "text_completion", "created": identity.created, "model": self.model_id(), "choices": [{"index": 0, "text": "", "finish_reason": finish}], "uor": {"stop": outcome.uor.stop}})
            }
            GenerationFamily::Responses => {
                json!({"type": "response.completed", "response": self.responses_json_with_identity(outcome, identity)})
            }
        }
    }

    pub fn stream_usage_chunk(
        &self,
        family: GenerationFamily,
        identity: &StreamIdentity,
        outcome: &GenerationOutcome,
    ) -> Value {
        let object = if family == GenerationFamily::Chat {
            "chat.completion.chunk"
        } else {
            "text_completion"
        };
        json!({"id": identity.id, "object": object, "created": identity.created, "model": self.model_id(), "choices": [], "usage": outcome.usage, "uor": outcome.uor})
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRequest {
    model: Option<String>,
    #[serde(default)]
    messages: Vec<InputMessage>,
    prompt: Option<Value>,
    input: Option<Value>,
    instructions: Option<String>,
    max_tokens: Option<usize>,
    max_completion_tokens: Option<usize>,
    max_output_tokens: Option<usize>,
    stream: Option<bool>,
    stream_options: Option<StreamOptions>,
    temperature: Option<f64>,
    n: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StreamOptions {
    include_usage: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputMessage {
    role: String,
    content: Value,
}

fn text_content(value: &Value) -> Result<String, OpenAiError> {
    match value {
        Value::String(text) if !text.is_empty() => Ok(text.clone()),
        Value::Array(parts) if !parts.is_empty() => {
            let mut text = String::new();
            for part in parts {
                let kind = part.get("type").and_then(Value::as_str).unwrap_or_default();
                if matches!(kind, "image" | "image_url" | "input_image") {
                    return Err(OpenAiError::unsupported("image content parts are not implemented; deterministic R4 image decomposition is a separate unit"));
                }
                if !matches!(kind, "text" | "input_text") {
                    return Err(OpenAiError::unsupported(format!(
                        "content-part type {kind:?} is not implemented"
                    )));
                }
                if part
                    .as_object()
                    .map(|object| object.len() != 2)
                    .unwrap_or(true)
                {
                    return Err(OpenAiError::new(
                        400,
                        "a text content part accepts only type and text",
                    ));
                }
                let piece = part.get("text").and_then(Value::as_str).ok_or_else(|| {
                    OpenAiError::new(400, "a text content part requires a string text field")
                })?;
                text.push_str(piece);
            }
            if text.is_empty() {
                return Err(OpenAiError::new(400, "message content must not be empty"));
            }
            Ok(text)
        }
        _ => Err(OpenAiError::new(
            400,
            "message content must be a nonempty string or text-part array",
        )),
    }
}

fn chat_error(error: StackChatError) -> OpenAiError {
    match error {
        StackChatError::Context { needed, context } => OpenAiError::new(
            413,
            format!("the turn needs {needed} positions of a {context}-position context"),
        )
        .with_code("context_overflow"),
        other => OpenAiError::new(400, other.to_string()),
    }
}

fn finish_reason(stop: StackStop) -> &'static str {
    match stop {
        StackStop::MaximumNewTokens => "length",
        StackStop::Eos | StackStop::ShortCycle { .. } => "stop",
    }
}

fn request_id(prefix: &str) -> String {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    format!("{prefix}-uor-{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn nanos(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}
