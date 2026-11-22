import type React from "react";
import { useState, type ReactNode } from "react";
import { ScrollText } from "lucide-react";
import { CustomTagState } from "./stateTypes";
import {
  PrismCard,
  PrismCardHeader,
  PrismBadge,
  PrismExpandIcon,
  PrismStateIndicator,
  PrismCardContent,
} from "./PrismCardPrimitives";

interface PrismMcpToolSchemaProps {
  children?: ReactNode;
  node?: {
    // Comma-separated tool names whose signatures were requested.
    properties?: { tools?: string; state?: CustomTagState };
  };
}

export const PrismMcpToolSchema: React.FC<PrismMcpToolSchemaProps> = ({
  children,
  node,
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const tools = node?.properties?.tools || "";
  const state = node?.properties?.state as CustomTagState;
  const inProgress = state === "pending";
  const resultText = typeof children === "string" ? children.trimEnd() : "";

  return (
    <PrismCard
      state={state}
      accentColor="indigo"
      onClick={() => setIsExpanded(!isExpanded)}
      isExpanded={isExpanded}
    >
      <PrismCardHeader icon={<ScrollText size={15} />} accentColor="indigo">
        <PrismBadge color="indigo">MCP Tool Schema</PrismBadge>
        {!isExpanded && tools && (
          <span className="text-sm text-muted-foreground italic truncate min-w-0">
            {tools}
          </span>
        )}
        {inProgress && (
          <PrismStateIndicator
            state="pending"
            pendingLabel="Loading schema..."
          />
        )}
        <div className="ml-auto">
          <PrismExpandIcon isExpanded={isExpanded} />
        </div>
      </PrismCardHeader>
      <PrismCardContent isExpanded={isExpanded}>
        <div className="text-sm text-muted-foreground space-y-2">
          {tools && (
            <div>
              <span className="text-xs font-medium text-muted-foreground">
                Tools:
              </span>
              <div className="italic mt-0.5 text-foreground">{tools}</div>
            </div>
          )}
          {children && (
            <div>
              <span className="text-xs font-medium text-muted-foreground">
                Signatures:
              </span>
              <pre className="mt-0.5 whitespace-pre-wrap font-mono text-xs text-foreground overflow-x-auto">
                {resultText || children}
              </pre>
            </div>
          )}
        </div>
      </PrismCardContent>
    </PrismCard>
  );
};
