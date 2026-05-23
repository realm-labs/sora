import { useEffect, useState } from "react";
import { AlertTriangle, Layers3, Pencil, Plus, Save, Trash2, X } from "lucide-react";

import { edgeColors, kindMeta } from "../constants";
import { translations } from "../i18n";
import {
  makeFieldDraft,
  type EditableFieldDraft,
  type StudioValidationIssue
} from "../schemaEditing";
import { unionVariants } from "../unionFields";
import type { Language, NodeKind, StudioEdge, StudioField, StudioNode, StudioSchema } from "../types";

export function Inspector({
  node,
  edges,
  language,
  onAddField,
  onDeleteField,
  onDeleteNode,
  onRenameNode,
  onUpdateField,
  schema,
  validationIssues
}: {
  node: StudioNode;
  edges: StudioEdge[];
  language: Language;
  onAddField: (ownerId: string, draft: EditableFieldDraft) => void;
  onDeleteField: (ownerId: string, fieldIndex: number) => void;
  onDeleteNode: (nodeId: string) => void;
  onRenameNode: (nodeId: string, name: string) => void;
  onUpdateField: (ownerId: string, fieldIndex: number, draft: EditableFieldDraft) => void;
  schema: StudioSchema;
  validationIssues: StudioValidationIssue[];
}) {
  const t = translations[language];
  const inbound = edges.filter((edge) => edge.target === node.id);
  const outbound = edges.filter((edge) => edge.source === node.id);
  const Icon = kindMeta[node.kind].icon;
  const variants = unionVariants(node);
  const [nameDraft, setNameDraft] = useState(node.name);
  const [addingField, setAddingField] = useState(false);
  const [editingField, setEditingField] = useState<{ index: number; draft: EditableFieldDraft } | null>(
    null
  );

  useEffect(() => {
    setNameDraft(node.name);
    setAddingField(false);
    setEditingField(null);
  }, [node.id, node.name]);

  const rename = () => {
    if (nameDraft.trim() && nameDraft.trim() !== node.name) {
      onRenameNode(node.id, nameDraft);
    }
  };
  const deleteNode = () => {
    if (window.confirm(t.deleteNodeConfirm.replace("{name}", node.name))) {
      onDeleteNode(node.id);
    }
  };
  const submitAddField = (draft: EditableFieldDraft) => {
    onAddField(node.id, draft);
    setAddingField(false);
  };
  const submitEditField = (draft: EditableFieldDraft) => {
    if (!editingField) return;
    onUpdateField(node.id, editingField.index, draft);
    setEditingField(null);
  };

  return (
    <div className="inspector-content">
      <div className="inspector-title">
        <span className="kind-icon" style={{ color: kindMeta[node.kind].color }}>
          <Icon size={20} />
        </span>
        <div className="node-title-editor">
          <p>{t.kindSingular[node.kind]}</p>
          <label>
            <input
              value={nameDraft}
              onBlur={rename}
              onChange={(event) => setNameDraft(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") rename();
              }}
            />
          </label>
        </div>
        <button className="icon-button danger icon-only" onClick={deleteNode} title={t.deleteNode}>
          <Trash2 size={16} />
        </button>
      </div>

      {validationIssues.length > 0 && (
        <section className="validation-panel">
          <h3>
            <AlertTriangle size={15} />
            {t.validation}
          </h3>
          {validationIssues.map((issue) => (
            <p key={issue.id}>{issue.message}</p>
          ))}
        </section>
      )}

      <section className="detail-section">
        <h3>{t.metadata}</h3>
        <dl>
          <div>
            <dt>{t.scope}</dt>
            <dd>{node.scope}</dd>
          </div>
          {Object.entries(node.metadata).map(([key, value]) => (
            <div key={key}>
              <dt>{key}</dt>
              <dd>{value}</dd>
            </div>
          ))}
        </dl>
      </section>

      <section className="detail-section">
        <h3>
          <Layers3 size={15} />
          {t.fields}
          <button className="section-action" onClick={() => setAddingField(true)} title={t.addField}>
            <Plus size={13} />
          </button>
        </h3>
        <div className="field-list">
          {addingField && (
            <FieldEditor
              draft={makeFieldDraft()}
              language={language}
              onCancel={() => setAddingField(false)}
              onSubmit={submitAddField}
              schema={schema}
            />
          )}
          {node.kind === "union"
            ? variants.map((variant) => (
                <section key={variant.name} className="variant-card">
                  <header>
                    <strong>{variant.name}</strong>
                    <span>{t.variant}</span>
                  </header>
                  {variant.fields.length === 0 ? (
                    <p className="muted">{t.emptyVariant}</p>
                  ) : (
                    variant.fields.map(({ field, fieldIndex, displayName }) =>
                      editingField?.index === fieldIndex ? (
                        <FieldEditor
                          key={`${field.name}:${fieldIndex}:editor`}
                          draft={editingField.draft}
                          language={language}
                          onCancel={() => setEditingField(null)}
                          onSubmit={(draft) =>
                            submitEditField(applyUnionFieldName(field.name, displayName, draft))
                          }
                          schema={schema}
                        />
                      ) : (
                        <FieldCard
                          key={`${field.name}:${field.ty}:${fieldIndex}`}
                          displayName={displayName}
                          field={field}
                          language={language}
                          onDelete={() => onDeleteField(node.id, fieldIndex)}
                          onEdit={() =>
                            setEditingField({
                              index: fieldIndex,
                              draft: makeUnionFieldDraft(field, displayName)
                            })
                          }
                        />
                      )
                    )
                  )}
                </section>
              ))
            : node.fields.map((field, index) =>
                editingField?.index === index ? (
                  <FieldEditor
                    key={`${field.name}:${index}:editor`}
                    draft={editingField.draft}
                    language={language}
                    onCancel={() => setEditingField(null)}
                    onSubmit={submitEditField}
                    schema={schema}
                  />
                ) : (
                  <FieldCard
                    key={`${field.name}:${field.ty}:${index}`}
                    field={field}
                    language={language}
                    onDelete={() => onDeleteField(node.id, index)}
                    onEdit={() => setEditingField({ index, draft: makeFieldDraft(field) })}
                  />
                )
              )}
        </div>
      </section>

      <section className="detail-section">
        <h3>{t.relations}</h3>
        <RelationList title={t.outgoing} edges={outbound} language={language} />
        <RelationList title={t.incoming} edges={inbound} language={language} />
      </section>
    </div>
  );
}

