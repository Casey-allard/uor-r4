import type React from "react";
import type { ReactNode } from "react";
import { ScanQrCode } from "lucide-react";
import { PrismCard, PrismCardHeader, PrismBadge } from "./PrismCardPrimitives";

interface PrismWebCrawlProps {
  children?: ReactNode;
  node?: any;
}

export const PrismWebCrawl: React.FC<PrismWebCrawlProps> = ({
  children,
  node: _node,
}) => {
  return (
    <PrismCard accentColor="blue">
      <PrismCardHeader icon={<ScanQrCode size={15} />} accentColor="blue">
        <PrismBadge color="blue">Web Crawl</PrismBadge>
      </PrismCardHeader>
      {children && (
        <div className="px-3 pb-2 text-sm italic text-muted-foreground">
          {children}
        </div>
      )}
    </PrismCard>
  );
};
