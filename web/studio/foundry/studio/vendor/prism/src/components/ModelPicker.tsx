// Upstream's model picker is replaced whole at build by ours, hologram.ModelPicker.tsx (vite.shell.config.mts
// resolveId; hologram.typecheck.mjs lists it as replaced), so its body is not kept here; it is upstream's
// src/components/ModelPicker.tsx at the commit VENDORED.md names. The path and its typed export stay, so
// ChatInputControls' import resolves and the typecheck's upstream baseline is upstream's own.
export function ModelPicker() { return null; }