function FieldCard({
  displayName,
  field,
  language,
  onDelete,
  onEdit
}: {
  displayName?: string;
  field: StudioField;
  language: Language;
  onDelete: () => void;
  onEdit: () => void;
}) {
  const t = translations[language];
  return (
    <article className="field-card">
      <div>
        <strong>{displayName ?? field.name}</strong>
        <code>{field.ty}</code>
        <span className="field-actions">
          <button className="mini-action" onClick={onEdit} title={t.editField}>
            <Pencil size={13} />
          </button>
          <button className="mini-action danger" onClick={onDelete} title={t.deleteField}>
            <Trash2 size={13} />
          </button>
        </span>
      </div>
      {field.parser && (
        <span className="pill">
          {t.parser} {field.parser}
        </span>
      )}
      {field.source && (
        <span className="pill derived">
          {t.from} {field.source}
        </span>
      )}
      {field.comment && <p>{field.comment}</p>}
    </article>
  );
}

function makeUnionFieldDraft(field: StudioField, displayName: string): EditableFieldDraft {
  return {
    ...makeFieldDraft(field),
    name: displayName
  };
}

function applyUnionFieldName(
  originalName: string,
  displayName: string,
  draft: EditableFieldDraft
): EditableFieldDraft {
  const separator = originalName.indexOf(".");
  if (separator < 1) return draft;
  return {
    ...draft,
    name: `${originalName.slice(0, separator)}.${draft.name.trim() || displayName}`
  };
}

function FieldEditor({
  draft,
  language,
  onCancel,
  onSubmit,
  schema
}: {
  draft: EditableFieldDraft;
  language: Language;
  onCancel: () => void;
  onSubmit: (draft: EditableFieldDraft) => void;
  schema: StudioSchema;
}) {
  const t = translations[language];
  const [value, setValue] = useState(draft);
  const [submitted, setSubmitted] = useState(false);
  const typeDraft = parseTypeDraft(value.ty, schema);
  const nameInvalid = submitted && !value.name.trim();
  const typeInvalid = submitted && !value.ty.trim();
  const submit = () => {
    setSubmitted(true);
    if (!value.name.trim() || !value.ty.trim()) return;
    onSubmit(value);
  };
  return (
    <article className="field-editor">
      <label className={nameInvalid ? "invalid" : ""}>
        <span>{t.name}</span>
        <input
          aria-invalid={nameInvalid}
          required
          value={value.name}
          onChange={(event) => setValue({ ...value, name: event.target.value })}
          placeholder="id"
        />
        {nameInvalid && <em>{t.required}</em>}
      </label>
      <label className={typeInvalid ? "invalid" : ""}>
        <span>{t.type}</span>
        <TypeEditor
          draft={typeDraft}
          language={language}
          onChange={(next) => setValue({ ...value, ty: serializeTypeDraft(next) })}
          schema={schema}
        />
        {typeInvalid && <em>{t.required}</em>}
      </label>
      <label>
        <span>{t.parser}</span>
        <input
          value={value.parser}
          onChange={(event) => setValue({ ...value, parser: event.target.value })}
          placeholder="split"
        />
      </label>
      <label>
        <span>{t.comment}</span>
        <textarea
          value={value.comment}
          onChange={(event) => setValue({ ...value, comment: event.target.value })}
          rows={3}
        />
      </label>
      <div className="editor-actions">
        <button className="icon-button" onClick={submit}>
          <Save size={14} />
          {t.apply}
        </button>
        <button className="icon-button subtle" onClick={onCancel}>
          <X size={14} />
          {t.cancel}
        </button>
      </div>
    </article>
  );
}

