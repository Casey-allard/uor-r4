// Upstream's sidebar is replaced whole at build by the shell's, hologram.screens/AppSidebar.tsx
// (hologram.screens.mjs SCREENS), so its body is not kept here; it is upstream's src/components/app-sidebar.tsx
// at the commit VENDORED.md names. The path and its typed export stay, so imports resolve and the typecheck's
// upstream baseline is upstream's own.
export function AppSidebar() { return null; }
