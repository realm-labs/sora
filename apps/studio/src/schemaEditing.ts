import type { EdgeKind, NodeKind, StudioEdge, StudioField, StudioNode, StudioSchema } from "./types";
import { unionVariantCount, unionVariants, type UnionVariantView } from "./unionFields";

const primitiveTypes = new Set(["bool", "i32", "i64", "f32", "f64", "string"]);
const kindPrefixes: Record<NodeKind, string> = {
  enum: "Enum",
  struct: "Struct",
  union: "Union",
  table: "Table"
};

export type StudioValidationIssue = {
  id: string;
  message: string;
  targetId?: string;
  fieldIndex?: number;
  setting?: string;
};

export type EditableFieldDraft = {
  name: string;
  ty: string;
  scope: string;
  parser: string;
  comment: string;
  defaultValue: string;
  rangeMin: string;
  rangeMax: string;
  lengthMin: string;
  lengthMax: string;
  sourceTable: string;
  parentKey: string;
  childKey: string;
  valueField: string;
  orderBy: string;
};

export type EditableNodeSettingsDraft = {
  schemaSource: string;
  scope: string;
  mode: string;
  key: string;
  source: string;
  sheet: string;
  tag: string;
};

export function makeFieldDraft(field?: StudioField): EditableFieldDraft {
  const source = parseSourceDraft(field?.source ?? null);
  return {
    name: field?.name ?? "",
    ty: field?.ty ?? "",
    scope: field?.scope ?? "",
    parser: field?.parser ?? "",
    comment: field?.comment ?? "",
    defaultValue: field?.default ?? "",
    rangeMin: field?.range?.[0]?.toString() ?? "",
    rangeMax: field?.range?.[1]?.toString() ?? "",
    lengthMin: field?.length?.[0]?.toString() ?? "",
    lengthMax: field?.length?.[1]?.toString() ?? "",
    sourceTable: source.table,
    parentKey: source.parentKey,
    childKey: source.childKey,
    valueField: source.valueField,
    orderBy: source.orderBy
  };
}

export function commitFieldDraft(field: StudioField | undefined, draft: EditableFieldDraft): StudioField {
  return {
    name: draft.name.trim(),
    ty: draft.ty.trim(),
    scope: draft.scope.trim(),
    parser: cleanOptional(draft.parser),
    comment: cleanOptional(draft.comment),
    default: cleanOptional(draft.defaultValue),
    range: numberPair(draft.rangeMin, draft.rangeMax),
    length: numberPair(draft.lengthMin, draft.lengthMax),
    source: sourceDraftValue(draft)
  };
}

export function makeNodeSettingsDraft(node: StudioNode): EditableNodeSettingsDraft {
  return {
    schemaSource: node.source,
    scope: node.scope,
    mode: node.metadata.mode ?? "map",
    key: node.metadata.key === "<none>" ? "" : (node.metadata.key ?? ""),
    source: node.metadata.source ?? "",
    sheet: node.metadata.sheet ?? "",
    tag: node.metadata.tag ?? "type"
  };
}

export function updateNodeSettings(
  schema: StudioSchema,
  nodeIdToUpdate: string,
  draft: EditableNodeSettingsDraft
): StudioSchema {
  return rebuildSchema({
    ...schema,
    nodes: schema.nodes.map((node) => {
      if (node.id !== nodeIdToUpdate) return node;
      const metadata = { ...node.metadata };
      if (node.kind === "table") {
        const mode = draft.mode.trim() || "map";
        metadata.mode = mode;
        metadata.key = mode === "map" ? draft.key.trim() || "<none>" : "<none>";
        setOptionalMetadata(metadata, "source", draft.source);
        setOptionalMetadata(metadata, "sheet", draft.sheet);
      }
      if (node.kind === "union") {
        metadata.tag = draft.tag.trim() || "type";
      }
      return {
        ...node,
        source: draft.schemaSource.trim() || node.source,
        scope: draft.scope.trim(),
        metadata
      };
    })
  });
}

export function updatePackage(schema: StudioSchema, packageName: string): StudioSchema {
  const cleanName = packageName.trim();
  if (!cleanName || cleanName === schema.package) return schema;
  return { ...schema, package: cleanName };
}

