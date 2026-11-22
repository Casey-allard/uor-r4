// FoundrySpecification.storageCommitBoundary: request success is not commit.
const failure = () => Object.assign(new Error("Storage transaction failed"), { code: "storageTransactionFailed" });
export function storageRequest(db, store, mode, operation) {
  return new Promise((resolve, reject) => {
    let transaction, result, received = false, failed = false;
    const refuse = (abort = false) => {
      failed = true;
      if (abort && transaction) { try { transaction.abort(); } catch {} }
      reject(failure());
    };
    try {
      transaction = db.transaction(store, mode);
      transaction.onabort = () => refuse();
      transaction.onerror = () => refuse(true);
      transaction.oncomplete = () => {
        if (failed || !received) refuse();
        else resolve(result);
      };
      const request = operation(transaction.objectStore(store));
      if (!request || typeof request !== "object" || !("onsuccess" in request) || !("onerror" in request)) { refuse(true); return; }
      request.onerror = () => refuse(true);
      request.onsuccess = () => {
        try { result = request.result; received = true; }
        catch { refuse(true); }
      };
    } catch { refuse(true); }
  });
}
