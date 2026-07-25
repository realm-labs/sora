import { useCallback, useEffect, useMemo, useState, type PointerEvent as ReactPointerEvent } from "react";
import { type NodeChange, type NodeMouseHandler } from "@xyflow/react";
import { AlertTriangle, CircleDot, Eye, X } from "lucide-react";

import { GraphCanvas } from "./components/GraphCanvas";
import { Inspector } from "./components/Inspector";
import { Sidebar } from "./components/Sidebar";
import { Topbar } from "./components/Topbar";
import { buildGraph, filterGraphNodes, filterVisibleNodes } from "./graph";
import { translations } from "./i18n";
import {
  addEnumValue,
  addField,
  addNode,
  addSchemaSource,
  addUnionVariant,
  addUnionVariantField,
  deleteField,
  deleteNode,
  deleteSchemaSource,
  deleteUnionVariant,
  moveField,
  moveUnionVariant,
  moveUnionVariantField,
  renameNode,
  updateEnumValue,
  updateNodeSettings,
  updateField,
  updateProjectId,
  updateUnionVariant,
  validateSchema,
  type EditableNodeSettingsDraft,
  type EditableFieldDraft,
  type StudioValidationIssue
} from "./schemaEditing";
import type {
  GraphMode,
  Language,
  NodeKind,
  StudioDiagnostic,
  StudioPreviewResponse,
  StudioResponse,
  StudioSaveResponse,
  StudioSchema,
  Theme
} from "./types";

const leftPanelMinWidth = 220;
const rightPanelMinWidth = 300;
const leftPanelMaxWidth = 420;
const rightPanelMaxWidth = 520;
const canvasMinWidth = 360;
const panelWidthStorageKey = "sora-studio-panel-widths";
const nodeSizeStorageKey = "sora-studio-node-sizes-v3";

type PanelWidths = {
  left: number;
  right: number;
};

