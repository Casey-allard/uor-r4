Preservation-only recovery after terminal setup refusal, no instrument binary invoked.
The reviewed operator omitted *.csv from its local-operator copy glob, so the
frozen REVIEW-SHA256SUMS failed on qualified-gpu.csv. The trap retained and
renewed vast-56332437; it did not upload, release or delete on failure.
Parent copied the exact frozen CSV after cmp with the complete remote archive.
Original operator bytes and initial manifest/failure log remain retained.
This recovery changes no execution condition, source, model input or dose.
All original and complete updated manifests are checked before full cloud
MD5 upload/readback; only then release/down is permitted. A recovery failure
leaves the identified pod retained for renewed recovery, never reruns the job.
