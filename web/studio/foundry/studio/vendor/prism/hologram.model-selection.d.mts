export function modelSelectionScope(
  pathname: unknown,
  basepath: unknown,
  chatId: unknown,
): { kind: "chat"; chatId: number } | { kind: "default" } | { kind: "unavailable" };
