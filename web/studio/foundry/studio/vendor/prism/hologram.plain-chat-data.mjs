// hologram-forge (ours, not upstream's; see VENDORED.md): what the chat-only screen decides, apart from how
// it is drawn (hologram.screens/PlainChat.tsx), so each decision is checked without a browser
// (tools/plain_chat_layout.test.mjs). Chat and Plan are the read-only modes
// (FoundrySpecification.readOnlyChatExecutionBoundary): they change nothing in the app, so their screen is
// the build screen's own chat section alone, without the preview beside it.

/** The modes shown as the chat section alone. Build, and any mode not named here, keeps the build screen. */
export const PLAIN_CHAT_MODES = Object.freeze(["ask", "plan"]);

export function isPlainChatMode(mode) {
  return PLAIN_CHAT_MODES.includes(mode);
}

/** The words of each chat-only mode: the empty conversation's heading and note, the composer's placeholder. */
export const PLAIN_CHAT_WORDS = Object.freeze({
  ask: Object.freeze({
    heading: "Ask about your app",
    note: "Chat answers questions. Nothing in your app changes.",
    placeholder: "Ask anything about your app",
  }),
  plan: Object.freeze({
    heading: "Plan before you build",
    note: "Plan maps the steps. Nothing in your app changes.",
    placeholder: "Describe what you want to plan",
  }),
});

// The suggestions, four a mode: `app` when the chat's app has rules (prism/spec.json), `general` when it has
// none yet. `label` is the chip; `prompt` is what the chip writes into the composer.
const chip = (label, prompt) => Object.freeze({ label, prompt });
export const PLAIN_CHAT_SUGGESTIONS = Object.freeze({
  ask: Object.freeze({
    app: Object.freeze([
      chip("Explain how this app decides", "Explain how this app decides, rule by rule, in plain words."),
      chip("What would break this rule?", "Which inputs would break one of this app's rules? Name the rule and the input."),
      chip("Suggest a guarantee to add", "Suggest one guarantee this app should prove that it does not prove yet, and why."),
      chip("Compare two ways to model this", "Compare two ways to model this app's rules, and say which you would choose."),
    ]),
    general: Object.freeze([
      chip("What is a Prism app?", "What is a Prism app, and what does it prove about itself?"),
      chip("What can a rule decide?", "What kinds of decision can a Prism app's rules make? Give two short examples."),
      chip("What does a guarantee prove?", "What does a guarantee prove about an app, and how is it checked?"),
      chip("Compare two ways to model this", "Compare two ways to model a decision as rules, and say when each fits."),
    ]),
  }),
  plan: Object.freeze({
    app: Object.freeze([
      chip("Plan the next change", "Plan the next change to this app, step by step, before anything is built."),
      chip("List the inputs I still need", "List the inputs this app still needs, with each one's type and allowed values."),
      chip("Draft a rule before building", "Draft one new rule for this app in plain words, with the reason for it."),
      chip("What should always be true?", "What should always be true of this app? Propose guarantees worth proving."),
    ]),
    general: Object.freeze([
      chip("Plan an app from one sentence", "Plan an app from this one sentence: "),
      chip("List the inputs I need", "List the inputs my app needs, with each one's type and allowed values."),
      chip("Draft the rules before building", "Draft the rules for my app in plain words before anything is built."),
      chip("What should always be true?", "What should always be true of my app? Propose guarantees worth proving."),
    ]),
  }),
});

/** The four suggestions for a mode; none for a mode that is not chat-only. */
export function plainChatSuggestions(mode, hasSpec) {
  const forMode = PLAIN_CHAT_SUGGESTIONS[mode];
  return forMode ? (hasSpec ? forMode.app : forMode.general) : [];
}

/** Suggestions show over an empty conversation or an empty composer, and never while a reply streams. */
export function showPlainChatSuggestions({ isStreaming, messageCount, draft }) {
  if (isStreaming) return false;
  return messageCount === 0 || String(draft ?? "").trim() === "";
}

// The mode a chat was last seen in, kept so the screen opens in that layout before the chat's record is
// read: a Chat or Plan chat never paints the split first.
const LAST_MODE = "holo.chat.mode.";
export function rememberedChatMode(storage, chatId) {
  try { return storage.getItem(LAST_MODE + chatId); } catch { return null; }
}
export function rememberChatMode(storage, chatId, mode) {
  try { storage.setItem(LAST_MODE + chatId, String(mode)); } catch { /* storage is a convenience */ }
}

/**
 * The layout a chat opens in: "plain" (the chat alone) or "build", from its own mode once its record is read
 * and until then from the mode it was last seen in. A chat never seen whose record is not read yet is
 * "unknown": it is drawn as Build is until the answer comes.
 */
export function plainChatLayout({ chatId, isLoading, selectedMode, remembered }) {
  if (chatId == null) return "build";
  if (isLoading && !remembered) return "unknown";
  return isPlainChatMode(isLoading ? remembered : selectedMode) ? "plain" : "build";
}
