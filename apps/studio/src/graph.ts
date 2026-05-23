import { MarkerType, type Edge, type Node } from "@xyflow/react";

import { edgeColors, fieldHandleId, kindOrder } from "./constants";
import type { GraphMode, Language, StudioEdge, StudioNode, StudioSchema } from "./types";

export type SchemaCardData = {
  node: StudioNode;
  selected: boolean;
  linkedFields: Set<string>;
  language: Language;
};

type BuildGraphOptions = {
  graphMode: GraphMode;
  graphNodes: StudioNode[];
  language: Language;
  manualPositions: Record<string, { x: number; y: number }>;
  schema: StudioSchema;
  selected: StudioNode | null;
};

export function buildGraph({
  graphMode,
  graphNodes,
  language,
  manualPositions,
  schema,
  selected
}: BuildGraphOptions): { nodes: Node<SchemaCardData>[]; edges: Edge[] } {
  const visibleIds = new Set(graphNodes.map((node) => node.id));
  const visibleEdges = schema.edges.filter((edge) => {
    if (!visibleIds.has(edge.source) || !visibleIds.has(edge.target)) return false;
    if (!selected || graphMode === "all") return true;
    if (graphMode === "fields") return edge.source === selected.id;
    return edge.target === selected.id;
  });
  const linkedFieldsByNode = collectLinkedFields(visibleEdges);
  const positions = buildDefaultPositions(graphMode, graphNodes, schema, selected);

  const nodes: Node<SchemaCardData>[] = graphNodes.map((node) => ({
    id: node.id,
    type: "schemaCard",
    position: manualPositions[node.id] ?? positions.get(node.id) ?? { x: 40, y: 40 },
    data: {
      node,
      selected: node.id === selected?.id,
      linkedFields: linkedFieldsByNode.get(node.id) ?? new Set<string>(),
      language
    }
  }));

  const edges: Edge[] = visibleEdges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    sourceHandle: fieldHandleId(edge.label),
    target: edge.target,
    targetHandle: "target",
    label: edge.label,
    animated: edge.kind === "derived",
    markerEnd: {
      type: MarkerType.ArrowClosed,
      color: edgeColors[edge.kind]
    },
    style: {
      stroke: edgeColors[edge.kind],
      strokeWidth: edge.kind === "derived" ? 2.6 : 1.8
    },
    labelStyle: { fill: "var(--edge-label-text)", fontWeight: 800 },
    labelBgPadding: [6, 3],
    labelBgBorderRadius: 6,
    labelBgStyle: {
      fill: "var(--edge-label-bg)",
      fillOpacity: 0.98,
      stroke: "var(--edge-label-border)",
      strokeWidth: 1
    },
    type: "smoothstep"
  }));

  return { nodes, edges };
}

export function filterVisibleNodes(schema: StudioSchema | null, query: string): StudioNode[] {
  if (!schema) return [];
  const needle = query.trim().toLowerCase();
  if (!needle) return schema.nodes;
  return schema.nodes.filter((node) => {
    return (
      node.name.toLowerCase().includes(needle) ||
      node.kind.includes(needle) ||
      node.fields.some(
        (field) =>
          field.name.toLowerCase().includes(needle) || field.ty.toLowerCase().includes(needle)
      )
    );
  });
}

export function filterGraphNodes(
  schema: StudioSchema | null,
  visibleNodes: StudioNode[],
  selected: StudioNode | null,
  graphMode: GraphMode
): StudioNode[] {
  if (!schema || graphMode === "all" || !selected) return visibleNodes;
  const visibleIds = new Set(visibleNodes.map((node) => node.id));
  const relatedIds = new Set<string>([selected.id]);
  for (const edge of schema.edges) {
    if (graphMode === "fields" && edge.source === selected.id && visibleIds.has(edge.target)) {
      relatedIds.add(edge.target);
    }
    if (graphMode === "usedBy" && edge.target === selected.id && visibleIds.has(edge.source)) {
      relatedIds.add(edge.source);
    }
  }
  return visibleNodes.filter((node) => relatedIds.has(node.id));
}

function collectLinkedFields(edges: StudioEdge[]) {
  const linkedFieldsByNode = new Map<string, Set<string>>();
  for (const edge of edges) {
    const fields = linkedFieldsByNode.get(edge.source) ?? new Set<string>();
    fields.add(edge.label);
    linkedFieldsByNode.set(edge.source, fields);
  }
  return linkedFieldsByNode;
}

function buildDefaultPositions(
  graphMode: GraphMode,
  graphNodes: StudioNode[],
  schema: StudioSchema,
  selected: StudioNode | null
) {
  const positions = new Map<string, { x: number; y: number }>();
  if (graphMode !== "all" && selected) {
    if (graphMode === "fields") {
      positions.set(selected.id, { x: 40, y: 180 });
      const outgoingNodes = graphNodes.filter((node) =>
        edgeTouchesSelected(schema, selected, node.id, "out")
      );
      for (const [id, position] of stackedPositions(outgoingNodes, 430)) {
        positions.set(id, position);
      }
    } else {
      const incomingNodes = graphNodes.filter((node) =>
        edgeTouchesSelected(schema, selected, node.id, "in")
      );
      for (const [id, position] of stackedPositions(incomingNodes, 40)) {
        positions.set(id, position);
      }
      positions.set(selected.id, { x: 430, y: 180 });
    }
    return positions;
  }

  for (const kind of kindOrder) {
    const lanes = [40, 40];
    const kindIndex = kindOrder.indexOf(kind);
    graphNodes
      .filter((node) => node.kind === kind)
      .sort((a, b) => a.name.localeCompare(b.name))
      .forEach((node, index) => {
        const lane = index % 2;
        positions.set(node.id, {
          x: 40 + kindIndex * 760 + lane * 340,
          y: lanes[lane]
        });
        lanes[lane] += cardHeight(node) + 34;
      });
  }
  return positions;
}

function stackedPositions(nodes: StudioNode[], x: number, startY = 70) {
  let y = startY;
  return new Map(
    nodes
      .sort((a, b) => a.name.localeCompare(b.name))
      .map((node) => {
        const position = [node.id, { x, y }] as const;
        y += cardHeight(node) + 36;
        return position;
      })
  );
}

function cardHeight(node: StudioNode) {
  return 70 + Math.max(node.fields.length, 1) * 34;
}

function edgeTouchesSelected(
  schema: StudioSchema,
  selected: StudioNode,
  nodeId: string,
  direction: "in" | "out"
) {
  return schema.edges.some((edge) =>
    direction === "in"
      ? edge.source === nodeId && edge.target === selected.id
      : edge.source === selected.id && edge.target === nodeId
  );
}
