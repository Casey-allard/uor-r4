// hologram-forge (ours, replaces src/components/AppShowcaseCard.tsx; see VENDORED.md): a gallery card as a
// gallery shows an app. The app's own first screen, edge to edge at 2:1, then its name, its one tag and one
// sentence of exactly two rows, so every card in a row ends on the same line. The picture, the sentence and
// the tag are the ones the host captured and checked (shell/covers.mjs); an app without them keeps the
// same shape, with its initial on a quiet ground and the two rows held empty.
import { useEffect, useState } from "react";
import type { ListedApp } from "@/ipc/types/app";
import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
import { useAppCard } from "@/hooks/useAppThumbnails";
import { cn } from "@/lib/utils";

interface AppShowcaseCardProps {
  app: ListedApp;
  thumbnailUrl: string | null;
  onClick: (appId: number) => void;
  isSelectionMode?: boolean;
  isSelected?: boolean;
  onToggleSelect?: (appId: number) => void;
}

function getInitial(name: string): string {
  const trimmed = name.trim();
  if (!trimmed) return "?";
  const codePoint = trimmed.codePointAt(0);
  return codePoint ? String.fromCodePoint(codePoint).toUpperCase() : trimmed[0].toUpperCase();
}

export function AppShowcaseCard({
  app,
  thumbnailUrl,
  onClick,
  isSelectionMode = false,
  isSelected = false,
  onToggleSelect,
}: AppShowcaseCardProps) {
  const card = useAppCard(app.id);
  const picture = thumbnailUrl ?? card.thumbnailUrl;
  const [imageBroken, setImageBroken] = useState(false);
  useEffect(() => {
    setImageBroken(false);
  }, [picture]);
  const showImage = picture && !imageBroken;

  const handleClick = () => {
    if (isSelectionMode) onToggleSelect?.(app.id);
    else onClick(app.id);
  };

  return (
    <button
      type="button"
      onClick={handleClick}
      title={app.name}
      data-testid={`app-showcase-card-${app.name}`}
      data-selected={isSelectionMode ? isSelected : undefined}
      role={isSelectionMode ? "checkbox" : undefined}
      aria-checked={isSelectionMode ? isSelected : undefined}
      className="group flex w-full min-w-0 flex-col gap-2.5 rounded-xl text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-4 focus-visible:ring-offset-background"
    >
      <div
        className={cn(
          "relative aspect-[2/1] w-full overflow-hidden rounded-xl border bg-muted transition-colors duration-200",
          isSelectionMode && isSelected ? "border-primary ring-2 ring-primary/40" : "border-border group-hover:border-foreground/20",
        )}
      >
        {showImage ? (
          <img
            src={picture!}
            alt=""
            loading="lazy"
            onError={() => setImageBroken(true)}
            className="absolute inset-0 h-full w-full object-cover object-top transition-transform duration-500 ease-out group-hover:scale-[1.02] motion-reduce:transition-none motion-reduce:group-hover:scale-100"
          />
        ) : (
          <div className="absolute inset-0 flex items-center justify-center">
            <span className="font-display text-3xl font-medium text-muted-foreground">{getInitial(app.name)}</span>
          </div>
        )}
        {isSelectionMode && (
          <div className="pointer-events-none absolute left-2 top-2 flex items-center justify-center rounded bg-background/90 p-1 shadow-sm">
            <Checkbox checked={isSelected} tabIndex={-1} aria-hidden="true" data-testid={`app-showcase-card-${app.name}-checkbox`} />
          </div>
        )}
      </div>
      <div className="flex min-w-0 flex-col gap-2 px-0.5">
        <p className="truncate text-[15px] font-medium leading-snug text-foreground">{app.name}</p>
        {card.tag && (
          <div className="flex flex-wrap gap-1.5">
            <Badge variant="secondary" className="rounded-md font-normal">
              {card.tag}
            </Badge>
          </div>
        )}
        <p
          data-testid={`app-showcase-card-${app.name}-description`}
          className="line-clamp-2 h-[3em] text-sm leading-[1.5] text-muted-foreground [text-wrap:pretty]"
        >
          {card.description ?? ""}
        </p>
      </div>
    </button>
  );
}