export function addSchemaSource(schema: StudioSchema, source: string): StudioSchema {
  const cleanSource = source.trim();
  if (!cleanSource || schema.sources.includes(cleanSource)) return schema;
  return { ...schema, sources: [...schema.sources, cleanSource] };
}

export function deleteSchemaSource(schema: StudioSchema, source: string): StudioSchema {
  if (schema.sources.length <= 1) return schema;
  if (schema.nodes.some((node) => node.source === source)) return schema;
  return { ...schema, sources: schema.sources.filter((item) => item !== source) };
}

export function addNode(schema: StudioSchema, kind: NodeKind): { schema: StudioSchema; nodeId: string } {
  const name = nextAvailableName(
    schema.nodes.filter((node) => node.kind === kind).map((node) => node.name),
    kindPrefixes[kind]
  );
  const node: StudioNode = {
    id: nodeId(kind, name),
    name,
    kind,
    source: schema.sources[0] ?? "",
    scope: "local",
    subtitle: subtitleFor(kind, 0),
    fields: kind === "enum" ? [{ name: "Value", ty: "enum value", scope: "local", parser: null, comment: null, default: null, range: null, length: null, source: null }] : [],
    metadata: defaultMetadata(kind)
  };
  return { schema: rebuildSchema({ ...schema, nodes: [...schema.nodes, node] }), nodeId: node.id };
}

export function renameNode(schema: StudioSchema, nodeIdToRename: string, nextName: string): StudioSchema {
  const current = schema.nodes.find((node) => node.id === nodeIdToRename);
  const cleanName = nextName.trim();
  if (!current || !cleanName || cleanName === current.name) return schema;
  const nextId = nodeId(current.kind, cleanName);
  const nodes = schema.nodes.map((node) => {
    if (node.id === nodeIdToRename) {
      return {
        ...node,
        id: nextId,
        name: cleanName
      };
    }
    return {
      ...node,
      fields: node.fields.map((field) => ({
        ...field,
        ty: renameTypeReference(field.ty, current, cleanName)
      }))
    };
  });
  return rebuildSchema({ ...schema, nodes });
}

export function deleteNode(schema: StudioSchema, nodeIdToDelete: string): StudioSchema {
  return rebuildSchema({
    ...schema,
    nodes: schema.nodes.filter((node) => node.id !== nodeIdToDelete)
  });
}

export function addField(schema: StudioSchema, ownerId: string, draft: EditableFieldDraft): StudioSchema {
  const field = commitFieldDraft(undefined, draft);
  if (!field.name || !field.ty) return schema;
  return rebuildSchema({
    ...schema,
    nodes: schema.nodes.map((node) =>
      node.id === ownerId ? { ...node, fields: [...node.fields, field] } : node
    )
  });
}

export function addEnumValue(schema: StudioSchema, ownerId: string, name: string): StudioSchema {
  const cleanName = name.trim();
  if (!cleanName) return schema;
  return rebuildSchema({
    ...schema,
    nodes: schema.nodes.map((node) =>
      node.id === ownerId && node.kind === "enum"
        ? { ...node, fields: [...node.fields, enumValueField(cleanName, node.scope)] }
        : node
    )
  });
}

export function updateEnumValue(
  schema: StudioSchema,
  ownerId: string,
  fieldIndex: number,
  name: string
): StudioSchema {
  const cleanName = name.trim();
  if (!cleanName) return schema;
  return rebuildSchema({
    ...schema,
    nodes: schema.nodes.map((node) => {
      if (node.id !== ownerId || node.kind !== "enum") return node;
      return {
        ...node,
        fields: node.fields.map((field, index) =>
          index === fieldIndex ? { ...field, name: cleanName, ty: "enum value" } : field
        )
      };
    })
  });
}

export function addUnionVariant(schema: StudioSchema, ownerId: string, name: string): StudioSchema {
  const cleanName = name.trim();
  if (!cleanName) return schema;
  return rebuildSchema({
    ...schema,
    nodes: schema.nodes.map((node) =>
      node.id === ownerId && node.kind === "union"
        ? { ...node, fields: [...node.fields, unionVariantMarker(cleanName, node.scope)] }
        : node
    )
  });
}

export function updateUnionVariant(
  schema: StudioSchema,
  ownerId: string,
  fieldIndex: number,
  name: string
): StudioSchema {
  const cleanName = name.trim();
  if (!cleanName) return schema;
  return updateUnionVariants(schema, ownerId, (variants) =>
    variants.map((variant) =>
      variant.fieldIndex === fieldIndex ? { ...variant, name: cleanName } : variant
    )
  );
}

