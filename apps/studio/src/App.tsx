import { useCallback, useEffect, useMemo, useState } from "react";
import { type NodeChange, type NodeMouseHandler } from "@xyflow/react";
import { AlertTriangle, CircleDot } from "lucide-react";

import { GraphCanvas } from "./components/GraphCanvas";
import { Inspector } from "./components/Inspector";
import { Sidebar } from "./components/Sidebar";
import { Topbar } from "./components/Topbar";
import { buildGraph, filterGraphNodes, filterVisibleNodes } from "./graph";
import { translations } from "./i18n";
import type { GraphMode, Language, StudioResponse, Theme } from "./types";

export function App() {
  const [response, setResponse] = useState<StudioResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [query, setQuery] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectionHistory, setSelectionHistory] = useState<{ items: string[]; index: number }>({
    items: [],
    index: -1
  });
  const [graphMode, setGraphMode] = useState<GraphMode>("fields");
  const [theme, setTheme] = useState<Theme>("dark");
  const [language, setLanguage] = useState<Language>("en");
  const [manualPositions, setManualPositions] = useState<Record<string, { x: number; y: number }>>(
    {}
  );
  const [layoutRevision, setLayoutRevision] = useState(0);
  const t = translations[language];

  const load = async () => {
    setLoading(true);
    try {
      const next: StudioResponse = await fetch("/api/schema").then((res) => res.json());
      setResponse(next);
      if (!selectedId && next.schema?.nodes?.length) {
        selectInitialNode(next.schema.nodes[0].id);
      }
    } catch (error) {
      setResponse({
        ok: false,
        project: "",
        diagnostics: [
          {
            level: "error",
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

  const schema = response?.schema ?? null;
  const visibleNodes = useMemo(() => filterVisibleNodes(schema, query), [query, schema]);
  const selected = useMemo(() => {
    if (!schema) return null;
    return schema.nodes.find((node) => node.id === selectedId) ?? schema.nodes[0] ?? null;
  }, [schema, selectedId]);
  const graphNodes = useMemo(
    () => filterGraphNodes(schema, visibleNodes, selected, graphMode),
    [graphMode, schema, selected, visibleNodes]
  );
  const graph = useMemo(() => {
    if (!schema) return { nodes: [], edges: [] };
    return buildGraph({
      graphMode,
      graphNodes,
      language,
      manualPositions,
      schema,
      selected
    });
  }, [graphMode, graphNodes, language, manualPositions, schema, selected]);

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
    setLayoutRevision((value) => value + 1);
  };
  const toggleTheme = () => setTheme((value) => (value === "dark" ? "light" : "dark"));

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
    <main className="studio-shell" data-theme={theme}>
      <Sidebar
        navigateToNode={navigateToNode}
        query={query}
        schema={schema}
        selectedId={selected?.id ?? null}
        setQuery={setQuery}
        t={t}
        visibleNodes={visibleNodes}
      />

      <section className="canvas-panel">
        <Topbar
          canGoBack={canGoBack}
          canGoForward={canGoForward}
          goBack={goBack}
          goForward={goForward}
          language={language}
          loading={loading}
          project={response?.project ?? ""}
          refresh={() => void load()}
          schema={schema}
          setLanguage={setLanguage}
          t={t}
          theme={theme}
          toggleTheme={toggleTheme}
        />

        {response?.diagnostics?.length ? (
          <div className={response.ok ? "diagnostics ok" : "diagnostics error"}>
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

      <aside className="inspector">
        {selected ? (
          <Inspector node={selected} edges={schema?.edges ?? []} language={language} />
        ) : (
          <div className="empty-state">{t.selectSchemaItem}</div>
        )}
      </aside>
    </main>
  );
}
