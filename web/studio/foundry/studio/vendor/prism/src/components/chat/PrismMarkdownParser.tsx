import React, { useDeferredValue, useMemo, useRef } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

import { PrismWrite } from "./PrismWrite";
import { PrismRename } from "./PrismRename";
import { PrismCopy } from "./PrismCopy";
import { PrismDelete } from "./PrismDelete";
import { PrismAddDependency } from "./PrismAddDependency";
import { PrismExecuteSql } from "./PrismExecuteSql";
import { PrismLogs } from "./PrismLogs";
import { PrismGrep } from "./PrismGrep";
import { PrismSearchChats } from "./PrismSearchChats";
import { PrismReadChat } from "./PrismReadChat";
import { PrismExploreCode } from "./PrismExploreCode";
import { PrismExploreChatHistory } from "./PrismExploreChatHistory";
import { PrismAddIntegration } from "./PrismAddIntegration";
import { PrismEnableNitro } from "./PrismEnableNitro";
import { PrismEdit } from "./PrismEdit";
import { PrismSearchReplace } from "./PrismSearchReplace";
import { PrismCodebaseContext } from "./PrismCodebaseContext";
import { PrismThink } from "./PrismThink";
import { CodeHighlight } from "./CodeHighlight";
import { useAtomValue } from "jotai";
import { selectedChatIdAtom } from "@/atoms/chatAtoms";
import {
  useChatStreamPreview,
  useChatStreamState,
} from "@/hooks/useChatStream";
import { isStreamActive } from "@/chat_stream/transition";
import { CustomTagState } from "./stateTypes";
import { PrismOutput } from "./PrismOutput";
import { PrismProblemSummary } from "./PrismProblemSummary";
import { PrismSecurityFinding } from "./PrismSecurityFinding";
import { ipc } from "@/ipc/types";
import { PrismMcpToolCall } from "./PrismMcpToolCall";
import { PrismMcpToolResult } from "./PrismMcpToolResult";
import {
  buildMcpPairing,
  EMPTY_MCP_PAIRING,
  type McpPairing,
  type CustomTagBlock,
} from "./mcpPairing";
import { PrismMcpToolSearch } from "./PrismMcpToolSearch";
import { PrismMcpToolSchema } from "./PrismMcpToolSchema";
import { PrismWebSearchResult } from "./PrismWebSearchResult";
import { PrismWebSearch } from "./PrismWebSearch";
import { PrismWebCrawl } from "./PrismWebCrawl";
import { PrismWebFetch } from "./PrismWebFetch";
import { PrismImageGeneration } from "./PrismImageGeneration";
import { PrismCodeSearchResult } from "./PrismCodeSearchResult";
import { PrismCodeSearch } from "./PrismCodeSearch";
import { PrismRead } from "./PrismRead";
import { PrismListFiles } from "./PrismListFiles";
import { PrismDatabaseSchema } from "./PrismDatabaseSchema";
import { PrismDbTableSchema } from "./PrismDbTableSchema";
import { PrismSupabaseProjectInfo } from "./PrismSupabaseProjectInfo";
import { PrismNeonProjectInfo } from "./PrismNeonProjectInfo";
import { PrismStatus } from "./PrismStatus";
import { PrismCompaction } from "./PrismCompaction";
import { PrismWritePlan } from "./PrismWritePlan";
import { PrismExitPlan } from "./PrismExitPlan";
import { PrismQuestionnaire } from "./PrismQuestionnaire";
import { PrismStepLimit } from "./PrismStepLimit";
import { PrismAppBlueprintCard } from "./PrismAppBlueprintCard";
import { PrismTestAssertionsCard } from "./PrismTestAssertionsCard";
import { PrismReadGuide } from "./PrismReadGuide";
import { PrismScript } from "./PrismScript";
import { PrismGit } from "./PrismGit";
import { PrismSubagent } from "./PrismSubagent";
import { mapActionToButton } from "./ChatInput";
import { SuggestedAction } from "@/lib/schemas";
import { FixAllErrorsButton } from "./FixAllErrorsButton";
import {
  advanceParser,
  type Block,
  getOpenBlock,
  initialParserState,
  parseFullMessage,
  type ParserState,
} from "@/lib/streamingMessageParser";