export function deleteUnionVariant(schema: StudioSchema, ownerId: string, fieldIndex: number): StudioSchema {
  return updateUnionVariants(schema, ownerId, (variants) =>
    variants.filter((variant) => variant.fieldIndex !== fieldIndex)
  );
}

export function addUnionVariantField(
  schema: StudioSchema,
  ownerId: string,
  variantName: string,
  draft: EditableFieldDraft
): StudioSchema {
  const field = commitFieldDraft(undefined, draft);
  if (!field.name || !field.ty) return schema;
  return updateUnionVariants(schema, ownerId, (variants) =>
    variants.map((variant) =>
      variant.name === variantName
        ? {
            ...variant,
            fields: [
              ...variant.fields,
              {
                field: { ...field, name: `${variant.name}.${field.name}` },
                fieldIndex: -1,
                displayName: field.name
              }
            ]
          }
        : variant
    )
  );
}

export function updateField(
  schema: StudioSchema,
  ownerId: string,
  fieldIndex: number,
  draft: EditableFieldDraft
): StudioSchema {
  return rebuildSchema({
    ...schema,
    nodes: schema.nodes.map((node) => {
      if (node.id !== ownerId) return node;
      return {
        ...node,
        fields: node.fields.map((field, index) =>
          index === fieldIndex ? commitFieldDraft(field, draft) : field
        )
      };
    })
  });
}

export function deleteField(schema: StudioSchema, ownerId: string, fieldIndex: number): StudioSchema {
  return rebuildSchema({
    ...schema,
    nodes: schema.nodes.map((node) => {
      if (node.id !== ownerId) return node;
      return { ...node, fields: node.fields.filter((_, index) => index !== fieldIndex) };
    })
  });
}

export function moveField(
  schema: StudioSchema,
  ownerId: string,
  fieldIndex: number,
  direction: -1 | 1
): StudioSchema {
  return rebuildSchema({
    ...schema,
    nodes: schema.nodes.map((node) => {
      if (node.id !== ownerId || node.kind === "union") return node;
      const targetIndex = fieldIndex + direction;
      if (targetIndex < 0 || targetIndex >= node.fields.length) return node;
      return { ...node, fields: moveItem(node.fields, fieldIndex, targetIndex) };
    })
  });
}

export function moveUnionVariant(
  schema: StudioSchema,
  ownerId: string,
  fieldIndex: number,
  direction: -1 | 1
): StudioSchema {
  return updateUnionVariants(schema, ownerId, (variants) => {
    const index = variants.findIndex((variant) => variant.fieldIndex === fieldIndex);
    const targetIndex = index + direction;
    if (index < 0 || targetIndex < 0 || targetIndex >= variants.length) return variants;
    return moveItem(variants, index, targetIndex);
  });
}

export function moveUnionVariantField(
  schema: StudioSchema,
  ownerId: string,
  variantName: string,
  fieldIndex: number,
  direction: -1 | 1
): StudioSchema {
  return updateUnionVariants(schema, ownerId, (variants) =>
    variants.map((variant) => {
      if (variant.name !== variantName) return variant;
      const index = variant.fields.findIndex((item) => item.fieldIndex === fieldIndex);
      const targetIndex = index + direction;
      if (index < 0 || targetIndex < 0 || targetIndex >= variant.fields.length) return variant;
      return { ...variant, fields: moveItem(variant.fields, index, targetIndex) };
    })
  );
}

export function rebuildSchema(schema: StudioSchema): StudioSchema {
  const nodes = schema.nodes.map((node) => ({
    ...node,
    subtitle: subtitleFor(node.kind, node.fields.length),
    metadata: {
      ...node.metadata,
      [node.kind === "union" ? "variants" : node.kind === "enum" ? "values" : "fields"]: (
        node.kind === "union" ? unionVariantCount(node) : node.fields.length
      ).toString()
    }
  }));
  const edges = buildEdges(nodes);
  return {
    ...schema,
    nodes,
    edges,
    summary: {
      enums: nodes.filter((node) => node.kind === "enum").length,
      structs: nodes.filter((node) => node.kind === "struct").length,
      unions: nodes.filter((node) => node.kind === "union").length,
      tables: nodes.filter((node) => node.kind === "table").length,
      edges: edges.length
    }
  };
}

