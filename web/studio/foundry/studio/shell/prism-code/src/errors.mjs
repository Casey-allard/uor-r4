/**
 * Every error the package throws is a `PrismError`. Each failure kind has its
 * own subclass with a stable `code`, so a caller branches on the class or the
 * code and never parses a message.
 *
 * - `PrismCodeError`: load, dispatch and archive failures. Its `code` names
 *   the check that failed (`archive.magic`, `core.exports`, `closure.provenance`, ...).
 * - `VerificationError`: no verdict could be reached, or the holo failed a
 *   check that is about the archive itself, its documents or its provenance.
 * - `ProofMismatchError`: a `proof.*` check of a verdict failed.
 * - `CapabilityDeniedError`: the declared world exceeds the granted capabilities,
 *   or the runtime denied a guest import or a resource (`capabilities.*`, issue #279).
 * - `WorldMismatchError`: the guest does not have the interface of the WIT world it declares,
 *   or the world is not the one the caller pinned (issue #278).
 * - `PolicyParseError`, `PolicyEvaluationError`: a policy file could not be parsed or
 *   evaluated, so no verdict was reached. `PolicyDeniedError`: a rule of a policy file
 *   denied the holo (issue #285). All three are `VerificationError`s.
 * - `DeterminismViolationError`: a replay diverged from its recorded history.
 * - `ReplayError`: a replay could not be carried out or its statement failed.
 * - `SigningError`: `attest` could not produce a signature (#272).
 * - `SignatureInvalidError`, `IdentityMismatchError`, `TransparencyError`,
 *   `SubjectMismatchError`, `AttestationNotFoundError`, `TrustRootError`: the
 *   reasons `verify-attestation` can refuse a bundle (#272).
 * - `RegistryError`, `RegistryAuthError`, `ReferrersUnsupportedError`,
 *   `ManifestNotFoundError`, `DigestMismatchError`, `PullVerificationError`,
 *   `ReferenceInvalidError`: the reasons `push` and `pull` can stop (#292).
 * - `ProviderNotFoundError`, `ProviderReplyError`, `ProviderFailedError`: the
 *   ways running a provider service can fail (#294). `ProviderError` is their base.
 * - `ExportMismatchError`: a browser export's bytes differ from the release's
 *   build manifest, its layers or the files written (#290).
 * - `MiddlewareError`: a middleware on the stack threw something that is not a `PrismError` (#295).
 *
 * The last five map one-to-one to the check ids of a `prism/verdict/1`
 * reason (docs/proposals/269-assurance-levels.md); `errorForReason` is the map.
 */
export class PrismError extends Error {
  /**
   * @param {string} message
   * @param {{ code?: string, check?: string, diagnostic?: string, details?: unknown, verdict?: object, reasons?: object[], cause?: unknown }} [options]
   */
  constructor(message, options = {}) {
    super(message, options.cause === undefined ? undefined : { cause: options.cause });
    this.name = new.target.name;
    this.code = options.code ?? new.target.code;
    if (options.check !== undefined) this.check = options.check;
    if (options.diagnostic !== undefined) this.diagnostic = options.diagnostic;
    if (options.details !== undefined) this.details = options.details;
    if (options.verdict !== undefined) this.verdict = options.verdict;
    if (options.reasons !== undefined) this.reasons = options.reasons;
  }
}
PrismError.code = 'prism_error';

/**
 * One error type for load, dispatch and archive failures. `code` names the
 * check that failed (`archive.magic`, `core.exports`, `closure.provenance`, ...).
 */
export class PrismCodeError extends PrismError {
  constructor(code, message, details = undefined) {
    super(message, { code, details });
  }
}

/** No verdict was reached, or an archive, document or provenance check failed. `check` names it. */
export class VerificationError extends PrismError {}
VerificationError.code = 'verification_failed';

/**
 * The policy file does not parse: its syntax is wrong, or it uses a construct outside the
 * subset this evaluator accepts. `check` is `policy.parse`, and the message names the line
 * and the construct. No verdict was reached, so `check` exits 2.
 */
export class PolicyParseError extends VerificationError {}
PolicyParseError.code = 'policy_parse_failed';