interface PrismMarkdownParserProps {
  content: string;
  messageId?: number;
  showStreamingPreview?: boolean;
}

const customLink = ({
  node: _node,
  ...props
}: {
  node?: any;
  [key: string]: any;
}) => (
  <a
    {...props}
    onClick={(e) => {
      const url = props.href;
      if (url) {
        e.preventDefault();
        ipc.system.openExternalUrl(url);
      }
    }}
  />
);

export const VanillaMarkdownParser = ({ content }: { content: string }) => {
  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      components={{
        code: CodeHighlight,
        a: customLink,
      }}
    >
      {content}
    </ReactMarkdown>
  );
};

/**
 * Custom component to parse markdown content with Prism-specific tags.
 *
 * The block list is sourced from a component-local incremental parser. Completed
 * blocks keep referential identity across streaming chunks, so React.memo can
 * skip prior blocks and leave only the open trailing block to re-render.
 */
export const PrismMarkdownParser: React.FC<PrismMarkdownParserProps> = ({
  content,
  messageId,
  showStreamingPreview = false,
}) => {
  const chatId = useAtomValue(selectedChatIdAtom);
  const streamState = useChatStreamState(chatId ?? undefined) ?? {
    type: "idle",
  };
  const isStreaming = isStreamActive(streamState);
  const deferredContent = useDeferredValue(content);
  const contentToParse = isStreaming ? deferredContent : content;

  // Component-local parser cache. Closed-block refs stay stable across chunks
  // so MemoClosedBlocks can skip its subtree; only the open trailing block
  // changes shape per chunk. On prefix-mismatch (full-message replace, etc.)
  // we restart from initialParserState — same correctness as a one-shot parse.
  //
  // Note: we write to parserCacheRef inside useMemo. React docs flag this as
  // a side effect during render; in practice the cache is purely advisory and
  // advanceParser is deterministic on (state, content), so the worst case
  // (StrictMode dev double-render, discarded concurrent render) is a wasted
  // re-parse, not a correctness issue.
  const parserCacheRef = useRef<{
    messageId?: number;
    content: string;
    state: ParserState;
  } | null>(null);

  const parserState = useMemo(() => {
    const cached = parserCacheRef.current;
    if (
      cached &&
      cached.messageId === messageId &&
      contentToParse.startsWith(cached.content)
    ) {
      const state = advanceParser(cached.state, contentToParse);
      parserCacheRef.current = { messageId, content: contentToParse, state };
      return state;
    }
    const state = advanceParser(initialParserState(), contentToParse);
    parserCacheRef.current = { messageId, content: contentToParse, state };
    return state;
  }, [messageId, contentToParse]);

  const closedBlocks = parserState.blocks;
  const openBlock = getOpenBlock(parserState);

  // Pair MCP tool-call blocks with their tool-result blocks by call-id so the
  // renderer can collapse the two into one card. Keyed on `closedBlocks`, which
  // only changes when a block closes (not per streamed token), so the scan
  // stays off the streaming hot path.
  const mcpPairing = useMemo(
    () => buildMcpPairing(closedBlocks),
    [closedBlocks],
  );

  // The button is hidden while streaming, so avoid scanning the block list on
  // every chunk. Do the full scan only for settled content.
  const { errorMessages, errorCount, lastErrorIndex } = useMemo(() => {
    if (isStreaming) {
      return EMPTY_ERROR_SCAN;
    }
    const errors: string[] = [];
    let lastIndex = -1;
    closedBlocks.forEach((block, index) => {
      if (
        block.kind === "custom-tag" &&
        block.tag === "prism-output" &&
        block.attributes.type === "error"
      ) {
        const msg = block.attributes.message?.trim();
        if (msg) {
          errors.push(msg);
          lastIndex = index;
        }
      }
    });
    return {
      errorMessages: errors,
      errorCount: errors.length,
      lastErrorIndex: lastIndex,
    };
  }, [closedBlocks, isStreaming]);

  const showFixAll =
    errorCount > 1 && !isStreaming && chatId !== null && chatId !== undefined;

  return (
    <>
      <MemoClosedBlocks
        blocks={closedBlocks}
        lastErrorIndex={lastErrorIndex}
        errorMessages={errorMessages}
        showFixAll={showFixAll}
        chatId={chatId ?? null}
        resultByCallId={mcpPairing.resultByCallId}
        callIds={mcpPairing.callIds}
        isStreaming={isStreaming}
      />
      {openBlock ? renderOpenBlock(openBlock, isStreaming, mcpPairing) : null}
      {showStreamingPreview && chatId !== null && chatId !== undefined && (
        <StreamingPreviewBlocks chatId={chatId} isStreaming={isStreaming} />
      )}
    </>
  );
};

