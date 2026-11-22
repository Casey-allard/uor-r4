// FoundrySpecification.chatModelPickerScopeBoundary: route representation only.
export function modelSelectionScope(pathname, basepath = "/", chatId) {
  if (typeof pathname !== "string" || !pathname.startsWith("/") || typeof basepath !== "string")
    return { kind: "unavailable" };
  const base = basepath.replace(/\/+$/, "");
  if (base && !base.startsWith("/")) return { kind: "unavailable" };
  if (base && pathname !== base && !pathname.startsWith(base + "/")) return { kind: "unavailable" };
  const relative = pathname.slice(base.length).replace(/\/+$/, "") || "/";
  if (relative !== "/chat") return { kind: "default" };
  return Number.isSafeInteger(chatId) && chatId > 0
    ? { kind: "chat", chatId }
    : { kind: "unavailable" };
}
