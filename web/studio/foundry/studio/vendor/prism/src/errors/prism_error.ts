/**
 * Classified application errors for IPC/main-process code.
 * Use {@link PrismError} with a {@link PrismErrorKind} so telemetry can ignore
 * high-volume, non-actionable failures (see `shouldFilterTelemetryException`).
 */

export enum PrismErrorKind {
  Validation = "validation",
  NotFound = "not_found",
  Auth = "auth",
  Precondition = "precondition",
  Conflict = "conflict",
  UserCancelled = "user_cancelled",
  RateLimited = "rate_limited",
  /** Upstream failures; reported to PostHog by default unless you add finer metadata later. */
  External = "external",
  /** Bugs, invariant violations, unexpected failures — always reported. */
  Internal = "internal",
  /** Unclassified; treated as reportable until call sites are migrated. */
  Unknown = "unknown",
}

const TELEMETRY_FILTERED_KINDS: ReadonlySet<PrismErrorKind> = new Set([
  PrismErrorKind.Validation,
  PrismErrorKind.NotFound,
  PrismErrorKind.Auth,
  PrismErrorKind.Precondition,
  PrismErrorKind.Conflict,
  PrismErrorKind.UserCancelled,
  PrismErrorKind.RateLimited,
]);

/**
 * Returns true if this kind should not be sent to PostHog as an `$exception` event.
 */
export function isPrismErrorKindFilteredFromTelemetry(
  kind: PrismErrorKind,
): boolean {
  return TELEMETRY_FILTERED_KINDS.has(kind);
}

export class PrismError extends Error {
  readonly kind: PrismErrorKind;
  readonly cause?: unknown;

  constructor(
    message: string,
    kind: PrismErrorKind,
    options?: { cause?: unknown },
  ) {
    super(message);
    this.name = "PrismError";
    this.kind = kind;
    this.cause = options?.cause;
  }
}

export function isPrismError(error: unknown): error is PrismError {
  return error instanceof PrismError;
}
