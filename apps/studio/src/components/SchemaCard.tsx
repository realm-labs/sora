import { Handle, Position, type Node, type NodeProps } from "@xyflow/react";

import { fieldHandleId, kindMeta } from "../constants";
import type { SchemaCardData } from "../graph";
import { translations } from "../i18n";

export function SchemaCard({ data }: NodeProps<Node<SchemaCardData>>) {
  const { node, selected, linkedFields, language } = data;
  const t = translations[language];
  const Icon = kindMeta[node.kind].icon;
  return (
    <article
      className={`schema-card ${node.kind}${selected ? " selected" : ""}`}
      style={{ borderColor: kindMeta[node.kind].color }}
    >
      <Handle id="target" type="target" position={Position.Left} className="node-port target" />
      <header className="schema-card-header">
        <span className="schema-card-icon" style={{ color: kindMeta[node.kind].color }}>
          <Icon size={16} />
        </span>
        <div>
          <p>{t.kindSingular[node.kind]}</p>
          <strong>{node.name}</strong>
        </div>
        <span className="schema-card-count">{node.fields.length}</span>
      </header>
      <div className="schema-card-fields">
        {node.fields.map((field) => {
          const linked = linkedFields.has(field.name);
          return (
            <div
              key={`${field.name}:${field.ty}`}
              className={linked ? "node-field linked" : "node-field"}
            >
              <span className="field-name">{field.name}</span>
              <code>{field.ty}</code>
              <Handle
                id={fieldHandleId(field.name)}
                type="source"
                position={Position.Right}
                className={linked ? "node-port source linked" : "node-port source"}
              />
            </div>
          );
        })}
      </div>
    </article>
  );
}
