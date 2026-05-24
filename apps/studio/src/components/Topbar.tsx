import { useEffect, useState } from "react";
import { ChevronLeft, ChevronRight, Eye, Moon, RefreshCw, RotateCcw, Save, Sun } from "lucide-react";

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
  updatePackage
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
  updatePackage: (packageName: string) => void;
}) {
  const [packageDraft, setPackageDraft] = useState(schema?.package ?? "");

  useEffect(() => {
    setPackageDraft(schema?.package ?? "");
  }, [schema?.package]);

  const commitPackage = () => {
    const clean = packageDraft.trim();
    if (clean && clean !== schema?.package) updatePackage(clean);
    else setPackageDraft(schema?.package ?? "");
  };

  return (
    <header className="topbar">
      <div>
        <p>{project || t.noProjectLoaded}</p>
        {schema ? (
          <label className="package-editor">
            <span>{t.package}</span>
            <input
              aria-label={t.package}
              value={packageDraft}
              onBlur={commitPackage}
              onChange={(event) => setPackageDraft(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") commitPackage();
              }}
            />
          </label>
        ) : (
          <h2>{t.schemaUnavailable}</h2>
        )}
      </div>
      <div className="topbar-actions">
        {dirty && (
          <div className="dirty-state">
            <span>{t.unsaved}</span>
            <button
              className="icon-button"
              disabled={saving || previewing || saveDisabled}
              onClick={previewLocalChanges}
              title={saveDisabled ? t.saveDisabled : t.preview}
            >
              <Eye size={14} />
              {previewing ? t.previewing : t.preview}
            </button>
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
