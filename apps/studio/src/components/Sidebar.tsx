import { Network, Plus, Search } from "lucide-react";

import { Metric } from "./Metric";
import { kindMeta, kindOrder } from "../constants";
import type { Translation } from "../i18n";
import type { NodeKind, StudioNode, StudioSchema } from "../types";

export function Sidebar({
  issueCounts,
  navigateToNode,
  onAddNode,
  query,
  schema,
  selectedId,
  setQuery,
  t,
  visibleNodes
}: {
  issueCounts: Record<string, number>;
  navigateToNode: (id: string) => void;
  onAddNode: (kind: NodeKind) => void;
  query: string;
  schema: StudioSchema | null;
  selectedId: string | null;
  setQuery: (query: string) => void;
  t: Translation;
  visibleNodes: StudioNode[];
}) {
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
