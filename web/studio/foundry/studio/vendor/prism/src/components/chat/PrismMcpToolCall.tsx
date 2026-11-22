import React, { useMemo, useState } from "react";
import { Wrench } from "lucide-react";
import { useTranslation } from "react-i18next";
import { CodeHighlight } from "./CodeHighlight";
import {
  PrismCard,
  PrismCardHeader,
  PrismBadge,
  PrismExpandIcon,
  PrismCardContent,
  PrismStateIndicator,
} from "./PrismCardPrimitives";
import { CustomTagState } from "./stateTypes";

interface PrismMcpToolCallProps {
  node?: any;
  children?: React.ReactNode;
  /** Raw result string once the paired result arrives; undefined while pending. */
  resultContent?: string;
  /**
   * When set, the card is a merged call+result card: it shows a spinner/
   * checkmark and a Result section. When undefined, it renders call-only
   * (legacy messages that predate call-id pairing).
   */
  state?: CustomTagState;
  /** The result is a tool error; shows a failure label. */
  isError?: boolean;
}

function prettyJson(raw: string): string {
  try {
    return JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    return raw;
  }
}

// Tool results can be plain strings, not just JSON, so highlight as text when
// the content does not parse as JSON.
function formatResult(raw: string): {
  text: string;
  language: "json" | "text";
} {
  try {
    return { text: JSON.stringify(JSON.parse(raw), null, 2), language: "json" };
  } catch {
    return { text: raw, language: "text" };
  }
}

export const PrismMcpToolCall: React.FC<PrismMcpToolCallProps> = ({
  node,
  children,
  resultContent,
  state,
  isError,
}) => {
  const { t } = useTranslation("chat");
  const serverName: string = node?.properties?.serverName || "";
  const toolName: string = node?.properties?.toolName || "";
  const autoApprovedReason: string = node?.properties?.autoApprovedReason || "";
  const [expanded, setExpanded] = useState(false);

  const merged = state !== undefined;
  const raw = typeof children === "string" ? children : String(children ?? "");

  const prettyInput = useMemo(
    () => (expanded ? prettyJson(raw) : ""),
    [expanded, raw],
  );
  const result = useMemo(
    () =>
      expanded && resultContent !== undefined
        ? formatResult(resultContent)
        : null,
    [expanded, resultContent],
  );

  return (
    <PrismCard
      accentColor="blue"
      state={state}
      isExpanded={expanded}
      onClick={() => setExpanded((v) => !v)}
    >
      <PrismCardHeader icon={<Wrench size={15} />} accentColor="blue">
        <PrismBadge color="blue">{merged ? "Tool" : "Tool Call"}</PrismBadge>
        {serverName && (
          <span className="text-xs px-2 py-0.5 rounded-full bg-blue-50 dark:bg-blue-950/40 text-blue-700 dark:text-blue-300 ring-1 ring-inset ring-blue-200 dark:ring-blue-800">
            {serverName}
          </span>
        )}
        {toolName && (
          <span className="text-xs px-2 py-0.5 rounded-full bg-muted/50 text-muted-foreground ring-1 ring-inset ring-border">
            {toolName}
          </span>
        )}
        {autoApprovedReason && (
          <span className="text-xs px-2 py-0.5 rounded-full bg-green-50 dark:bg-green-950/40 text-green-700 dark:text-green-300 ring-1 ring-inset ring-green-200 dark:ring-green-800 flex-shrink-0">
            {t("autoApproved")}
          </span>
        )}
        <div className="ml-auto flex items-center gap-2 flex-shrink-0">
          {merged && (
            <PrismStateIndicator
              state={state}
              pendingLabel="Running"
              abortedLabel={isError ? "Failed" : "No result"}
            />
          )}
          <PrismExpandIcon isExpanded={expanded} />
        </div>
      </PrismCardHeader>
      {autoApprovedReason && (
        <div className="px-3 pb-2 -mt-1 text-xs text-green-700 dark:text-green-300 whitespace-pre-wrap break-words">
          {autoApprovedReason}
        </div>
      )}
      <PrismCardContent isExpanded={expanded}>
        {merged ? (
          <>
            <div className="text-[11px] font-semibold text-muted-foreground mb-1">
              Input
            </div>
            <CodeHighlight className="language-json">
              {prettyInput}
            </CodeHighlight>
            <div className="text-[11px] font-semibold text-muted-foreground mt-3 mb-1">
              Result
            </div>
            {result ? (
              <CodeHighlight className={`language-${result.language}`}>
                {result.text}
              </CodeHighlight>
            ) : resultContent !== undefined ? (
              <CodeHighlight className="language-json">{""}</CodeHighlight>
            ) : (
              <div className="text-xs text-muted-foreground italic">
                {state === "aborted" ? "No result." : "Running…"}
              </div>
            )}
          </>
        ) : (
          <CodeHighlight className="language-json">{prettyInput}</CodeHighlight>
        )}
      </PrismCardContent>
    </PrismCard>
  );
};