type TypeDraft =
  | { kind: "primitive"; name: string }
  | { kind: "enum" | "struct" | "union"; name: string }
  | { kind: "ref"; table: string; field: string }
  | { kind: "optional" | "list" | "set"; inner: TypeDraft }
  | { kind: "array"; inner: TypeDraft; length: string }
  | { kind: "map"; key: TypeDraft; value: TypeDraft };

const primitiveTypeNames = ["bool", "i32", "i64", "f32", "f64", "string"];
const typeKindOptions = [
  "primitive",
  "enum",
  "struct",
  "union",
  "ref",
  "optional",
  "list",
  "set",
  "array",
  "map"
] as const;

function TypeEditor({
  draft,
  language,
  onChange,
  schema
}: {
  draft: TypeDraft;
  language: Language;
  onChange: (draft: TypeDraft) => void;
  schema: StudioSchema;
}) {
  const t = translations[language];
  return (
    <div className="type-editor">
      <select
        aria-label={t.typeKind}
        value={draft.kind}
        onChange={(event) => onChange(defaultTypeDraft(event.target.value as TypeDraft["kind"], schema))}
      >
        {typeKindOptions.map((kind) => (
          <option key={kind} value={kind}>
            {t.typeKinds[kind]}
          </option>
        ))}
      </select>
      {draft.kind === "primitive" && (
        <select
          aria-label={t.typeName}
          value={draft.name}
          onChange={(event) => onChange({ kind: "primitive", name: event.target.value })}
        >
          {primitiveTypeNames.map((name) => (
            <option key={name} value={name}>
              {name}
            </option>
          ))}
        </select>
      )}
      {(draft.kind === "enum" || draft.kind === "struct" || draft.kind === "union") && (
        <select
          aria-label={t.typeName}
          value={draft.name}
          onChange={(event) => onChange({ kind: draft.kind, name: event.target.value })}
        >
          {nodesOfKind(schema, draft.kind).map((node) => (
            <option key={node.id} value={node.name}>
              {node.name}
            </option>
          ))}
        </select>
      )}
      {draft.kind === "ref" && (
        <div className="type-editor-row">
          <select
            aria-label={t.table}
            value={draft.table}
            onChange={(event) => {
              const table = schema.nodes.find((node) => node.kind === "table" && node.name === event.target.value);
              onChange({
                kind: "ref",
                table: event.target.value,
                field: table?.fields[0]?.name ?? "id"
              });
            }}
          >
            {nodesOfKind(schema, "table").map((node) => (
              <option key={node.id} value={node.name}>
                {node.name}
              </option>
            ))}
          </select>
          <select
            aria-label={t.field}
            value={draft.field}
            onChange={(event) => onChange({ ...draft, field: event.target.value })}
          >
            {fieldsForTable(schema, draft.table).map((field) => (
              <option key={field.name} value={field.name}>
                {field.name}
              </option>
            ))}
          </select>
        </div>
      )}
      {(draft.kind === "optional" || draft.kind === "list" || draft.kind === "set") && (
        <TypeEditor
          draft={draft.inner}
          language={language}
          onChange={(inner) => onChange({ ...draft, inner })}
          schema={schema}
        />
      )}
      {draft.kind === "array" && (
        <>
          <input
            aria-label={t.length}
            min={1}
            onChange={(event) => onChange({ ...draft, length: event.target.value })}
            placeholder="3"
            type="number"
            value={draft.length}
          />
          <TypeEditor
            draft={draft.inner}
            language={language}
            onChange={(inner) => onChange({ ...draft, inner })}
            schema={schema}
          />
        </>
      )}
      {draft.kind === "map" && (
        <div className="type-editor-map">
          <div>
            <span>{t.keyType}</span>
            <TypeEditor
              draft={draft.key}
              language={language}
              onChange={(key) => onChange({ ...draft, key })}
              schema={schema}
            />
          </div>
          <div>
            <span>{t.valueType}</span>
            <TypeEditor
              draft={draft.value}
              language={language}
              onChange={(value) => onChange({ ...draft, value })}
              schema={schema}
            />
          </div>
        </div>
      )}
    </div>
  );
}

