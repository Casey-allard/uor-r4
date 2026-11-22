// FoundrySpecification.publicationIdentityBoundary. The imported Prism archive
// reader recomputes the fingerprint; this adapter checks its declared identity
// against the page and checks that runtime acceptance is complete and nonempty.
const canonicalDigest = value => typeof value === "string" && value.length === 64 && /^[0-9a-f]{64}$/.test(value);
export const sameArchiveIdentity = (named, actual) => canonicalDigest(named) && canonicalDigest(actual) && named === actual;
const refuse = (code, error) => ({ ok: false, code, error });

export function admitPublication(named, fingerprint, acceptance) {
  if (!canonicalDigest(named) || !canonicalDigest(fingerprint)) {
    return refuse("invalidArchiveIdentity", "The page or archive has no canonical BLAKE3 archive fingerprint");
  }
  if (!sameArchiveIdentity(named, fingerprint)) {
    return refuse("archiveIdentityMismatch", "The page and retrieved archive identify different artifacts");
  }
  if (!acceptance || !Number.isSafeInteger(acceptance.total) || acceptance.total <= 0 ||
      !Number.isSafeInteger(acceptance.passed) || acceptance.passed < 0 || acceptance.passed > acceptance.total ||
      !Array.isArray(acceptance.failed) || acceptance.passed + acceptance.failed.length !== acceptance.total) {
    return refuse("invalidAcceptance", "Runtime acceptance is missing, empty or inconsistent");
  }
  if (acceptance.failed.length !== 0) {
    return refuse("acceptanceFailed", "The retrieved archive failed runtime acceptance");
  }
  return {
    ok: true,
    identity: { kind: "archiveFingerprint", algorithm: "blake3", profile: "holo-v4-body", version: 1, digest: fingerprint },
    vectors: { passed: acceptance.passed, total: acceptance.total },
  };
}
