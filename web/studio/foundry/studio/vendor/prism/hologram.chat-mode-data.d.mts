import type { ChatMode } from "./src/lib/schemas";
export const chatModeCatalog: {
  profile: string;
  modes: ReadonlyArray<{ id: ChatMode; label: string }>;
};
