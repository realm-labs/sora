(function () {
  function normalizedRelativePath() {
    var path = window.location.pathname;
    var marker = "/sora/";
    var index = path.indexOf(marker);
    var relative = index >= 0 ? path.slice(index + marker.length) : path.replace(/^\/+/, "");

    if (relative === "" || relative.endsWith("/")) {
      relative += "index.html";
    }
    return relative;
  }

  function pageExists(relativePath) {
    var knownPages = new Set([
      "index.html",
      "quick-start.html",
      "concepts.html",
      "schema.html",
      "project-config.html",
      "exports.html",
      "codegen/overview.html",
      "codegen/runtime-formats.html",
      "codegen/adapters.html",
      "tutorial/overview.html",
      "tutorial/first-config.html",
      "tutorial/excel-workflow.html",
      "tutorial/load-generated-code.html",
      "schema/types.html",
      "schema/tables.html",
      "schema/enums-structs-unions.html",
      "schema/references.html",
      "export/formats.html",
      "design/overview.html",
      "design/schema-as-source-of-truth.html",
      "design/excel-header-projection.html",
      "design/ir-boundaries.html",
      "extension.html",
      "extension/generators.html",
      "extension/exporters.html"
    ]);

    var englishPath = relativePath.startsWith("zh/")
      ? relativePath.slice(3)
      : relativePath;
    return knownPages.has(englishPath);
  }

  function targetFor(relativePath) {
    if (relativePath.startsWith("zh/")) {
      return relativePath.slice(3);
    }
    return "zh/" + relativePath;
  }

  function addLanguageToggle() {
    var relative = normalizedRelativePath();
    if (!pageExists(relative)) {
      return;
    }

    var isChinese = relative.startsWith("zh/");
    var link = document.createElement("a");
    var root = typeof path_to_root === "string" ? path_to_root : "";
    link.href = root + targetFor(relative);
    link.textContent = isChinese ? "English" : "中文";
    link.setAttribute("aria-label", isChinese ? "Switch to English" : "切换到中文");
    link.style.marginLeft = "0.75rem";
    link.style.padding = "0.2rem 0.55rem";
    link.style.border = "1px solid var(--icons)";
    link.style.borderRadius = "4px";
    link.style.fontSize = "0.85rem";
    link.style.textDecoration = "none";

    var rightButtons = document.querySelector(".right-buttons");
    if (rightButtons) {
      rightButtons.prepend(link);
      return;
    }

    var menuBar = document.getElementById("menu-bar");
    if (menuBar) {
      menuBar.appendChild(link);
    }
  }

  function languageForPath(path) {
    var marker = "/sora/";
    var index = path.indexOf(marker);
    var relative = index >= 0 ? path.slice(index + marker.length) : path.replace(/^\/+/, "");
    return relative.startsWith("zh/") ? "zh" : "en";
  }

  function filterSidebar() {
    var currentLanguage = languageForPath(window.location.pathname);
    var sidebar = document.getElementById("sidebar");
    if (!sidebar) {
      return;
    }

    sidebar.querySelectorAll("li.chapter-item").forEach(function (item) {
      if (item.querySelector(".part-title")) {
        item.hidden = true;
        return;
      }

      var link = item.querySelector(".chapter-link-wrapper > a");
      if (!link) {
        return;
      }

      var linkLanguage = languageForPath(new URL(link.href, window.location.href).pathname);
      item.hidden = linkLanguage !== currentLanguage;
    });
  }

  function initialize() {
    addLanguageToggle();
    filterSidebar();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initialize);
  } else {
    initialize();
  }
})();
