// hologram-forge (ours, see VENDORED.md): Vite serves an imported .svg as its URL; upstream declares no type for it.
declare module "*.svg" {
  const url: string;
  export default url;
}
// Vite replaces import.meta.env.BASE_URL with the shell's base at build (the covers hook keys its cache by it).
interface ImportMetaEnv {
  readonly BASE_URL: string;
}
interface ImportMeta {
  readonly env: ImportMetaEnv;
}
