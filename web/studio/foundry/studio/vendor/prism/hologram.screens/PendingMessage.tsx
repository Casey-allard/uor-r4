// hologram-forge (ours, not upstream's; see VENDORED.md): the message a send shows before the host has
// stored it. A turn is admitted through the durable chat queue (two guarded commits) before its first chunk
// carries the user's message back, some 200 to 400 ms after Enter. Until then the conversation shows the
// prompt here, drawn by upstream's own ChatMessage so the stored message replaces it without a change on
// screen. It is presentation only: nothing is written to the conversation's own state, so a submission the
// host refuses leaves no message behind, and the composer gets its words back (hologram.pending-message.mjs
// holds the decisions).
import { useEffect } from "react";
import { atom, useAtomValue, useSetAtom, type createStore } from "jotai";
import { chatMessagesByIdAtom } from "@/atoms/chatAtoms";
import ChatMessage from "@/components/chat/ChatMessage";
import { useChatMessageCount } from "@/hooks/useChatMessages";
import { useChatStreamState } from "@/hooks/useChatStream";
import type { Message } from "@/ipc/types";
import { pendingUserMessageShown, type PendingUserMessage as Pending } from "../hologram.pending-message.mjs";

type Store = ReturnType<typeof createStore>;
const pendingByChatAtom = atom<Map<number, Pending>>(new Map());

/** A text-only prompt was sent: show it now. `messageCount` is the conversation's length at that moment. */
export function showPendingUserMessage(store: Store, chatId: number, prompt: string): void {
  const messageCount = store.get(chatMessagesByIdAtom).get(chatId)?.length ?? 0;
  store.set(pendingByChatAtom, (pending) => new Map(pending).set(chatId, { prompt, messageCount }));
}

/** The submission was refused or failed: there is no message to stand in for. */
export function clearPendingUserMessage(store: Store, chatId: number): void {
  store.set(pendingByChatAtom, (pending) => {
    if (!pending.has(chatId)) return pending;
    const next = new Map(pending);
    next.delete(chatId);
    return next;
  });
}

/** The prompt to show for this chat, or null: none sent, or the conversation already holds it. */
export function usePendingUserMessage(chatId: number | null | undefined): string | null {
  const pending = useAtomValue(pendingByChatAtom).get(chatId ?? -1) ?? null;
  const messageCount = useChatMessageCount(chatId);
  const setPending = useSetAtom(pendingByChatAtom);
  const queuePaused = useChatStreamState(chatId ?? undefined)?.queuePaused === true;
  const shown = pendingUserMessageShown(pending, messageCount, queuePaused);
  // Once the conversation has grown past the send, the entry has done its work.
  useEffect(() => {
    if (chatId == null || !pending || shown) return;
    setPending((all) => {
      if (all.get(chatId) !== pending) return all;
      const next = new Map(all);
      next.delete(chatId);
      return next;
    });
  }, [chatId, pending, shown, setPending]);
  return shown ? pending!.prompt : null;
}

/** The sent prompt as the conversation's next message, until the stored one arrives. */
export function PendingUserMessage({ chatId }: { chatId: number | null }) {
  const prompt = usePendingUserMessage(chatId);
  if (prompt === null) return null;
  const message = { id: -1, role: "user", content: prompt, approvalState: null, commitHash: null } as unknown as Message;
  return (
    // Not yet a stored message: it cannot be restored to or acted on.
    <div data-slot="pending-user-message" className="pointer-events-none px-4" inert>
      <ChatMessage message={message} isLastMessage={false} />
    </div>
  );
}
