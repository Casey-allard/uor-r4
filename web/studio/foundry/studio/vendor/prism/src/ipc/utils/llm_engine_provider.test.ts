import { describe, expect, test } from "vitest";

import type {
  LanguageModelV3,
  LanguageModelV3CallOptions,
} from "@ai-sdk/provider";

import type { UserSettings } from "../../lib/schemas";
import {
  createPrismEngine,
  transcribeWithPrismEngine,
} from "./llm_engine_provider";

describe("createPrismEngine", () => {
  test("uses Anthropic messages API for Anthropic engine models", async () => {
    const requests: Array<{
      input: RequestInfo | URL;
      init?: RequestInit;
    }> = [];

    const provider = createPrismEngine({
      apiKey: "prism-pro-key",
      baseURL: "https://engine.example.test/v1",
      prismOptions: {
        enableLazyEdits: true,
        enableSmartFilesContext: true,
        enableWebSearch: false,
      },
      settings: {
        selectedModel: {
          provider: "anthropic",
          name: "claude-sonnet-4-20250514",
          effortLevel: "medium",
        },
      } as unknown as UserSettings,
      fetch: async (input, init) => {
        requests.push({ input, init });

        return new Response(
          JSON.stringify({
            type: "message",
            id: "msg_123",
            model: "claude-sonnet-4-20250514",
            content: [{ type: "text", text: "ok", citations: [] }],
            stop_reason: "end_turn",
            usage: {
              input_tokens: 1,
              output_tokens: 1,
            },
          }),
          {
            headers: { "content-type": "application/json" },
          },
        );
      },
    });

    const model = provider.anthropic("claude-sonnet-4-20250514", {
      providerId: "anthropic",
    }) as LanguageModelV3;

    await model.doGenerate({
      prompt: [
        { role: "system", content: "You are concise." },
        { role: "user", content: [{ type: "text", text: "Hello" }] },
      ],
      providerOptions: {
        "prism-engine": {
          prismAppId: 42,
          prismRequestId: "request-1",
          prismFiles: [{ path: "src/App.tsx", content: "export {}" }],
        },
      },
    } satisfies LanguageModelV3CallOptions);

    expect(requests).toHaveLength(1);
    const request = requests[0];
    expect(String(request.input)).toBe(
      "https://engine.example.test/v1/messages",
    );
    expect(request.init?.headers).toMatchObject({
      authorization: "Bearer prism-pro-key",
      "X-Prism-Request-Id": "request-1:attempt-1",
    });

    const body = JSON.parse(String(request.init?.body));
    expect(body).toMatchObject({
      model: "claude-sonnet-4-20250514",
      messages: [{ role: "user", content: [{ type: "text", text: "Hello" }] }],
      system: [{ type: "text", text: "You are concise." }],
      thinking: { type: "adaptive", display: "summarized" },
      output_config: { effort: "medium" },
      prism_options: {
        app_id: 42,
        enable_lazy_edits: true,
        enable_smart_files_context: true,
        enable_web_search: false,
        files: [{ path: "src/App.tsx", content: "export {}" }],
      },
    });
    expect(body).not.toHaveProperty("prismAppId");
    expect(body).not.toHaveProperty("prismRequestId");
    expect(body).not.toHaveProperty("prismFiles");
    expect(body).not.toHaveProperty("reasoning_effort");
  });

  test("applies query params to Anthropic engine requests", async () => {
    const requests: Array<{
      input: RequestInfo | URL;
      init?: RequestInit;
    }> = [];

    const provider = createPrismEngine({
      apiKey: "prism-pro-key",
      baseURL: "https://engine.example.test/v1",
      queryParams: {
        feature: "anthropic-direct",
        source: "test",
      },
      prismOptions: {},
      settings: {} as UserSettings,
      fetch: async (input, init) => {
        requests.push({ input, init });

        return new Response(
          JSON.stringify({
            type: "message",
            id: "msg_123",
            model: "claude-sonnet-4-20250514",
            content: [{ type: "text", text: "ok", citations: [] }],
            stop_reason: "end_turn",
            usage: {
              input_tokens: 1,
              output_tokens: 1,
            },
          }),
          {
            headers: { "content-type": "application/json" },
          },
        );
      },
    });

    const model = provider.anthropic("claude-sonnet-4-20250514", {
      providerId: "anthropic",
    }) as LanguageModelV3;

    await model.doGenerate({
      prompt: [{ role: "user", content: [{ type: "text", text: "Hello" }] }],
    } satisfies LanguageModelV3CallOptions);

    expect(requests).toHaveLength(1);
    const url = new URL(String(requests[0].input));
    expect(url.origin + url.pathname).toBe(
      "https://engine.example.test/v1/messages",
    );
    expect(url.searchParams.get("feature")).toBe("anthropic-direct");
    expect(url.searchParams.get("source")).toBe("test");
  });

  test("sends stable free quota key separately from attempt request id", async () => {
    const requests: Array<{
      input: RequestInfo | URL;
      init?: RequestInit;
    }> = [];

    const provider = createPrismEngine({
      apiKey: "prism-pro-key",
      baseURL: "https://engine.example.test/v1",
      prismOptions: {},
      settings: {} as UserSettings,
      fetch: async (input, init) => {
        requests.push({ input, init });

        return new Response(
          JSON.stringify({
            id: "chatcmpl_123",
            object: "chat.completion",
            created: 1,
            model: "free-pro",
            choices: [
              {
                index: 0,
                message: { role: "assistant", content: "ok" },
                finish_reason: "stop",
              },
            ],
            usage: {
              prompt_tokens: 1,
              completion_tokens: 1,
              total_tokens: 2,
            },
          }),
          {
            headers: { "content-type": "application/json" },
          },
        );
      },
    });

    const model = provider.freeChatModel("free-pro", {
      providerId: "auto",
    }) as LanguageModelV3;

    await model.doGenerate({
      prompt: [{ role: "user", content: [{ type: "text", text: "Hello" }] }],
      providerOptions: {
        "prism-engine": {
          prismRequestId: "visible-turn-1",
        },
      },
    } satisfies LanguageModelV3CallOptions);

    expect(requests).toHaveLength(1);
    expect(String(requests[0].input)).toBe(
      "https://engine.example.test/v1/free/chat/completions",
    );
    expect(requests[0].init?.headers).toMatchObject({
      "X-Prism-Request-Id": "visible-turn-1:attempt-1",
      "X-Prism-Free-Quota-Key": "visible-turn-1",
    });
  });
});

describe("transcribeWithPrismEngine", () => {
  test("uses the Prism transcription model alias", async () => {
    let request: { input: RequestInfo | URL; init?: RequestInit } | undefined;

    const text = await transcribeWithPrismEngine(
      Buffer.from("audio"),
      "recording.webm",
      "request-1",
      {
        apiKey: "prism-pro-key",
        baseURL: "https://engine.example.test/v1",
        prismOptions: {},
        settings: {} as UserSettings,
        fetch: async (input, init) => {
          request = { input, init };
          return Response.json({ text: "transcribed" });
        },
      },
    );

    expect(text).toBe("transcribed");
    expect(String(request?.input)).toBe(
      "https://engine.example.test/v1/audio/transcriptions",
    );
    const formData = request?.init?.body as FormData;
    expect(formData.get("model")).toBe("prism/transcribe");
  });
});