export function validateSchema(schema: StudioSchema): StudioValidationIssue[] {
  const issues: StudioValidationIssue[] = [];
  const namesByKind = new Map<NodeKind, Set<string>>();
  const tableNames = new Set(schema.nodes.filter((node) => node.kind === "table").map((node) => node.name));

  if (!schema.package.trim()) {
    issues.push({ id: "schema:package", message: "Schema package is required.", setting: "package" });
  }
  for (const issue of validateSchemaSources(schema)) {
    issues.push(issue);
  }

  for (const node of schema.nodes) {
    const names = namesByKind.get(node.kind) ?? new Set<string>();
    if (!node.name.trim()) {
      issues.push({ id: `${node.id}:empty-name`, targetId: node.id, message: "Schema item name is required." });
    } else if (names.has(node.name)) {
      issues.push({
        id: `${node.id}:duplicate-node`,
        targetId: node.id,
        message: `${node.kind} name "${node.name}" is duplicated.`
      });
    }
    names.add(node.name);
    namesByKind.set(node.kind, names);
    if (!schema.sources.includes(node.source)) {
      issues.push({
        id: `${node.id}:schema-source`,
        targetId: node.id,
        message: `${node.name} belongs to unknown schema file "${node.source}".`
      });
    }

    const fieldNames = new Set<string>();
    node.fields.forEach((field, index) => {
      if (!field.name.trim()) {
        issues.push({
          id: `${node.id}:field-${index}:empty-name`,
          targetId: node.id,
          fieldIndex: index,
          setting: "name",
          message: `${node.name} has a field without a name.`
        });
      } else if (fieldNames.has(field.name)) {
        issues.push({
          id: `${node.id}:field-${index}:duplicate`,
          targetId: node.id,
          fieldIndex: index,
          setting: "name",
          message: `${node.name}.${field.name} is duplicated.`
        });
      }
      fieldNames.add(field.name);
      if (field.ty !== "enum value" && field.ty !== "variant" && !field.ty.trim()) {
        issues.push({
          id: `${node.id}:field-${index}:empty-type`,
          targetId: node.id,
          fieldIndex: index,
          setting: "type",
          message: `${node.name}.${field.name || "<unnamed>"} needs a type.`
        });
      }
      if (field.ty !== "enum value" && field.ty !== "variant") {
        for (const issue of validateFieldType(schema, node, field, index)) {
          issues.push(issue);
        }
        for (const issue of validateFieldParser(node, field, index)) {
          issues.push(issue);
        }
      }
      for (const refTarget of refTargets(field.ty)) {
        if (!tableNames.has(refTarget.table)) {
          issues.push({
            id: `${node.id}:field-${index}:missing-ref:${refTarget.table}`,
            targetId: node.id,
            fieldIndex: index,
            setting: "type",
            message: `${node.name}.${field.name} references missing table "${refTarget.table}".`
          });
        } else {
          const table = schema.nodes.find((item) => item.kind === "table" && item.name === refTarget.table);
          if (table && !table.fields.some((item) => item.name === refTarget.field)) {
            issues.push({
              id: `${node.id}:field-${index}:missing-ref-field:${refTarget.table}.${refTarget.field}`,
              targetId: node.id,
              fieldIndex: index,
              setting: "type",
              message: `${node.name}.${field.name} references missing field "${refTarget.table}.${refTarget.field}".`
            });
          }
        }
      }
      if (field.source && !parseSourceEdge(field.source)) {
        issues.push({
          id: `${node.id}:field-${index}:invalid-source`,
          targetId: node.id,
          fieldIndex: index,
          setting: "source",
          message: `${node.name}.${field.name} has an invalid derived source.`
        });
      }
    });
    if (node.kind === "table" && node.metadata.mode === "map") {
      const key = node.metadata.key;
      if (!key || key === "<none>") {
        issues.push({
          id: `${node.id}:missing-key`,
          targetId: node.id,
          setting: "key",
          message: `${node.name} map table must declare a key.`
        });
      } else if (!node.fields.some((field) => field.name === key)) {
        issues.push({
          id: `${node.id}:missing-key:${key}`,
          targetId: node.id,
          setting: "key",
          message: `${node.name} map key "${key}" does not match any field.`
        });
      }
    }
  }

  return issues;
}

