import type { FC, ReactNode } from "react";
import { Globe } from "lucide-react";
import {
  PrismCard,
  PrismCardHeader,
  PrismBadge,
  PrismStateIndicator,
} from "./PrismCardPrimitives";
import { CustomTagState } from "./stateTypes";

interface PrismWebFetchProps {
  children?: ReactNode;
  node?: {
    properties: {
      state?: CustomTagState;
    };
  };
}

export const PrismWebFetch: FC<PrismWebFetchProps> = ({ children, node }) => {
  const state = node?.properties?.state as CustomTagState;

  return (
    <PrismCard state={state} accentColor="blue">
      <PrismCardHeader icon={<Globe size={15} />} accentColor="blue">
        <PrismBadge color="blue">Web Fetch</PrismBadge>
        {state && (
          <PrismStateIndicator
            state={state}
            pendingLabel="Fetching..."
            finishedLabel="Done"
            abortedLabel="Aborted"
          />
        )}
      </PrismCardHeader>
      {children && (
        <div className="px-3 pb-2 text-sm italic text-muted-foreground">
          {children}
        </div>
      )}
    </PrismCard>
  );
};
