// hologram-forge (ours, not upstream's; see VENDORED.md): a sent prompt is on screen at once
// (hologram.screens/PendingMessage.tsx). Upstream keeps the prompt in the composer until the host accepts
// the turn and draws the message when the host's first chunk arrives. Here the composer hands the prompt to
// the conversation when it is sent, and takes it back if the host refuses it or the send fails; a prompt
// sent while a turn is running, or left in a paused queue, is shown by upstream's queue, not here.
const PENDING = "../../../hologram.screens/PendingMessage";
const DATA = "../../../hologram.pending-message.mjs";

export const PENDING_MESSAGE_PATCHES = [
  {
    file: "src/components/chat/ChatInput.tsx",
    from: `import { ChatErrorBox } from "./ChatErrorBox";`,
    to: `import { ChatErrorBox } from "./ChatErrorBox";
import { clearPendingUserMessage, showPendingUserMessage } from "${PENDING}";
import { pendingUserMessageFor, restoredDraft } from "${DATA}";`,
  },
  {
    file: "src/components/chat/ChatInput.tsx",
    from: `    isAwaitingTurnAcceptanceRef.current = true;
    hideSubmittedAnnotations();
    let wasAccepted = false;`,
    to: `    isAwaitingTurnAcceptanceRef.current = true;
    hideSubmittedAnnotations();
    let wasAccepted = false;
    // The prompt goes to the conversation now, when it is exactly what was typed and nothing is attached;
    // a refused or failed send takes it back into the composer.
    const pendingPrompt = pendingUserMessageFor({
      prompt: currentInput === submittedInputValue ? currentInput : "",
      attachmentCount: attachments.length,
    });
    const withdrawPendingPrompt = () => {
      if (pendingPrompt === null) return;
      clearPendingUserMessage(store, chatId);
      setInputValue((current) => restoredDraft(current, submittedInputValue));
    };
    if (pendingPrompt !== null) {
      showPendingUserMessage(store, chatId, pendingPrompt);
      setInputValue((current) => (current === submittedInputValue ? "" : current));
    }`,
  },
  {
    file: "src/components/chat/ChatInput.tsx",
    from: `      onAcceptanceRejected: () => {
        isAwaitingTurnAcceptanceRef.current = false;
        restoreSubmittedAnnotations();
      },
      onSettled: ({ success, queued }) => {
        isAwaitingTurnAcceptanceRef.current = false;
        if (queued) clearAcceptedPayload();
        if (!success && !queued && !wasAccepted) {
          restoreSubmittedAnnotations();
        }
      },`,
    to: `      onAcceptanceRejected: () => {
        isAwaitingTurnAcceptanceRef.current = false;
        restoreSubmittedAnnotations();
        withdrawPendingPrompt();
      },
      onSettled: ({ success, queued }) => {
        isAwaitingTurnAcceptanceRef.current = false;
        // The host reports every turn as queued for a moment before it runs it: the prompt stays in the
        // conversation through that, and gives way to the queue only if the queue is paused
        // (hologram.screens/PendingMessage.tsx).
        if (queued) clearAcceptedPayload();
        if (!success && !queued && !wasAccepted) {
          restoreSubmittedAnnotations();
          withdrawPendingPrompt();
        }
      },`,
  },
  {
    file: "src/components/chat/MessagesList.tsx",
    from: `import { ModifiedFilesCard } from "./ModifiedFilesCard";`,
    to: `import { ModifiedFilesCard } from "./ModifiedFilesCard";
import { PendingUserMessage, usePendingUserMessage } from "${PENDING}";`,
  },
  {
    // The last thing in the conversation, where the stored message will be.
    file: "src/components/chat/MessagesList.tsx",
    from: `  return (
    <>
      {!isStreaming && showModifiedFilesCard && (`,
    to: `  return (
    <>
      <PendingUserMessage chatId={selectedChatId} />
      {!isStreaming && showModifiedFilesCard && (`,
  },
  {
    file: "src/components/chat/MessagesList.tsx",
    from: `    const { chat: selectedChat } = useChatMode(selectedChatId);`,
    to: `    const { chat: selectedChat } = useChatMode(selectedChatId);
    const pendingPrompt = usePendingUserMessage(selectedChatId);`,
  },
  {
    // A first message is a conversation: the empty state gives way to it at once.
    file: "src/components/chat/MessagesList.tsx",
    from: `    // Render empty state or setup banner
    if (messages.length === 0) {`,
    to: `    // Render empty state or setup banner
    if (messages.length === 0 && pendingPrompt === null) {`,
  },
  {
    file: "src/components/chat/MessagesList.tsx",
    from: `          initialTopMostItemIndex={messages.length - 1}`,
    to: `          initialTopMostItemIndex={Math.max(0, messages.length - 1)}`,
  },
];
