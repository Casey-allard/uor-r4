import React from "react";
import { CustomTagState } from "./stateTypes";
import { Database } from "lucide-react";
import {
  PrismCard,
  PrismCardHeader,
  PrismBadge,
  PrismStateIndicator,
} from "./PrismCardPrimitives";

interface PrismDatabaseSchemaProps {
  node: {
    properties: {
      state?: CustomTagState;
    };
  };
  children: React.ReactNode;
}

export function PrismDatabaseSchema({
  node,
  children,
}: PrismDatabaseSchemaProps) {
  const { state } = node.properties;
  const isLoading = state === "pending";
  const content = typeof children === "string" ? children : "";

  return (
    <PrismCard state={state} accentColor="teal">
      <PrismCardHeader icon={<Database size={15} />} accentColor="teal">
        <PrismBadge color="teal">Database Schema</PrismBadge>
        {isLoading && <PrismStateIndicator state="pending" />}
      </PrismCardHeader>
      {content && (
        <div className="px-3 pb-3">
          <div className="p-3 text-xs font-mono whitespace-pre-wrap max-h-60 overflow-y-auto bg-muted/20 rounded-lg">
            {content}
          </div>
        </div>
      )}
    </PrismCard>
  );
}
