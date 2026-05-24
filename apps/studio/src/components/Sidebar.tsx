import { useState } from "react";
import {
  Check,
  ChevronDown,
  ChevronRight,
  FileText,
  Network,
  Plus,
  Search,
  Trash2,
  X
} from "lucide-react";

import { Metric } from "./Metric";
import { kindMeta, kindOrder } from "../constants";
import type { Translation } from "../i18n";
import type { NodeKind, StudioNode, StudioSchema } from "../types";

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
  visibleNodes
}: {
  issueCounts: Record<string, number>;
  onAddSchemaSource: (source: string) => void;
  navigateToNode: (id: string) => void;
  onAddNode: (kind: NodeKind) => void;
  onDeleteSchemaSource: (source: string) => void;
  query: string;
  schema: StudioSchema | null;
  selectedId: string | null;
  setQuery: (query: string) => void;
  t: Translation;
  visibleNodes: StudioNode[];
}) {
  const [sourceDraft, setSourceDraft] = useState<string | null>(null);
  const [schemaFilesOpen, setSchemaFilesOpen] = useState(false);
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
      <div className="brand">
        <div className="brand-mark">
          <Network size={20} />
        </div>
        <div>
          <h1>Sora Studio</h1>
          <p>{t.schemaVisualizer}</p>
        </div>
      </div>

      <label className="search">
        <Search size={16} />
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder={t.searchSchema}
        />
      </label>

      {schema && (
        <div className="summary-grid">
          <Metric label={t.kindPlural.table} value={schema.summary.tables} />
          <Metric label={t.kindPlural.struct} value={schema.summary.structs} />
          <Metric label={t.kindPlural.union} value={schema.summary.unions} />
          <Metric label={t.edges} value={schema.summary.edges} />
        </div>
      )}

      {schema && (
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
                  <button
                    className="mini-action danger"
                    disabled={blocked}
                    onClick={() => onDeleteSchemaSource(source)}
                    title={blocked ? t.deleteSchemaFileBlocked : t.deleteSchemaFile}
                    type="button"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              );
            })}
          </div>
        </section>
      )}

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
              <button className="list-item add-item" onClick={() => onAddNode(kind)} type="button">
                <Plus size={14} />
                <span>{t.addKind[kind]}</span>
              </button>
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
    </aside>
  );
}

function nextSchemaSource(sources: string[]) {
  const existing = new Set(sources);
  let index = sources.length + 1;
  for (;;) {
    const source = `schema/schema${index}.toml`;
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