function validateSchemaSources(schema: StudioSchema): StudioValidationIssue[] {
  const issues: StudioValidationIssue[] = [];
  if (schema.sources.length === 0) {
    issues.push({ id: "schema:sources:empty", message: "At least one schema file is required." });
  }
  const seen = new Set<string>();
  schema.sources.forEach((source, index) => {
    if (!source.trim()) {
      issues.push({ id: `schema:sources:${index}:empty`, message: "Schema file path cannot be empty." });
      return;
    }
    if (source.trim() !== source) {
      issues.push({
        id: `schema:sources:${index}:whitespace`,
        message: `Schema file "${source}" cannot contain surrounding whitespace.`
      });
    }
    if (source.startsWith("/") || source.includes("\\") || source.split("/").some((part) => part === "." || part === ".." || part === "")) {
      issues.push({
        id: `schema:sources:${index}:relative`,
        message: `Schema file "${source}" must be a plain relative path.`
      });
    }
    if (!schemaSourceExtension(source)) {
      issues.push({
        id: `schema:sources:${index}:extension`,
        message: `Schema file "${source}" must end with .toml, .yaml, .yml, .json, or .lua.`
      });
    }
    if (seen.has(source)) {
      issues.push({
        id: `schema:sources:${index}:duplicate`,
        message: `Schema file "${source}" is duplicated.`
      });
    }
    seen.add(source);
  });
  return issues;
}

function schemaSourceExtension(source: string): boolean {
  return /\.(toml|ya?ml|json|lua)$/.test(source);
}

function buildEdges(nodes: StudioNode[]): StudioEdge[] {
  const edges = new Map<string, StudioEdge>();
  for (const node of nodes) {
    for (const field of node.fields) {
      for (const target of targetsForField(nodes, field)) {
        const id = `${node.id}->${target.node.id}:${target.kind}:${field.name}`;
        edges.set(id, {
          id,
          source: node.id,
          target: target.node.id,
          kind: target.kind,
          label: field.name,
          targetLabel: target.targetLabel
        });
      }
      if (field.source) {
        const source = parseSourceEdge(field.source);
        const sourceTable = source?.table ?? field.source.split(":")[0]?.trim();
        const target = nodes.find((item) => item.kind === "table" && item.name === sourceTable);
        if (target) {
          const id = `${node.id}->${target.id}:derived:${field.name}`;
          edges.set(id, {
            id,
            source: node.id,
            target: target.id,
            kind: "derived",
            label: field.name,
            targetLabel: source?.childKey ?? null
          });
        }
      }
    }
  }
  return [...edges.values()].sort((a, b) => a.id.localeCompare(b.id));
}

function targetsForField(
  nodes: StudioNode[],
  field: StudioField
): Array<{ node: StudioNode; kind: EdgeKind; targetLabel: string | null }> {
  const targets: Array<{ node: StudioNode; kind: EdgeKind; targetLabel: string | null }> = [];
  for (const refTarget of refTargets(field.ty)) {
    const node = nodes.find((item) => item.kind === "table" && item.name === refTarget.table);
    if (node) targets.push({ node, kind: "ref", targetLabel: refTarget.field });
  }
  for (const node of nodes) {
    if (node.kind === "table") continue;
    if (typeMentionsSymbol(field.ty, node.name)) {
      targets.push({ node, kind: "type", targetLabel: null });
    }
  }
  return targets;
}

function validateFieldType(
  schema: StudioSchema,
  owner: StudioNode,
  field: StudioField,
  fieldIndex: number
): StudioValidationIssue[] {
  const issues: StudioValidationIssue[] = [];
  for (const reference of namedTypeReferences(field.ty)) {
    if (!schema.nodes.some((node) => node.kind === reference.kind && node.name === reference.name)) {
      issues.push({
        id: `${owner.id}:field-${fieldIndex}:missing-type:${reference.kind}:${reference.name}`,
        targetId: owner.id,
        fieldIndex,
        setting: "type",
        message: `${owner.name}.${field.name} references missing ${reference.kind} "${reference.name}".`
      });
    }
  }
  return issues;
}

