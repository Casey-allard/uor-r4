// Chat and Plan show the build screen with its preview collapsed, and suggestions above the composer:
// the decisions (vendor/prism/hologram.plain-chat-data.mjs) and the patches that apply them
// (vendor/prism/hologram.plain-chat-patches.mjs). The browser half is tools/plain_chat_layout_browser.mjs.
import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { chatExecutionPolicy } from "../shell/chat-execution.mjs";
import { chatModeCatalog } from "../shell/chat-mode-catalog.mjs";
import {
  PLAIN_CHAT_MODES,
  PLAIN_CHAT_SUGGESTIONS,
  PLAIN_CHAT_WORDS,
  isPlainChatMode,
  plainChatLayout,
  plainChatSuggestions,
  rememberChatMode,
  rememberedChatMode,
  showPlainChatSuggestions,
} from "../vendor/prism/hologram.plain-chat-data.mjs";
import { PLAIN_CHAT_PATCHES } from "../vendor/prism/hologram.plain-chat-patches.mjs";
import { screenSource } from "../vendor/prism/hologram.screens.mjs";

const prism = (p) => fileURLToPath(new URL("../vendor/prism/" + p, import.meta.url));
const read = (p) => fs.readFileSync(prism(p), "utf8");
const built = (p) => screenSource(prism(p), read(p));

test("the layout is chosen by mode: exactly the catalog's read-only modes show the chat alone", () => {
  for (const { id } of chatModeCatalog.modes) {
    // Read-only still holds: the chat-only modes are the ones the host gives no source effects.
    assert.equal(isPlainChatMode(id), !chatExecutionPolicy(id, null).sourceEffects, id);
  }
  assert.deepEqual([...PLAIN_CHAT_MODES], ["ask", "plan"]);
  for (const mode of ["build", "local-agent", "", null, undefined, 7]) assert.equal(isPlainChatMode(mode), false);
  assert.deepEqual(Object.keys(PLAIN_CHAT_WORDS), [...PLAIN_CHAT_MODES]);
  assert.deepEqual(Object.keys(PLAIN_CHAT_SUGGESTIONS), [...PLAIN_CHAT_MODES]);
});

test("a chat opens in the layout it was last seen in, then in its own mode", () => {
  const base = { chatId: 4, isLoading: false, selectedMode: "build", remembered: null };
  assert.equal(plainChatLayout(base), "build");
  assert.equal(plainChatLayout({ ...base, selectedMode: "ask" }), "plain");
  assert.equal(plainChatLayout({ ...base, selectedMode: "plan" }), "plain");
  // While the chat's record is read, the remembered mode decides; once read, the record does.
  assert.equal(plainChatLayout({ ...base, isLoading: true, remembered: "ask" }), "plain");
  assert.equal(plainChatLayout({ ...base, isLoading: true, selectedMode: "ask", remembered: "build" }), "build");
  assert.equal(plainChatLayout({ ...base, isLoading: false, selectedMode: "build", remembered: "ask" }), "build");
  // A chat never seen is not guessed: the default mode is not its mode.
  assert.equal(plainChatLayout({ ...base, isLoading: true, selectedMode: "build", remembered: null }), "unknown");
  assert.equal(plainChatLayout({ ...base, isLoading: true, selectedMode: "ask", remembered: null }), "unknown");
  assert.equal(plainChatLayout({ ...base, chatId: null, selectedMode: "ask" }), "build");
  const kept = new Map(), storage = { getItem: (k) => kept.get(k) ?? null, setItem: (k, v) => void kept.set(k, v) };
  assert.equal(rememberedChatMode(storage, 4), null);
  rememberChatMode(storage, 4, "plan");
  assert.equal(rememberedChatMode(storage, 4), "plan");
  assert.equal(rememberedChatMode(storage, 5), null);
  const refusing = { getItem() { throw new Error("no storage"); }, setItem() { throw new Error("no storage"); } };
  assert.equal(rememberedChatMode(refusing, 4), null);
  assert.doesNotThrow(() => rememberChatMode(refusing, 4, "ask"));
});

