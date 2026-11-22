import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { CustomTagState } from "./stateTypes";
import { BookOpen } from "lucide-react";
import {
  PrismCard,
  PrismCardHeader,
  PrismBadge,
  PrismExpandIcon,
  PrismStateIndicator,
  PrismCardContent,
} from "./PrismCardPrimitives";

interface PrismReadGuideProps {
  node: {
    properties: {
      name?: string;
      state?: CustomTagState;
    };
  };
  children: React.ReactNode;
}

export function PrismReadGuide({ node, children }: PrismReadGuideProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const { t } = useTranslation("chat");
  const { name, state } = node.properties;
  const isLoading = state === "pending";
  const isAborted = state === "aborted";

  return (
    <PrismCard
      state={state}
      accentColor="indigo"
      isExpanded={isExpanded}
      onClick={() => setIsExpanded(!isExpanded)}
    >
      <PrismCardHeader icon={<BookOpen size={15} />} accentColor="indigo">
        <PrismBadge color="indigo">{t("guide")}</PrismBadge>
        {name && (
          <span className="text-sm text-foreground truncate">{name}</span>
        )}
        {isLoading && <PrismStateIndicator state="pending" />}
        {isAborted && <PrismStateIndicator state="aborted" />}
        <div className="ml-auto">
          <PrismExpandIcon isExpanded={isExpanded} />
        </div>
      </PrismCardHeader>
      <PrismCardContent isExpanded={isExpanded}>
        {children && (
          <div className="p-3 text-xs font-mono whitespace-pre-wrap max-h-80 overflow-y-auto bg-muted/20 rounded-lg">
            {children}
          </div>
        )}
      </PrismCardContent>
    </PrismCard>
  );
}
