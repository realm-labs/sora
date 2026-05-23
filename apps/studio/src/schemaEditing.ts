import type { EdgeKind, NodeKind, StudioEdge, StudioField, StudioNode, StudioSchema } from "./types";
import { unionVariantCount } from "./unionFields";

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
};

export type EditableFieldDraft = {
  name: string;
  ty: string;
  parser: string;
  comment: string;
};

export function makeFieldDraft(field?: StudioField): EditableFieldDraft {
  return {
    name: field?.name ?? "",
    ty: field?.ty ?? "",
    parser: field?.parser ?? "",
    comment: field?.comment ?? ""
  };
}

export function commitFieldDraft(field: StudioField | undefined, draft: EditableFieldDraft): StudioField {
  return {
    name: draft.name.trim(),
    ty: draft.ty.trim(),
    scope: field?.scope ?? "",
    parser: cleanOptional(draft.parser),
    comment: cleanOptional(draft.comment),
    default: field?.default ?? null,
    range: field?.range ?? null,
    length: field?.length ?? null,
    source: field?.source ?? null
  };
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

    const fieldNames = new Set<string>();
    node.fields.forEach((field, index) => {
      if (!field.name.trim()) {
        issues.push({
          id: `${node.id}:field-${index}:empty-name`,
          targetId: node.id,
          message: `${node.name} has a field without a name.`
        });
      } else if (fieldNames.has(field.name)) {
        issues.push({
          id: `${node.id}:field-${index}:duplicate`,
          targetId: node.id,
          message: `${node.name}.${field.name} is duplicated.`
        });
      }
      fieldNames.add(field.name);
      if (!field.ty.trim()) {
        issues.push({
          id: `${node.id}:field-${index}:empty-type`,
          targetId: node.id,
          message: `${node.name}.${field.name || "<unnamed>"} needs a type.`
        });
      }
      for (const refTarget of refTargets(field.ty)) {
        if (!tableNames.has(refTarget.table)) {
          issues.push({
            id: `${node.id}:field-${index}:missing-ref:${refTarget.table}`,
            targetId: node.id,
            message: `${node.name}.${field.name} references missing table "${refTarget.table}".`
          });
        }
      }
    });
  }

  return issues;
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

function refTargets(ty: string) {
  return [...ty.matchAll(/\bref<\s*([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*>/g)].map(
    (match) => ({ table: match[1], field: match[2] })
  );
}

function parseSourceEdge(source: string) {
  const [table, rest] = source.split(":");
  const [keys] = (rest ?? "").split(",");
  const [childKey] = keys.trim().split(" -> ");
  if (!table?.trim() || !childKey?.trim()) return null;
  return { table: table.trim(), childKey: childKey.trim() };
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

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
