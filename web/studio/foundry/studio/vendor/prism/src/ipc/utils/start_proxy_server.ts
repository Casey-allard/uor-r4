// startProxy.js – helper to launch proxy.js as a worker

import { Worker } from "worker_threads";
import path from "path";
import log from "electron-log";
import { PrismError, PrismErrorKind } from "@/errors/prism_error";
import {
  PROXY_FALLBACK_MAX_ATTEMPTS,
  getProxyFallbackPortStart,
} from "../../../shared/ports";

const logger = log.scope("start_proxy_server");

export async function startProxy(
  targetOrigin: string,
  opts: {
    port: number;
    onStarted?: (proxyUrl: string) => void;
    onError?: (error: PrismError) => void;
    fixedHeaders?: Record<string, string>;
    authBootstrapToken: string;
  },
) {
  if (!/^https?:\/\//.test(targetOrigin))
    throw new PrismError(
      "startProxy: targetOrigin must be absolute http/https URL",
      PrismErrorKind.Validation,
    );
  const { port, onStarted, onError, fixedHeaders, authBootstrapToken } = opts;
  const fallbackPortStart = getProxyFallbackPortStart();
  logger.info("Starting proxy on port", port);

  const worker = new Worker(
    path.resolve(__dirname, "..", "..", "worker", "proxy_server.js"),
    {
      workerData: {
        targetOrigin,
        port,
        fallbackPortStart,
        maxPortAttempts: PROXY_FALLBACK_MAX_ATTEMPTS,
        fixedHeaders,
        authBootstrapToken,
      },
    },
  );

  worker.on("message", (m) => {
    logger.info("[proxy]", m);
    if (typeof m === "string" && m.startsWith("proxy-server-start url=")) {
      const url = m.substring("proxy-server-start url=".length);
      onStarted?.(url);
    } else if (typeof m === "string" && m.startsWith("proxy-server-error")) {
      logger.error("[proxy] failed to bind:", m);
      onError?.(
        new PrismError(
          `Could not start the preview proxy: every port from ${port} to ${fallbackPortStart + PROXY_FALLBACK_MAX_ATTEMPTS - 1} is in use. Free up a port and restart the app.`,
          PrismErrorKind.Conflict,
        ),
      );
    }
  });
  worker.on("error", (e) => logger.error("[proxy] error:", e));
  worker.on("exit", (c) => logger.info("[proxy] exit", c));

  return worker; // let the caller keep a handle if desired
}
