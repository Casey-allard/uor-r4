// hologram-forge (ours, see VENDORED.md): posthog-js and posthog-js/react as the shell builds them, a client
// that sends nothing. Upstream's calls (init, capture, captureException, identify, people.set, usePostHog,
// PostHogProvider) keep their shape: every property at any depth is a function that answers undefined,
// and init answers the client itself.
import type { ReactNode } from "react";

// "then" stays undefined so no part of the client is mistaken for a promise.
const inert = (): any => new Proxy(function () {}, {
  get: (_target, key) => (key === "then" || typeof key === "symbol" ? undefined : key === "init" ? () => client : inert()),
  apply: () => undefined,
});
const client: any = inert();

export class PostHog {}
export const usePostHog = () => client;
export const PostHogProvider = ({ children }: { children?: ReactNode }) => children;
export default client;
