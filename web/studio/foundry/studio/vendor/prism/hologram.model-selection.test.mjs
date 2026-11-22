import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { modelSelectionScope } from "./hologram.model-selection.mjs";
test("mounted and root chat routes retain exact active chat identity", () => {
  for (const [pathname, base] of [
    ["/chat", "/"],
    ["/foundry/chat", "/foundry/"],
    ["/a/b/chat", "/a/b"],
    ["/foundry/chat/", "/foundry"],
  ])
    assert.deepEqual(modelSelectionScope(pathname, base, 7), { kind: "chat", chatId: 7 });
});
test("Home and non-chat routes have only default scope", () => {
  for (const [pathname, base] of [
    ["/", "/"],
    ["/foundry/", "/foundry"],
    ["/foundry/settings", "/foundry/"],
    ["/foundry/chatty", "/foundry"],
  ])
    assert.deepEqual(modelSelectionScope(pathname, base, 7), { kind: "default" });
});
test("malformed chat identity and paths cannot fall through into defaults", () => {
  for (const id of [undefined, null, 0, -1, 1.5, "1", Number.MAX_SAFE_INTEGER + 1])
    assert.deepEqual(modelSelectionScope("/foundry/chat", "/foundry", id), { kind: "unavailable" });
  for (const pathname of ["/foundryish/chat", "/chat", null])
    assert.deepEqual(modelSelectionScope(pathname, "/foundry", 1), { kind: "unavailable" });
});
test("actual picker uses the canonical scope and imported mutations with real browser acceptance", () => {
  const read = (p) => fs.readFileSync(new URL("../../../" + p, import.meta.url), "utf8");
  const picker = read("studio/vendor/prism/hologram.ModelPicker.tsx").replace(/\s+/g, " ");
  for (const text of [
    "modelSelectionScope(",
    "router.options.basepath",
    'scope.kind === "chat"',
    "setChatSelection({ modelSelection:",
    'scope.kind === "default"',
    "disabled={",
  ])
    assert.ok(picker.includes(text), text);
  assert.ok(!picker.includes("chat.messages.length > 0"));
  assert.ok(!picker.includes('pathname === "/chat"'));
  assert.ok(
    read("app/src/FoundrySpecification.lex.tex").includes("name: chatModelPickerScopeBoundary,"),
  );
  assert.ok(
    read(".github/workflows/studio.yml").includes("node tools/chat_model_picker_scope_browser.mjs"),
  );
});