// Stable ref for the "nothing to scan" return path so MemoClosedBlocks's
// memo doesn't invalidate every render during streaming.
const EMPTY_ERROR_SCAN: {
  errorMessages: string[];
  errorCount: number;
  lastErrorIndex: number;
} = { errorMessages: [], errorCount: 0, lastErrorIndex: -1 };

function StreamingPreviewBlocks({
  chatId,
  isStreaming,
}: {
  chatId: number;
  isStreaming: boolean;
}) {
  const previewXml = useChatStreamPreview(chatId);
  const previewBlocks = useMemo<Block[] | null>(() => {
    if (!previewXml) return null;
    return parseFullMessage(previewXml).blocks;
  }, [previewXml]);

  const previewPairing = useMemo(
    () => (previewBlocks ? buildMcpPairing(previewBlocks) : EMPTY_MCP_PAIRING),
    [previewBlocks],
  );

  if (!previewBlocks) return null;

  return (
    <>
      {previewBlocks.map((block) => (
        <React.Fragment key={`preview-${block.id}`}>
          {renderOpenBlock(block, isStreaming, previewPairing)}
        </React.Fragment>
      ))}
    </>
  );
}

function renderBlock(block: Block, isStreaming: boolean): React.ReactNode {
  if (block.kind === "markdown") {
    return block.content ? <MemoMarkdown content={block.content} /> : null;
  }
  return <MemoBlockCustomTag block={block} isStreaming={isStreaming} />;
}

// Render the trailing open block, accounting for MCP pairing: an open
// tool-call shows as a pending card; an open tool-result whose call already
// has a card is hidden (the call card will absorb it once it closes).
function renderOpenBlock(
  block: Block,
  isStreaming: boolean,
  pairing: McpPairing,
): React.ReactNode {
  if (block.kind === "custom-tag") {
    const callId = block.attributes["call-id"];
    if (callId && block.tag === "prism-mcp-tool-call") {
      return (
        <MemoMcpToolPair
          callBlock={block}
          resultBlock={pairing.resultByCallId.get(callId)}
          isStreaming={isStreaming}
        />
      );
    }
    if (
      callId &&
      block.tag === "prism-mcp-tool-result" &&
      pairing.callIds.has(callId)
    ) {
      return null;
    }
  }
  return renderBlock(block, isStreaming);
}

