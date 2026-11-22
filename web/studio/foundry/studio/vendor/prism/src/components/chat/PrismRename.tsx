import type React from "react";
import type { ReactNode } from "react";
import { FileEdit } from "lucide-react";
import {
  PrismCard,
  PrismCardHeader,
  PrismBadge,
  PrismFilePath,
  PrismDescription,
} from "./PrismCardPrimitives";
import { CustomTagState } from "./stateTypes";

interface PrismRenameProps {
  children?: ReactNode;
  node?: any;
  from?: string;
  to?: string;
}

export const PrismRename: React.FC<PrismRenameProps> = ({
  children,
  node,
  from: fromProp,
  to: toProp,
}) => {
  const from = fromProp || node?.properties?.from || "";
  const to = toProp || node?.properties?.to || "";
  const state = node?.properties?.state as CustomTagState;

  const fromFileName = from ? from.split("/").pop() : "";
  const toFileName = to ? to.split("/").pop() : "";

  const displayTitle =
    fromFileName && toFileName
      ? `${fromFileName} → ${toFileName}`
      : fromFileName || toFileName || "";

  return (
    <PrismCard accentColor="amber" state={state}>
      <PrismCardHeader icon={<FileEdit size={15} />} accentColor="amber">
        {displayTitle && (
          <span className="font-medium text-sm text-foreground truncate">
            {displayTitle}
          </span>
        )}
        <PrismBadge color="amber">Rename</PrismBadge>
      </PrismCardHeader>
      {from && <PrismFilePath path={`From: ${from}`} />}
      {to && <PrismFilePath path={`To: ${to}`} />}
      {children && <PrismDescription>{children}</PrismDescription>}
    </PrismCard>
  );
};
