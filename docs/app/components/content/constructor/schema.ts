/** Schema-driven inspector helpers for widget-config.v1.json */

export type JsonSchema = {
  $ref?: string;
  type?: string | string[];
  enum?: unknown[];
  const?: unknown;
  default?: unknown;
  description?: string;
  properties?: Record<string, JsonSchema>;
  required?: string[];
  oneOf?: JsonSchema[];
  anyOf?: JsonSchema[];
  allOf?: JsonSchema[];
  items?: JsonSchema;
  definitions?: Record<string, JsonSchema>;
  format?: string;
};

/** Structural keys owned by the tree — not shown as free-form inspector fields. */
const TREE_KEYS = new Set(["type", "children"]);
/** `list.items` is schema-editable JSON (typed ListItem rows), not a WidgetElement tree. */
const JSON_ARRAY_KEYS = new Set(["items"]);

const STYLE_KEYS = new Set([
  "padding",
  "background",
  "cornerRadius",
  "opacity",
  "frame",
  "border",
  "shadow",
  "clipShape",
  "flex",
  "spacing",
  "alignment",
]);

export type FieldKind = "string" | "number" | "boolean" | "enum" | "json";

export type InspectorField = {
  key: string;
  description?: string;
  required: boolean;
  kind: FieldKind;
  enumValues?: string[];
  group: "props" | "style";
};

function resolveRef(schema: JsonSchema, root: JsonSchema): JsonSchema {
  if (!schema.$ref) return schema;
  const m = schema.$ref.match(/^#\/definitions\/(.+)$/);
  if (!m || !root.definitions?.[m[1]]) return schema;
  return root.definitions[m[1]];
}

function branchForType(root: JsonSchema, type: string): JsonSchema | null {
  const we = root.definitions?.WidgetElement;
  if (!we?.oneOf) return null;
  for (const branch of we.oneOf) {
    const resolved = resolveRef(branch, root);
    const t = resolved.properties?.type;
    const enumVals = t?.enum;
    if (Array.isArray(enumVals) && enumVals[0] === type) return resolved;
    if (t?.const === type) return resolved;
  }
  return null;
}

function unwrapNull(schema: JsonSchema, root: JsonSchema): JsonSchema {
  let s = resolveRef(schema, root);
  if (s.anyOf) {
    const nonNull = s.anyOf
      .map((x) => resolveRef(x, root))
      .find((x) => x.type !== "null" && x.const !== null);
    if (nonNull) s = nonNull;
  }
  if (Array.isArray(s.type)) {
    const t = s.type.find((x) => x !== "null");
    if (t) s = { ...s, type: t };
  }
  if (s.allOf?.length === 1) s = resolveRef(s.allOf[0], root);
  return resolveRef(s, root);
}

function fieldKind(schema: JsonSchema, root: JsonSchema): FieldKind {
  const s = unwrapNull(schema, root);
  if (Array.isArray(s.enum) && s.enum.every((v) => typeof v === "string")) {
    return "enum";
  }
  const t = s.type;
  if (t === "string") return "string";
  if (t === "number" || t === "integer") return "number";
  if (t === "boolean") return "boolean";
  return "json";
}

function enumValues(schema: JsonSchema, root: JsonSchema): string[] | undefined {
  const s = unwrapNull(schema, root);
  if (Array.isArray(s.enum) && s.enum.every((v) => typeof v === "string")) {
    return s.enum as string[];
  }
  return undefined;
}

export function fieldsForType(root: JsonSchema, type: string): InspectorField[] {
  const branch = branchForType(root, type);
  if (!branch?.properties) return [];
  const required = new Set(branch.required ?? []);
  const fields: InspectorField[] = [];
  for (const [key, prop] of Object.entries(branch.properties)) {
    if (TREE_KEYS.has(key)) continue;
    const resolved = resolveRef(prop, root);
    const kind = JSON_ARRAY_KEYS.has(key) ? "json" : fieldKind(prop, root);
    fields.push({
      key,
      description: resolved.description ?? prop.description,
      required: required.has(key),
      kind,
      enumValues: kind === "enum" ? enumValues(prop, root) : undefined,
      group: STYLE_KEYS.has(key) ? "style" : "props",
    });
  }
  fields.sort((a, b) => {
    if (a.required !== b.required) return a.required ? -1 : 1;
    if (a.group !== b.group) return a.group === "props" ? -1 : 1;
    return a.key.localeCompare(b.key);
  });
  return fields;
}

export function allTypesFromSchema(root: JsonSchema): string[] {
  const we = root.definitions?.WidgetElement;
  if (!we?.oneOf) return [];
  const out: string[] = [];
  for (const branch of we.oneOf) {
    const resolved = resolveRef(branch, root);
    const t = resolved.properties?.type;
    if (Array.isArray(t?.enum) && typeof t.enum[0] === "string") {
      out.push(t.enum[0] as string);
    } else if (typeof t?.const === "string") {
      out.push(t.const);
    }
  }
  return out;
}
