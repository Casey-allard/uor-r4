"use strict";

const API = "/uor/v1/workbench";
const ui = Object.fromEntries([
  "provider", "execution", "model-id", "generation", "qualification", "operation",
  "state-pill", "notice", "load", "unload", "cancel", "submit", "composer",
  "raw-text", "byte-count", "result-card", "result-heading", "result"
].map(id => [id, document.getElementById(id)]));

let capabilities = null;
let model = null;
let activeJob = null;
let polling = null;

function requireSnapshot(next, previous, kind) {
  if (!next || typeof next !== "object") throw { tag: "BAD_RESPONSE", message: `${kind} snapshot is missing.`, native: null };
  if (capabilities && next.instance_id !== capabilities.instance_id) {
    throw { tag: "STALE_INSTANCE", message: `${kind} belongs to another service instance.`, native: null };
  }
  if (previous && next.instance_id === previous.instance_id && next.revision < previous.revision) {
    throw { tag: "BAD_RESPONSE", message: `${kind} revision moved backwards.`, native: null };
  }
  return next;
}

function adoptCapabilities(next) {
  if (!next || typeof next !== "object") {
    throw { tag: "BAD_RESPONSE", message: "Capability snapshot is missing.", native: null };
  }
  if (capabilities && next.instance_id === capabilities.instance_id && next.revision < capabilities.revision) {
    throw { tag: "BAD_RESPONSE", message: "Capability revision moved backwards.", native: null };
  }
  if (capabilities && next.instance_id !== capabilities.instance_id) {
    clearTimeout(polling);
    polling = null;
    activeJob = null;
    model = null;
    showResult(null);
  }
  capabilities = next;
}

function adoptModel(next) { model = requireSnapshot(next, model, "Model"); }
function adoptJob(next) { activeJob = requireSnapshot(next, activeJob, "Job"); }

function exactBytes(text) { return new TextEncoder().encode(text); }
async function sha256Hex(bytes) {
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
  return Array.from(digest, byte => byte.toString(16).padStart(2, "0")).join("");
}
function paddedBase64(bytes) {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}
function messageOf(error) {
  if (!error) return "No error detail supplied.";
  const native = error.native ? ` · native ${error.native.tag}` : "";
  return `${error.tag}: ${error.message}${native}`;
}
function setNotice(text, error = false) {
  ui.notice.textContent = text;
  ui.notice.classList.toggle("error", error);
}
function showResult(job) {
  if (!job || !job.result) {
    ui["result-card"].hidden = true;
    ui["result-heading"].textContent = "Result";
    ui.result.textContent = "";
    return;
  }
  ui["result-card"].hidden = false;
  ui["result-heading"].textContent = job.result.status === "MODEL_TOKEN" ? "Model token" : "Typed refusal";
  ui.result.textContent = JSON.stringify(job.result, null, 2);
}

function render() {
  if (!capabilities || !model) return;
  ui.provider.textContent = capabilities.provider;
  ui.execution.textContent = capabilities.execution;
  ui["model-id"].textContent = model.model_id;
  ui.generation.textContent = String(model.model_generation);
  ui.qualification.textContent = model.qualification_receipt_sha256 || "not installed";
  ui.operation.textContent = capabilities.operations[0]?.id || "No operation advertised";
  ui["state-pill"].textContent = model.state;
  ui["state-pill"].dataset.state = model.state;

  const busy = ["loading", "running", "stopping", "unloading"].includes(model.state);
  ui.load.disabled = busy || model.state === "ready" || model.state === "unavailable";
  ui.unload.disabled = model.state !== "ready";
  ui.submit.disabled = model.state !== "ready" || exactBytes(ui["raw-text"].value).length > 4096;
  ui.cancel.hidden = !(activeJob && ["load", "answer"].includes(activeJob.kind) && !["completed", "cancelled", "failed"].includes(activeJob.state));

  if (model.error) setNotice(messageOf(model.error), true);
  else if (activeJob && !["completed", "cancelled", "failed"].includes(activeJob.state)) {
    setNotice(`${activeJob.kind} · ${activeJob.state} · ${activeJob.progress.stage}`);
  } else if (model.state === "ready") setNotice("Qualified native worker is ready for the bounded Four-fact operation.");
  else setNotice(`Native model state: ${model.state}.`);
}

async function request(path, options = {}) {
  const response = await fetch(`${API}${path}`, {
    cache: "no-store",
    headers: options.body ? { "Content-Type": "application/json" } : undefined,
    ...options
  });
  const payload = await response.json();
  if (!response.ok) {
    const error = payload.error || { tag: "BAD_RESPONSE", message: `HTTP ${response.status}`, native: null };
    error.deliveryKnown = true;
    throw error;
  }
  return payload;
}

async function refresh() {
  try {
    const [nextCapabilities, nextModel] = await Promise.all([request("/capabilities"), request("/model")]);
    adoptCapabilities(nextCapabilities);
    adoptModel(nextModel);
    if (model.active_job_id) await watchJob(model.active_job_id);
    render();
  } catch (error) {
    if (error.deliveryKnown) {
      setNotice(messageOf(error), true);
      return;
    }
    setNotice(messageOf(error), true);
  }
}

async function watchJob(jobId) {
  adoptJob(await request(`/jobs/${encodeURIComponent(jobId)}`));
  adoptModel(await request("/model"));
  showResult(activeJob);
  render();
  const terminal = ["completed", "cancelled", "failed"].includes(activeJob.state);
  if (terminal) {
    clearTimeout(polling);
    polling = null;
    if (activeJob.error) setNotice(messageOf(activeJob.error), true);
    else setNotice(`${activeJob.kind} job ${activeJob.state}.`);
    return;
  }
  clearTimeout(polling);
  polling = setTimeout(() => watchJob(jobId).catch(error => setNotice(messageOf(error), true)), 300);
}