/**
 * The policy file parsed but could not produce a decision. `check` is `policy.evaluation`, and
 * the message names the rule. No verdict was reached, so `check` exits 2.
 */
export class PolicyEvaluationError extends VerificationError {}
PolicyEvaluationError.code = 'policy_evaluation_failed';

/**
 * A rule of a policy file denied the holo. `check` is the rule, such as
 * `policy.pure-only.effects`. A verdict was reached, and it failed, so `check` exits 1.
 */
export class PolicyDeniedError extends VerificationError {}
PolicyDeniedError.code = 'policy_denied';

/** A `proof.*` check failed: the proof statement is absent, malformed, or does not belong to this holo. */
export class ProofMismatchError extends PrismError {}
ProofMismatchError.code = 'proof_mismatch';

/**
 * The world the holo declares exceeds the capabilities granted to it
 * (`world.capabilities`), or the runtime denied what the grant does not allow:
 * a guest import (`capabilities.import`) or a resource
 * (`capabilities.outbound-hosts`, `capabilities.key-value-stores`,
 * `capabilities.variables`). `details` names the import or resource and the set.
 */
export class CapabilityDeniedError extends PrismError {}
CapabilityDeniedError.code = 'capability_denied';

/**
 * The guest does not have the imports and exports of its declared world
 * (`world.contract`), or the declared world differs from the world hash the
 * caller pinned (`world.hash`). `check` says which.
 */
export class WorldMismatchError extends PrismError {}
WorldMismatchError.code = 'world_mismatch';

/** A replay produced output that differs from the recorded history. */
export class DeterminismViolationError extends PrismError {}
DeterminismViolationError.code = 'determinism_violation';

/** A replay could not be carried out, or the holo's replay statement failed. */
export class ReplayError extends PrismError {}
ReplayError.code = 'replay_failed';

/**
 * `attest` could not produce a signature: the key is unusable, no identity token
 * was available, or Fulcio, Rekor or the timestamp authority refused. `check`
 * is `sign.key`, `sign.identity`, `sign.service`, `sign.config`, `sign.predicate`
 * or `sign.subject`.
 */
export class SigningError extends PrismError {}
SigningError.code = 'signing_failed';

/** The DSSE signature, the signing key or the certificate chain of a bundle does not verify. */
export class SignatureInvalidError extends PrismError {}
SignatureInvalidError.code = 'signature_invalid';

/** The signer is valid but is not the identity (or issuer) the policy names. */
export class IdentityMismatchError extends PrismError {}
IdentityMismatchError.code = 'identity_mismatch';

/** A transparency log entry or signed timestamp is missing, untrusted or not about this bundle. `reasons` lists each one. */
export class TransparencyError extends PrismError {}
TransparencyError.code = 'transparency_failed';

/** A statement subject is not a content blob of this holo, or its digests differ from the blob's. */
export class SubjectMismatchError extends PrismError {}
SubjectMismatchError.code = 'subject_mismatch';

/** The holo carries no attestation, or none of the requested predicate type. */
export class AttestationNotFoundError extends PrismError {}
AttestationNotFoundError.code = 'attestation_not_found';

/** The pinned trusted root cannot be read or is not a usable root. */
export class TrustRootError extends PrismError {}
TrustRootError.code = 'trust_root_invalid';

/** A registry or a local store could not do what was asked. `check` is `registry.*` or `store.*`. */
export class RegistryError extends PrismError {}
RegistryError.code = 'registry_failed';

/** The registry refused the credentials, or asked for them where they must not be sent. `check` is `registry.auth`, `registry.denied` or `registry.insecure`. */
export class RegistryAuthError extends PrismError {}
RegistryAuthError.code = 'registry_auth_failed';

/** The registry has no referrers API and its referrers tag cannot be read or written, or the caller turned the fallback off. */
export class ReferrersUnsupportedError extends PrismError {}
ReferrersUnsupportedError.code = 'referrers_unsupported';

/** The reference names nothing in the registry or the store. */
export class ManifestNotFoundError extends PrismError {}
ManifestNotFoundError.code = 'manifest_not_found';

/** Bytes differ from the digest something claimed for them: a descriptor, a reference, a registry header or an annotation. */
export class DigestMismatchError extends PrismError {}
DigestMismatchError.code = 'digest_mismatch';

