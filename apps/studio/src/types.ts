export type NodeKind = "enum" | "struct" | "union" | "table";
export type EdgeKind = "type" | "ref" | "derived";
export type Language = "en" | "zh";
export type GraphMode = "fields" | "usedBy" | "all";
export type Theme = "dark" | "light";

export type StudioField = {
  name: string;
  ty: string;
  scope: string;
  parser?: string | null;
  comment?: string | null;
  source?: string | null;
};

export type StudioNode = {
  id: string;
  name: string;
  kind: NodeKind;
  scope: string;
  subtitle: string;
  fields: StudioField[];
  metadata: Record<string, string>;
};

export type StudioEdge = {
  id: string;
  source: string;
  target: string;
  kind: EdgeKind;
  label: string;
};

export type StudioSchema = {
  package: string;
  summary: {
    enums: number;
    structs: number;
    unions: number;
    tables: number;
    edges: number;
  };
  nodes: StudioNode[];
  edges: StudioEdge[];
};

export type StudioResponse = {
  ok: boolean;
  project: string;
  diagnostics: Array<{ level: "error" | "info"; message: string }>;
  schema?: StudioSchema | null;
};