export function App() {
  const [response, setResponse] = useState<StudioResponse | null>(null);
  const [editableSchema, setEditableSchema] = useState<StudioSchema | null>(null);
  const [dirty, setDirty] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [previewing, setPreviewing] = useState(false);
  const [preview, setPreview] = useState<StudioPreviewResponse | null>(null);
  const [activeView, setActiveView] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectionHistory, setSelectionHistory] = useState<{ items: string[]; index: number }>({
    items: [],
    index: -1
  });
  const [graphMode, setGraphMode] = useState<GraphMode>("fields");
  const [theme, setTheme] = useState<Theme>("dark");
  const [language, setLanguage] = useState<Language>("en");
  const [panelWidths, setPanelWidths] = useState<PanelWidths>(loadPanelWidths);
  const [manualPositions, setManualPositions] = useState<Record<string, { x: number; y: number }>>(
    {}
  );
  const [manualSizes, setManualSizes] = useState<Record<string, { width: number; height: number }>>(
    loadNodeSizes
  );
  const [layoutRevision, setLayoutRevision] = useState(0);
  const t = translations[language];

  const load = async (view = activeView) => {
    setLoading(true);
    try {
      const queryString = view ? `?view=${encodeURIComponent(view)}` : "";
      const next: StudioResponse = await fetch(`/api/schema${queryString}`).then((res) => res.json());
      setResponse(next);
      setEditableSchema(next.schema ? structuredClone(next.schema) : null);
      setDirty(false);
      const nextSelection =
        next.schema?.nodes.find((node) => node.id === selectedId)?.id ??
        next.schema?.nodes[0]?.id ??
        null;
      setSelectedId(nextSelection);
      setSelectionHistory(
        nextSelection ? { items: [nextSelection], index: 0 } : { items: [], index: -1 }
      );
    } catch (error) {
      setResponse({
        ok: false,
        project: "",
        diagnostics: [
          {
            severity: "error",
            message: error instanceof Error ? error.message : String(error)
          }
        ],
        schema: null
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  useEffect(() => {
    try {
      window.localStorage.setItem(panelWidthStorageKey, JSON.stringify(panelWidths));
    } catch {
      // Panel resizing still works when localStorage is unavailable.
    }
  }, [panelWidths]);

  useEffect(() => {
    const onResize = () => setPanelWidths((current) => fitPanelWidths(current));
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  useEffect(() => {
    try {
      window.localStorage.setItem(nodeSizeStorageKey, JSON.stringify(manualSizes));
    } catch {
      // Card resizing still works when localStorage is unavailable.
    }
  }, [manualSizes]);

  const schema = editableSchema ?? response?.schema ?? null;
  const readOnly = activeView !== null;
  const contractId = activeView
    ? schema?.views[activeView]?.contract ?? schema?.project_id ?? ""
    : schema?.project_id ?? "";
  const localValidationIssues = useMemo(() => (schema ? validateSchema(schema) : []), [schema]);
  const backendValidationIssues = useMemo(
    () => (dirty ? [] : diagnosticsToValidationIssues(response?.diagnostics ?? [])),
    [dirty, response?.diagnostics]
  );
  const validationIssues = useMemo(
    () => [...backendValidationIssues, ...localValidationIssues],
    [backendValidationIssues, localValidationIssues]
  );
  const nodeIssueCounts = useMemo(() => issueCountsByNode(validationIssues), [validationIssues]);
  const visibleNodes = useMemo(() => filterVisibleNodes(schema, query), [query, schema]);
  const selected = useMemo(() => {
    if (!schema) return null;
    return schema.nodes.find((node) => node.id === selectedId) ?? schema.nodes[0] ?? null;
  }, [schema, selectedId]);
  const graphNodes = useMemo(
    () => filterGraphNodes(schema, visibleNodes, selected, graphMode),
    [graphMode, schema, selected, visibleNodes]
  );
  const resizeNode = useCallback(
    (
      id: string,
      next: { position: { x: number; y: number }; size: { width: number; height: number } }
    ) => {
      setManualPositions((positions) => ({
        ...positions,
        [id]: {
          x: Math.round(next.position.x),
          y: Math.round(next.position.y)
        }
      }));
      setManualSizes((sizes) => ({
        ...sizes,
        [id]: {
          width: Math.round(next.size.width),
          height: Math.round(next.size.height)
        }
      }));
    },
    []
  );
  const graph = useMemo(() => {
    if (!schema) return { nodes: [], edges: [] };
    return buildGraph({
      graphMode,
      graphNodes,
      language,
      manualPositions,
      manualSizes,
      nodeIssueCounts,
      resizeNode,
      schema,
      selected
    });
  }, [
    graphMode,
    graphNodes,
    language,
    manualPositions,
    manualSizes,
    nodeIssueCounts,
    resizeNode,
    schema,
    selected
  ]);

  const navigateToNode = useCallback(
    (id: string) => {
      if (id === selectedId) return;
      setSelectedId(id);
      setSelectionHistory((history) => {
        if (history.items[history.index] === id) return history;
        const base = history.index >= 0 ? history.items.slice(0, history.index + 1) : [];
        const items = [...base, id];
        return { items, index: items.length - 1 };
      });
    },
    [selectedId]
  );
  const onNodeClick: NodeMouseHandler = (_, node) => navigateToNode(node.id);
  const onNodesChange = useCallback((changes: NodeChange[]) => {
    setManualPositions((positions) => {
      let next = positions;
      for (const change of changes) {
        if (change.type !== "position" || !change.position) continue;
        if (next === positions) {
          next = { ...positions };
        }
        next[change.id] = change.position;
      }
      return next;
    });
  }, []);

  const canGoBack = selectionHistory.index > 0;
  const canGoForward =
    selectionHistory.index >= 0 && selectionHistory.index < selectionHistory.items.length - 1;
  const goBack = () => navigateHistory(selectionHistory.index - 1);
  const goForward = () => navigateHistory(selectionHistory.index + 1);
  const resetLayout = () => {
    setManualPositions({});
    setManualSizes({});
    setLayoutRevision((value) => value + 1);
  };
  const toggleTheme = () => setTheme((value) => (value === "dark" ? "light" : "dark"));
  const switchView = (view: string | null) => {
    if (view === activeView) return;
    if (dirty && !window.confirm(t.switchViewDiscard)) return;
    setActiveView(view);
    void load(view);
  };
  const startPanelResize = (side: "left" | "right", event: ReactPointerEvent) => {
    event.preventDefault();
    document.body.classList.add("resizing-panel");
    const onPointerMove = (moveEvent: PointerEvent) => {
      setPanelWidths((current) => resizePanel(side, moveEvent.clientX, current));
    };
    const stopResize = () => {
      document.body.classList.remove("resizing-panel");
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", stopResize);
      window.removeEventListener("pointercancel", stopResize);
    };
    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", stopResize);
    window.addEventListener("pointercancel", stopResize);
  };
  const discardLocalChanges = () => {
    setEditableSchema(response?.schema ? structuredClone(response.schema) : null);
    setDirty(false);
  };
  const saveLocalChanges = async () => {
    if (!schema || !response?.revision || localValidationIssues.length > 0) return;
    setSaving(true);
    try {
      const plan: StudioPreviewResponse = await fetch("/api/schema/preview", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          schema,
          expected_schema_revision: response.revision.schema,
          expected_manifest_revision: response.revision.manifest
        })
      }).then((res) => res.json());
      if (!plan.ok || !plan.plan_id) {
        setPreview(plan);
        return;
      }
      const next: StudioSaveResponse = await fetch("/api/schema/apply", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          plan_id: plan.plan_id,
          idempotency_key: crypto.randomUUID()
        })
      }).then((res) => res.json());
      setResponse(next);
      if (next.ok && next.schema) {
        setEditableSchema(structuredClone(next.schema));
        setDirty(false);
      }
    } catch (error) {
      setResponse({
        ok: false,
        project: response?.project ?? "",
        diagnostics: [
          {
            severity: "error",
            message: error instanceof Error ? error.message : String(error)
          }
        ],
        schema
      });
    } finally {
      setSaving(false);
    }
  };

  const previewLocalChanges = async () => {
    if (!schema || !response?.revision || localValidationIssues.length > 0) return;
    setPreviewing(true);
    try {
      const next: StudioPreviewResponse = await fetch("/api/schema/preview", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          schema,
          expected_schema_revision: response.revision.schema,
          expected_manifest_revision: response.revision.manifest
        })
      }).then((res) => res.json());
      setPreview(next);
    } catch (error) {
      setPreview({
        ok: false,
        project: response?.project ?? "",
        target: null,
        content: null,
        diff: null,
        diagnostics: [
          {
            severity: "error",
            message: error instanceof Error ? error.message : String(error)
          }
        ]
      });
    } finally {
      setPreviewing(false);
    }
  };

  const createNode = (kind: NodeKind) => {
    if (!schema) return;
    const result = addNode(schema, kind);
    setEditableSchema(result.schema);
    setDirty(true);
    navigateToNode(result.nodeId);
  };

  const renameSelectedNode = (id: string, name: string) => {
    const node = schema?.nodes.find((item) => item.id === id);
    if (!schema || !node) return;
    const nextName = name.trim();
    if (!nextName) return;
    const nextId = `${node.kind}:${nextName}`;
    setEditableSchema(renameNode(schema, id, nextName));
    setManualPositions((positions) => {
      if (!positions[id]) return positions;
      const next = { ...positions, [nextId]: positions[id] };
      delete next[id];
      return next;
    });
    setManualSizes((sizes) => {
      if (!sizes[id]) return sizes;
      const next = { ...sizes, [nextId]: sizes[id] };
      delete next[id];
      return next;
    });
    setDirty(true);
    navigateToNode(nextId);
  };

  const deleteSelectedNode = (id: string) => {
    if (!schema) return;
    const next = deleteNode(schema, id);
    setEditableSchema(next);
    setDirty(true);
    const fallback = next.nodes[0]?.id ?? null;
    setSelectedId(fallback);
    setSelectionHistory(fallback ? { items: [fallback], index: 0 } : { items: [], index: -1 });
  };

  const editNodeSettings = (id: string, draft: EditableNodeSettingsDraft) => {
    if (!schema) return;
    setEditableSchema(updateNodeSettings(schema, id, draft));
    setDirty(true);
  };

  const editProjectId = (projectId: string) => {
    if (!schema) return;
    setEditableSchema(updateProjectId(schema, projectId));
    setDirty(true);
  };

  const createSchemaSource = (source: string) => {
    if (!schema) return;
    const next = addSchemaSource(schema, source);
    if (next === schema) return;
    setEditableSchema(next);
    setDirty(true);
  };

  const removeSchemaSource = (source: string) => {
    if (!schema) return;
    if (!window.confirm(t.deleteSchemaFileConfirm.replace("{source}", source))) return;
    const next = deleteSchemaSource(schema, source);
    if (next === schema) return;
    setEditableSchema(next);
    setDirty(true);
  };

  const createField = (ownerId: string, draft: EditableFieldDraft) => {
    if (!schema) return;
    setEditableSchema(addField(schema, ownerId, draft));
    setDirty(true);
  };

  const createEnumValue = (ownerId: string, id: number, name: string) => {
    if (!schema) return;
    setEditableSchema(addEnumValue(schema, ownerId, id, name));
    setDirty(true);
  };

  const editEnumValue = (ownerId: string, fieldIndex: number, id: number, name: string) => {
    if (!schema) return;
    setEditableSchema(updateEnumValue(schema, ownerId, fieldIndex, id, name));
    setDirty(true);
  };

  const createUnionVariant = (ownerId: string, name: string) => {
    if (!schema) return;
    setEditableSchema(addUnionVariant(schema, ownerId, name));
    setDirty(true);
  };

  const editUnionVariant = (ownerId: string, fieldIndex: number, name: string) => {
    if (!schema) return;
    setEditableSchema(updateUnionVariant(schema, ownerId, fieldIndex, name));
    setDirty(true);
  };

  const removeUnionVariant = (ownerId: string, fieldIndex: number) => {
    if (!schema) return;
    setEditableSchema(deleteUnionVariant(schema, ownerId, fieldIndex));
    setDirty(true);
  };

  const createUnionVariantField = (ownerId: string, variantName: string, draft: EditableFieldDraft) => {
    if (!schema) return;
    setEditableSchema(addUnionVariantField(schema, ownerId, variantName, draft));
    setDirty(true);
  };

  const editField = (ownerId: string, fieldIndex: number, draft: EditableFieldDraft) => {
    if (!schema) return;
    setEditableSchema(updateField(schema, ownerId, fieldIndex, draft));
    setDirty(true);
  };

  const removeField = (ownerId: string, fieldIndex: number) => {
    if (!schema) return;
    setEditableSchema(deleteField(schema, ownerId, fieldIndex));
    setDirty(true);
  };

  const reorderField = (ownerId: string, fieldIndex: number, direction: -1 | 1) => {
    if (!schema) return;
    setEditableSchema(moveField(schema, ownerId, fieldIndex, direction));
    setDirty(true);
  };

  const reorderUnionVariant = (ownerId: string, fieldIndex: number, direction: -1 | 1) => {
    if (!schema) return;
    setEditableSchema(moveUnionVariant(schema, ownerId, fieldIndex, direction));
    setDirty(true);
  };

  const reorderUnionVariantField = (
    ownerId: string,
    variantName: string,
    fieldIndex: number,
    direction: -1 | 1
  ) => {
    if (!schema) return;
    setEditableSchema(moveUnionVariantField(schema, ownerId, variantName, fieldIndex, direction));
    setDirty(true);
  };

  function selectInitialNode(id: string) {
    setSelectedId(id);
    setSelectionHistory({ items: [id], index: 0 });
  }

  function navigateHistory(index: number) {
    if (index < 0 || index >= selectionHistory.items.length) return;
    setSelectionHistory((history) => ({ ...history, index }));
    setSelectedId(selectionHistory.items[index]);
  }

  return (
    <main
      className="studio-shell"
      data-theme={theme}
      style={{
        gridTemplateColumns: `${panelWidths.left}px 8px minmax(${canvasMinWidth}px, 1fr) 8px ${panelWidths.right}px`
      }}
    >
      <Topbar
        canGoBack={canGoBack}
        canGoForward={canGoForward}
        dirty={dirty}
        discardLocalChanges={discardLocalChanges}
        goBack={goBack}
        goForward={goForward}
        language={language}
        loading={loading}
        previewLocalChanges={() => void previewLocalChanges()}
        previewing={previewing}
        project={response?.project ?? ""}
        refresh={() => void load(activeView)}
        schema={schema}
        activeView={activeView}
        onSelectView={switchView}
        setLanguage={setLanguage}
        updateProjectId={editProjectId}
        saveDisabled={localValidationIssues.length > 0}
        saveLocalChanges={() => void saveLocalChanges()}
        saving={saving}
        t={t}
        theme={theme}
        toggleTheme={toggleTheme}
      />

      <Sidebar
        issueCounts={nodeIssueCounts}
        navigateToNode={navigateToNode}
        onAddNode={createNode}
        onAddSchemaSource={createSchemaSource}
        onDeleteSchemaSource={removeSchemaSource}
        query={query}
        schema={schema}
        selectedId={selected?.id ?? null}
        setQuery={setQuery}
        t={t}
        visibleNodes={visibleNodes}
        readOnly={readOnly}
      />

      <div
        aria-label={t.resizeLeftPanel}
        aria-orientation="vertical"
        className="panel-resizer left-resizer"
        onPointerDown={(event) => startPanelResize("left", event)}
        role="separator"
        title={t.resizeLeftPanel}
      />

      <section className="canvas-panel">
        {response?.diagnostics?.length ? (
          <div className={response.ok ? "workspace-notice ok" : "workspace-notice error"}>
            {response.ok ? <CircleDot size={16} /> : <AlertTriangle size={16} />}
            <span>{response.ok ? t.schemaLoaded : response.diagnostics[0].message}</span>
          </div>
        ) : null}

        {schema ? (
          <GraphCanvas
            edges={graph.edges}
            graphMode={graphMode}
            layoutRevision={layoutRevision}
            nodes={graph.nodes}
            onNodeClick={onNodeClick}
            onNodesChange={onNodesChange}
            query={query}
            resetLayout={resetLayout}
            selectedId={selected?.id ?? null}
            setGraphMode={setGraphMode}
            t={t}
            theme={theme}
          />
        ) : (
          <div className="graph-wrap">
            <div className="empty-state">{t.startStudioApi}</div>
          </div>
        )}
      </section>

      <div
        aria-label={t.resizeRightPanel}
        aria-orientation="vertical"
        className="panel-resizer right-resizer"
        onPointerDown={(event) => startPanelResize("right", event)}
        role="separator"
        title={t.resizeRightPanel}
      />

      <aside className={readOnly ? "inspector read-only" : "inspector"}>
        {readOnly ? (
          <div className="read-only-banner">
            <Eye size={13} />
            <span>{t.readOnlyView}</span>
          </div>
        ) : null}
        {selected && schema ? (
          <fieldset className="inspector-fieldset" disabled={readOnly}>
            <Inspector
              edges={schema?.edges ?? []}
              language={language}
              node={selected}
              onAddEnumValue={createEnumValue}
              onAddField={createField}
              onAddUnionVariant={createUnionVariant}
              onAddUnionVariantField={createUnionVariantField}
              onDeleteField={removeField}
              onDeleteNode={deleteSelectedNode}
              onDeleteUnionVariant={removeUnionVariant}
              onMoveField={reorderField}
              onMoveUnionVariant={reorderUnionVariant}
              onMoveUnionVariantField={reorderUnionVariantField}
              onRenameNode={renameSelectedNode}
              onUpdateEnumValue={editEnumValue}
              onUpdateNodeSettings={editNodeSettings}
              onUpdateField={editField}
              onUpdateUnionVariant={editUnionVariant}
              schema={schema}
              validationIssues={validationIssues.filter((issue) => issue.targetId === selected.id)}
            />
          </fieldset>
        ) : (
          <div className="empty-state">{t.selectSchemaItem}</div>
        )}
      </aside>

      <footer className="statusbar">
        <span className={validationIssues.length > 0 ? "status-health issue" : "status-health valid"}>
          <CircleDot size={11} />
          {validationIssues.length > 0
            ? `${validationIssues.length} ${t.issues}`
            : t.valid}
        </span>
        <strong className="status-view">{activeView ?? t.canonical}</strong>
        {contractId ? <code title={contractId}>{contractId}</code> : null}
        <span>
          {schema?.summary.tables ?? 0} {t.kindPlural.table.toLowerCase()}
        </span>
        <span>
          {schema?.nodes.length ?? 0} {t.entities}
        </span>
        <span>
          {schema?.summary.edges ?? 0} {t.edges.toLowerCase()}
        </span>
        <span className="statusbar-spacer" />
        <span>{response?.project ? fileName(response.project) : t.noProjectLoaded}</span>
        {response?.revision?.schema ? (
          <code title={response.revision.schema}>
            {t.revision} {shortRevision(response.revision.schema)}
          </code>
        ) : null}
      </footer>

      {preview && (
        <div className="modal-backdrop" role="presentation">
          <section aria-modal="true" className="preview-modal" role="dialog">
            <header>
              <div>
                <p>{t.targetFile}</p>
                <h3>{preview.target ?? t.schemaPreview}</h3>
              </div>
              <button className="icon-button icon-only" onClick={() => setPreview(null)} title={t.close}>
                <X size={16} />
              </button>
            </header>
            {preview.diagnostics.length > 0 && (
              <div className={preview.ok ? "diagnostics ok" : "diagnostics error"}>
                {preview.diagnostics[0].message}
              </div>
            )}
            <div className="preview-grid">
              <section>
                <h4>{t.previewDiff}</h4>
                <DiffView value={preview.diff ?? ""} />
              </section>
              <section>
                <h4>{t.renderedToml}</h4>
                <pre>{preview.content ?? ""}</pre>
              </section>
            </div>
          </section>
        </div>
      )}
    </main>
  );
}

