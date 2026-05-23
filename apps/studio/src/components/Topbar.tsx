import { ChevronLeft, ChevronRight, Moon, RefreshCw, Sun } from "lucide-react";

import type { Translation } from "../i18n";
import type { Language, StudioSchema, Theme } from "../types";

export function Topbar({
  canGoBack,
  canGoForward,
  goBack,
  goForward,
  language,
  loading,
  project,
  refresh,
  schema,
  setLanguage,
  t,
  theme,
  toggleTheme
}: {
  canGoBack: boolean;
  canGoForward: boolean;
  goBack: () => void;
  goForward: () => void;
  language: Language;
  loading: boolean;
  project: string;
  refresh: () => void;
  schema: StudioSchema | null;
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
