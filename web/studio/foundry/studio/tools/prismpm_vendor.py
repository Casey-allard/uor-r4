"""The vendored PrismPM archive code is PrismPM's, byte for byte, at the pinned commit.

core/src/holo/{archive,canonical,model_document,validate}.rs are `crates/prismpm/src/holo/*.rs` verbatim;
core/src/error.rs is `crates/prismpm/src/error.rs` with one function removed, `PrismError::from_lexlean`
(it converts LexLean's error type and would pull the LexLean crate into the core; the composer never
calls it), and a note added at the top. This script normalises both sides the same way (line endings,
that one function, the note) and compares digests with the record in tools/prismpm_vendor.sha256.

    python3 tools/prismpm_vendor.py <PrismPM checkout>            # check (the lane's gate)
    python3 tools/prismpm_vendor.py <PrismPM checkout> --record   # write the record from PrismPM
"""
import hashlib, io, pathlib, sys

root = pathlib.Path(__file__).resolve().parent.parent
prismpm = pathlib.Path(sys.argv[1])
record = root / "tools" / "prismpm_vendor.sha256"
FILES = ["error.rs", "holo/archive.rs", "holo/canonical.rs", "holo/model_document.rs", "holo/validate.rs"]
NOTE = "//! hologram-forge: PrismPM's"


def normalised(path, text):
    text = text.replace("\r\n", "\n")
    if path == "error.rs":
        start = text.find("    /// Convert a LexLean failure while preserving structured diagnostics up to")
        if start >= 0:
            end = text.index("\n    }\n", start) + len("\n    }\n")
            text = text[:start] + text[end:]
        if NOTE in text:
            head, rest = text.split("//!\n" + NOTE, 1)
            rest = rest.split("\n", 4)[4]  # the note is four lines after its opener
            text = head + rest
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def digest(base, path):
    return normalised(path, (base / path).read_text(encoding="utf-8"))


if "--record" in sys.argv:
    rows = [f"{digest(prismpm / 'crates' / 'prismpm' / 'src', p)}  {p}\n" for p in FILES]
    io.open(record, "w", encoding="utf-8", newline="\n").write("".join(rows))
    print(f"recorded {len(FILES)} digests from {prismpm}")
    sys.exit(0)

recorded = dict(reversed(line.split()) for line in record.read_text(encoding="utf-8").split("\n") if line.strip())
bad = []
for p in FILES:
    upstream = digest(prismpm / "crates" / "prismpm" / "src", p)
    vendored = digest(root / "core" / "src", p)
    if upstream != recorded.get(p) or vendored != recorded.get(p):
        bad.append(f"core/src/{p}: recorded {recorded.get(p)}, PrismPM {upstream}, vendored {vendored}")
if bad:
    sys.exit("vendored PrismPM archive code is not PrismPM's at the pinned commit:\n" + "\n".join(bad))
print(f"vendored PrismPM archive code matches ({len(FILES)} files)")
