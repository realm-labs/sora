import {
  Background,
  Controls,
  MiniMap,
  ReactFlow,
  type Edge,
  type Node,
  type NodeChange,
  type NodeMouseHandler
} from "@xyflow/react";
import { RotateCcw } from "lucide-react";

import { SchemaCard } from "./SchemaCard";
import { SchemaEdge } from "./SchemaEdge";
import type { SchemaCardData } from "../graph";
import type { Translation } from "../i18n";
import type { GraphMode, Theme } from "../types";

const nodeTypes = {
  schemaCard: SchemaCard
};

const edgeTypes = {
  schemaEdge: SchemaEdge
};

export function GraphCanvas({
  edges,
  graphMode,
  layoutRevision,
  nodes,
  onNodeClick,
  onNodesChange,
  query,
  resetLayout,
  selectedId,
  setGraphMode,
  t,
  theme
}: {
  edges: Edge[];
  graphMode: GraphMode;
  layoutRevision: number;
  nodes: Node<SchemaCardData>[];
  onNodeClick: NodeMouseHandler;
  onNodesChange: (changes: NodeChange[]) => void;
  query: string;
  resetLayout: () => void;
  selectedId: string | null;
  setGraphMode: (mode: GraphMode) => void;
  t: Translation;
  theme: Theme;
}) {
  return (
    <div className="graph-wrap">
      <div className="graph-toolbar">
        <div className="segment-control" aria-label={t.graphMode}>
          <button className={graphMode === "fields" ? "active" : ""} onClick={() => setGraphMode("fields")}>
            {t.fieldsMode}
          </button>
          <button className={graphMode === "usedBy" ? "active" : ""} onClick={() => setGraphMode("usedBy")}>
            {t.usedByMode}
          </button>
          <button className={graphMode === "all" ? "active" : ""} onClick={() => setGraphMode("all")}>
            {t.allMode}
          </button>
        </div>
        <span>
          {nodes.length}
          {t.nodesAbbr} / {edges.length}
          {t.edgesAbbr}
        </span>
        <button className="toolbar-button" onClick={resetLayout}>
          <RotateCcw size={14} />
          {t.reset}
        </button>
      </div>
      <ReactFlow
        key={`${graphMode}:${query}:${selectedId ?? "none"}:${layoutRevision}`}
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onNodeClick={onNodeClick}
        edgeTypes={edgeTypes}
        nodeTypes={nodeTypes}
        fitView
        fitViewOptions={{ padding: 0.08 }}
        minZoom={0.08}
        maxZoom={1.35}
        nodesDraggable
        panOnDrag={false}
        panOnScroll
        preventScrolling
        zoomOnPinch
        zoomOnScroll={false}
      >
        <Background color={theme === "dark" ? "#344054" : "#cbd5e1"} gap={22} />
        {graphMode === "all" && <MiniMap pannable zoomable />}
        <Controls />
      </ReactFlow>
    </div>
  );
}