function validateFieldParser(
  owner: StudioNode,
  field: StudioField,
  fieldIndex: number
): StudioValidationIssue[] {
  if (!field.parser) return [];
  const parser = parseParser(field.parser);
  if (!parser.kind) {
    return [
      {
        id: `${owner.id}:field-${fieldIndex}:parser-kind`,
        targetId: owner.id,
        fieldIndex,
        setting: "parser",
        message: `${owner.name}.${field.name} parser needs a kind.`
      }
    ];
  }
  const allowedOptions = parserAllowedOptions(parser.kind);
  if (!allowedOptions) {
    return [
      {
        id: `${owner.id}:field-${fieldIndex}:parser-unsupported:${parser.kind}`,
        targetId: owner.id,
        fieldIndex,
        setting: "parser",
        message: `${owner.name}.${field.name} declares unsupported parser "${parser.kind}".`
      }
    ];
  }

  const issues: StudioValidationIssue[] = [];
  for (const [key, value] of parser.options) {
    if (!allowedOptions.includes(key)) {
      issues.push({
        id: `${owner.id}:field-${fieldIndex}:parser-option:${key}`,
        targetId: owner.id,
        fieldIndex,
        setting: "parser",
        message: `${owner.name}.${field.name} parser "${parser.kind}" does not support option "${key}".`
      });
    }
    if (key !== "prefix" && !value.trim()) {
      issues.push({
        id: `${owner.id}:field-${fieldIndex}:parser-option-empty:${key}`,
        targetId: owner.id,
        fieldIndex,
        setting: "parser",
        message: `${owner.name}.${field.name} parser option "${key}" cannot be empty.`
      });
    }
  }

  if (!parserMatchesType(parser.kind, field.ty)) {
    issues.push({
      id: `${owner.id}:field-${fieldIndex}:parser-target:${parser.kind}`,
      targetId: owner.id,
      fieldIndex,
      setting: "parser",
      message: `${owner.name}.${field.name} parser "${parser.kind}" is not compatible with type "${field.ty}".`
    });
  }
  if ((parser.kind === "columns" || parser.kind === "tagged_columns") && owner.kind !== "table") {
    issues.push({
      id: `${owner.id}:field-${fieldIndex}:parser-table-only:${parser.kind}`,
      targetId: owner.id,
      fieldIndex,
      setting: "parser",
      message: `${owner.name}.${field.name} parser "${parser.kind}" is only supported on table fields.`
    });
  }
  return issues;
}

function parseParser(value: string): { kind: string; options: Array<[string, string]> } {
  const clean = value.trim();
  const match = clean.match(/^([^()]+?)(?:\s+\((.*)\))?$/);
  const kind = match?.[1]?.trim() ?? clean;
  const options = parseParserOptionPairs(match?.[2] ?? "");
  return { kind, options };
}

function parseParserOptionPairs(optionsText: string): Array<[string, string]> {
  return splitParserOptions(optionsText).flatMap((part) => {
    const clean = part.trim();
    if (!clean || !clean.includes("=")) return [];
    const [key, ...valueParts] = clean.split("=");
    return [[key.trim(), parseParserOptionValue(valueParts.join("=").trim())]];
  });
}

function splitParserOptions(optionsText: string) {
  const parts: string[] = [];
  let start = 0;
  let quoted = false;
  let escaped = false;
  for (let index = 0; index < optionsText.length; index += 1) {
    const char = optionsText[index];
    if (escaped) {
      escaped = false;
      continue;
    }
    if (char === "\\") {
      escaped = true;
      continue;
    }
    if (char === "\"") quoted = !quoted;
    if (char === "," && !quoted) {
      parts.push(optionsText.slice(start, index));
      start = index + 1;
    }
  }
  parts.push(optionsText.slice(start));
  return parts;
}

function parseParserOptionValue(value: string) {
  if (value.startsWith("\"") && value.endsWith("\"")) {
    try {
      const parsed = JSON.parse(value);
      if (typeof parsed === "string") return parsed;
    } catch {
      return value;
    }
  }
  return value;
}

function parserAllowedOptions(kind: string): string[] | null {
  if (kind === "json") return [];
  if (kind === "split" || kind === "tuple") return ["separator"];
  if (kind === "tuple_list" || kind === "map") return ["separator", "item_separator"];
  if (kind === "columns" || kind === "tagged_columns") return ["prefix"];
  return null;
}

