// hologram-forge (ours, not upstream's; see VENDORED.md): Chat and Plan show the build screen with its
// preview collapsed, the state upstream's own toggle already gives (hologram.screens/PlainChat.tsx says
// which chats). Nothing is drawn differently: the same panels, header, messages and composer, so the chat
// stays mounted when the mode changes and Build only has to expand the preview. The preview itself is
// mounted and started exactly as before (FoundrySpecification.previewAutoStartBoundary): it is only
// collapsed, so going to Build is immediate. The collapse happens before the first paint, so the split
// never flashes.
// Applied after the screens' own patches to src/pages/chat.tsx (hologram.screens.mjs).
const CHAT = "src/pages/chat.tsx";
const PLAIN = "../../hologram.screens/PlainChat";

export const PLAIN_CHAT_PATCHES = [
  {
    file: CHAT,
    from: `import { MobileBuild } from "../../hologram.screens/MobileBuild";`,
    to: `import { MobileBuild } from "../../hologram.screens/MobileBuild";
import { useLayoutEffect } from "react";
import { usePlainChatLayout } from "${PLAIN}";`,
  },
  {
    file: CHAT,
    from: `  const isInitialMountRef = useRef(true);`,
    to: `  const isInitialMountRef = useRef(true);
  // Chat and Plan are read-only: the preview starts collapsed there, whatever Build's own setting is, and
  // the toggle still opens it for this visit. Build keeps upstream's setting.
  const chatLayout = usePlainChatLayout(chatId);
  const plain = chatLayout === "plain";
  const [plainPreviewOpen, setPlainPreviewOpen] = useState(false);
  useEffect(() => {
    setPlainPreviewOpen(false);
  }, [chatId, plain]);
  const previewOpen = plain ? plainPreviewOpen : isPreviewOpen;`,
  },
  {
    file: CHAT,
    from: `  useEffect(() => {
    if (isPreviewOpen) {
      ref.current?.expand();
    } else {
      ref.current?.collapse();
    }
  }, [isPreviewOpen]);`,
    to: `  useLayoutEffect(() => {
    if (previewOpen) {
      ref.current?.expand();
    } else {
      ref.current?.collapse();
    }
  }, [previewOpen]);`,
  },
  {
    file: CHAT,
    from: `        <div className="h-full w-full">
          {!isChatPanelHidden && (`,
    to: `        <div className="h-full w-full" data-chat-layout={plain ? "plain" : "build"}>
          {(!isChatPanelHidden || (plain && !plainPreviewOpen)) && (`,
  },
  {
    file: CHAT,
    from: `              isPreviewOpen={isPreviewOpen}
              onTogglePreview={() => {
                setIsPreviewOpen(!isPreviewOpen);`,
    to: `              isPreviewOpen={previewOpen}
              onTogglePreview={() => {
                if (plain) {
                  setPlainPreviewOpen(!plainPreviewOpen);
                  return;
                }
                setIsPreviewOpen(!isPreviewOpen);`,
  },
  {
    // Dragging the handle opens and closes the preview too: the contents follow it.
    file: CHAT,
    from: `        ref={ref}
        id="preview-panel"`,
    to: `        ref={ref}
        id="preview-panel"
        onExpand={() => {
          if (plain) setPlainPreviewOpen(true);
        }}
        onCollapse={() => {
          if (plain) setPlainPreviewOpen(false);
        }}`,
  },
  {
    // The narrow screen shows one pane at a time already; it carries the layout's name for the gates.
    file: CHAT,
    from: `        chat={<ChatPanel chatId={chatId} isPreviewOpen={true} onTogglePreview={() => {}} />}`,
    to: `        chat={<div className="h-full w-full" data-chat-layout={plain ? "plain" : "build"}><ChatPanel chatId={chatId} isPreviewOpen={true} onTogglePreview={() => {}} /></div>}`,
  },
  // The composer, with the suggestions above it in Chat and Plan.
  {
    file: "src/components/ChatPanel.tsx",
    from: `import { ChatInput } from "./chat/ChatInput";`,
    to: `import { ChatInput } from "./chat/ChatInput";
import { ChatComposer } from "../../hologram.screens/PlainChat";`,
  },
  {
    file: "src/components/ChatPanel.tsx",
    from: `                  <ChatInput chatId={chatId} />`,
    to: `                  <ChatComposer chatId={chatId}><ChatInput chatId={chatId} /></ChatComposer>`,
  },
  // The composer and an empty conversation say what the mode does.
  {
    file: "src/components/chat/ChatInput.tsx",
    from: `import { ChatInputControls } from "../ChatInputControls";`,
    to: `import { ChatInputControls } from "../ChatInputControls";
import { usePlainChatPlaceholder } from "../${PLAIN}";`,
  },
  {
    file: "src/components/chat/ChatInput.tsx",
    from: `  const { t } = useTranslation("chat");
  const posthog = usePostHog();`,
    to: `  const { t } = useTranslation("chat");
  const composerPlaceholder = usePlainChatPlaceholder(chatId, t("askPrismToBuild"));
  const posthog = usePostHog();`,
  },
  {
    file: "src/components/chat/ChatInput.tsx",
    from: `              placeholder={t("askPrismToBuild")}`,
    to: `              placeholder={composerPlaceholder}`,
  },
  {
    file: "src/components/chat/MessagesList.tsx",
    from: `import ChatMessage from "./ChatMessage";`,
    to: `import ChatMessage from "./ChatMessage";
import { ChatEmptyWords } from "../${PLAIN}";`,
  },
  {
    file: "src/components/chat/MessagesList.tsx",
    from: `            <div className="flex flex-1 items-center justify-center text-gray-500">
              No messages yet
            </div>`,
    to: `            <div className="flex flex-1 items-center justify-center text-gray-500">
              <ChatEmptyWords chatId={selectedChatId} fallback="No messages yet" />
            </div>`,
  },
];
