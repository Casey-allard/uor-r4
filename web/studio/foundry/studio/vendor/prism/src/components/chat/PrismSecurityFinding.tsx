import React, { useState } from "react";
import { ShieldAlert } from "lucide-react";
import {
  PrismCard,
  PrismCardHeader,
  PrismExpandIcon,
  PrismCardContent,
  type PrismAccentColor,
} from "./PrismCardPrimitives";
import {
  SeverityBadge,
  type SecurityLevel,
} from "@/components/security/severity";
import { VanillaMarkdownParser } from "./PrismMarkdownParser";

const VALID_LEVELS: readonly SecurityLevel[] = [
  "critical",
  "high",
  "medium",
  "low",
];

function isSecurityLevel(value: string | undefined): value is SecurityLevel {
  return value != null && (VALID_LEVELS as readonly string[]).includes(value);
}

// Map a finding's severity onto the card's left-accent color. The exact level
// is still conveyed precisely by the SeverityBadge; the accent is a coarser cue.
const ACCENT_BY_LEVEL: Record<SecurityLevel, PrismAccentColor> = {
  critical: "red",
  high: "red",
  medium: "amber",
  low: "slate",
};

interface PrismSecurityFindingProps {
  title?: string;
  level?: string;
  children?: React.ReactNode;
}

export function PrismSecurityFinding({
  title,
  level,
  children,
}: PrismSecurityFindingProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  const validLevel = isSecurityLevel(level) ? level : undefined;
  const accentColor: PrismAccentColor = validLevel
    ? ACCENT_BY_LEVEL[validLevel]
    : "slate";
  const content = typeof children === "string" ? children : "";

  return (
    <PrismCard
      accentColor={accentColor}
      showAccent
      isExpanded={isExpanded}
      onClick={() => setIsExpanded(!isExpanded)}
      data-testid="security-finding"
    >
      <PrismCardHeader
        icon={<ShieldAlert size={15} />}
        accentColor={accentColor}
      >
        {validLevel && <SeverityBadge level={validLevel} />}
        <span className="font-medium text-sm text-foreground truncate">
          {title || "Security finding"}
        </span>
        <div className="ml-auto">
          <PrismExpandIcon isExpanded={isExpanded} />
        </div>
      </PrismCardHeader>
      <PrismCardContent isExpanded={isExpanded}>
        {content && (
          <div
            className="prose prose-sm dark:prose-invert max-w-none cursor-text"
            onClick={(e) => e.stopPropagation()}
          >
            <VanillaMarkdownParser content={content} />
          </div>
        )}
      </PrismCardContent>
    </PrismCard>
  );
}
