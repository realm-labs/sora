(function () {
  var knownPages = new Set([
    "index.html",
    "concepts.html",
    "quick-start.html",
    "tutorial/overview.html",
    "tutorial/first-config.html",
    "tutorial/excel-workflow.html",
    "tutorial/load-generated-code.html",
    "schema.html",
    "schema/formats.html",
    "schema/tables.html",
    "schema/types.html",
    "schema/enums-structs-unions.html",
    "schema/references.html",
    "schema/parsers.html",
    "project-config.html",
    "localization.html",
    "studio.html",
    "cli.html",
    "exports.html",
    "export/formats.html",
    "codegen/overview.html",
    "codegen/runtime-formats.html",
    "codegen/adapters.html",
    "versioning.html",
    "extension.html",
    "extension/generators.html",
    "extension/exporters.html",
    "design/overview.html",
    "design/schema-as-source-of-truth.html",
    "design/excel-header-projection.html",
    "design/ir-boundaries.html"
  ]);

  function normalizedRelativePath() {
    var relative = relativePathFromLocation(window.location);

    if (relative === "" || relative.endsWith("/")) {
      relative += "index.html";
    }
    return relative;
  }

  function pageExists(relativePath) {
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
    link.href = siteRootFromLocation(window.location) + targetFor(relative);
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

  function relativePathFromLocation(location) {
    return relativePathFromPath(location.pathname);
  }

  function relativePathFromPath(path) {
    var marker = "/sora/";
    var index = path.indexOf(marker);
    if (index >= 0) {
      return path.slice(index + marker.length);
    }

    var bookIndex = path.indexOf("/docs/book/");
    if (bookIndex >= 0) {
      return path.slice(bookIndex + "/docs/book/".length);
    }

    var root = typeof path_to_root === "string" ? path_to_root : "";
    var rootPath = new URL(root || ".", window.location.href).pathname;
    if (!rootPath.endsWith("/")) {
      rootPath += "/";
    }
    if (path.startsWith(rootPath)) {
      return path.slice(rootPath.length);
    }

    return path.replace(/^\/+/, "");
  }

  function siteRootFromLocation(location) {
    var path = location.pathname;
    var marker = "/sora/";
    var index = path.indexOf(marker);
    if (index >= 0) {
      return path.slice(0, index + marker.length);
    }

    var bookIndex = path.indexOf("/docs/book/");
    if (bookIndex >= 0) {
      return path.slice(0, bookIndex + "/docs/book/".length);
    }

    var root = typeof path_to_root === "string" ? path_to_root : "";
    var rootPath = new URL(root || ".", location.href).pathname;
    if (!rootPath.endsWith("/")) {
      rootPath += "/";
    }
    if (rootPath.endsWith("/zh/")) {
      return rootPath.slice(0, -3);
    }
    return rootPath;
  }

  function initialize() {
    addLanguageToggle();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initialize);
  } else {
    initialize();
  }
})();
