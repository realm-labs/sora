import { Network, Search } from "lucide-react";

import { Metric } from "./Metric";
import { kindMeta, kindOrder } from "../constants";
import type { Translation } from "../i18n";
import type { StudioNode, StudioSchema } from "../types";

export function Sidebar({
  navigateToNode,
  query,
  schema,
  selectedId,
  setQuery,
  t,
  visibleNodes
}: {
  navigateToNode: (id: string) => void;
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
              {items.map((node) => (
                <button
                  key={node.id}
                  className={node.id === selectedId ? "list-item active" : "list-item"}
                  onClick={() => navigateToNode(node.id)}
                >
                  <span className="dot" style={{ background: kindMeta[node.kind].color }} />
                  <span>{node.name}</span>
                </button>
              ))}
            </section>
          );
        })}
      </nav>
    </aside>
  );
}