/** `pull` fetched a holo that failed its verdict, its attestations or the signer policy, and wrote nothing. `check` is `pull.*`. */
export class PullVerificationError extends PrismError {}
PullVerificationError.code = 'pull_verification_failed';

/** The text is not a registry or store reference. `check` is `reference.*`. */
export class ReferenceInvalidError extends PrismError {}
ReferenceInvalidError.code = 'reference_invalid';

/** Running a provider service failed (#294). `details` names the capability and the provider type. */
export class ProviderError extends PrismError {}
ProviderError.code = 'provider_error';

/** No `prism-provider-<type>` is on the PATH. Check `provider.not-found`. */
export class ProviderNotFoundError extends ProviderError {}
ProviderNotFoundError.code = 'provider_not_found';

/**
 * A provider broke the protocol (`provider.reply`), or a variable it returned
 * has a second source (`provider.collision`). The message never quotes the
 * reply, because a reply may hold a secret.
 */
export class ProviderReplyError extends ProviderError {}
ProviderReplyError.code = 'provider_reply_invalid';

/** A provider exited non-zero or sent an `error` message (`provider.failed`), would not start (`provider.spawn`) or ran out of time (`provider.timeout`). */
export class ProviderFailedError extends ProviderError {}
ProviderFailedError.code = 'provider_failed';
/**
 * A browser export does not byte-match its release: a file differs from its
 * build manifest row (`export.manifest`) or its release layer (`export.layer`),
 * the written set of files differs (`export.files`), or the tree digest does
 * (`export.tree`). `check` says which; `details` names the file.
 */
export class ExportMismatchError extends PrismError {}
ExportMismatchError.code = 'export_mismatch';
/**
 * A middleware threw something that is not a `PrismError` (#295). `details`
 * names it (`name`, `step`, `tags`) and `cause` is what it threw. A
 * `PrismError` a middleware throws, such as `CapabilityDeniedError`, passes
 * through as it is.
 */
export class MiddlewareError extends PrismError {}
MiddlewareError.code = 'middleware_failed';

/** Every class with a code of its own, by code. */
export const ERROR_CLASSES = Object.freeze({
  [VerificationError.code]: VerificationError,
  [PolicyParseError.code]: PolicyParseError,
  [PolicyEvaluationError.code]: PolicyEvaluationError,
  [PolicyDeniedError.code]: PolicyDeniedError,
  [ProofMismatchError.code]: ProofMismatchError,
  [CapabilityDeniedError.code]: CapabilityDeniedError,
  [WorldMismatchError.code]: WorldMismatchError,
  [DeterminismViolationError.code]: DeterminismViolationError,
  [ReplayError.code]: ReplayError,
  [SigningError.code]: SigningError,
  [SignatureInvalidError.code]: SignatureInvalidError,
  [IdentityMismatchError.code]: IdentityMismatchError,
  [TransparencyError.code]: TransparencyError,
  [SubjectMismatchError.code]: SubjectMismatchError,
  [AttestationNotFoundError.code]: AttestationNotFoundError,
  [TrustRootError.code]: TrustRootError,
  [RegistryError.code]: RegistryError,
  [RegistryAuthError.code]: RegistryAuthError,
  [ReferrersUnsupportedError.code]: ReferrersUnsupportedError,
  [ManifestNotFoundError.code]: ManifestNotFoundError,
  [DigestMismatchError.code]: DigestMismatchError,
  [PullVerificationError.code]: PullVerificationError,
  [ReferenceInvalidError.code]: ReferenceInvalidError,
  [ProviderNotFoundError.code]: ProviderNotFoundError,
  [ProviderReplyError.code]: ProviderReplyError,
  [ProviderFailedError.code]: ProviderFailedError,
  [ExportMismatchError.code]: ExportMismatchError,
  [MiddlewareError.code]: MiddlewareError,
});

/**
 * The class a verdict reason maps to, by its check id. `replay.divergence` is
 * the one replay check that is a determinism violation; every other `replay.*`
 * check is a replay error. A `policy.*` check is a rule of a policy file, a
 * denial, unless it is one of the checks a verdict held before policy files.
 * Anything else, `holo.*` and `provenance.*`, is a verification error.
 */
