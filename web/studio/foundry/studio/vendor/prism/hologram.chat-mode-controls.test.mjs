import { CHAT_MODE_PATCHES } from "./hologram.chat-mode-patches.mjs";
import assert from "node:assert/strict";
import fs from "node:fs";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import {
  browserChatMode,
  assertBrowserChatModes,
  browserChatModeSettings,
} from "../../shell/browser-chat-modes.mjs";
import { chatModeCatalog } from "../../shell/chat-mode-catalog.mjs";
test("canonical catalog reproduces exactly and offers only implemented browser modes", () => {
  assert.deepEqual(chatModeCatalog.modes, [
    { id: "build", label: "Build" },
    { id: "ask", label: "Chat" },
    { id: "plan", label: "Plan" },
  ]);
  const r = spawnSync(
    "python3",
    [new URL("../../tools/chat_mode_catalog_projection.py", import.meta.url).pathname, "--check"],
    { encoding: "utf8" },
  );
  assert.equal(r.status, 0, r.stdout + r.stderr);
});
test("explicit mode precedes captured default and absent preferences preserve Build", () => {
  for (const mode of ["build", "ask", "plan"]) {
    assert.equal(browserChatMode(mode, { defaultChatMode: "ask" }), mode);
    assert.equal(browserChatMode(undefined, { defaultChatMode: mode }), mode);
    assert.doesNotThrow(() =>
      assertBrowserChatModes({ selectedChatMode: mode, defaultChatMode: mode }),
    );
  }
  assert.deepEqual(browserChatModeSettings({ selectedChatMode: "ask" }), {
    selectedChatMode: "ask",
    defaultChatMode: "ask",
  });
  assert.equal(
    browserChatModeSettings({ defaultChatMode: "local-agent" }).defaultChatMode,
    "local-agent",
  );
  assert.equal(browserChatMode(undefined, {}), "build");
  assert.equal(browserChatMode(undefined, { selectedChatMode: "plan" }), "plan");
  assert.equal(browserChatMode("ask", { defaultChatMode: "invalid" }), "ask");
});
test("unsupported malformed and null modes fail before writes", () => {
  for (const mode of ["local-agent", "agent", "", {}, [], 7, null]) {
    assert.throws(() => browserChatMode(mode, {}), { code: "invalidChatExecutionMode" });
    assert.throws(() => assertBrowserChatModes({ selectedChatMode: mode }), {
      code: "invalidChatExecutionMode",
    });
    assert.throws(() => assertBrowserChatModes({ defaultChatMode: mode }), {
      code: "invalidChatExecutionMode",
    });
  }
  assert.throws(() => browserChatMode(undefined, { defaultChatMode: "invalid" }), {
    code: "invalidChatExecutionMode",
  });
});
test("actual selector creation settings and mandatory browser bind to canonical ownership", () => {
  const read = (p) => fs.readFileSync(new URL("../../../" + p, import.meta.url), "utf8"),
    host = read("studio/shell/host.js"),
    picker = read("studio/vendor/prism/hologram.ChatModeSelector.tsx");
  assert.ok(!host.includes("...BUILD_MODE"));
  assert.ok(host.includes("assertBrowserChatModes(patch)"));
  assert.ok(host.includes("browserChatMode(initialChatMode,"));
  assert.ok(host.includes("browserChatMode(undefined,"));
  assert.ok(host.includes("initialChatMode: capturedMode"));
  assert.ok(read("studio/shell/project-creation.mjs").includes("chatMode: capturedMode"));
  assert.ok(read("studio/vendor/prism/hologram.screens.mjs").includes("...CHAT_MODE_PATCHES,"));
  for (const x of [
    "chatModeCatalog.modes",
    "modelSelectionScope(",
    "setChatMode(",
    "defaultChatMode: mode",
    "disabled={",
  ])
    assert.ok(picker.includes(x), x);
  for (const patch of CHAT_MODE_PATCHES) {
    const original = read("studio/vendor/prism/" + patch.file);
    assert.equal(original.split(patch.from).length, 2);
    assert.ok(!original.replace(patch.from, patch.to).includes("useChatModeToggle();"));
  }
  assert.equal(CHAT_MODE_PATCHES.length, 2);
  assert.deepEqual(CHAT_MODE_PATCHES.map((p) => p.file).sort(), [
    "src/components/chat/ChatInput.tsx",
    "src/components/chat/HomeChatInput.tsx",
  ]);
  assert.ok(
    !read("studio/vendor/prism/vite.shell.config.mts").includes(
      'out.replace("<ChatModeSelector />", "{null}")',
    ),
  );
  assert.ok(
    read("app/src/FoundrySpecification.lex.tex").includes("name: browserChatModeControlsBoundary,"),
  );
  assert.ok(
    read(".github/workflows/studio.yml").includes(
      "node tools/browser_chat_mode_controls_browser.mjs",
    ),
  );
  assert.ok(read("studio/tools/shell_parse.mjs").includes("browser-chat-modes.mjs"));
});
