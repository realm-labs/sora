import {
  BaseEdge,
  Position,
  getSmoothStepPath,
  type EdgeProps,
  type XYPosition
} from "@xyflow/react";

const endpointOffset = 6;

export function SchemaEdge({
  id,
  markerEnd,
  sourcePosition,
  sourceX,
  sourceY,
  style,
  targetPosition,
  targetX,
  targetY
}: EdgeProps) {
  const source = offsetEndpoint({ x: sourceX, y: sourceY }, sourcePosition);
  const target = offsetEndpoint({ x: targetX, y: targetY }, targetPosition);
  const [edgePath] = getSmoothStepPath({
    sourcePosition,
    sourceX: source.x,
    sourceY: source.y,
    targetPosition,
    targetX: target.x,
    targetY: target.y
  });

  return <BaseEdge id={id} markerEnd={markerEnd} path={edgePath} style={style} />;
}

function offsetEndpoint(point: XYPosition, position: Position | undefined) {
  switch (position) {
    case Position.Left:
      return { ...point, x: point.x - endpointOffset };
    case Position.Right:
      return { ...point, x: point.x + endpointOffset };
    case Position.Top:
      return { ...point, y: point.y - endpointOffset };
    case Position.Bottom:
      return { ...point, y: point.y + endpointOffset };
    default:
      return point;
  }
}
