import { useEffect, useState } from "react";
import {
  ChevronLeft,
  ChevronRight,
  Eye,
  Languages,
  Moon,
  MoreHorizontal,
  Network,
  RefreshCw,
  RotateCcw,
  Save,
  Sun
} from "lucide-react";

import type { Translation } from "../i18n";
import type { Language, StudioSchema, Theme } from "../types";

export function Topbar({
  canGoBack,
  canGoForward,
  dirty,
  discardLocalChanges,
  goBack,
  goForward,
  language,
  loading,
  project,
  previewLocalChanges,
  previewing,
  refresh,
  schema,
  saveDisabled,
  saveLocalChanges,
  saving,
  setLanguage,
  t,
  theme,
  toggleTheme,
  updateProjectId
}: {
  canGoBack: boolean;
  canGoForward: boolean;
  dirty: boolean;
  discardLocalChanges: () => void;
  goBack: () => void;
  goForward: () => void;
  language: Language;
  loading: boolean;
  project: string;
  previewLocalChanges: () => void;
  previewing: boolean;
  refresh: () => void;
  schema: StudioSchema | null;
  saveDisabled: boolean;
  saveLocalChanges: () => void;
  saving: boolean;
  setLanguage: (language: Language) => void;
  t: Translation;
  theme: Theme;
  toggleTheme: () => void;
  updateProjectId: (projectId: string) => void;
}) {
  const [projectIdDraft, setProjectIdDraft] = useState(schema?.project_id ?? "");

  useEffect(() => {
    setProjectIdDraft(schema?.project_id ?? "");
  }, [schema?.project_id]);

  const commitProjectId = () => {
    const clean = projectIdDraft.trim();
    if (clean && clean !== schema?.project_id) updateProjectId(clean);
    else setProjectIdDraft(schema?.project_id ?? "");
  };
  const projectFile = project.split(/[\\/]/).pop() || t.noProjectLoaded;

  return (
    <header className="app-header">
      <div className="app-identity">
        <span className="app-mark" aria-hidden="true">
          <Network size={16} />
        </span>
        <strong>Sora Studio</strong>
      </div>

      <div className="header-history">
        <button
          aria-label={t.goBack}
          className="icon-button icon-only"
          onClick={goBack}
          disabled={!canGoBack}
          title={t.goBack}
        >
          <ChevronLeft size={17} />
        </button>
        <button
          aria-label={t.goForward}
          className="icon-button icon-only"
          onClick={goForward}
          disabled={!canGoForward}
          title={t.goForward}
        >
          <ChevronRight size={17} />
        </button>
      </div>

      <div className="project-breadcrumb">
        <span>{projectFile}</span>
        <strong>{schema?.project_id ?? t.schemaUnavailable}</strong>
      </div>

      <div className="header-spacer" />

      {dirty ? (
        <div className="header-dirty">
          <span>{t.unsaved}</span>
          <button
            className="text-button"
            disabled={saving || previewing || saveDisabled}
            onClick={previewLocalChanges}
            title={saveDisabled ? t.saveDisabled : t.preview}
          >
            <Eye size={14} />
            {previewing ? t.previewing : t.preview}
          </button>
          <button
            className="text-button primary"
            disabled={saving || saveDisabled}
            onClick={saveLocalChanges}
            title={saveDisabled ? t.saveDisabled : t.save}
          >
            <Save size={14} />
            {saving ? t.saving : t.save}
          </button>
          <button
            aria-label={t.discard}
            className="icon-button icon-only"
            onClick={discardLocalChanges}
            title={t.discard}
          >
            <RotateCcw size={14} />
          </button>
        </div>
      ) : null}

      <details className="app-menu">
        <summary aria-label={t.moreActions} title={t.moreActions}>
          <MoreHorizontal size={17} />
        </summary>
        <div className="app-menu-popover">
          {schema ? (
            <label className="menu-field">
              <span>{t.projectId}</span>
              <input
                aria-label={t.projectId}
                value={projectIdDraft}
                onBlur={commitProjectId}
                onChange={(event) => setProjectIdDraft(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") commitProjectId();
                }}
              />
            </label>
          ) : null}
          <div className="menu-separator" />
          <button onClick={() => setLanguage(language === "en" ? "zh" : "en")}>
            <Languages size={15} />
            {language === "en" ? "中文" : "English"}
          </button>
          <button onClick={toggleTheme}>
            {theme === "dark" ? <Sun size={15} /> : <Moon size={15} />}
            {theme === "dark" ? t.light : t.dark}
          </button>
          <button onClick={refresh} disabled={loading}>
            <RefreshCw size={15} />
            {t.refresh}
          </button>
          {project ? <code title={project}>{project}</code> : null}
        </div>
      </details>
    </header>
  );
}
