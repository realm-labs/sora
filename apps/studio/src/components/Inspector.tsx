import { useEffect, useState } from "react";
import {
  AlertTriangle,
  ChevronDown,
  ChevronUp,
  CircleHelp,
  Layers3,
  Pencil,
  Plus,
  Save,
  Trash2,
  X
} from "lucide-react";

import { edgeColors, kindMeta } from "../constants";
import { translations, type Translation } from "../i18n";
import {
  makeNodeSettingsDraft,
  makeFieldDraft,
  type EditableNodeSettingsDraft,
  type EditableFieldDraft,
  type StudioValidationIssue
} from "../schemaEditing";
import { unionVariants } from "../unionFields";
import type { Language, NodeKind, StudioEdge, StudioField, StudioNode, StudioSchema } from "../types";

export function Inspector({
  node,
  edges,
  language,
  onAddEnumValue,
  onAddField,
  onAddUnionVariant,
  onAddUnionVariantField,
  onDeleteField,
  onDeleteNode,
  onDeleteUnionVariant,
  onMoveField,
  onMoveUnionVariant,
  onMoveUnionVariantField,
  onRenameNode,
  onUpdateEnumValue,
  onUpdateNodeSettings,
  onUpdateField,
  onUpdateUnionVariant,
  schema,
  validationIssues
}: {
  node: StudioNode;
  edges: StudioEdge[];
  language: Language;
  onAddEnumValue: (ownerId: string, name: string) => void;
  onAddField: (ownerId: string, draft: EditableFieldDraft) => void;
  onAddUnionVariant: (ownerId: string, name: string) => void;
  onAddUnionVariantField: (ownerId: string, variantName: string, draft: EditableFieldDraft) => void;
  onDeleteField: (ownerId: string, fieldIndex: number) => void;
  onDeleteNode: (nodeId: string) => void;
  onDeleteUnionVariant: (ownerId: string, fieldIndex: number) => void;
  onMoveField: (ownerId: string, fieldIndex: number, direction: -1 | 1) => void;
  onMoveUnionVariant: (ownerId: string, fieldIndex: number, direction: -1 | 1) => void;
  onMoveUnionVariantField: (
    ownerId: string,
    variantName: string,
    fieldIndex: number,
    direction: -1 | 1
  ) => void;
  onRenameNode: (nodeId: string, name: string) => void;
  onUpdateEnumValue: (ownerId: string, fieldIndex: number, name: string) => void;
  onUpdateNodeSettings: (nodeId: string, draft: EditableNodeSettingsDraft) => void;
  onUpdateField: (ownerId: string, fieldIndex: number, draft: EditableFieldDraft) => void;
  onUpdateUnionVariant: (ownerId: string, fieldIndex: number, name: string) => void;
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
  const [addingEnumValue, setAddingEnumValue] = useState(false);
  const [addingVariant, setAddingVariant] = useState(false);
  const [addingVariantField, setAddingVariantField] = useState<string | null>(null);
  const [editingEnumValue, setEditingEnumValue] = useState<{ index: number; name: string } | null>(null);
  const [editingVariant, setEditingVariant] = useState<{ index: number; name: string } | null>(null);
  const [settingsDraft, setSettingsDraft] = useState(makeNodeSettingsDraft(node));
  const [editingField, setEditingField] = useState<{ index: number; draft: EditableFieldDraft } | null>(
    null
  );

  useEffect(() => {
    setNameDraft(node.name);
    setSettingsDraft(makeNodeSettingsDraft(node));
    setAddingField(false);
    setAddingEnumValue(false);
    setAddingVariant(false);
    setAddingVariantField(null);
    setEditingEnumValue(null);
    setEditingVariant(null);
    setEditingField(null);
  }, [node.id, node.name, node.scope, node.metadata]);

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
  const submitAddEnumValue = (name: string) => {
    onAddEnumValue(node.id, name);
    setAddingEnumValue(false);
  };
  const submitEditEnumValue = (name: string) => {
    if (!editingEnumValue) return;
    onUpdateEnumValue(node.id, editingEnumValue.index, name);
    setEditingEnumValue(null);
  };
  const submitAddVariant = (name: string) => {
    onAddUnionVariant(node.id, name);
    setAddingVariant(false);
  };
  const submitEditVariant = (name: string) => {
    if (!editingVariant) return;
    onUpdateUnionVariant(node.id, editingVariant.index, name);
    setEditingVariant(null);
  };
  const submitAddVariantField = (variantName: string, draft: EditableFieldDraft) => {
    onAddUnionVariantField(node.id, variantName, draft);
    setAddingVariantField(null);
  };
  const submitEditField = (draft: EditableFieldDraft) => {
    if (!editingField) return;
    onUpdateField(node.id, editingField.index, draft);
    setEditingField(null);
  };
  const applySettings = () => onUpdateNodeSettings(node.id, settingsDraft);
  const sectionTitle = node.kind === "enum" ? t.values : node.kind === "union" ? t.variants : t.fields;
  const addTitle = node.kind === "enum" ? t.addValue : node.kind === "union" ? t.addVariant : t.addField;
  const startAdd = () => {
    if (node.kind === "enum") setAddingEnumValue(true);
    else if (node.kind === "union") setAddingVariant(true);
    else setAddingField(true);
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
        <NodeSettingsEditor
          draft={settingsDraft}
          language={language}
          node={node}
          onChange={setSettingsDraft}
          onSubmit={applySettings}
          schema={schema}
          validationIssues={validationIssues}
        />
      </section>

      <section className="detail-section">
        <h3>
          <Layers3 size={15} />
          {sectionTitle}
          <button className="section-action" onClick={startAdd} title={addTitle}>
            <Plus size={13} />
          </button>
        </h3>
        <div className="field-list">
          {node.kind === "enum" && addingEnumValue && (
            <NameEditor
              initialName=""
              language={language}
              onCancel={() => setAddingEnumValue(false)}
              onSubmit={submitAddEnumValue}
              placeholder="Value"
            />
          )}
          {node.kind === "enum" &&
            node.fields.map((field, index) =>
              editingEnumValue?.index === index ? (
                <NameEditor
                  key={`${field.name}:${index}:editor`}
                  initialName={editingEnumValue.name}
                  language={language}
                  onCancel={() => setEditingEnumValue(null)}
                  onSubmit={submitEditEnumValue}
                  placeholder="Value"
                />
              ) : (
                <ValueCard
                  key={`${field.name}:${index}`}
                  issues={issuesForField(validationIssues, index)}
                  language={language}
                  moveDownDisabled={index === node.fields.length - 1}
                  moveUpDisabled={index === 0}
                  name={field.name}
                  onDelete={() => onDeleteField(node.id, index)}
                  onEdit={() => setEditingEnumValue({ index, name: field.name })}
                  onMoveDown={() => onMoveField(node.id, index, 1)}
                  onMoveUp={() => onMoveField(node.id, index, -1)}
                />
              )
            )}
          {node.kind !== "enum" && node.kind !== "union" && addingField && (
            <FieldEditor
              draft={makeFieldDraft()}
              language={language}
              onCancel={() => setAddingField(false)}
              onSubmit={submitAddField}
              owner={node}
              ownerKind={node.kind}
              schema={schema}
              validationIssues={[]}
            />
          )}
          {node.kind === "union" && addingVariant && (
            <NameEditor
              initialName=""
              language={language}
              onCancel={() => setAddingVariant(false)}
              onSubmit={submitAddVariant}
              placeholder="Variant"
            />
          )}
          {node.kind === "union"
            ? variants.map((variant, variantIndex) => (
                <section key={variant.name} className="variant-card">
                  <header>
                    {editingVariant?.index === variant.fieldIndex ? (
                      <NameEditor
                        compact
                        initialName={editingVariant.name}
                        language={language}
                        onCancel={() => setEditingVariant(null)}
                        onSubmit={submitEditVariant}
                        placeholder="Variant"
                      />
                    ) : (
                      <>
                        <strong>{variant.name}</strong>
                        <span>{t.variant}</span>
                        <div className="variant-actions">
                          <button className="mini-action" onClick={() => setAddingVariantField(variant.name)} title={t.addField}>
                            <Plus size={13} />
                          </button>
                          <button className="mini-action" disabled={variantIndex === 0} onClick={() => onMoveUnionVariant(node.id, variant.fieldIndex, -1)} title={t.moveUp}>
                            <ChevronUp size={13} />
                          </button>
                          <button className="mini-action" disabled={variantIndex === variants.length - 1} onClick={() => onMoveUnionVariant(node.id, variant.fieldIndex, 1)} title={t.moveDown}>
                            <ChevronDown size={13} />
                          </button>
                          <button className="mini-action" onClick={() => setEditingVariant({ index: variant.fieldIndex, name: variant.name })} title={t.editVariant}>
                            <Pencil size={13} />
                          </button>
                          <button className="mini-action danger" onClick={() => onDeleteUnionVariant(node.id, variant.fieldIndex)} title={t.deleteVariant}>
                            <Trash2 size={13} />
                          </button>
                        </div>
                      </>
                    )}
                  </header>
                  {addingVariantField === variant.name && (
                    <FieldEditor
                      draft={makeFieldDraft()}
                      language={language}
                      onCancel={() => setAddingVariantField(null)}
                      onSubmit={(draft) => submitAddVariantField(variant.name, draft)}
                      owner={node}
                      ownerKind={node.kind}
                      schema={schema}
                      validationIssues={[]}
                    />
                  )}
                  {variant.fields.length === 0 ? (
                    <p className="muted">{t.emptyVariant}</p>
                  ) : (
                    variant.fields.map(({ field, fieldIndex, displayName }, fieldPosition) =>
                      editingField?.index === fieldIndex ? (
                        <FieldEditor
                          key={`${field.name}:${fieldIndex}:editor`}
                          draft={editingField.draft}
                          language={language}
                          onCancel={() => setEditingField(null)}
                          onSubmit={(draft) =>
                            submitEditField(applyUnionFieldName(field.name, displayName, draft))
                          }
                          owner={node}
                          ownerKind={node.kind}
                          schema={schema}
                          validationIssues={issuesForField(validationIssues, fieldIndex)}
                        />
                      ) : (
                        <FieldCard
                          key={`${field.name}:${field.ty}:${fieldIndex}`}
                          displayName={displayName}
                          field={field}
                          issues={issuesForField(validationIssues, fieldIndex)}
                          language={language}
                          onDelete={() => onDeleteField(node.id, fieldIndex)}
                          onEdit={() =>
                            setEditingField({
                              index: fieldIndex,
                              draft: makeUnionFieldDraft(field, displayName)
                            })
                          }
                          onMoveDown={() => onMoveUnionVariantField(node.id, variant.name, fieldIndex, 1)}
                          onMoveUp={() => onMoveUnionVariantField(node.id, variant.name, fieldIndex, -1)}
                          moveDownDisabled={fieldPosition === variant.fields.length - 1}
                          moveUpDisabled={fieldPosition === 0}
                        />
                      )
                    )
                  )}
                </section>
              ))
            : node.kind !== "enum" && node.fields.map((field, index) =>
                editingField?.index === index ? (
                  <FieldEditor
                    key={`${field.name}:${index}:editor`}
                    draft={editingField.draft}
                    language={language}
                    onCancel={() => setEditingField(null)}
                    onSubmit={submitEditField}
                    owner={node}
                    ownerKind={node.kind}
                    schema={schema}
                    validationIssues={issuesForField(validationIssues, index)}
                  />
                ) : (
                  <FieldCard
                    key={`${field.name}:${field.ty}:${index}`}
                    field={field}
                    issues={issuesForField(validationIssues, index)}
                    language={language}
                    onDelete={() => onDeleteField(node.id, index)}
                    onEdit={() => setEditingField({ index, draft: makeFieldDraft(field) })}
                    onMoveDown={() => onMoveField(node.id, index, 1)}
                    onMoveUp={() => onMoveField(node.id, index, -1)}
                    moveDownDisabled={index === node.fields.length - 1}
                    moveUpDisabled={index === 0}
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

function NodeSettingsEditor({
  draft,
  language,
  node,
  onChange,
  onSubmit,
  schema,
  validationIssues
}: {
  draft: EditableNodeSettingsDraft;
  language: Language;
  node: StudioNode;
  onChange: (draft: EditableNodeSettingsDraft) => void;
  onSubmit: () => void;
  schema: StudioSchema;
  validationIssues: StudioValidationIssue[];
}) {
  const t = translations[language];
  const keyIssues = validationIssues.filter((issue) => issue.setting === "key");
  return (
    <article className="field-editor metadata-editor">
      <label>
        <span>{t.schemaFile}</span>
        <select
          value={draft.schemaSource}
          onChange={(event) => onChange({ ...draft, schemaSource: event.target.value })}
        >
          {schema.sources.map((source) => (
            <option key={source} value={source}>
              {source}
            </option>
          ))}
        </select>
      </label>
      <label>
        <span>{t.scope}</span>
        <input
          value={draft.scope}
          onChange={(event) => onChange({ ...draft, scope: event.target.value })}
          placeholder="all"
        />
      </label>
      {node.kind === "table" && (
        <>
          <div className="editor-grid two">
            <label>
              <FieldLabel required>{t.mode}</FieldLabel>
              <select
                aria-required="true"
                value={draft.mode}
                onChange={(event) => onChange({ ...draft, mode: event.target.value })}
              >
                <option value="map">map</option>
                <option value="list">list</option>
                <option value="singleton">singleton</option>
              </select>
            </label>
            {draft.mode === "map" && (
              <label className={keyIssues.length ? "invalid" : ""}>
                <FieldLabel required>{t.key}</FieldLabel>
                <select
                  aria-required="true"
                  value={draft.key}
                  onChange={(event) => onChange({ ...draft, key: event.target.value })}
                >
                  <option value="">{t.none}</option>
                  {node.fields.map((field) => (
                    <option key={field.name} value={field.name}>
                      {field.name}
                    </option>
                  ))}
                </select>
                {keyIssues.map((issue) => (
                  <em key={issue.id}>{issue.message}</em>
                ))}
              </label>
            )}
          </div>
          <div className="editor-grid two">
            <label>
              <span>{t.sourceFile}</span>
              <input
                value={draft.source}
                onChange={(event) => onChange({ ...draft, source: event.target.value })}
                placeholder={`${node.name}.xlsx`}
              />
            </label>
            <label>
              <span>{t.sheet}</span>
              <input
                value={draft.sheet}
                onChange={(event) => onChange({ ...draft, sheet: event.target.value })}
                placeholder={node.name}
              />
            </label>
          </div>
        </>
      )}
      {node.kind === "union" && (
        <label>
          <span>{t.unionTag}</span>
          <input
            value={draft.tag}
            onChange={(event) => onChange({ ...draft, tag: event.target.value })}
            placeholder="type"
          />
        </label>
      )}
      {node.kind !== "table" && node.kind !== "union" && (
        <dl>
          {Object.entries(node.metadata).map(([key, value]) => (
            <div key={key}>
              <dt>{key}</dt>
              <dd>{value}</dd>
            </div>
          ))}
        </dl>
      )}
      <div className="editor-actions">
        <button className="icon-button" onClick={onSubmit}>
          <Save size={14} />
          {t.apply}
        </button>
      </div>
    </article>
  );
}

function FieldLabel({
  children,
  required = false,
  tooltip
}: {
  children: string;
  required?: boolean;
  tooltip?: string;
}) {
  return (
    <span className="field-label">
      {children}
      {required && (
        <b aria-hidden="true" className="required-mark">
          *
        </b>
      )}
      {tooltip && (
        <span aria-label={tooltip} className="tooltip-anchor" tabIndex={0} title={tooltip}>
          <CircleHelp aria-hidden="true" size={12} />
          <span className="tooltip-panel" role="tooltip">
            {tooltip}
          </span>
        </span>
      )}
    </span>
  );
}

function FieldCard({
  displayName,
  field,
  issues,
  language,
  moveDownDisabled = false,
  moveUpDisabled = false,
  onDelete,
  onEdit,
  onMoveDown,
  onMoveUp
}: {
  displayName?: string;
  field: StudioField;
  issues: StudioValidationIssue[];
  language: Language;
  moveDownDisabled?: boolean;
  moveUpDisabled?: boolean;
  onDelete: () => void;
  onEdit: () => void;
  onMoveDown?: () => void;
  onMoveUp?: () => void;
}) {
  const t = translations[language];
  return (
    <article className="field-card">
      <div>
        <strong>{displayName ?? field.name}</strong>
        <code>{field.ty}</code>
        <span className="field-actions">
          {onMoveUp && (
            <button className="mini-action" disabled={moveUpDisabled} onClick={onMoveUp} title={t.moveUp}>
              <ChevronUp size={13} />
            </button>
          )}
          {onMoveDown && (
            <button className="mini-action" disabled={moveDownDisabled} onClick={onMoveDown} title={t.moveDown}>
              <ChevronDown size={13} />
            </button>
          )}
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
      {field.scope && field.scope !== "all" && (
        <span className="pill">
          {t.scope} {field.scope}
        </span>
      )}
      {field.default && (
        <span className="pill">
          {t.defaultValue} {field.default}
        </span>
      )}
      {field.range && (
        <span className="pill">
          {t.range} {field.range[0]}..{field.range[1]}
        </span>
      )}
      {field.length && (
        <span className="pill">
          {t.length} {field.length[0]}..{field.length[1]}
        </span>
      )}
      {field.source && (
        <span className="pill derived">
          {t.from} {field.source}
        </span>
      )}
      {field.comment && <p>{field.comment}</p>}
      {issues.map((issue) => (
        <p key={issue.id} className="inline-issue">
          {issue.message}
        </p>
      ))}
    </article>
  );
}

function ValueCard({
  issues,
  language,
  moveDownDisabled = false,
  moveUpDisabled = false,
  name,
  onDelete,
  onEdit,
  onMoveDown,
  onMoveUp
}: {
  issues: StudioValidationIssue[];
  language: Language;
  moveDownDisabled?: boolean;
  moveUpDisabled?: boolean;
  name: string;
  onDelete: () => void;
  onEdit: () => void;
  onMoveDown: () => void;
  onMoveUp: () => void;
}) {
  const t = translations[language];
  return (
    <article className="field-card value-card">
      <div>
        <strong>{name}</strong>
        <code>{t.value}</code>
        <span className="field-actions">
          <button className="mini-action" disabled={moveUpDisabled} onClick={onMoveUp} title={t.moveUp}>
            <ChevronUp size={13} />
          </button>
          <button className="mini-action" disabled={moveDownDisabled} onClick={onMoveDown} title={t.moveDown}>
            <ChevronDown size={13} />
          </button>
          <button className="mini-action" onClick={onEdit} title={t.editValue}>
            <Pencil size={13} />
          </button>
          <button className="mini-action danger" onClick={onDelete} title={t.deleteValue}>
            <Trash2 size={13} />
          </button>
        </span>
      </div>
      {issues.map((issue) => (
        <p key={issue.id} className="inline-issue">
          {issue.message}
        </p>
      ))}
    </article>
  );
}

function NameEditor({
  compact = false,
  initialName,
  language,
  onCancel,
  onSubmit,
  placeholder
}: {
  compact?: boolean;
  initialName: string;
  language: Language;
  onCancel: () => void;
  onSubmit: (name: string) => void;
  placeholder: string;
}) {
  const t = translations[language];
  const [name, setName] = useState(initialName);
  const [submitted, setSubmitted] = useState(false);
  const invalid = submitted && !name.trim();
  const submit = () => {
    setSubmitted(true);
    if (!name.trim()) return;
    onSubmit(name);
  };
  return (
    <article className={compact ? "field-editor compact-editor" : "field-editor"}>
      <label className={invalid ? "invalid" : ""}>
        <FieldLabel required>{t.name}</FieldLabel>
        <input
          aria-invalid={invalid}
          aria-required="true"
          onChange={(event) => setName(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") submit();
          }}
          placeholder={placeholder}
          value={name}
        />
        {invalid && <em>{t.required}</em>}
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
  owner,
  ownerKind,
  schema,
  validationIssues
}: {
  draft: EditableFieldDraft;
  language: Language;
  onCancel: () => void;
  onSubmit: (draft: EditableFieldDraft) => void;
  owner: StudioNode;
  ownerKind: NodeKind;
  schema: StudioSchema;
  validationIssues: StudioValidationIssue[];
}) {
  const t = translations[language];
  const [value, setValue] = useState(draft);
  const [submitted, setSubmitted] = useState(false);
  const typeDraft = parseTypeDraft(value.ty, schema);
  const updateType = (next: TypeDraft) => {
    const ty = serializeTypeDraft(next);
    const parser = parserCompatibleWithType(value.parser, ty, ownerKind) ? value.parser : "";
    setValue({ ...value, ty, parser });
  };
  const nameInvalid =
    (submitted && !value.name.trim()) || validationIssues.some((issue) => issue.setting === "name");
  const typeInvalid =
    (submitted && !value.ty.trim()) || validationIssues.some((issue) => issue.setting === "type");
  const sourceIssue = submitted ? derivedSourceIssue(value, t) : null;
  const submit = () => {
    setSubmitted(true);
    if (!value.name.trim() || !value.ty.trim() || derivedSourceIssue(value, t)) return;
    onSubmit(value);
  };
  return (
    <article className="field-editor">
      <label className={nameInvalid ? "invalid" : ""}>
        <FieldLabel required>{t.name}</FieldLabel>
        <input
          aria-invalid={nameInvalid}
          aria-required="true"
          required
          value={value.name}
          onChange={(event) => setValue({ ...value, name: event.target.value })}
          placeholder="id"
        />
        {nameInvalid && <em>{t.required}</em>}
      </label>
      <label className={typeInvalid ? "invalid" : ""}>
        <FieldLabel required>{t.type}</FieldLabel>
        <TypeEditor
          draft={typeDraft}
          language={language}
          onChange={updateType}
          schema={schema}
        />
        {typeInvalid && <em>{t.required}</em>}
      </label>
      {validationIssues
        .filter((issue) => issue.setting !== "name" && issue.setting !== "type")
        .map((issue) => (
          <p key={issue.id} className="inline-issue">
            {issue.message}
          </p>
        ))}
      <label>
        <span>{t.scope}</span>
        <input
          value={value.scope}
          onChange={(event) => setValue({ ...value, scope: event.target.value })}
          placeholder="all"
        />
      </label>
      <ParserEditor
        language={language}
        onChange={(parser) => setValue({ ...value, parser })}
        ownerKind={ownerKind}
        ty={value.ty}
        value={value.parser}
      />
      <div className="editor-grid two">
        <label>
          <span>{t.defaultValue}</span>
          <input
            value={value.defaultValue}
            onChange={(event) => setValue({ ...value, defaultValue: event.target.value })}
            placeholder="0"
          />
        </label>
        <label>
          <span>{t.comment}</span>
          <input
            value={value.comment}
            onChange={(event) => setValue({ ...value, comment: event.target.value })}
            placeholder={t.comment}
          />
        </label>
      </div>
      <div className="editor-grid two">
        <label>
          <span>{t.range}</span>
          <div className="inline-pair">
            <input
              value={value.rangeMin}
              onChange={(event) => setValue({ ...value, rangeMin: event.target.value })}
              placeholder={t.min}
              type="number"
            />
            <input
              value={value.rangeMax}
              onChange={(event) => setValue({ ...value, rangeMax: event.target.value })}
              placeholder={t.max}
              type="number"
            />
          </div>
        </label>
        <label>
          <span>{t.length}</span>
          <div className="inline-pair">
            <input
              value={value.lengthMin}
              onChange={(event) => setValue({ ...value, lengthMin: event.target.value })}
              placeholder={t.min}
              type="number"
            />
            <input
              value={value.lengthMax}
              onChange={(event) => setValue({ ...value, lengthMax: event.target.value })}
              placeholder={t.max}
              type="number"
            />
          </div>
        </label>
      </div>
      {ownerKind === "table" && (
        <DerivedSourceEditor
          draft={value}
          language={language}
          onChange={setValue}
          owner={owner}
          schema={schema}
          issue={sourceIssue}
        />
      )}
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

type ParserDraft = {
  kind: string;
  prefix: string;
  prefixSet: boolean;
  separator: string;
  itemSeparator: string;
  options: string;
};

const parserKinds = ["json", "split", "tuple", "tuple_list", "map", "columns", "tagged_columns"];

function parserKindAllowedForType(kind: string, ty: string, ownerKind: NodeKind) {
  if (kind === "json") return true;
  if (kind === "split") return typeHasCollectionShape(ty);
  if (kind === "tuple") return typeHasNamedShape(ty, "struct");
  if (kind === "tuple_list") return typeHasCollectionOfNamedShape(ty, "struct");
  if (kind === "map") return typeHasMapShape(ty);
  if (kind === "columns") return ownerKind === "table" && typeHasNamedShape(ty, "struct");
  if (kind === "tagged_columns") return ownerKind === "table" && typeHasNamedShape(ty, "union");
  return false;
}

function parserCompatibleWithType(parser: string, ty: string, ownerKind: NodeKind) {
  const kind = parseParserDraft(parser).kind;
  return !kind || parserKindAllowedForType(kind, ty, ownerKind);
}

function ParserEditor({
  language,
  onChange,
  ownerKind,
  ty,
  value
}: {
  language: Language;
  onChange: (value: string) => void;
  ownerKind: NodeKind;
  ty: string;
  value: string;
}) {
  const t = translations[language];
  const draft = parseParserDraft(value);
  const availableParserKinds = parserKinds.filter((kind) => parserKindAllowedForType(kind, ty, ownerKind));
  const setDraft = (next: ParserDraft) => onChange(serializeParserDraft(next));
  const supportsSeparator =
    draft.kind === "split" || draft.kind === "tuple" || draft.kind === "tuple_list" || draft.kind === "map";
  const supportsItemSeparator = draft.kind === "tuple_list" || draft.kind === "map";
  const supportsPrefix = draft.kind === "columns" || draft.kind === "tagged_columns";
  return (
    <div className="nested-editor parser-editor">
      <header>{t.parser}</header>
      <div className="editor-grid three">
        <label>
          <span>{t.parserKind}</span>
          <select
            value={draft.kind}
            onChange={(event) =>
              setDraft({
                kind: event.target.value,
                prefix: "",
                prefixSet: false,
                separator: "",
                itemSeparator: "",
                options: ""
              })
            }
          >
            <option value="">{t.none}</option>
            {availableParserKinds.map((kind) => (
              <option key={kind} value={kind}>
                {kind}
              </option>
            ))}
          </select>
        </label>
        <label>
          <span>{t.separator}</span>
          <input
            disabled={!supportsSeparator}
            value={draft.separator}
            onChange={(event) => setDraft({ ...draft, separator: event.target.value })}
            placeholder={draft.kind === "map" ? ":" : ","}
          />
        </label>
        <label>
          <span>{t.itemSeparator}</span>
          <input
            disabled={!supportsItemSeparator}
            value={draft.itemSeparator}
            onChange={(event) => setDraft({ ...draft, itemSeparator: event.target.value })}
            placeholder="|"
          />
        </label>
      </div>
      {supportsPrefix && (
        <div className="editor-grid two">
          <label>
            <span>{t.prefix}</span>
            <input
              value={draft.prefix}
              onChange={(event) => setDraft({ ...draft, prefix: event.target.value })}
              placeholder="price_"
            />
          </label>
          <label className="checkbox-row">
            <input
              checked={draft.prefixSet}
              onChange={(event) => setDraft({ ...draft, prefixSet: event.target.checked })}
              type="checkbox"
            />
            <span>{t.overridePrefix}</span>
          </label>
        </div>
      )}
      <label>
        <span>{t.advancedOptions}</span>
        <input
          value={draft.options}
          onChange={(event) => setDraft({ ...draft, options: event.target.value })}
          placeholder="key=value"
        />
      </label>
    </div>
  );
}

function parseParserDraft(value: string): ParserDraft {
  const clean = value.trim();
  if (!clean) {
    return { kind: "", prefix: "", prefixSet: false, separator: "", itemSeparator: "", options: "" };
  }
  const match = clean.match(/^([^()]+?)(?:\s+\((.*)\))?$/);
  const kind = match?.[1]?.trim() ?? clean;
  const optionPairs = parseParserOptionPairs(match?.[2] ?? "");
  const prefixIndex = optionPairs.findIndex(([key]) => key === "prefix");
  const prefix = prefixIndex >= 0 ? optionPairs.splice(prefixIndex, 1)[0][1] : "";
  const separatorIndex = optionPairs.findIndex(([key]) => key === "separator");
  const separator = separatorIndex >= 0 ? optionPairs.splice(separatorIndex, 1)[0][1] : "";
  const itemSeparatorIndex = optionPairs.findIndex(([key]) => key === "item_separator");
  const itemSeparator =
    itemSeparatorIndex >= 0 ? optionPairs.splice(itemSeparatorIndex, 1)[0][1] : "";
  return {
    kind,
    prefix,
    prefixSet: prefixIndex >= 0,
    separator,
    itemSeparator,
    options: optionPairs.map(([key, optionValue]) => `${key}=${quoteParserOptionValue(optionValue)}`).join(", ")
  };
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

function serializeParserDraft(draft: ParserDraft): string {
  const kind = draft.kind.trim();
  if (!kind) return "";
  const options = draft.options
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean);
  if (draft.itemSeparator && (kind === "tuple_list" || kind === "map")) {
    options.unshift(`item_separator=${quoteParserOptionValue(draft.itemSeparator)}`);
  }
  if (
    draft.separator &&
    (kind === "split" || kind === "tuple" || kind === "tuple_list" || kind === "map")
  ) {
    options.unshift(`separator=${quoteParserOptionValue(draft.separator)}`);
  }
  if ((kind === "columns" || kind === "tagged_columns") && draft.prefixSet) {
    options.unshift(`prefix=${quoteParserOptionValue(draft.prefix)}`);
  }
  return options.length ? `${kind} (${options.join(", ")})` : kind;
}

function quoteParserOptionValue(value: string) {
  return JSON.stringify(value);
}

function DerivedSourceEditor({
  draft,
  issue,
  language,
  onChange,
  owner,
  schema
}: {
  draft: EditableFieldDraft;
  issue: string | null;
  language: Language;
  onChange: (draft: EditableFieldDraft) => void;
  owner: StudioNode;
  schema: StudioSchema;
}) {
  const t = translations[language];
  const sourceTable = schema.nodes.find((node) => node.kind === "table" && node.name === draft.sourceTable);
  const sourceFields = sourceTable?.fields ?? [];
  const active = derivedSourceActive(draft);
  const showInvalid = Boolean(issue);
  const tableMissing = showInvalid && active && !draft.sourceTable.trim();
  const childKeyMissing = showInvalid && active && !draft.childKey.trim();
  const parentKeyMissing = showInvalid && active && !draft.parentKey.trim();
  const sourceFieldExists = (name: string) => sourceFields.some((field) => field.name === name);
  return (
    <section className="nested-editor">
      <header>{t.derivedSource}</header>
      {issue && <p className="inline-issue">{issue}</p>}
      <div className="editor-grid two">
        <label className={tableMissing ? "invalid" : ""}>
          <FieldLabel required={active}>{t.table}</FieldLabel>
          <select
            aria-required={active}
            required={active}
            value={draft.sourceTable}
            onChange={(event) => {
              const table = schema.nodes.find((node) => node.kind === "table" && node.name === event.target.value);
              onChange({
                ...draft,
                sourceTable: event.target.value,
                childKey: table?.fields[0]?.name ?? "",
                valueField: table?.fields.some((field) => field.name === draft.valueField)
                  ? draft.valueField
                  : "",
                orderBy: table?.fields.some((field) => field.name === draft.orderBy) ? draft.orderBy : ""
              });
            }}
          >
            <option value="">{t.none}</option>
            {nodesOfKind(schema, "table").map((node) => (
              <option key={node.id} value={node.name}>
                {node.name}
              </option>
            ))}
          </select>
        </label>
        <label className={childKeyMissing ? "invalid" : ""}>
          <FieldLabel required={active}>{t.childKey}</FieldLabel>
          <select
            aria-required={active}
            required={active}
            value={sourceFieldExists(draft.childKey) ? draft.childKey : ""}
            onChange={(event) => onChange({ ...draft, childKey: event.target.value })}
          >
            <option value="">{t.none}</option>
            {sourceFields.map((field) => (
              <option key={field.name} value={field.name}>
                {field.name}
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="editor-grid three">
        <label className={parentKeyMissing ? "invalid" : ""}>
          <FieldLabel required={active}>{t.parentKey}</FieldLabel>
          <select
            aria-required={active}
            required={active}
            value={draft.parentKey}
            onChange={(event) => onChange({ ...draft, parentKey: event.target.value })}
          >
            <option value="">{t.none}</option>
            {owner.fields.map((field) => (
              <option key={field.name} value={field.name}>
                {field.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          <FieldLabel tooltip={t.valueFieldHelp}>{t.valueField}</FieldLabel>
          <select
            disabled={!sourceTable}
            value={sourceFieldExists(draft.valueField) ? draft.valueField : ""}
            onChange={(event) => onChange({ ...draft, valueField: event.target.value })}
          >
            <option value="">{t.none}</option>
            {sourceFields.map((field) => (
              <option key={field.name} value={field.name}>
                {field.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          <FieldLabel tooltip={t.orderByHelp}>{t.orderBy}</FieldLabel>
          <select
            disabled={!sourceTable}
            value={sourceFieldExists(draft.orderBy) ? draft.orderBy : ""}
            onChange={(event) => onChange({ ...draft, orderBy: event.target.value })}
          >
            <option value="">{t.none}</option>
            {sourceFields.map((field) => (
              <option key={field.name} value={field.name}>
                {field.name}
              </option>
            ))}
          </select>
        </label>
      </div>
    </section>
  );
}

function derivedSourceActive(draft: EditableFieldDraft) {
  return Boolean(
    draft.sourceTable.trim() ||
      draft.parentKey.trim() ||
      draft.childKey.trim() ||
      draft.valueField.trim() ||
      draft.orderBy.trim()
  );
}

function derivedSourceIssue(draft: EditableFieldDraft, t: Translation) {
  if (!derivedSourceActive(draft)) return null;
  if (!draft.sourceTable.trim() || !draft.parentKey.trim() || !draft.childKey.trim()) {
    return t.derivedSourceRequired;
  }
  return null;
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

function issuesForField(issues: StudioValidationIssue[], fieldIndex: number) {
  return issues.filter((issue) => issue.fieldIndex === fieldIndex);
}

function typeHasCollectionShape(ty: string): boolean {
  const value = unwrapOptionalType(ty);
  return /^(list|set|array)</.test(value);
}

function typeHasCollectionOfNamedShape(ty: string, kind: "struct" | "union"): boolean {
  const value = unwrapOptionalType(ty);
  const generic = parseGeneric(value);
  if (!generic || (generic.name !== "list" && generic.name !== "array")) return false;
  return typeHasNamedShape(generic.args[0] ?? "", kind);
}

function typeHasMapShape(ty: string): boolean {
  return parseGeneric(unwrapOptionalType(ty))?.name === "map";
}

function typeHasNamedShape(ty: string, kind: "struct" | "union"): boolean {
  return parseGeneric(unwrapOptionalType(ty))?.name === kind;
}

function unwrapOptionalType(ty: string): string {
  let value = ty.trim();
  for (;;) {
    const generic = parseGeneric(value);
    if (generic?.name !== "optional") return value;
    value = generic.args[0] ?? "";
  }
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