export function classForCheck(check) {
  if (check === 'policy.parse') return PolicyParseError;
  if (check === 'policy.evaluation') return PolicyEvaluationError;
  if (check.startsWith('policy.') && !/^policy\.(level|effects|invalid|unreadable|digests(\..*)?)$/.test(check)) return PolicyDeniedError;
  if (check.startsWith('proof.')) return ProofMismatchError;
  if (check === 'world.capabilities' || check.startsWith('capabilities.')) return CapabilityDeniedError;
  if (check === 'world.contract' || check === 'world.hash') return WorldMismatchError;
  if (check === 'replay.divergence') return DeterminismViolationError;
  if (check.startsWith('replay.')) return ReplayError;
  return VerificationError;
}

/** The typed error for one reason of a verdict, carrying the reason's check id and diagnostic code. */
export function errorForReason(reason, verdict = undefined) {
  const Class = classForCheck(reason.check);
  return new Class(reason.message, {
    check: reason.check,
    diagnostic: reason.code,
    verdict,
    reasons: verdict ? verdict.reasons : [reason],
  });
}

/**
 * The error that explains a failed verdict, or null for one that passed. A
 * rule the caller pinned and the holo broke (a digest, the world) comes
 * first: the caller asked for that fact by name, and it holds whatever the
 * level is. A world outside the granted capabilities (`world.capabilities`)
 * comes next: the default policy requires it at every level (#283). Otherwise, when the level is below the policy's minimum, it is
 * the first reason that withholds a level up to the minimum, which is what
 * kept the holo down. The error carries the whole verdict and every reason.
 */
export function errorFromVerdict(verdict, ranks) {
  if (verdict.passed) return null;
  const minimum = ranks[verdict.policy.minimum];
  // Damage fails every policy (policy.integrity), and the damage itself is the reason to name.
  const damaged = verdict.reasons.find((r) => r.check === 'holo.integrity');
  if (damaged) return errorForReason(damaged, verdict);
  const pinned = verdict.reasons.find((r) => r.withholds === null && r.check !== 'policy.level');
  // The default policy requires the world within the grant at every level (#269, #283), so it is named before a level shortfall.
  const outside = verdict.reasons.find((r) => r.check === 'world.capabilities');
  const blocking = verdict.reasons.find((r) => r.withholds && ranks[r.withholds] <= minimum);
  const reason = pinned ?? outside ?? blocking ?? verdict.reasons.find((r) => r.check !== 'policy.level') ?? verdict.reasons[0];
  return errorForReason(reason, verdict);
}

/**
 * What a run exits with, one row per outcome: the single source of truth for
 * container exit codes and the `prism-code` process (issue #348). The table is
 * in docs/site/guides/run-a-holo.md, and a test holds the two equal. A row
 * names an error class, or none for success, and `when` narrows it. The first
 * row that matches decides.
 *
 * `check` reports a verdict, not an error, so its own contract stays 0, 1, 2.
 * 1 to 3 keep the meanings the command line already gave them. 10 to 13 are
 * new. 64 to 78 are the BSD sysexits and 125 and above are Docker's and the
 * shell's, so no typed error takes those, apart from `lock.*` as usage (64).
 */
