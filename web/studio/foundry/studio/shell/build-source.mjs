// FoundrySpecification.snapshotBuildBoundary: exact source before effects.
import { assertCanonicalBuild } from "./canonical-build.mjs";
import {applicationShape, shapeFailure} from './application-shape.mjs';
import { captureProjectSource } from "./project-edit.mjs";
import { prepareSnapshot } from "./snapshot-restore.mjs";
import { snapshotDependencies } from "./project-lifecycle.mjs";
const fail = code => Object.assign(new Error(code === "buildSourceChanged" ? "Project changed while building; build the current version again" : "Project bytes do not match their snapshot"), { code });
const shape = source => source && typeof source.name === "string" && typeof source.reference === "string" && Array.isArray(source.files) && source.files.every(row => Array.isArray(row) && row.length === 2 && row.every(v => typeof v === "string")) && new Set(source.files.map(([path]) => path)).size === source.files.length && Array.isArray(source.dependencies) && source.dependencies.every(v => typeof v === "string");
const rows = files => JSON.stringify([...files].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0));
export function sameBuildSource(left, right) {
  return !!(shape(left) && shape(right) && left.name === right.name && left.reference === right.reference && rows(left.files) === rows(right.files) && JSON.stringify(left.dependencies) === JSON.stringify(right.dependencies));
}
export function verifyCapturedSource(source, snapshot) {
  if (!shape(source) || !snapshot || !shape({ ...snapshot, reference: snapshot.oid, dependencies: [] }) || source.reference !== snapshot.oid || source.name !== snapshot.name || rows(source.files) !== rows(snapshot.files) || JSON.stringify(source.dependencies) !== JSON.stringify(snapshotDependencies(snapshot.files))) throw fail("buildSourceMismatch");
  assertCanonicalBuild(source.files);
  return source;
}
export async function captureBuildSource(db, appId, boundary) {
  const source = await captureProjectSource(db, appId);
  const snapshot = await prepareSnapshot(source.reference, boundary);
  return verifyCapturedSource(source, snapshot);
}
export async function assertBuildSourceCurrent(db, appId, source) {
  let current;
  try { current = await captureProjectSource(db, appId); }
  catch (error) { if (error?.code === "projectEditStateInvalid") throw fail("buildSourceChanged"); throw error; }
  if (!sameBuildSource(source, current)) throw fail("buildSourceChanged");
}
export function requiresRulePublication(source) {
  if (applicationShape(source) === 'stateful') throw shapeFailure('statefulBuildUnsupported');
  return source.files.some(([path]) => path === "prism/spec.json");
}
