import type { StudioField, StudioNode } from "./types";

export type UnionVariantView = {
  name: string;
  fieldIndex: number;
  marker: StudioField;
  fields: Array<{
    field: StudioField;
    fieldIndex: number;
    displayName: string;
  }>;
};

export function unionVariants(node: StudioNode): UnionVariantView[] {
  if (node.kind !== "union") return [];
  const variants: UnionVariantView[] = [];
  let current: UnionVariantView | null = null;

  node.fields.forEach((field, fieldIndex) => {
    if (field.ty === "variant") {
      current = { name: field.name, fieldIndex, marker: field, fields: [] };
      variants.push(current);
      return;
    }

    const [variantName, fieldName] = splitVariantField(field.name);
    if (variantName) {
      let variant = variants.find((item) => item.name === variantName);
      if (!variant) {
        variant = {
          name: variantName,
          fieldIndex,
          marker: {
            name: variantName,
            ty: "variant",
            scope: field.scope,
            parser: null,
            comment: null,
            default: null,
            range: null,
            length: null,
            source: null
          },
          fields: []
        };
        variants.push(variant);
      }
      variant.fields.push({ field, fieldIndex, displayName: fieldName });
      current = variant;
      return;
    }

    if (current) {
      current.fields.push({ field, fieldIndex, displayName: field.name });
    }
  });

  return variants;
}

export function unionFieldCount(node: StudioNode) {
  if (node.kind !== "union") return node.fields.length;
  return unionVariants(node).reduce((count, variant) => count + variant.fields.length, 0);
}

export function unionVariantCount(node: StudioNode) {
  if (node.kind !== "union") return node.fields.length;
  return unionVariants(node).length;
}

function splitVariantField(name: string): [string | null, string] {
  const index = name.indexOf(".");
  if (index < 1 || index === name.length - 1) return [null, name];
  return [name.slice(0, index), name.slice(index + 1)];
}