function parserMatchesType(kind: string, ty: string) {
  if (kind === "json") return true;
  if (kind === "split") return typeHasCollectionShape(ty);
  if (kind === "tuple") return typeHasNamedShape(ty, "struct");
  if (kind === "tuple_list") return typeHasCollectionOfNamedShape(ty, "struct");
  if (kind === "map") return typeHasMapShape(ty);
  if (kind === "columns") return typeHasNamedShape(ty, "struct");
  if (kind === "tagged_columns") return typeHasNamedShape(ty, "union");
  return false;
}

function typeHasCollectionShape(ty: string): boolean {
  const value = unwrapOptionalType(ty);
  return /^(list|set|array)</.test(value);
}

function typeHasCollectionOfNamedShape(ty: string, kind: "struct" | "union"): boolean {
  const value = unwrapOptionalType(ty);
  const generic = parseTopLevelGeneric(value);
  if (!generic || (generic.name !== "list" && generic.name !== "array")) return false;
  return typeHasNamedShape(generic.args[0] ?? "", kind);
}

function typeHasMapShape(ty: string): boolean {
  return parseTopLevelGeneric(unwrapOptionalType(ty))?.name === "map";
}

function typeHasNamedShape(ty: string, kind: "struct" | "union"): boolean {
  return parseTopLevelGeneric(unwrapOptionalType(ty))?.name === kind;
}

function unwrapOptionalType(ty: string): string {
  let value = ty.trim();
  for (;;) {
    const generic = parseTopLevelGeneric(value);
    if (generic?.name !== "optional") return value;
    value = generic.args[0] ?? "";
  }
}

function parseTopLevelGeneric(value: string): { name: string; args: string[] } | null {
  const open = value.indexOf("<");
  if (open < 1 || !value.endsWith(">")) return null;
  return { name: value.slice(0, open).trim(), args: splitTopLevelArgs(value.slice(open + 1, -1)) };
}

function namedTypeReferences(ty: string): Array<{ kind: Exclude<NodeKind, "table">; name: string }> {
  const references: Array<{ kind: Exclude<NodeKind, "table">; name: string }> = [];
  for (const match of ty.matchAll(/\b(enum|struct|union)<\s*([A-Za-z_][A-Za-z0-9_]*)\s*>/g)) {
    references.push({ kind: match[1] as Exclude<NodeKind, "table">, name: match[2] });
  }
  return references;
}

