import React from "react";
import { Server } from "lucide-react";
import {
  PrismCard,
  PrismCardHeader,
  PrismBadge,
  PrismStateIndicator,
} from "./PrismCardPrimitives";
import { CustomTagState } from "./stateTypes";

interface PrismEnableNitroProps {
  state?: CustomTagState;
}

export const PrismEnableNitro: React.FC<PrismEnableNitroProps> = ({ state }) => {
  const isPending = state === "pending";
  const isAborted = state === "aborted";
  const headline = isPending
    ? "Adding Nitro server layer"
    : isAborted
      ? "Nitro server layer setup aborted"
      : "Added Nitro server layer";
  return (
    <PrismCard accentColor="emerald" state={state}>
      <PrismCardHeader icon={<Server size={15} />} accentColor="emerald">
        <PrismBadge color="emerald">Server layer</PrismBadge>
        <span className="text-sm font-medium text-foreground">{headline}</span>
        {state && (
          <PrismStateIndicator state={state} abortedLabel="Did not finish" />
        )}
      </PrismCardHeader>
      {!isPending && !isAborted && (
        <div className="px-3 pb-3">
          <p className="text-xs text-muted-foreground leading-snug">
            API routes can now live under{" "}
            <code className="font-mono text-[11px] px-1 py-0.5 rounded bg-muted">
              server/routes/api/
            </code>
          </p>
        </div>
      )}
    </PrismCard>
  );
};
