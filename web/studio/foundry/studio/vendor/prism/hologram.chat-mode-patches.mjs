// FoundrySpecification.browserChatModeControlsBoundary: one shortcut owner.
export const CHAT_MODE_PATCHES = [
  {
    file: "src/components/chat/ChatInput.tsx",
    from: "  useChatModeToggle();",
    to: "  // The browser mode selector owns its canonical keyboard cycle.",
  },
  {
    file: "src/components/chat/HomeChatInput.tsx",
    from: "  useChatModeToggle();",
    to: "  // The browser mode selector owns its canonical keyboard cycle.",
  },
];