// Render a closed block, collapsing MCP call/result pairs into one card and
// hiding the standalone result block that the call card now renders.
function renderClosedBlock(
  block: Block,
  {
    resultByCallId,
    callIds,
    isStreaming,
  }: {
    resultByCallId: Map<string, CustomTagBlock>;
    callIds: Set<string>;
    isStreaming: boolean;
  },
): React.ReactNode {
  if (block.kind === "custom-tag") {
    const callId = block.attributes["call-id"];
    if (callId && block.tag === "prism-mcp-tool-call") {
      return (
        <MemoMcpToolPair
          callBlock={block}
          resultBlock={resultByCallId.get(callId)}
          isStreaming={isStreaming}
        />
      );
    }
    // Hide the standalone result only when its call is on screen to absorb it;
    // an unmatched result still renders on its own.
    if (callId && block.tag === "prism-mcp-tool-result" && callIds.has(callId)) {
      return null;
    }
  }
  return renderBlock(block, false);
}

// One card for an MCP tool call + its result. Memoizes on both block refs;
// once the result is present the card is "finished" regardless of streaming,
// so isStreaming is only compared while still waiting for a result.
const MemoMcpToolPair = React.memo(
  function MemoMcpToolPair({
    callBlock,
    resultBlock,
    isStreaming,
  }: {
    callBlock: CustomTagBlock;
    resultBlock: CustomTagBlock | undefined;
    isStreaming: boolean;
  }) {
    const isError = resultBlock?.attributes["is-error"] === "true";
    const state: CustomTagState = !resultBlock
      ? isStreaming
        ? "pending"
        : "aborted"
      : isError
        ? "aborted"
        : "finished";
    return (
      <PrismMcpToolCall
        node={{
          properties: {
            serverName: callBlock.attributes.server || "",
            toolName: callBlock.attributes.tool || "",
            autoApprovedReason:
              callBlock.attributes["auto-approved-reason"] || "",
          },
        }}
        resultContent={resultBlock?.content}
        state={state}
        isError={isError}
      >
        {callBlock.content}
      </PrismMcpToolCall>
    );
  },
  (prev, next) =>
    prev.callBlock === next.callBlock &&
    prev.resultBlock === next.resultBlock &&
    (next.resultBlock != null || prev.isStreaming === next.isStreaming),
);

// Memoized wrapper for closed blocks. Memo hits when blocks ref + error
// props are unchanged, so the closed-block subtree is skipped per chunk.
// Closed children also memo on `prev.block === next.block` and skip their
// subtrees on commit chunks.
const MemoClosedBlocks = React.memo(function MemoClosedBlocks({
  blocks,
  lastErrorIndex,
  errorMessages,
  showFixAll,
  chatId,
  resultByCallId,
  callIds,
  isStreaming,
}: {
  blocks: Block[];
  lastErrorIndex: number;
  errorMessages: string[];
  showFixAll: boolean;
  chatId: number | null;
  resultByCallId: Map<string, CustomTagBlock>;
  callIds: Set<string>;
  isStreaming: boolean;
}) {
  // Hoisted once per render rather than allocated per block in the map.
  const mcpCtx = { resultByCallId, callIds, isStreaming };
  return (
    <>
      {blocks.map((block, index) => (
        <React.Fragment key={block.id}>
          {renderClosedBlock(block, mcpCtx)}
          {showFixAll &&
            index === lastErrorIndex &&
            chatId !== null &&
            chatId !== undefined && (
              <div className="mt-3 w-full flex">
                <FixAllErrorsButton
                  errorMessages={errorMessages}
                  chatId={chatId}
                />
              </div>
            )}
        </React.Fragment>
      ))}
    </>
  );
});

// Module-level constants so MemoMarkdown never gets fresh refs for these
// props, which would defeat ReactMarkdown's internal prop-equality checks.
const REMARK_PLUGINS = [remarkGfm];
const MARKDOWN_COMPONENTS = { code: CodeHighlight, a: customLink };

// Memoized markdown piece. Without this, ReactMarkdown re-parses every
// completed segment's text into an AST on every streaming chunk.
const MemoMarkdown = React.memo(function MemoMarkdown({
  content,
}: {
  content: string;
}) {
  return (
    <ReactMarkdown
      remarkPlugins={REMARK_PLUGINS}
      components={MARKDOWN_COMPONENTS}
    >
      {content}
    </ReactMarkdown>
  );
});

