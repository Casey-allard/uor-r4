import type React from "react";
import type { ReactNode } from "react";
import { Copy } from "lucide-react";
import {
  PrismCard,
  PrismCardHeader,
  PrismBadge,
  PrismFilePath,
  PrismDescription,
  PrismStateIndicator,
} from "./PrismCardPrimitives";
import { CustomTagState } from "./stateTypes";

interface PrismCopyProps {
  children?: ReactNode;
  node?: any;
}

export const PrismCopy: React.FC<PrismCopyProps> = ({ children, node }) => {
  const from = node?.properties?.from || "";
  const to = node?.properties?.to || "";
  const description = node?.properties?.description || "";
  const state = node?.properties?.state as CustomTagState;

  const toFileName = to ? to.split("/").pop() : "";
  // Hide the "From" line for temp attachment paths (absolute paths) since they
  // show cryptic hash filenames that mean nothing to the user.
  const isTempAttachment =
    /^(\/|[A-Za-z]:\\)/.test(from) || from.includes(".prism/media/");

  return (
    <PrismCard accentColor="teal" state={state}>
      <PrismCardHeader icon={<Copy size={15} />} accentColor="teal">
        {toFileName && (
          <span className="font-medium text-sm text-foreground truncate">
            {toFileName}
          </span>
        )}
        <PrismBadge color="teal">Copy</PrismBadge>
        <span className="ml-auto">
          {state === "pending" && (
            <PrismStateIndicator state="pending" pendingLabel="Copying..." />
          )}
          {state === "aborted" && (
            <PrismStateIndicator state="aborted" abortedLabel="Did not finish" />
          )}
          {state === "finished" && (
            <PrismStateIndicator state="finished" finishedLabel="Copied" />
          )}
        </span>
      </PrismCardHeader>
      {from && !isTempAttachment && <PrismFilePath path={`From: ${from}`} />}
      {to && <PrismFilePath path={`To: ${to}`} />}
      {description && <PrismDescription>{description}</PrismDescription>}
      {children && <PrismDescription>{children}</PrismDescription>}
    </PrismCard>
  );
};
