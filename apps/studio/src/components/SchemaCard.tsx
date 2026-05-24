import type { PointerEvent as ReactPointerEvent } from "react";
import { Handle, Position, useViewport, type Node, type NodeProps } from "@xyflow/react";

import { fieldHandleId, kindMeta } from "../constants";
import type { SchemaCardData } from "../graph";
import { translations } from "../i18n";
import { unionFieldCount, unionVariantCount, unionVariants } from "../unionFields";

type ResizeDirection = "n" | "ne" | "e" | "se" | "s" | "sw" | "w" | "nw";

const minCardWidth = 240;
const minCardHeight = 120;
const maxCardWidth = 900;
const maxCardHeight = 1000;

export function SchemaCard({
  data,
  height,
  id,
  positionAbsoluteX,
  positionAbsoluteY,
  width
}: NodeProps<Node<SchemaCardData>>) {
  const {
    node,
    selected,
    linkedSourceFields,
    linkedTargetFields,
    linkedTargetNode,
    issueCount,
    language
  } = data;
  const { zoom } = useViewport();
  const t = translations[language];
  const Icon = kindMeta[node.kind].icon;
  const variants = unionVariants(node);
  const count = node.kind === "union" ? unionVariantCount(node) : node.fields.length;
  const currentWidth = width ?? 310;
  const currentHeight = height ?? naturalCardHeight(node);

  const startResize = (direction: ResizeDirection, event: ReactPointerEvent) => {
    event.preventDefault();
    event.stopPropagation();
    const startX = event.clientX;
    const startY = event.clientY;
    const startPosition = { x: positionAbsoluteX, y: positionAbsoluteY };
    const startSize = { width: currentWidth, height: currentHeight };
    document.body.classList.add("resizing-card");

    const onPointerMove = (moveEvent: PointerEvent) => {
      const dx = (moveEvent.clientX - startX) / zoom;
      const dy = (moveEvent.clientY - startY) / zoom;
      const next = resizeRect(direction, startPosition, startSize, dx, dy);
      data.resizeNode(id, next);
    };
    const stopResize = () => {
      document.body.classList.remove("resizing-card");
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", stopResize);
      window.removeEventListener("pointercancel", stopResize);
    };
    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", stopResize);
    window.addEventListener("pointercancel", stopResize);
  };

  return (
    <article
      className={`schema-card ${node.kind}${selected ? " selected" : ""}${issueCount ? " has-issue" : ""}`}
      style={{ borderColor: issueCount ? "#ef4444" : kindMeta[node.kind].color }}
    >
      {resizeDirections.map((direction) => (
        <div
          key={direction}
          className={`card-resize-zone nodrag nopan ${direction}`}
          onPointerDown={(event) => startResize(direction, event)}
        />
      ))}
      <Handle
        id="target"
        type="target"
        position={Position.Left}
        className={linkedTargetNode ? "node-port target node-target linked" : "node-port target node-target"}
      />
      <header className="schema-card-header">
        <span className="schema-card-icon" style={{ color: kindMeta[node.kind].color }}>
          <Icon size={16} />
        </span>
        <div>
          <p>{t.kindSingular[node.kind]}</p>
          <strong>{node.name}</strong>
        </div>
        <span className={issueCount ? "schema-card-count issue" : "schema-card-count"}>
          {issueCount || count}
        </span>
      </header>
      <div className="schema-card-fields">
        {node.kind === "union"
          ? variants.map((variant) => (
              <section key={variant.name} className="node-variant">
                <div className="node-variant-title">
                  <span>{variant.name}</span>
                  <code>{t.variant}</code>
                </div>
                {variant.fields.length === 0 ? (
                  <p>{t.emptyVariant}</p>
                ) : (
                  variant.fields.map(({ field, displayName }) => {
                    const sourceLinked = linkedSourceFields.has(field.name);
                    const targetLinked = linkedTargetFields.has(field.name);
                    const linked = sourceLinked || targetLinked;
                    return (
                      <div
                        key={`${field.name}:${field.ty}`}
                        className={linked ? "node-field linked" : "node-field"}
                      >
                        <Handle
                          id={fieldHandleId(field.name)}
                          type="target"
                          position={Position.Left}
                          className={
                            targetLinked
                              ? "node-port target field-target linked"
                              : "node-port target field-target"
                          }
                        />
                        <span className="field-name">{displayName}</span>
                        <code>{field.ty}</code>
                        <Handle
                          id={fieldHandleId(field.name)}
                          type="source"
                          position={Position.Right}
                          className={sourceLinked ? "node-port source linked" : "node-port source"}
                        />
                      </div>
                    );
                  })
                )}
              </section>
            ))
          : node.fields.map((field) => {
              const sourceLinked = linkedSourceFields.has(field.name);
              const targetLinked = linkedTargetFields.has(field.name);
              const linked = sourceLinked || targetLinked;
              return (
                <div
                  key={`${field.name}:${field.ty}`}
                  className={linked ? "node-field linked" : "node-field"}
                >
                  <Handle
                    id={fieldHandleId(field.name)}
                    type="target"
                    position={Position.Left}
                    className={
                      targetLinked ? "node-port target field-target linked" : "node-port target field-target"
                    }
                  />
                  <span className="field-name">{field.name}</span>
                  <code>{field.ty}</code>
                  <Handle
                    id={fieldHandleId(field.name)}
                    type="source"
                    position={Position.Right}
                    className={sourceLinked ? "node-port source linked" : "node-port source"}
                  />
                </div>
              );
            })}
      </div>
      {node.kind === "union" && (
        <footer className="schema-card-footer">
          {unionFieldCount(node)}
          {t.fieldsAbbr}
        </footer>
      )}
    </article>
  );
}

const resizeDirections: ResizeDirection[] = ["n", "ne", "e", "se", "s", "sw", "w", "nw"];

function resizeRect(
  direction: ResizeDirection,
  startPosition: { x: number; y: number },
  startSize: { width: number; height: number },
  dx: number,
  dy: number
) {
  let width = startSize.width;
  let height = startSize.height;
  let x = startPosition.x;
  let y = startPosition.y;

  if (direction.includes("e")) {
    width = clamp(startSize.width + dx, minCardWidth, maxCardWidth);
  }
  if (direction.includes("s")) {
    height = clamp(startSize.height + dy, minCardHeight, maxCardHeight);
  }
  if (direction.includes("w")) {
    width = clamp(startSize.width - dx, minCardWidth, maxCardWidth);
    x = startPosition.x + startSize.width - width;
  }
  if (direction.includes("n")) {
    height = clamp(startSize.height - dy, minCardHeight, maxCardHeight);
    y = startPosition.y + startSize.height - height;
  }

  return {
    position: { x, y },
    size: { width, height }
  };
}

function naturalCardHeight(node: SchemaCardData["node"]) {
  if (node.kind !== "union") return 70 + Math.max(node.fields.length, 1) * 34;
  const variants = unionVariants(node);
  const fieldCount = variants.reduce((count, variant) => count + variant.fields.length, 0);
  return 96 + variants.length * 34 + Math.max(fieldCount, 1) * 34;
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}