function DiffView({ value }: { value: string }) {
  const lines = value.split("\n");
  return (
    <pre className="diff-view">
      {lines.map((line, index) => (
        <span className={`diff-line ${diffLineClass(line)}`} key={`${index}:${line}`}>
          {line || " "}
        </span>
      ))}
    </pre>
  );
}

function diffLineClass(line: string) {
  if (line === "No changes.") return "diff-empty";
  if (line.startsWith("@@")) return "diff-hunk";
  if (line.startsWith("project:") || line.startsWith("include:")) return "diff-file";
  if (line.startsWith("+++") || line.startsWith("---")) return "diff-marker";
  if (line.startsWith("+")) return "diff-add";
  if (line.startsWith("-")) return "diff-delete";
  return "diff-context";
}

function loadPanelWidths(): PanelWidths {
  try {
    const parsed = JSON.parse(window.localStorage.getItem(panelWidthStorageKey) ?? "");
    return fitPanelWidths({
      left: clamp(Number(parsed.left), leftPanelMinWidth, leftPanelMaxWidth),
      right: clamp(Number(parsed.right), rightPanelMinWidth, rightPanelMaxWidth)
    });
  } catch {
    return fitPanelWidths({ left: 256, right: 340 });
  }
}

function fileName(path: string) {
  return path.split(/[\\/]/).pop() ?? path;
}

