// @ts-nocheck
export function auditSchemaDocs(elements) {
  const errors = [];
  const enumBlurb = elements[0]?.rootDesc || "A UI element that can be a layout container or a leaf widget.";
  for (const el of elements) {
    if (!el.description || !el.description.trim()) {
      errors.push(`element \`${el.name}\` missing variant description in schema`);
    } else if (el.description.trim() === enumBlurb.trim()) {
      errors.push(`element \`${el.name}\` still uses enum-level description`);
    }
    for (const [name, prop] of Object.entries(el.properties)) {
      if (name === "type") continue;
      if (!prop.description || !String(prop.description).trim()) {
        errors.push(`\`${el.name}.${name}\` missing description`);
      }
    }
  }
  return errors;
}
