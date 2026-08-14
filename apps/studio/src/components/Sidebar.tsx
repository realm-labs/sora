import { useState, type ReactNode } from "react";
import {
  Boxes,
  Check,
  ChevronDown,
  ChevronRight,
  FileText,
  Files,
  Plus,
  Search,
  Tags,
  Trash2,
  X
} from "lucide-react";

import { kindMeta, kindOrder } from "../constants";
import type { Translation } from "../i18n";
import type { NavigatorMode, NodeKind, StudioNode, StudioSchema } from "../types";

export function Sidebar({
  issueCounts,
  onAddSchemaSource,
  navigateToNode,
  onAddNode,
  onDeleteSchemaSource,
  query,
  schema,
  selectedId,
  setQuery,
  t,
  visibleNodes,
  readOnly
}: {
  issueCounts: Record<string, number>;
  onAddSchemaSource: (source: string) => void;
  navigateToNode: (id: string) => void;
  onAddNode: (kind: NodeKind) => void;
  onDeleteSchemaSource: (source: string) => void;
  query: string;
  readOnly: boolean;
  schema: StudioSchema | null;
  selectedId: string | null;
  setQuery: (query: string) => void;
  t: Translation;
  visibleNodes: StudioNode[];
}) {
  const [sourceDraft, setSourceDraft] = useState<string | null>(null);
  const [schemaFilesOpen, setSchemaFilesOpen] = useState(false);
  const [mode, setMode] = useState<NavigatorMode>("entities");
  const startAddingSource = () => setSourceDraft(nextSchemaSource(schema?.sources ?? []));
  const applySourceDraft = () => {
    if (!sourceDraft) return;
    onAddSchemaSource(sourceDraft);
    setSchemaFilesOpen(true);
    setSourceDraft(null);
  };
  const schemaFileNodeCount = schema?.nodes.length ?? 0;
  return (
    <aside className="sidebar">
      <div className="navigator-title">
        <strong>{t.explorer}</strong>
        <span>{schema?.nodes.length ?? 0}</span>
      </div>

      <div className="navigator-tabs" aria-label={t.navigatorMode}>
        <NavigatorTab
          active={mode === "entities"}
          icon={<Boxes size={14} />}
          label={t.entitiesMode}
          onClick={() => setMode("entities")}
        />
        <NavigatorTab
          active={mode === "files"}
          icon={<Files size={14} />}
          label={t.filesMode}
          onClick={() => setMode("files")}
        />
        <NavigatorTab
          active={mode === "groups"}
          icon={<Tags size={14} />}
          label={t.groupsMode}
          onClick={() => setMode("groups")}
        />
      </div>

      {mode === "entities" ? (
        <label className="search">
          <Search size={16} />
          <input
            data-studio-search
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t.searchSchema}
          />
        </label>
      ) : null}

      {schema && mode === "files" ? (
        <section className={schemaFilesOpen ? "schema-files open" : "schema-files"}>
          <h2>
            <button
              aria-expanded={schemaFilesOpen}
              className="schema-files-toggle"
              onClick={() => setSchemaFilesOpen((value) => !value)}
              type="button"
            >
              {schemaFilesOpen ? <ChevronDown size={15} /> : <ChevronRight size={15} />}
              <FileText size={15} />
              {t.schemaFiles}
              <span>{schema.sources.length}</span>
              <small>{schemaFileNodeCount}</small>
            </button>
            {!readOnly ? (
              <button
                className="section-action"
                onClick={() => {
                  setSchemaFilesOpen(true);
                  startAddingSource();
                }}
                title={t.addSchemaFile}
                type="button"
              >
                <Plus size={14} />
              </button>
            ) : null}
          </h2>
          <div className="schema-file-list" hidden={!schemaFilesOpen}>
            {sourceDraft !== null && (
              <div className="schema-file-item schema-file-editor">
                <input
                  aria-label={t.schemaFilePrompt}
                  autoFocus
                  onChange={(event) => setSourceDraft(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") applySourceDraft();
                    if (event.key === "Escape") setSourceDraft(null);
                  }}
                  value={sourceDraft}
                />
                <button
                  className="mini-action"
                  disabled={!sourceDraft.trim()}
                  onClick={applySourceDraft}
                  title={t.apply}
                  type="button"
                >
                  <Check size={14} />
                </button>
                <button
                  className="mini-action"
                  onClick={() => setSourceDraft(null)}
                  title={t.cancel}
                  type="button"
                >
                  <X size={14} />
                </button>
              </div>
            )}
            {schema.sources.map((source) => {
              const nodeCount = schema.nodes.filter((node) => node.source === source).length;
              const blocked = nodeCount > 0 || schema.sources.length <= 1;
              return (
                <div className="schema-file-item" key={source}>
                  <span title={source}>{source}</span>
                  <small>{nodeCount}</small>
                  {!readOnly ? (
                    <button
                      className="mini-action danger"
                      disabled={blocked}
                      onClick={() => onDeleteSchemaSource(source)}
                      title={blocked ? t.deleteSchemaFileBlocked : t.deleteSchemaFile}
                      type="button"
                    >
                      <Trash2 size={14} />
                    </button>
                  ) : null}
                </div>
              );
            })}
          </div>
        </section>
      ) : null}

      {schema && mode === "groups" ? (
        <div className="group-list">
          {Object.entries(schema.groups).map(([group, enabled]) => {
            const viewCount = Object.values(schema.views).filter((view) =>
              view.groups.includes(group)
            ).length;
            return (
              <div className="group-row" key={group}>
                <span className={enabled ? "group-state enabled" : "group-state"} />
                <strong>{group}</strong>
                <small>
                  {enabled ? t.defaultEnabled : t.defaultDisabled} · {viewCount} {t.views}
                </small>
              </div>
            );
          })}
        </div>
      ) : null}

      {mode === "entities" ? (
        <nav className="schema-list">
        {kindOrder.map((kind) => {
          const items = visibleNodes
            .filter((node) => node.kind === kind)
            .sort((a, b) => a.name.localeCompare(b.name));
          const Icon = kindMeta[kind].icon;
          return (
            <section key={kind}>
              <h2>
                <Icon size={15} />
                {t.kindPlural[kind]}
                <span>{items.length}</span>
              </h2>
              {!readOnly ? (
                <button className="list-item add-item" onClick={() => onAddNode(kind)} type="button">
                  <Plus size={14} />
                  <span>{t.addKind[kind]}</span>
                </button>
              ) : null}
              {items.map((node) => (
                <button
                  key={node.id}
                  className={listItemClass(node.id, selectedId, issueCounts)}
                  onClick={() => navigateToNode(node.id)}
                >
                  <span className="dot" style={{ background: kindMeta[node.kind].color }} />
                  <span>{node.name}</span>
                  {issueCounts[node.id] ? <span className="issue-badge">{issueCounts[node.id]}</span> : null}
                </button>
              ))}
            </section>
          );
        })}
        </nav>
      ) : null}
    </aside>
  );
}

function NavigatorTab({
  active,
  icon,
  label,
  onClick
}: {
  active: boolean;
  icon: ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button className={active ? "active" : ""} onClick={onClick} title={label} type="button">
      {icon}
      <span>{label}</span>
    </button>
  );
}

function nextSchemaSource(sources: string[]) {
  const existing = new Set(sources);
  let index = sources.length + 1;
  for (;;) {
    const source = `schema/schema${index}.scon`;
    if (!existing.has(source)) return source;
    index += 1;
  }
}

function listItemClass(
  nodeId: string,
  selectedId: string | null,
  issueCounts: Record<string, number>
) {
  return [
    "list-item",
    nodeId === selectedId ? "active" : "",
    issueCounts[nodeId] ? "has-issue" : ""
  ]
    .filter(Boolean)
    .join(" ");
}