test("suggestions are four a mode, short, and differ by mode and by whether the app has rules", () => {
  const seen = new Set();
  for (const mode of PLAIN_CHAT_MODES) {
    for (const hasSpec of [true, false]) {
      const chips = plainChatSuggestions(mode, hasSpec);
      assert.equal(chips.length, 4);
      assert.equal(new Set(chips.map((c) => c.label)).size, 4);
      for (const chip of chips) {
        assert.ok(chip.label.length > 0 && chip.label.length <= 32, chip.label);
        assert.ok(chip.prompt.trim().length >= chip.label.length - 1, chip.label);
      }
      seen.add(JSON.stringify(chips.map((c) => c.label)));
    }
  }
  assert.equal(seen.size, 4, "each mode and each kind of app has its own set");
  assert.notDeepEqual(plainChatSuggestions("ask", true), plainChatSuggestions("plan", true));
  // Only an app with rules is asked about "this app".
  for (const mode of PLAIN_CHAT_MODES) assert.ok(plainChatSuggestions(mode, false).every((c) => !/this app/i.test(c.label + c.prompt)));
  for (const mode of ["build", "local-agent", null]) assert.deepEqual(plainChatSuggestions(mode, true), []);
});

test("suggestions show over an empty conversation or an empty composer, never while a reply streams", () => {
  const show = showPlainChatSuggestions;
  assert.equal(show({ isStreaming: false, messageCount: 0, draft: "" }), true);
  assert.equal(show({ isStreaming: false, messageCount: 0, draft: "half a thought" }), true);
  assert.equal(show({ isStreaming: false, messageCount: 6, draft: "" }), true);
  assert.equal(show({ isStreaming: false, messageCount: 6, draft: "   \n" }), true);
  assert.equal(show({ isStreaming: false, messageCount: 6, draft: "half a thought" }), false);
  assert.equal(show({ isStreaming: true, messageCount: 0, draft: "" }), false);
  assert.equal(show({ isStreaming: true, messageCount: 6, draft: "" }), false);
});

test("every patch applies once, and the build screen is the one screen in every mode", () => {
  assert.ok(read("hologram.screens.mjs").includes("...PLAIN_CHAT_PATCHES,"));
  assert.deepEqual([...new Set(PLAIN_CHAT_PATCHES.map((p) => p.file))].sort(), [
    "src/components/ChatPanel.tsx",
    "src/components/chat/ChatInput.tsx",
    "src/components/chat/MessagesList.tsx",
    "src/pages/chat.tsx",
  ]);
  // screenSource refuses a passage it does not find; a passage that is still there afterwards was ambiguous.
  for (const file of new Set(PLAIN_CHAT_PATCHES.map((p) => p.file))) {
    const out = built(file);
    for (const patch of PLAIN_CHAT_PATCHES.filter((p) => p.file === file)) assert.ok(out.includes(patch.to), `${file}: ${patch.from.split("\n")[0]}`);
  }
  const page = built("src/pages/chat.tsx");
  // One chat panel, in the same place in every mode: it is never unmounted by a change of mode.
  assert.equal(page.split("<ChatPanel").length - 1, 2, "the wide screen's chat panel and the narrow screen's");
  assert.ok(page.includes(`data-chat-layout={plain ? "plain" : "build"}`));
  // The preview: collapsed in Chat and Plan, upstream's own setting in Build, and mounted in every mode as
  // upstream mounts it, so its start and its permission prompt are the preview boundary's, unchanged.
  assert.ok(page.includes("const previewOpen = plain ? plainPreviewOpen : isPreviewOpen;"));
  assert.equal(page.split("<PreviewPanel />").length - 1, 2, "the wide screen's preview and the narrow screen's, unconditionally");
  assert.ok(!page.includes("previewMounted"));
  assert.ok(page.includes("useLayoutEffect(() => {\n    if (previewOpen) {"), "collapsed before the first paint");
  // The suggestions sit directly above the composer, inside the chat section.
  assert.ok(built("src/components/ChatPanel.tsx").includes("<ChatComposer chatId={chatId}><ChatInput chatId={chatId} /></ChatComposer>"));
  const screen = read("hologram.screens/PlainChat.tsx");
  for (const text of ['role="group"', 'aria-label="Suggestions"', "showPlainChatSuggestions({ isStreaming, messageCount, draft })", "plainChatSuggestions(mode, hasSpec)", "overflow-x-auto", "h-12"]) assert.ok(screen.includes(text), text);
  // Nothing here reaches the host's execution boundary: read-only is the host's, unchanged.
  for (const file of ["hologram.screens/PlainChat.tsx", "hologram.plain-chat-patches.mjs", "hologram.plain-chat-data.mjs"]) {
    for (const word of ["streamMessage", "requestedChatMode", "sourceEffects", "setChatMode"]) assert.ok(!read(file).includes(word), `${file}: ${word}`);
  }
  assert.ok(fs.readFileSync(new URL("../../.github/workflows/studio.yml", import.meta.url), "utf8").includes("node tools/plain_chat_layout_browser.mjs"));
  assert.ok(read("VENDORED.md").includes("hologram.plain-chat-patches.mjs"));
});