function shortRevision(revision: string) {
  return revision.startsWith("sha256:") ? revision.slice(7, 15) : revision.slice(0, 8);
}

function loadNodeSizes(): Record<string, { width: number; height: number }> {
  try {
    const parsed = JSON.parse(window.localStorage.getItem(nodeSizeStorageKey) ?? "");
    if (!parsed || typeof parsed !== "object") return {};
    return Object.fromEntries(
      Object.entries(parsed)
        .map(([id, size]) => {
          const value = size as { width?: unknown; height?: unknown };
          const width = clamp(Number(value.width), 240, 720);
          const height = clamp(Number(value.height), 120, 900);
          return [id, { width, height }] as const;
        })
        .filter(([id]) => id.includes(":"))
    );
  } catch {
    return {};
  }
}

function resizePanel(side: "left" | "right", clientX: number, current: PanelWidths): PanelWidths {
  const viewportWidth = window.innerWidth;
  if (side === "left") {
    const maxWidth = Math.min(leftPanelMaxWidth, viewportWidth - current.right - canvasMinWidth);
    return {
      ...current,
      left: clamp(clientX, leftPanelMinWidth, Math.max(leftPanelMinWidth, maxWidth))
    };
  }

  const maxWidth = Math.min(rightPanelMaxWidth, viewportWidth - current.left - canvasMinWidth);
  return {
    ...current,
    right: clamp(viewportWidth - clientX, rightPanelMinWidth, Math.max(rightPanelMinWidth, maxWidth))
  };
}

