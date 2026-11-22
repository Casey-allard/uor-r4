import {assertBrowserChatModes,browserChatModeSettings} from "../shell/browser-chat-modes.mjs";
import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import vm from "node:vm";
import { browserSettings, assertBrowserSettingsPatch } from "../shell/privacy-policy.mjs";
import { invokeHost } from "../shell/host-boundary.mjs";
const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
test("fresh browser settings explicitly refuse collection without an identifier", () => {
  for (const value of [undefined, null, {}]) assert.deepEqual(browserSettings(value), { telemetryConsent: "opted_out" });
});
test("legacy settings cannot restore collection or leak a telemetry identifier", () => {
  const value = { telemetryConsent: "opted_in", telemetryUserId: "secret", selectedModel: { name: "model", provider: "provider" }, providerSettings: { owned: true } };
  const result = browserSettings(value);
  assert.equal(result.telemetryConsent, "opted_out"); assert.ok(!Object.hasOwn(result, "telemetryUserId")); assert.deepEqual(result.selectedModel, value.selectedModel); assert.deepEqual(result.providerSettings, value.providerSettings); assert.equal(value.telemetryUserId, "secret");
});
test("enable and malformed consent writes refuse with a fixed typed failure", () => {
  for (const telemetryConsent of ["opted_in", null, true, "secret"]) assert.throws(() => assertBrowserSettingsPatch({ telemetryConsent }), error => error.code === "browserTelemetryUnavailable" && !error.message.includes("secret"));
  for (const patch of [{}, undefined, { telemetryConsent: "opted_out" }, { telemetryConsent: "unset" }, { selectedModel: { name: "owned" } }]) assert.doesNotThrow(() => assertBrowserSettingsPatch(patch));
});
test("actual host settings normalize reads and writes and refuse before storage", async () => {
  const start = host.indexOf('// ---- settings'), end = host.indexOf('// What the renderer takes', start);
  let stored = { telemetryConsent: "opted_in", telemetryUserId: "secret" }, writes = 0;
  const handlers = new Map();
  vm.runInNewContext(host.slice(start, end), { LOCAL_MODEL: "foundry:reader", handlers, browserSettings, assertBrowserSettingsPatch, assertBrowserChatModes, browserChatModeSettings, inference: { settingsGet: async () => stored }, dbReady: {}, changeUserSettings: async (_db, patch, apply) => { writes++; stored = apply(stored, structuredClone(patch)); return stored; } });
  const read = await handlers.get("get-user-settings")(); assert.equal(read.telemetryConsent, "opted_out"); assert.equal(read.telemetryUserId, undefined);
  await assert.rejects(async () => handlers.get("set-user-settings")({ telemetryConsent: "opted_in" }), { code: "browserTelemetryUnavailable" }); assert.equal(writes, 0);
  const saved = await handlers.get("set-user-settings")({ telemetryUserId: "secret replacement", autoExpandPreviewPanel: false });
  assert.equal(writes, 1); assert.equal(saved.telemetryConsent, "opted_out"); assert.equal(saved.telemetryUserId, undefined); assert.equal(stored.autoExpandPreviewPanel, false);
  const reset = await handlers.get("set-user-settings")({ telemetryConsent: "unset" }); assert.equal(reset.telemetryConsent, "opted_out"); assert.equal(writes, 2);
});
test("typed host envelope preserves the policy refusal and never forwards private error text", async () => {
  const result = await invokeHost("set-user-settings", [{}], () => { throw Object.assign(new Error("secret provider prompt"), { code: "browserTelemetryUnavailable" }); }, null);
  assert.equal(result.ok, false); assert.equal(result.error.code, "browserTelemetryUnavailable"); assert.equal(result.error.kind, "validation"); assert.ok(!JSON.stringify(result).includes("secret"));
});
test("the actual browser projection and its independent SDK browser gates are wired", () => {
  const screens = fs.readFileSync(new URL("../vendor/prism/hologram.screens.mjs", import.meta.url), "utf8"), typecheck = fs.readFileSync(new URL("../vendor/prism/hologram.typecheck.mjs", import.meta.url), "utf8"), vite = fs.readFileSync(new URL("../vendor/prism/vite.shell.config.mts", import.meta.url), "utf8");
  assert.match(screens, /return privacySource\(rel, out\)/); assert.match(typecheck, /screenSource\(file, text\)/); assert.match(vite, /screenSource\(file, code\)/);
  const parser = fs.readFileSync(new URL("./shell_parse.mjs", import.meta.url), "utf8"); assert.match(parser, /"privacy-policy.mjs": "mjs"/);
  const workflow = fs.readFileSync(new URL("../../.github/workflows/studio.yml", import.meta.url), "utf8"); assert.match(workflow, /node tools\/privacy_projection.mjs/); assert.match(workflow, /node tools\/privacy_browser.mjs/); assert.match(workflow, /node tools\/privacy_shell_browser.mjs/);
});