// Memoized custom-tag block. The incremental parser preserves the Block
// reference for any completed (closed) tag across streaming patches, so
// referential equality on `block` is sufficient — completed blocks
// short-circuit and skip renderCustomTag entirely.
const MemoBlockCustomTag = React.memo(
  function MemoBlockCustomTag({
    block,
    isStreaming,
  }: {
    block: CustomTagBlock;
    isStreaming: boolean;
  }) {
    return <>{renderCustomTag(block, { isStreaming })}</>;
  },
  (prev, next) =>
    prev.block === next.block &&
    // Completed tags ignore isStreaming (getState returns "finished"
    // regardless), so skip the check to avoid one-time re-renders of every
    // completed tag when streaming ends.
    (prev.block.inProgress === false || prev.isStreaming === next.isStreaming),
);

function getState({
  isStreaming,
  inProgress,
  explicitState,
}: {
  isStreaming?: boolean;
  inProgress?: boolean;
  explicitState?: string;
}): CustomTagState {
  if (
    explicitState === "aborted" ||
    explicitState === "error" ||
    explicitState === "finished" ||
    explicitState === "warning"
  ) {
    return explicitState;
  }
  if (explicitState === "in-progress" || explicitState === "pending") {
    return "pending";
  }
  if (!inProgress) {
    return "finished";
  }
  return isStreaming ? "pending" : "aborted";
}

/**
 * Render a custom tag based on its type
 */
