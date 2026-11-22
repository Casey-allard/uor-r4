export function shouldSimulateFreeAgentQuotaExceeded(
  environment: NodeJS.ProcessEnv = process.env,
): boolean {
  return (
    environment.NODE_ENV === "development" &&
    environment.PRISM_SIMULATE_FREE_AGENT_QUOTA_EXCEEDED === "true"
  );
}
