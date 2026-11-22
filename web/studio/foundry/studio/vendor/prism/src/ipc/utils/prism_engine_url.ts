export function getPrismEngineBaseUrl(): string {
  return process.env.PRISM_ENGINE_URL ?? "https://engine.dyad.sh/v1";
}
