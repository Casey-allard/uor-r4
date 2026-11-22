import type React from "react";
import type { ReactNode } from "react";
import { Trash2 } from "lucide-react";
import {
  PrismCard,
  PrismCardHeader,
  PrismBadge,
  PrismFilePath,
  PrismDescription,
} from "./PrismCardPrimitives";
import { CustomTagState } from "./stateTypes";

interface PrismDeleteProps {
  children?: ReactNode;
  node?: any;
  path?: string;
}

export const PrismDelete: React.FC<PrismDeleteProps> = ({
  children,
  node,
  path: pathProp,
}) => {
  const path = pathProp || node?.properties?.path || "";
  const state = node?.properties?.state as CustomTagState;
  const fileName = path ? path.split("/").pop() : "";

  return (
    <PrismCard accentColor="red" state={state}>
      <PrismCardHeader icon={<Trash2 size={15} />} accentColor="red">
        {fileName && (
          <span className="font-medium text-sm text-foreground truncate">
            {fileName}
          </span>
        )}
        <PrismBadge color="red">Delete</PrismBadge>
      </PrismCardHeader>
      <PrismFilePath path={path} />
      {children && <PrismDescription>{children}</PrismDescription>}
    </PrismCard>
  );
};
