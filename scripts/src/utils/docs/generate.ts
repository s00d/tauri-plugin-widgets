// @ts-nocheck — facade: split generators live in ./generate/
export {
  ROOT,
  DOCS,
  PUBLIC,
  REQUIRED_PLATFORMS,
  ALL_PLATFORMS,
  CORE_ELEMENTS,
  EXTENDED_ELEMENTS,
  ELEMENT_GROUPS,
  writeOrCheck,
  writeBlock,
  listCases,
  loadCase,
  caseRequiredPlatforms,
  resolveFixture,
  loadFixture,
  walkTypes,
  collectElementUsage,
  loadSchema,
  extractElementsFromSchema,
} from "./generate/shared.js";
export { genElements } from "./generate/elements.js";
export { copyShots, copySchemas, copySandbox } from "./generate/sandbox.js";
export { groupCasesByPreset, genShowcase } from "./generate/showcase.js";
export { genPermissions } from "./generate/permissions.js";
export { genCapabilityMatrix } from "./generate/capability-matrix.js";
export { auditSchemaDocs, auditCoverage } from "./generate/audit.js";
export { runGenerate } from "./generate/index.js";
