import { Layers3 } from "lucide-react";

import { edgeColors, kindMeta } from "../constants";
import { translations } from "../i18n";
import type { Language, StudioEdge, StudioNode } from "../types";

export function Inspector({
  node,
  edges,
  language
}: {
  node: StudioNode;
  edges: StudioEdge[];
  language: Language;
}) {
  const t = translations[language];
  const inbound = edges.filter((edge) => edge.target === node.id);
  const outbound = edges.filter((edge) => edge.source === node.id);
  const Icon = kindMeta[node.kind].icon;
  return (
    <div className="inspector-content">
      <div className="inspector-title">
        <span className="kind-icon" style={{ color: kindMeta[node.kind].color }}>
          <Icon size={20} />
        </span>
        <div>
          <p>{t.kindSingular[node.kind]}</p>
          <h2>{node.name}</h2>
        </div>
      </div>

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
        </h3>
        <div className="field-list">
          {node.fields.map((field) => (
            <article key={`${field.name}:${field.ty}`} className="field-card">
              <div>
                <strong>{field.name}</strong>
                <code>{field.ty}</code>
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
          ))}
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
