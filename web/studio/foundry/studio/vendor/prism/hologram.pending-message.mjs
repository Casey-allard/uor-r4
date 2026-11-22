// hologram-forge (ours, not upstream's; see VENDORED.md): when a sent prompt stands in for its message
// (hologram.screens/PendingMessage.tsx), and what the composer does with its words meanwhile. Checked without
// a browser by tools/instant_user_message.test.mjs.

/** Only a text-only prompt is shown early: its stored message is exactly its words. */
export function pendingUserMessageFor({ prompt, attachmentCount }) {
  const text = String(prompt ?? "");
  return attachmentCount === 0 && text.trim() !== "" ? text : null;
}

/**
 * A pending prompt is shown until the conversation is longer than it was at the send: the host's first
 * chunk carries the stored user message, and from then on the conversation shows it. A paused queue will
 * not run the prompt, so there the queue's own row stands for it.
 */
export function pendingUserMessageShown(pending, messageCount, queuePaused = false) {
  return !queuePaused && !!pending && typeof pending.prompt === "string" && pending.prompt !== "" && messageCount <= pending.messageCount;
}

/** After a refused or failed send the composer takes its words back, unless something was typed since. */
export function restoredDraft(current, submitted) {
  return current === "" ? submitted : current;
}
