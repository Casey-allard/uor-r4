import { PrismError, PrismErrorKind, isPrismError } from "@/errors/prism_error";
import type { SandboxHostCallName } from "./capabilities";
import type { SandboxRunResult } from "./execution";

export interface SandboxWorkerInput {
  appPath: string;
  script: string;
  timeoutMs: number;
  persistFullOutput?: boolean;
}

export interface SandboxWorkerHostCall {
  name: SandboxHostCallName;
  path?: string;
}

export interface SerializedSandboxWorkerError {
  name?: string;
  message: string;
  kind?: PrismErrorKind;
  stack?: string;
}

export type SandboxWorkerMessage =
  | { type: "vmBudgetStart" }
  | { type: "vmBudgetPause" }
  | { type: "vmBudgetResume" }
  | { type: "hostCall"; hostCall: SandboxWorkerHostCall }
  | { type: "result"; result: SandboxRunResult }
  | { type: "error"; error: SerializedSandboxWorkerError };

export function serializeSandboxWorkerError(
  error: unknown,
): SerializedSandboxWorkerError {
  if (isPrismError(error)) {
    return {
      name: error.name,
      message: error.message,
      kind: error.kind,
      stack: error.stack,
    };
  }
  if (error instanceof Error) {
    return {
      name: error.name,
      message: error.message,
      stack: error.stack,
    };
  }
  return {
    message: String(error),
  };
}

function isPrismErrorKind(value: unknown): value is PrismErrorKind {
  return (
    typeof value === "string" &&
    Object.values(PrismErrorKind).includes(value as PrismErrorKind)
  );
}

export function deserializeSandboxWorkerError(
  error: SerializedSandboxWorkerError,
): Error {
  if (isPrismErrorKind(error.kind)) {
    const prismError = new PrismError(error.message, error.kind);
    prismError.stack = error.stack;
    return prismError;
  }

  const genericError = new Error(error.message);
  genericError.name = error.name ?? genericError.name;
  genericError.stack = error.stack;
  return genericError;
}