function refTargets(ty: string) {
  return [...ty.matchAll(/\bref<\s*([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*>/g)].map(
    (match) => ({ table: match[1], field: match[2] })
  );
}

function parseSourceEdge(source: string) {
  const [table, rest] = source.split(":");
  const [keys] = (rest ?? "").split(",");
  const [childKey, parentKey] = keys.trim().split(" -> ");
  if (!table?.trim() || !childKey?.trim() || !parentKey?.trim()) return null;
  return { table: table.trim(), childKey: childKey.trim(), parentKey: parentKey.trim() };
}

function typeMentionsSymbol(ty: string, name: string) {
  if (primitiveTypes.has(name)) return false;
  return new RegExp(`(^|[^A-Za-z0-9_])${escapeRegExp(name)}([^A-Za-z0-9_]|$)`).test(ty);
}

function renameTypeReference(ty: string, node: StudioNode, nextName: string) {
  if (node.kind === "table") {
    return ty.replace(
      new RegExp(`\\bref<\\s*${escapeRegExp(node.name)}\\s*\\.`, "g"),
      `ref<${nextName}.`
    );
  }
  return ty.replace(
    new RegExp(`(^|[^A-Za-z0-9_])${escapeRegExp(node.name)}(?=[^A-Za-z0-9_]|$)`, "g"),
    `$1${nextName}`
  );
}

function defaultMetadata(kind: NodeKind): Record<string, string> {
  if (kind === "table") return { mode: "map", key: "id", fields: "0" };
  if (kind === "union") return { tag: "type", variants: "0" };
  if (kind === "enum") return { values: "1" };
  return { fields: "0" };
}

function enumValueField(name: string, scope: string): StudioField {
  return {
    name,
    ty: "enum value",
    scope: scope || "all",
    parser: null,
    comment: null,
    default: null,
    range: null,
    length: null,
    source: null
  };
}

function unionVariantMarker(name: string, scope: string): StudioField {
  return {
    name,
    ty: "variant",
    scope: scope || "all",
    parser: null,
    comment: null,
    default: null,
    range: null,
    length: null,
    source: null
  };
}

function updateUnionVariants(
  schema: StudioSchema,
  ownerId: string,
  update: (variants: UnionVariantView[]) => UnionVariantView[]
): StudioSchema {
  return rebuildSchema({
    ...schema,
    nodes: schema.nodes.map((node) => {
      if (node.id !== ownerId || node.kind !== "union") return node;
      return { ...node, fields: flattenUnionVariants(update(unionVariants(node)), node.scope) };
    })
  });
}

function flattenUnionVariants(variants: UnionVariantView[], scope: string): StudioField[] {
  return variants.flatMap((variant) => [
    {
      ...unionVariantMarker(variant.name, variant.marker.scope || scope),
      comment: variant.marker.comment
    },
    ...variant.fields.map(({ field, displayName }) => ({
      ...field,
      name: `${variant.name}.${displayName}`
    }))
  ]);
}

function moveItem<T>(items: T[], from: number, to: number): T[] {
  const next = [...items];
  const [item] = next.splice(from, 1);
  next.splice(to, 0, item);
  return next;
}

function subtitleFor(kind: NodeKind, count: number) {
  if (kind === "table") return `map table, ${count} fields`;
  if (kind === "union") return `${count} variants`;
  if (kind === "enum") return `${count} values`;
  return `${count} fields`;
}

function nodeId(kind: NodeKind, name: string) {
  return `${kind}:${name}`;
}

function nextAvailableName(existingNames: string[], prefix: string) {
  const existing = new Set(existingNames);
  if (!existing.has(prefix)) return prefix;
  let index = 2;
  while (existing.has(`${prefix}${index}`)) index += 1;
  return `${prefix}${index}`;
}

function cleanOptional(value: string) {
  const clean = value.trim();
  return clean ? clean : null;
}

function numberPair(min: string, max: string): [number, number] | null {
  const cleanMin = min.trim();
  const cleanMax = max.trim();
  if (!cleanMin && !cleanMax) return null;
  const minValue = Number(cleanMin);
  const maxValue = Number(cleanMax);
  if (!Number.isFinite(minValue) || !Number.isFinite(maxValue)) return null;
  return [minValue, maxValue];
}

function sourceDraftValue(draft: EditableFieldDraft): string | null {
  const table = draft.sourceTable.trim();
  const parentKey = draft.parentKey.trim();
  const childKey = draft.childKey.trim();
  if (!table && !parentKey && !childKey && !draft.valueField.trim() && !draft.orderBy.trim()) {
    return null;
  }
  if (!table || !parentKey || !childKey) return null;
  const parts = [`${table}: ${childKey} -> ${parentKey}`];
  if (draft.valueField.trim()) parts.push(`field=${draft.valueField.trim()}`);
  if (draft.orderBy.trim()) parts.push(`order_by=${draft.orderBy.trim()}`);
  return parts.join(", ");
}

function parseSourceDraft(source: string | null) {
  const empty = { table: "", parentKey: "", childKey: "", valueField: "", orderBy: "" };
  if (!source) return empty;
  const [table, rest] = source.split(":");
  const [keys, ...options] = (rest ?? "").split(",");
  const [childKey, parentKey] = keys.trim().split(" -> ");
  const parsed = { ...empty, table: table.trim(), parentKey: parentKey?.trim() ?? "", childKey: childKey?.trim() ?? "" };
  for (const option of options) {
    const [key, value] = option.trim().split("=");
    if (key === "field") parsed.valueField = value?.trim() ?? "";
    if (key === "order_by") parsed.orderBy = value?.trim() ?? "";
  }
  return parsed;
}

function setOptionalMetadata(metadata: Record<string, string>, key: string, value: string) {
  const clean = value.trim();
  if (clean) {
    metadata[key] = clean;
  } else {
    delete metadata[key];
  }
}

function splitTopLevelArgs(value: string) {
  const parts: string[] = [];
  let depth = 0;
  let start = 0;
  for (let index = 0; index < value.length; index += 1) {
    const char = value[index];
    if (char === "<") depth += 1;
    if (char === ">") depth -= 1;
    if (char === "," && depth === 0) {
      parts.push(value.slice(start, index).trim());
      start = index + 1;
    }
  }
  parts.push(value.slice(start).trim());
  return parts;
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