export const EXIT_CODES = Object.freeze(
  [
    { exit: 0, error: null, meaning: 'The guest returned its output.' },
    { exit: 1, error: 'VerificationError', when: 'with a verdict', meaning: 'The holo failed check, for an archive, document, provenance or policy reason.' },
    { exit: 2, error: 'VerificationError', when: 'no verdict', meaning: 'Check could not reach a verdict: an unreadable policy, a closure, an unknown format revision.' },
    { exit: 2, error: 'PrismCodeError', meaning: 'The artifact was refused, or dispatch refused the request: a bad archive, a request over the guest allocation.' },
    { exit: 2, error: 'CapabilityDeniedError', meaning: 'The world exceeds the granted capabilities, or the runtime denied an import or a resource.' },
    { exit: 3, error: 'SigningError', meaning: 'A signature could not be produced.' },
    { exit: 10, error: 'ProofMismatchError', meaning: 'A proof check failed: the statement is absent, malformed, or about another holo.' },
    { exit: 11, error: 'WorldMismatchError', meaning: 'The guest does not have the interface of its declared world, or the world is not the one pinned.' },
    { exit: 12, error: 'DeterminismViolationError', meaning: 'A replay produced output that differs from the recorded history.' },
    { exit: 13, error: 'ReplayError', meaning: 'A replay could not be carried out, or the replay statement failed.' },
    { exit: 1, error: 'SignatureInvalidError', meaning: 'An attestation did not verify: its signature.' },
    { exit: 1, error: 'IdentityMismatchError', meaning: 'An attestation did not verify: its signer.' },
    { exit: 1, error: 'TransparencyError', meaning: 'An attestation did not verify: its log entry or timestamp.' },
    { exit: 1, error: 'SubjectMismatchError', meaning: 'An attestation did not verify: its subject.' },
    { exit: 1, error: 'AttestationNotFoundError', meaning: 'An attestation did not verify: there is none.' },
    { exit: 1, error: 'TrustRootError', meaning: 'An attestation did not verify: the trusted root is unusable.' },
    { exit: 1, error: 'PolicyDeniedError', when: 'with a verdict', meaning: 'A policy rule denied the holo.' },
    { exit: 2, error: 'PolicyParseError', when: 'no verdict', meaning: 'A policy did not parse.' },
    { exit: 2, error: 'PolicyEvaluationError', when: 'no verdict', meaning: 'A policy could not be evaluated.' },
    { exit: 2, error: 'RegistryError', meaning: 'The registry or store failed.' },
    { exit: 2, error: 'RegistryAuthError', meaning: 'The registry refused the credentials.' },
    { exit: 2, error: 'ReferrersUnsupportedError', meaning: 'No referrers API or tag.' },
    { exit: 2, error: 'ManifestNotFoundError', meaning: 'The reference names nothing.' },
    { exit: 1, error: 'DigestMismatchError', meaning: 'Bytes differ from their digest.' },
    { exit: 1, error: 'PullVerificationError', meaning: 'A pulled holo failed verification.' },
    { exit: 1, error: 'ReferenceInvalidError', meaning: 'Not a registry or store reference.' },
    { exit: 2, error: 'ProviderNotFoundError', meaning: 'No provider on the PATH.' },
    { exit: 2, error: 'ProviderReplyError', meaning: 'A provider broke the protocol.' },
    { exit: 2, error: 'ProviderFailedError', meaning: 'A provider failed.' },
    { exit: 2, error: 'ExportMismatchError', meaning: 'A browser export differs from its release.' },
    { exit: 1, error: 'MiddlewareError', meaning: 'A middleware threw something untyped.' },
    { exit: 1, error: 'PrismError', meaning: 'Any other typed error.' },
    { exit: 70, error: 'anything else', meaning: 'Dispatch threw something untyped: a guest trap, or a bug in the runtime.' },
  ].map(Object.freeze),
);

const classNamed = (name) => (name === 'PrismError' ? PrismError : name === 'PrismCodeError' ? PrismCodeError : Object.values(ERROR_CLASSES).find((C) => C.name === name));

/**
 * The exit code for how a run ended: `exitCodeFor(null)` is 0, a typed error
 * takes its row of `EXIT_CODES`, and anything else is 70. A `lock.*` code is
 * the one `PrismCodeError` that is the caller's to fix, so it is usage (64),
 * and `attest.unavailable` means no signature could be produced (3).
 */
export function exitCodeFor(error) {
  if (error === null || error === undefined) return 0;
  if (!(error instanceof PrismError)) return 70;
  if (error instanceof PrismCodeError && error.code.startsWith('lock.')) return 64;
  if (error.code === 'attest.unavailable') return 3;
  return EXIT_CODES.find((row) => {
    const Class = row.error === null ? undefined : classNamed(row.error);
    if (!Class || !(error instanceof Class)) return false;
    if (row.when === 'with a verdict') return error.verdict !== undefined;
    if (row.when === 'no verdict') return error.verdict === undefined;
    return true;
  }).exit;
}

export function fail(code, message, details) {
  throw new PrismCodeError(code, message, details);
}
