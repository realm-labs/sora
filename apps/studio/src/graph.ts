import { MarkerType, type Edge, type Node } from "@xyflow/react";

import { edgeColors, fieldHandleId, kindOrder } from "./constants";
import type { GraphMode, Language, StudioEdge, StudioNode, StudioSchema } from "./types";
import { unionVariants } from "./unionFields";

export type SchemaCardData = {
  node: StudioNode;
  selected: boolean;
  linkedSourceFields: Set<string>;
  linkedTargetFields: Set<string>;
  linkedTargetNode: boolean;
  issueCount: number;
  language: Language;
  resizeNode: (
    id: string,
    next: { position: { x: number; y: number }; size: { width: number; height: number } }
  ) => void;
};

type BuildGraphOptions = {
  graphMode: GraphMode;
  graphNodes: StudioNode[];
  language: Language;
  manualPositions: Record<string, { x: number; y: number }>;
  manualSizes: Record<string, { width: number; height: number }>;
  nodeIssueCounts: Record<string, number>;
  resizeNode: SchemaCardData["resizeNode"];
  schema: StudioSchema;
  selected: StudioNode | null;
};

export function buildGraph({
  graphMode,
  graphNodes,
  language,
  manualPositions,
  manualSizes,
  nodeIssueCounts,
  resizeNode,
  schema,
  selected
}: BuildGraphOptions): { nodes: Node<SchemaCardData>[]; edges: Edge[] } {
  const visibleIds = new Set(graphNodes.map((node) => node.id));
  const visibleEdges = schema.edges.filter(
    (edge) => visibleIds.has(edge.source) && visibleIds.has(edge.target)
  );
  const linkedPortsByNode = collectLinkedPorts(visibleEdges);
  const positions = buildDefaultPositions(graphMode, graphNodes, schema, selected, manualSizes);

  const nodes: Node<SchemaCardData>[] = graphNodes.map((node) => {
    const linkedPorts = linkedPortsByNode.get(node.id);
    return {
      id: node.id,
      type: "schemaCard",
      position: manualPositions[node.id] ?? positions.get(node.id) ?? { x: 40, y: 40 },
      style: {
        width: manualSizes[node.id]?.width ?? 310,
        height: manualSizes[node.id]?.height
      },
      data: {
        node,
        selected: node.id === selected?.id,
        linkedSourceFields: linkedPorts?.sourceFields ?? new Set<string>(),
        linkedTargetFields: linkedPorts?.targetFields ?? new Set<string>(),
        linkedTargetNode: linkedPorts?.targetNode ?? false,
        issueCount: nodeIssueCounts[node.id] ?? 0,
        language,
        resizeNode
      }
    };
  });

  const edges: Edge[] = visibleEdges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    sourceHandle: fieldHandleId(edge.label),
    target: edge.target,
    targetHandle: edge.targetLabel ? fieldHandleId(edge.targetLabel) : "target",
    animated: edge.kind === "derived",
    markerEnd: {
      type: MarkerType.ArrowClosed,
      color: edgeColors[edge.kind]
    },
    style: {
      stroke: edgeColors[edge.kind],
      strokeWidth: edge.kind === "derived" ? 2.6 : 1.8
    },
    type: "schemaEdge"
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

type LinkedPorts = {
  sourceFields: Set<string>;
  targetFields: Set<string>;
  targetNode: boolean;
};

function collectLinkedPorts(edges: StudioEdge[]) {
  const linkedPortsByNode = new Map<string, LinkedPorts>();
  for (const edge of edges) {
    const sourcePorts = ensureLinkedPorts(linkedPortsByNode, edge.source);
    sourcePorts.sourceFields.add(edge.label);

    const targetPorts = ensureLinkedPorts(linkedPortsByNode, edge.target);
    if (edge.targetLabel) {
      targetPorts.targetFields.add(edge.targetLabel);
    } else {
      targetPorts.targetNode = true;
    }
  }
  return linkedPortsByNode;
}

function ensureLinkedPorts(portsByNode: Map<string, LinkedPorts>, nodeId: string) {
  const existing = portsByNode.get(nodeId);
  if (existing) return existing;
  const ports: LinkedPorts = {
    sourceFields: new Set<string>(),
    targetFields: new Set<string>(),
    targetNode: false
  };
  portsByNode.set(nodeId, ports);
  return ports;
}

function buildDefaultPositions(
  graphMode: GraphMode,
  graphNodes: StudioNode[],
  schema: StudioSchema,
  selected: StudioNode | null,
  manualSizes: Record<string, { width: number; height: number }>
) {
  const positions = new Map<string, { x: number; y: number }>();
  if (graphMode !== "all" && selected) {
    if (graphMode === "fields") {
      positions.set(selected.id, { x: 40, y: 140 });
      const gap = nodeWidth(selected.id, manualSizes) + 90;
      const outgoingNodes = graphNodes.filter((node) =>
        edgeTouchesSelected(schema, selected, node.id, "out")
      );
      for (const [id, position] of multiColumnPositions(outgoingNodes, 40 + gap, manualSizes)) {
        positions.set(id, position);
      }
    } else {
      const incomingNodes = graphNodes.filter((node) =>
        edgeTouchesSelected(schema, selected, node.id, "in")
      );
      const incomingPositions = multiColumnPositions(incomingNodes, 40, manualSizes);
      const gap = columnGap(incomingNodes, manualSizes);
      for (const [id, position] of incomingPositions) {
        positions.set(id, position);
      }
      positions.set(selected.id, {
        x: 40 + columnCount(incomingNodes, manualSizes) * gap,
        y: 140
      });
    }
    return positions;
  }

  for (const kind of kindOrder) {
    const lanes = [40, 40];
    const kindIndex = kindOrder.indexOf(kind);
    const kindNodes = graphNodes.filter((node) => node.kind === kind);
    const laneGap = columnGap(kindNodes, manualSizes);
    const kindGap = laneGap * 2 + 110;
    kindNodes
      .sort((a, b) => a.name.localeCompare(b.name))
      .forEach((node, index) => {
        const lane = index % 2;
        positions.set(node.id, {
          x: 40 + kindIndex * kindGap + lane * laneGap,
          y: lanes[lane]
        });
        lanes[lane] += cardHeight(node, manualSizes) + 34;
      });
  }
  return positions;
}

function multiColumnPositions(
  nodes: StudioNode[],
  startX: number,
  manualSizes: Record<string, { width: number; height: number }>,
  startY = 54
) {
  const sortedNodes = [...nodes].sort((a, b) => a.name.localeCompare(b.name));
  const columns = columnCount(sortedNodes, manualSizes);
  const gap = columnGap(sortedNodes, manualSizes);
  const heights = Array.from({ length: columns }, () => startY);
  const positions = new Map<string, { x: number; y: number }>();
  for (const node of sortedNodes) {
    const column = shortestColumn(heights);
    positions.set(node.id, { x: startX + column * gap, y: heights[column] });
    heights[column] += cardHeight(node, manualSizes) + 34;
  }
  return positions;
}

function columnCount(nodes: StudioNode[], manualSizes: Record<string, { width: number; height: number }>) {
  if (nodes.length <= 2) return 1;
  const totalHeight = nodes.reduce((sum, node) => sum + cardHeight(node, manualSizes) + 34, 0);
  return Math.min(3, Math.max(1, Math.ceil(totalHeight / 680)));
}

function shortestColumn(heights: number[]) {
  let best = 0;
  for (let index = 1; index < heights.length; index += 1) {
    if (heights[index] < heights[best]) best = index;
  }
  return best;
}

function cardHeight(node: StudioNode, manualSizes: Record<string, { width: number; height: number }>) {
  return manualSizes[node.id]?.height ?? naturalCardHeight(node);
}

function naturalCardHeight(node: StudioNode) {
  if (node.kind === "union") {
    const variants = unionVariants(node);
    const fieldCount = variants.reduce((count, variant) => count + variant.fields.length, 0);
    return 96 + variants.length * 34 + Math.max(fieldCount, 1) * 34;
  }
  return 70 + Math.max(node.fields.length, 1) * 34;
}

function columnGap(nodes: StudioNode[], manualSizes: Record<string, { width: number; height: number }>) {
  const width = nodes.reduce((max, node) => Math.max(max, nodeWidth(node.id, manualSizes)), 310);
  return width + 70;
}

function nodeWidth(nodeId: string, manualSizes: Record<string, { width: number; height: number }>) {
  return manualSizes[nodeId]?.width ?? 310;
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
