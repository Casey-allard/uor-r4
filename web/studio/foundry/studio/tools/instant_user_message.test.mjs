// A sent prompt is on screen at once: the decisions (vendor/prism/hologram.pending-message.mjs) and the
// patches that apply them (vendor/prism/hologram.pending-message-patches.mjs). The browser half is
// tools/instant_user_message_browser.mjs.
import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { pendingUserMessageFor, pendingUserMessageShown, restoredDraft } from "../vendor/prism/hologram.pending-message.mjs";
import { PENDING_MESSAGE_PATCHES } from "../vendor/prism/hologram.pending-message-patches.mjs";
import { screenSource } from "../vendor/prism/hologram.screens.mjs";

const prism = (p) => fileURLToPath(new URL("../vendor/prism/" + p, import.meta.url));
const read = (p) => fs.readFileSync(prism(p), "utf8").replace(/\r\n/g, "\n");

test("only a text-only prompt stands in for its message", () => {
  assert.equal(pendingUserMessageFor({ prompt: "What does this rule decide?", attachmentCount: 0 }), "What does this rule decide?");
  assert.equal(pendingUserMessageFor({ prompt: "  keep my spaces ", attachmentCount: 0 }), "  keep my spaces ");
  for (const prompt of ["", "   \n", null, undefined]) assert.equal(pendingUserMessageFor({ prompt, attachmentCount: 0 }), null);
  assert.equal(pendingUserMessageFor({ prompt: "with a file", attachmentCount: 1 }), null);
});

test("it shows until the conversation is longer than it was at the send, and never beside its stored message", () => {
  const pending = { prompt: "hi", messageCount: 4 };
  assert.equal(pendingUserMessageShown(pending, 4), true);
  assert.equal(pendingUserMessageShown(pending, 3), true, "a conversation re-read shorter still lacks the message");
  assert.equal(pendingUserMessageShown(pending, 5), false, "the first chunk carries the stored user message");
  assert.equal(pendingUserMessageShown(pending, 6), false);
  assert.equal(pendingUserMessageShown({ prompt: "first", messageCount: 0 }, 0), true);
  for (const none of [null, undefined, { prompt: "", messageCount: 0 }, { messageCount: 0 }]) assert.equal(pendingUserMessageShown(none, 0), false);
  // The host calls every turn queued for a moment before it runs: only a paused queue takes the prompt over.
  assert.equal(pendingUserMessageShown(pending, 4, false), true);
  assert.equal(pendingUserMessageShown(pending, 4, true), false);
});

test("a refused send gives the composer its words back unless something was typed since", () => {
  assert.equal(restoredDraft("", "my question"), "my question");
  assert.equal(restoredDraft("something newer", "my question"), "something newer");
});

test("every patch applies once, and the pending prompt is presentation only", () => {
  assert.ok(read("hologram.screens.mjs").includes("...PENDING_MESSAGE_PATCHES,"));
  assert.deepEqual([...new Set(PENDING_MESSAGE_PATCHES.map((p) => p.file))].sort(), ["src/components/chat/ChatInput.tsx", "src/components/chat/MessagesList.tsx"]);
  for (const patch of PENDING_MESSAGE_PATCHES) assert.equal(read(patch.file).split(patch.from).length, 2, `${patch.file}: ${patch.from.split("\n")[0]}`);
  const input = screenSource(prism("src/components/chat/ChatInput.tsx"), read("src/components/chat/ChatInput.tsx"));
  const list = screenSource(prism("src/components/chat/MessagesList.tsx"), read("src/components/chat/MessagesList.tsx"));
  for (const patch of PENDING_MESSAGE_PATCHES) assert.ok((patch.file.endsWith("ChatInput.tsx") ? input : list).includes(patch.to), patch.from.split("\n")[0]);
  // Shown before the send, withdrawn on every path that ends without a stored message, and left to the queue when queued.
  const at = (text) => { const i = input.indexOf(text); assert.ok(i >= 0, text); return i; };
  assert.ok(at("showPendingUserMessage(store, chatId, pendingPrompt);") < at("await streamMessage({\n      prompt: currentInput,"));
  assert.equal(input.split("withdrawPendingPrompt();").length - 1, 2, "refused, and failed before acceptance");
  assert.ok(input.includes("if (queued) clearAcceptedPayload();\n        if (!success && !queued && !wasAccepted) {"), "a turn reported queued keeps its prompt on screen");
  assert.ok(input.includes('prompt: currentInput === submittedInputValue ? currentInput : "",'), "only the words as typed");
  // The queued path, taken while a turn runs, returns before any of it.
  assert.ok(at("if (isStreaming) {\n      const queued = queueMessage({") < at("const pendingPrompt = pendingUserMessageFor({"));
  assert.ok(list.includes("<PendingUserMessage chatId={selectedChatId} />\n      {!isStreaming && showModifiedFilesCard && ("));
  assert.ok(list.includes("if (messages.length === 0 && pendingPrompt === null) {"));
  // Nothing is written to the conversation's own state, and nothing reaches the host.
  const screen = read("hologram.screens/PendingMessage.tsx");
  assert.ok(!/store\.set\(\s*chatMessagesByIdAtom/.test(screen) && !screen.includes("ipc.") && !screen.includes("streamMessage"));
  assert.ok(screen.includes("pendingUserMessageShown(pending, messageCount, queuePaused)") && screen.includes("inert"));
  assert.ok(fs.readFileSync(new URL("../../.github/workflows/studio.yml", import.meta.url), "utf8").includes("node tools/instant_user_message_browser.mjs"));
  assert.ok(read("VENDORED.md").includes("hologram.pending-message-patches.mjs"));
});