function renderCustomTag(
  block: CustomTagBlock,
  { isStreaming }: { isStreaming: boolean },
): React.ReactNode {
  const { tag, attributes, content, inProgress } = block;

  switch (tag) {
    case "prism-subagent": {
      const subagentChatId = Number(attributes["chat-id"]);
      if (!Number.isSafeInteger(subagentChatId)) return null;
      return (
        <PrismSubagent
          chatId={subagentChatId}
          threadId={attributes["thread-id"] || ""}
          persona={attributes.persona || "agent"}
          taskName={attributes["task-name"] || "Sub-agent task"}
          renderActivity={(xml, activityId) => (
            <PrismMarkdownParser content={xml} messageId={activityId} />
          )}
        />
      );
    }
    case "prism-read":
      return (
        <PrismRead
          node={{
            properties: {
              path: attributes.path || "",
              startLine: attributes.start_line || "",
              endLine: attributes.end_line || "",
              appName: attributes.app_name || "",
            },
          }}
        >
          {content}
        </PrismRead>
      );
    case "prism-git":
      return (
        <PrismGit
          node={{
            properties: {
              ...attributes,
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismGit>
      );
    case "prism-web-search":
      return (
        <PrismWebSearch
          node={{
            properties: {
              query: attributes.query || "",
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismWebSearch>
      );
    case "prism-search-chats":
      return (
        <PrismSearchChats
          node={{
            properties: {
              query: attributes.query || "",
              indexStatus: attributes["index-status"] || "",
              resultCount: attributes["result-count"],
              state: getState({
                isStreaming,
                inProgress,
                explicitState: attributes.state as CustomTagState,
              }),
            },
          }}
        >
          {content}
        </PrismSearchChats>
      );
    case "prism-read-chat":
      return (
        <PrismReadChat
          node={{
            properties: {
              chatId: attributes["chat-id"] || "",
              title: attributes.title || "",
              range: attributes.range || "",
              state: getState({
                isStreaming,
                inProgress,
                explicitState: attributes.state as CustomTagState,
              }),
            },
          }}
        >
          {content}
        </PrismReadChat>
      );
    case "prism-web-crawl":
      return (
        <PrismWebCrawl
          node={{
            properties: {},
          }}
        >
          {content}
        </PrismWebCrawl>
      );
    case "prism-web-fetch":
      return (
        <PrismWebFetch
          node={{
            properties: {
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismWebFetch>
      );
    case "prism-code-search":
      return (
        <PrismCodeSearch
          node={{
            properties: {
              query: attributes.query || "",
              state: getState({
                isStreaming,
                inProgress,
                explicitState: attributes.state,
              }),
              appName: attributes.app_name || "",
            },
          }}
        >
          {content}
        </PrismCodeSearch>
      );
    case "prism-code-search-result":
      return (
        <PrismCodeSearchResult
          node={{
            properties: {},
          }}
        >
          {content}
        </PrismCodeSearchResult>
      );
    case "prism-web-search-result":
      return (
        <PrismWebSearchResult
          node={{
            properties: {
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismWebSearchResult>
      );
    case "think":
      return (
        <PrismThink
          node={{
            properties: {
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismThink>
      );
    // "prism-generate-test" is legacy: no longer emitted, but historical chats
    // still contain it. Both tags carry a path/description and a file body, so
    // the old test cards render as plain file-write cards instead of raw markup.
    case "prism-generate-test":
    case "prism-write":
      return (
        <PrismWrite
          node={{
            properties: {
              path: attributes.path || "",
              description: attributes.description || "",
              state: getState({
                isStreaming,
                inProgress,
                explicitState: attributes.state,
              }),
            },
          }}
        >
          {content}
        </PrismWrite>
      );

    case "prism-rename":
      return (
        <PrismRename
          node={{
            properties: {
              from: attributes.from || "",
              to: attributes.to || "",
            },
          }}
        >
          {content}
        </PrismRename>
      );

    case "prism-copy":
      return (
        <PrismCopy
          node={{
            properties: {
              from: attributes.from || "",
              to: attributes.to || "",
              description: attributes.description || "",
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismCopy>
      );

    case "prism-delete":
      return (
        <PrismDelete
          node={{
            properties: {
              path: attributes.path || "",
            },
          }}
        >
          {content}
        </PrismDelete>
      );

    case "prism-add-dependency":
      return (
        <PrismAddDependency
          node={{
            properties: {
              packages: attributes.packages || "",
            },
          }}
        >
          {content}
        </PrismAddDependency>
      );

    case "prism-execute-sql":
      return (
        <PrismExecuteSql
          node={{
            properties: {
              state: getState({ isStreaming, inProgress }),
              description: attributes.description || "",
            },
          }}
        >
          {content}
        </PrismExecuteSql>
      );

    case "prism-read-logs":
      return (
        <PrismLogs
          node={{
            properties: {
              state: getState({ isStreaming, inProgress }),
              time: attributes.time || "",
              type: attributes.type || "",
              level: attributes.level || "",
              count: attributes.count || "",
            },
          }}
        >
          {content}
        </PrismLogs>
      );

    case "prism-grep":
      return (
        <PrismGrep
          node={{
            properties: {
              state: getState({
                isStreaming,
                inProgress,
                explicitState: attributes.state,
              }),
              query: attributes.query || "",
              include: attributes.include || "",
              exclude: attributes.exclude || "",
              "case-sensitive": attributes["case-sensitive"] || "",
              count: attributes.count || "",
              total: attributes.total || "",
              truncated: attributes.truncated || "",
              appName: attributes.app_name || "",
            },
          }}
        >
          {content}
        </PrismGrep>
      );

    case "prism-explore-chat-history":
      return (
        <PrismExploreChatHistory
          node={{
            properties: {
              state: getState({ isStreaming, inProgress }),
              query: attributes.query || "",
              chats: attributes.chats || "",
              evidence: attributes.evidence || "",
              outcome: attributes.outcome || "",
            },
          }}
        >
          {content}
        </PrismExploreChatHistory>
      );

    case "prism-explore-code":
      return (
        <PrismExploreCode
          node={{
            properties: {
              state: getState({
                isStreaming,
                inProgress,
                explicitState: attributes.state,
              }),
              query: attributes.query || "",
              appName: attributes.app_name || "",
              files: attributes.files || "",
              symbols: attributes.symbols || "",
              indexMs: attributes.index_ms || "",
              searchMs: attributes.search_ms || "",
              truncated: attributes.truncated || "",
            },
          }}
        >
          {content}
        </PrismExploreCode>
      );

    case "prism-add-integration":
      return (
        <PrismAddIntegration
          provider={
            attributes.provider === "neon" || attributes.provider === "supabase"
              ? attributes.provider
              : undefined
          }
          outcome={
            attributes.outcome === "pending" ||
            attributes.outcome === "skipped" ||
            attributes.outcome === "completed" ||
            attributes.outcome === "dismissed"
              ? attributes.outcome
              : undefined
          }
        >
          {content}
        </PrismAddIntegration>
      );

    case "prism-enable-nitro":
      return <PrismEnableNitro state={getState({ isStreaming, inProgress })} />;

    case "prism-edit":
      return (
        <PrismEdit
          node={{
            properties: {
              path: attributes.path || "",
              description: attributes.description || "",
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismEdit>
      );

    case "prism-search-replace":
      return (
        <PrismSearchReplace
          node={{
            properties: {
              path: attributes.path || "",
              description: attributes.description || "",
              state: getState({
                isStreaming,
                inProgress,
                explicitState: attributes.state,
              }),
            },
          }}
        >
          {content}
        </PrismSearchReplace>
      );

    case "prism-codebase-context":
      return (
        <PrismCodebaseContext
          node={{
            properties: {
              files: attributes.files || "",
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismCodebaseContext>
      );

    case "prism-mcp-tool-search":
      return (
        <PrismMcpToolSearch
          node={{
            properties: {
              query: attributes.query || "",
              server: attributes.server || "",
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismMcpToolSearch>
      );

    case "prism-mcp-tool-schema":
      return (
        <PrismMcpToolSchema
          node={{
            properties: {
              tools: attributes.tools || "",
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismMcpToolSchema>
      );
    case "prism-mcp-tool-call":
      return (
        <PrismMcpToolCall
          node={{
            properties: {
              serverName: attributes.server || "",
              toolName: attributes.tool || "",
              autoApprovedReason: attributes["auto-approved-reason"] || "",
            },
          }}
        >
          {content}
        </PrismMcpToolCall>
      );

    case "prism-mcp-tool-result":
      return (
        <PrismMcpToolResult
          node={{
            properties: {
              serverName: attributes.server || "",
              toolName: attributes.tool || "",
            },
          }}
        >
          {content}
        </PrismMcpToolResult>
      );

    case "prism-output":
      return (
        <PrismOutput
          type={attributes.type as "warning" | "error"}
          message={attributes.message}
        >
          {content}
        </PrismOutput>
      );

    case "prism-script":
      return (
        <PrismScript
          node={{
            properties: {
              description: attributes.description || "",
              truncated: attributes.truncated || "",
              executionMs: attributes["execution-ms"] || "",
              fullOutputPath: attributes["full-output-path"] || "",
            },
          }}
        >
          {content}
        </PrismScript>
      );

    case "prism-problem-report":
      return (
        <PrismProblemSummary summary={attributes.summary}>
          {content}
        </PrismProblemSummary>
      );

    case "prism-security-finding":
      return (
        <PrismSecurityFinding title={attributes.title} level={attributes.level}>
          {content}
        </PrismSecurityFinding>
      );

    case "prism-chat-summary":
      // Don't render anything for prism-chat-summary
      return null;

    case "prism-command":
      if (attributes.type) {
        const action = {
          id: attributes.type,
        } as SuggestedAction;
        return <>{mapActionToButton(action)}</>;
      }
      return null;

    case "prism-list-files":
      return (
        <PrismListFiles
          node={{
            properties: {
              directory: attributes.directory || "",
              recursive: attributes.recursive || "",
              include_ignored:
                attributes.include_ignored || attributes.include_hidden || "",
              state: getState({
                isStreaming,
                inProgress,
                explicitState: attributes.state,
              }),
              appName: attributes.app_name || "",
            },
          }}
        >
          {content}
        </PrismListFiles>
      );

    case "prism-database-schema":
      return (
        <PrismDatabaseSchema
          node={{
            properties: {
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismDatabaseSchema>
      );

    case "prism-db-table-schema":
    // Backward compat: old messages used provider-specific tags
    case "prism-supabase-table-schema":
    case "prism-neon-table-schema":
      return (
        <PrismDbTableSchema
          provider={
            tag === "prism-supabase-table-schema"
              ? "Supabase"
              : tag === "prism-neon-table-schema"
                ? "Neon"
                : (attributes.provider as string) || ""
          }
          node={{
            properties: {
              table: attributes.table || "",
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismDbTableSchema>
      );

    case "prism-supabase-project-info":
      return (
        <PrismSupabaseProjectInfo
          node={{
            properties: {
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismSupabaseProjectInfo>
      );

    case "prism-neon-project-info":
      return (
        <PrismNeonProjectInfo
          node={{
            properties: {
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismNeonProjectInfo>
      );

    case "prism-read-guide":
      return (
        <PrismReadGuide
          node={{
            properties: {
              name: attributes.name || "",
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismReadGuide>
      );

    case "prism-image-generation":
      return (
        <PrismImageGeneration
          node={{
            properties: {
              prompt: attributes.prompt || "",
              path: attributes.path || "",
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismImageGeneration>
      );

    case "prism-status":
      return (
        <PrismStatus
          node={{
            properties: {
              title: attributes.title || "Processing...",
              state: getState({
                isStreaming,
                inProgress,
                explicitState: attributes.state,
              }),
            },
          }}
        >
          {content}
        </PrismStatus>
      );

    case "prism-compaction":
      return (
        <PrismCompaction
          node={{
            properties: {
              title: attributes.title || "Compacting conversation",
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismCompaction>
      );

    case "prism-write-plan":
      return (
        <PrismWritePlan
          node={{
            properties: {
              title: attributes.title || "Implementation Plan",
              summary: attributes.summary,
              complete: attributes.complete,
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismWritePlan>
      );

    case "prism-exit-plan":
      return (
        <PrismExitPlan
          node={{
            properties: {
              notes: attributes.notes,
            },
          }}
        />
      );

    case "prism-questionnaire":
      return <PrismQuestionnaire>{content}</PrismQuestionnaire>;

    case "prism-step-limit":
      return (
        <PrismStepLimit
          node={{
            properties: {
              steps: attributes.steps,
              limit: attributes.limit,
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismStepLimit>
      );

    case "prism-app-blueprint":
      return (
        <PrismAppBlueprintCard
          node={{
            properties: {
              "app-name": attributes["app-name"] || "",
              template: attributes.template || "react",
              theme: attributes.theme || "default",
              "design-direction": attributes["design-direction"] || "",
              "primary-color": attributes["primary-color"] || "",
              complete: attributes.complete,
              state: getState({ isStreaming, inProgress }),
            },
          }}
        />
      );

    case "prism-test-assertions":
      return (
        <PrismTestAssertionsCard
          node={{
            properties: {
              "proposal-id": attributes["proposal-id"] || "",
              "request-id": attributes["request-id"] || "",
              status: attributes.status || "proposed",
              "spec-path": attributes["spec-path"] || "",
              state: getState({ isStreaming, inProgress }),
            },
          }}
        >
          {content}
        </PrismTestAssertionsCard>
      );

    default:
      return null;
  }
}
