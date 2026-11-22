import React, { useEffect, useState } from "react";
import { Globe } from "lucide-react";
import { VanillaMarkdownParser } from "./PrismMarkdownParser";
import { CustomTagState } from "./stateTypes";
import {
  PrismCard,
  PrismCardHeader,
  PrismBadge,
  PrismExpandIcon,
  PrismStateIndicator,
  PrismCardContent,
} from "./PrismCardPrimitives";

interface PrismWebSearchResultProps {
  node?: any;
  children?: React.ReactNode;
}

export const PrismWebSearchResult: React.FC<PrismWebSearchResultProps> = ({
  children,
  node,
}) => {
  const state = node?.properties?.state as CustomTagState;
  const inProgress = state === "pending";
  const [isExpanded, setIsExpanded] = useState(inProgress);

  useEffect(() => {
    if (!inProgress && isExpanded) {
      setIsExpanded(false);
    }
  }, [inProgress]);

  return (
    <PrismCard
      state={state}
      accentColor="blue"
      onClick={() => setIsExpanded(!isExpanded)}
      isExpanded={isExpanded}
    >
      <PrismCardHeader icon={<Globe size={15} />} accentColor="blue">
        <PrismBadge color="blue">Web Search Result</PrismBadge>
        {inProgress && (
          <PrismStateIndicator state="pending" pendingLabel="Loading..." />
        )}
        <div className="ml-auto">
          <PrismExpandIcon isExpanded={isExpanded} />
        </div>
      </PrismCardHeader>
      <PrismCardContent isExpanded={isExpanded}>
        <div className="text-sm text-muted-foreground">
          {typeof children === "string" ? (
            <VanillaMarkdownParser content={children} />
          ) : (
            children
          )}
        </div>
      </PrismCardContent>
    </PrismCard>
  );
};