function fitPanelWidths(current: PanelWidths): PanelWidths {
  const leftMaxWidth = Math.min(leftPanelMaxWidth, window.innerWidth - current.right - canvasMinWidth);
  const left = clamp(current.left, leftPanelMinWidth, Math.max(leftPanelMinWidth, leftMaxWidth));
  const rightMaxWidth = Math.min(rightPanelMaxWidth, window.innerWidth - left - canvasMinWidth);
  return {
    left,
    right: clamp(current.right, rightPanelMinWidth, Math.max(rightPanelMinWidth, rightMaxWidth))
  };
}

function clamp(value: number, min: number, max: number) {
  if (!Number.isFinite(value)) return min;
  return Math.min(Math.max(value, min), max);
}

function diagnosticsToValidationIssues(diagnostics: StudioDiagnostic[]): StudioValidationIssue[] {
  return diagnostics
    .filter((diagnostic) => diagnostic.severity === "error")
    .map((diagnostic, index) => ({
      id: `backend:${index}:${diagnostic.targetId ?? "global"}`,
      message: diagnostic.message,
      targetId: diagnostic.targetId ?? undefined
    }));
}

function issueCountsByNode(issues: StudioValidationIssue[]): Record<string, number> {
  const counts: Record<string, number> = {};
  for (const issue of issues) {
    if (!issue.targetId) continue;
    counts[issue.targetId] = (counts[issue.targetId] ?? 0) + 1;
  }
  return counts;
}
