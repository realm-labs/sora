import { ChevronLeft, ChevronRight, Moon, RefreshCw, RotateCcw, Save, Sun } from "lucide-react";

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
  refresh,
  schema,
  saveDisabled,
  saveLocalChanges,
  saving,
  setLanguage,
  t,
  theme,
  toggleTheme
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
  refresh: () => void;
  schema: StudioSchema | null;
  saveDisabled: boolean;
  saveLocalChanges: () => void;
  saving: boolean;
  setLanguage: (language: Language) => void;
  t: Translation;
  theme: Theme;
  toggleTheme: () => void;
}) {
  return (
    <header className="topbar">
      <div>
        <p>{project || t.noProjectLoaded}</p>
        <h2>{schema ? schema.package : t.schemaUnavailable}</h2>
      </div>
      <div className="topbar-actions">
        {dirty && (
          <div className="dirty-state">
            <span>{t.unsaved}</span>
            <button
              className="icon-button"
              disabled={saving || saveDisabled}
              onClick={saveLocalChanges}
              title={saveDisabled ? t.saveDisabled : t.save}
            >
              <Save size={14} />
              {saving ? t.saving : t.save}
            </button>
            <button className="icon-button" onClick={discardLocalChanges}>
              <RotateCcw size={14} />
              {t.discard}
            </button>
          </div>
        )}
        <button className="icon-button icon-only" onClick={goBack} disabled={!canGoBack}>
          <ChevronLeft size={17} />
        </button>
        <button className="icon-button icon-only" onClick={goForward} disabled={!canGoForward}>
          <ChevronRight size={17} />
        </button>
        <div className="language-switch" aria-label={t.language}>
          <button className={language === "en" ? "active" : ""} onClick={() => setLanguage("en")}>
            EN
          </button>
          <button className={language === "zh" ? "active" : ""} onClick={() => setLanguage("zh")}>
            中文
          </button>
        </div>
        <button className="icon-button" onClick={toggleTheme}>
          {theme === "dark" ? <Sun size={16} /> : <Moon size={16} />}
          {theme === "dark" ? t.light : t.dark}
        </button>
        <button className="icon-button" onClick={refresh} disabled={loading}>
          <RefreshCw size={16} />
          {t.refresh}
        </button>
      </div>
    </header>
  );
}