function RelationList({
  title,
  edges,
  language
}: {
  title: string;
  edges: StudioEdge[];
  language: Language;
}) {
  const t = translations[language];
  return (
    <div className="relations">
      <h4>{title}</h4>
      {edges.length === 0 ? (
        <p className="muted">{t.noRelations}</p>
      ) : (
        edges.map((edge) => (
          <div key={edge.id} className="relation-row">
            <span style={{ background: edgeColors[edge.kind] }}>{t.edgeKind[edge.kind]}</span>
            <code>{edge.label}</code>
          </div>
        ))
      )}
    </div>
  );
}

function parseTypeDraft(ty: string, schema: StudioSchema): TypeDraft {
  const value = ty.trim();
  if (!value) return defaultTypeDraft("primitive", schema);
  const generic = parseGeneric(value);
  if (generic) {
    if (generic.name === "optional" || generic.name === "list" || generic.name === "set") {
      return { kind: generic.name, inner: parseTypeDraft(generic.args[0] ?? "", schema) };
    }
    if (generic.name === "array") {
      return {
        kind: "array",
        inner: parseTypeDraft(generic.args[0] ?? "", schema),
        length: generic.args[1] ?? "1"
      };
    }
    if (generic.name === "map") {
      return {
        kind: "map",
        key: parseTypeDraft(generic.args[0] ?? "string", schema),
        value: parseTypeDraft(generic.args[1] ?? "string", schema)
      };
    }
    if (generic.name === "enum" || generic.name === "struct" || generic.name === "union") {
      return { kind: generic.name, name: generic.args[0] ?? firstNodeName(schema, generic.name) };
    }
    if (generic.name === "ref") {
      const [table, field] = (generic.args[0] ?? "").split(".");
      const fallback = defaultTypeDraft("ref", schema);
      return {
        kind: "ref",
        table: table?.trim() || (fallback.kind === "ref" ? fallback.table : ""),
        field: field?.trim() || (fallback.kind === "ref" ? fallback.field : "id")
      };
    }
  }
  if (primitiveTypeNames.includes(value)) return { kind: "primitive", name: value };
  for (const kind of ["enum", "struct", "union"] as const) {
    if (schema.nodes.some((node) => node.kind === kind && node.name === value)) {
      return { kind, name: value };
    }
  }
  return defaultTypeDraft("primitive", schema);
}

function serializeTypeDraft(draft: TypeDraft): string {
  switch (draft.kind) {
    case "primitive":
      return draft.name;
    case "enum":
    case "struct":
    case "union":
      return `${draft.kind}<${draft.name}>`;
    case "ref":
      return `ref<${draft.table}.${draft.field}>`;
    case "optional":
    case "list":
    case "set":
      return `${draft.kind}<${serializeTypeDraft(draft.inner)}>`;
    case "array":
      return `array<${serializeTypeDraft(draft.inner)},${draft.length || "1"}>`;
    case "map":
      return `map<${serializeTypeDraft(draft.key)},${serializeTypeDraft(draft.value)}>`;
  }
}

function defaultTypeDraft(kind: TypeDraft["kind"], schema: StudioSchema): TypeDraft {
  switch (kind) {
    case "primitive":
      return { kind, name: "string" };
    case "enum":
    case "struct":
    case "union":
      return { kind, name: firstNodeName(schema, kind) };
    case "ref": {
      const table = nodesOfKind(schema, "table")[0];
      return {
        kind,
        table: table?.name ?? "",
        field: table?.fields[0]?.name ?? "id"
      };
    }
    case "optional":
    case "list":
    case "set":
      return { kind, inner: defaultTypeDraft("primitive", schema) };
    case "array":
      return { kind, inner: defaultTypeDraft("primitive", schema), length: "1" };
    case "map":
      return {
        kind,
        key: defaultTypeDraft("primitive", schema),
        value: defaultTypeDraft("primitive", schema)
      };
  }
}

function nodesOfKind(schema: StudioSchema, kind: NodeKind) {
  return schema.nodes.filter((node) => node.kind === kind).sort((a, b) => a.name.localeCompare(b.name));
}

function firstNodeName(schema: StudioSchema, kind: Exclude<NodeKind, "table">) {
  return nodesOfKind(schema, kind)[0]?.name ?? "";
}

function fieldsForTable(schema: StudioSchema, tableName: string) {
  return schema.nodes.find((node) => node.kind === "table" && node.name === tableName)?.fields ?? [];
}

function parseGeneric(value: string): { name: string; args: string[] } | null {
  const open = value.indexOf("<");
  if (open < 1 || !value.endsWith(">")) return null;
  const name = value.slice(0, open).trim();
  const inner = value.slice(open + 1, -1);
  return { name, args: splitTopLevel(inner) };
}

function splitTopLevel(value: string) {
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
