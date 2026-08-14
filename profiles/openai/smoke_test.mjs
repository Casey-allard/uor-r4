#!/usr/bin/env node
// #654 phase F — official OpenAI **JS/TS** SDK smoke test against a live R4 server.
//
// The JavaScript counterpart to smoke_test.py: it drives the pinned wire
// surfaces (POST /v1/chat/completions non-streaming and streaming, and
// POST /v1/responses) with the official `openai` npm package and asserts the SDK
// round-trips against a running R4 server. Like the Python script it is NOT wired
// into CI — it needs a server with a compiled model loaded; it is the developer-run
// companion to the deterministic DTO/response fixtures in src/server.rs.
//
// Usage:
//     npm install openai
//     node profiles/openai/smoke_test.mjs --base-url http://127.0.0.1:8080/v1 --model <compiled-model-id>
//
// --base-url/--model default to the UOR_OPENAI_BASE_URL / UOR_OPENAI_MODEL
// environment variables. The API key is a placeholder (the R4 server does not
// authenticate the pinned profile). Exit code 0 = every surface round-tripped.

import OpenAI from "openai";

function arg(name, fallback) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && i + 1 < process.argv.length ? process.argv[i + 1] : fallback;
}

function check(label, ok, detail) {
  console.log(`[${ok ? "PASS" : "FAIL"}] ${label}: ${detail}`);
  return ok;
}

const baseURL = arg("base-url", process.env.UOR_OPENAI_BASE_URL || "http://127.0.0.1:8080/v1");
const model = arg("model", process.env.UOR_OPENAI_MODEL || "uor-r4");
const client = new OpenAI({ baseURL, apiKey: "sk-uor-r4-smoke", maxRetries: 0 });
const results = [];

// 1. Chat Completions, non-streaming.
const chat = await client.chat.completions.create({
  model,
  messages: [{ role: "user", content: "Say hi in one word." }],
});
const choice = chat.choices[0];
results.push(
  check(
    "chat.completions (non-stream)",
    Boolean(choice.message.content) && ["stop", "length"].includes(choice.finish_reason),
    `content=${JSON.stringify(choice.message.content)} finish_reason=${choice.finish_reason} usage.total_tokens=${chat.usage.total_tokens}`,
  ),
);

// 2. Chat Completions, streaming (phase D SSE).
let reconstructed = "";
let sawRole = false;
let streamFinish = null;
const stream = await client.chat.completions.create({
  model,
  messages: [{ role: "user", content: "Say hi in one word." }],
  stream: true,
});
for await (const event of stream) {
  const delta = event.choices[0].delta;
  if (delta.role) sawRole = true;
  if (delta.content) reconstructed += delta.content;
  if (event.choices[0].finish_reason) streamFinish = event.choices[0].finish_reason;
}
results.push(
  check(
    "chat.completions (stream)",
    sawRole && ["stop", "length"].includes(streamFinish),
    `reconstructed=${JSON.stringify(reconstructed)} finish_reason=${streamFinish}`,
  ),
);

// 3. Responses.
const resp = await client.responses.create({ model, input: "Say hi in one word." });
results.push(
  check(
    "responses",
    Boolean(resp.output_text) && ["completed", "incomplete"].includes(resp.status),
    `output_text=${JSON.stringify(resp.output_text)} status=${resp.status} usage.total_tokens=${resp.usage.total_tokens}`,
  ),
);

const ok = results.every(Boolean);
console.log(
  `\n${ok ? "ALL SURFACES ROUND-TRIPPED" : "ONE OR MORE SURFACES FAILED"} (${results.filter(Boolean).length}/${results.length})`,
);
process.exit(ok ? 0 : 1);
