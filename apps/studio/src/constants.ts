import { Boxes, Braces, GitBranch, Table2, type LucideIcon } from "lucide-react";

import type { EdgeKind, NodeKind } from "./types";

export const kindOrder: NodeKind[] = ["table", "struct", "union", "enum"];

export const kindMeta: Record<NodeKind, { color: string; icon: LucideIcon }> = {
  table: { color: "#2563eb", icon: Table2 },
  struct: { color: "#059669", icon: Boxes },
  union: { color: "#dc2626", icon: GitBranch },
  enum: { color: "#7c3aed", icon: Braces }
};

export const edgeColors: Record<EdgeKind, string> = {
  type: "#64748b",
  ref: "#2563eb",
  derived: "#d97706"
};

export function fieldHandleId(fieldName: string) {
  return `field:${fieldName}`;
}
