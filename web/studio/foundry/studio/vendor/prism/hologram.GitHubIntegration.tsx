// prism-prism (ours, not upstream's; see VENDORED.md): Settings → Integrations → GitHub, replacing upstream's at build
// time through `resolveId` in vite.shell.config.mts. Foundry publishes on the device (shell/host.js, holo:publish),
// with no account anywhere, so there is nothing to connect: the row renders nothing.
export function GitHubIntegration() {
  return null;
}