function matchesAdmission(job, expected) {
  return job
    && job.instance_id === expected.instanceId
    && job.kind === expected.kind
    && job.model_id === expected.modelId
    && job.admitted_generation === expected.admittedGeneration
    && job.raw_text_sha256 === expected.rawTextSha256;
}

async function recoverUncertainMutation(expected) {
  const nextCapabilities = await request("/capabilities");
  const nextModel = await request("/model");
  adoptCapabilities(nextCapabilities);
  adoptModel(nextModel);

  const ids = expected.jobId
    ? [expected.jobId]
    : [...new Set([nextModel.active_job_id, nextModel.last_job_id].filter(Boolean))];
  const candidates = [];
  for (const id of ids) {
    try {
      candidates.push(await request(`/jobs/${encodeURIComponent(id)}`));
    } catch (error) {
      if (!expected.jobId && error.deliveryKnown && error.tag === "JOB_NOT_FOUND") continue;
      throw error;
    }
  }
  const matches = candidates.filter(job => matchesAdmission(job, expected));
  if (matches.length !== 1) {
    render();
    throw {
      tag: "DELIVERY_UNCERTAIN",
      message: "The POST result was lost and discovery could not identify exactly one matching admitted job. The request was not resubmitted.",
      native: null
    };
  }
  adoptJob(matches[0]);
  showResult(activeJob);
  render();
  return activeJob;
}

async function mutate(path, body, expected) {
  showResult(null);
  try {
    const admitted = await request(path, {
      method: "POST",
      body: JSON.stringify(body)
    });
    expected.jobId = admitted.job_id;
    adoptJob(admitted);
    await watchJob(activeJob.job_id);
  } catch (error) {
    if (error.deliveryKnown) {
      try {
        const [nextCapabilities, nextModel] = await Promise.all([request("/capabilities"), request("/model")]);
        adoptCapabilities(nextCapabilities);
        adoptModel(nextModel);
        render();
      } catch (_) {
        // The received typed rejection remains authoritative even if the
        // subsequent discovery refresh is unavailable.
      }
      setNotice(messageOf(error), true);
      return;
    }
    // A lost POST response is delivery-uncertain. Discovery may reveal the
    // admitted job; the mutation is never submitted automatically again.
    try {
      const recovered = await recoverUncertainMutation(expected);
      if (!["completed", "cancelled", "failed"].includes(recovered.state)) {
        await watchJob(recovered.job_id);
      }
    } catch (recoveryError) {
      setNotice(messageOf(recoveryError), true);
    }
  }
}

ui.load.addEventListener("click", () => {
  if (!capabilities || !model) return;
  const expected = {
    instanceId: capabilities.instance_id,
    kind: "load",
    modelId: model.model_id,
    admittedGeneration: model.model_generation,
    rawTextSha256: null,
    jobId: null
  };
  mutate("/model/load", {
    schema: "uor-r4.workbench-load/1",
    instance_id: expected.instanceId,
    model_id: expected.modelId
  }, expected);
});
ui.unload.addEventListener("click", () => {
  if (!capabilities || !model) return;
  const expected = {
    instanceId: capabilities.instance_id,
    kind: "unload",
    modelId: model.model_id,
    admittedGeneration: model.model_generation,
    rawTextSha256: null,
    jobId: null
  };
  mutate("/model/unload", {
    schema: "uor-r4.workbench-unload/1",
    instance_id: expected.instanceId,
    model_id: expected.modelId,
    expected_generation: expected.admittedGeneration
  }, expected);
});
ui.cancel.addEventListener("click", () => {
  if (!activeJob) return;
  const expected = {
    instanceId: activeJob.instance_id,
    kind: activeJob.kind,
    modelId: activeJob.model_id,
    admittedGeneration: activeJob.admitted_generation,
    rawTextSha256: activeJob.raw_text_sha256,
    jobId: activeJob.job_id
  };
  mutate(`/jobs/${encodeURIComponent(activeJob.job_id)}/cancel`, {
    schema: "uor-r4.workbench-cancel/1",
    instance_id: capabilities.instance_id
  }, expected);
});
ui.composer.addEventListener("submit", async event => {
  event.preventDefault();
  if (!capabilities || !model) return;
  const bytes = exactBytes(ui["raw-text"].value);
  if (bytes.length > 4096) return setNotice("INPUT_LIMIT: UTF-8 input exceeds 4096 bytes.", true);
  const expected = {
    instanceId: capabilities.instance_id,
    kind: "answer",
    modelId: model.model_id,
    admittedGeneration: model.model_generation,
    rawTextSha256: await sha256Hex(bytes),
    jobId: null
  };
  mutate("/requests", {
    schema: "uor-r4.workbench-request/1",
    instance_id: expected.instanceId,
    model_id: expected.modelId,
    expected_generation: expected.admittedGeneration,
    operation: "answer_four_fact_raw_text/v1",
    input: {
      schema: "uor-r4.text-to-clauses/1",
      encoding: "base64",
      bytes_b64: paddedBase64(bytes)
    }
  }, expected);
});
ui["raw-text"].addEventListener("input", () => {
  const count = exactBytes(ui["raw-text"].value).length;
  ui["byte-count"].textContent = String(count);
  ui.submit.disabled = !model || model.state !== "ready" || count > 4096;
});

refresh();
