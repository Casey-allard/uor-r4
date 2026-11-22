// hologram-forge (ours, not upstream's; see VENDORED.md): in place of src/components/ui/back-button.tsx
// (hologram.screens.mjs). The shell's site header carries a breadcrumb to the page above (AppSidebar.tsx),
// as shadcn's dashboard layouts do, so a page draws no back link of its own.
export function BackButton(_props: {
  onClick?: () => void;
  label?: string;
  className?: string;
  fallbackTo?: string;
  "data-testid"?: string;
}) {
  return null;
}
