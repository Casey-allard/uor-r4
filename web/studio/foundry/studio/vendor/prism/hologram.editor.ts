// FoundrySpecification.editorAssetBoundary: the loader is configured before mount.
import { loader } from "@monaco-editor/react";

const rawBase = import.meta.env.BASE_URL || "/";
const applicationBase =
  rawBase === "./" || rawBase === "."
    ? new URL(import.meta.url.includes("/assets/") ? "../" : "./", import.meta.url)
    : new URL(rawBase, window.location.origin);
loader.config({ paths: { vs: new URL("editor/vs", applicationBase).href } });
