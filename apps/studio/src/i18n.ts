export const translations = {
  en: {
    schemaVisualizer: "Schema visualizer",
    searchSchema: "Search schema",
    noProjectLoaded: "No project loaded",
    schemaUnavailable: "Schema unavailable",
    schemaLoaded: "schema loaded successfully",
    language: "Language",
    light: "Light",
    dark: "Dark",
    refresh: "Refresh",
    graphMode: "Graph mode",
    fieldsMode: "Fields",
    usedByMode: "Used by",
    allMode: "All",
    reset: "Reset",
    edges: "Edges",
    nodesAbbr: "N",
    edgesAbbr: "E",
    startStudioApi: "Start the Studio API with a valid project.",
    selectSchemaItem: "Select a schema item.",
    metadata: "Metadata",
    scope: "Scope",
    fields: "Fields",
    relations: "Relations",
    outgoing: "Outgoing",
    incoming: "Incoming",
    noRelations: "No relations.",
    parser: "parser",
    from: "from",
    kindPlural: {
      table: "Tables",
      struct: "Structs",
      union: "Unions",
      enum: "Enums"
    },
    kindSingular: {
      table: "table",
      struct: "struct",
      union: "union",
      enum: "enum"
    },
    edgeKind: {
      type: "type",
      ref: "ref",
      derived: "derived"
    }
  },
  zh: {
    schemaVisualizer: "Schema 可视化",
    searchSchema: "搜索 Schema",
    noProjectLoaded: "未加载项目",
    schemaUnavailable: "Schema 不可用",
    schemaLoaded: "schema 加载成功",
    language: "语言",
    light: "浅色",
    dark: "深色",
    refresh: "刷新",
    graphMode: "图模式",
    fieldsMode: "字段",
    usedByMode: "被使用",
    allMode: "全部",
    reset: "重置",
    edges: "关系",
    nodesAbbr: "点",
    edgesAbbr: "线",
    startStudioApi: "请使用有效项目启动 Studio API。",
    selectSchemaItem: "请选择一个 Schema 项。",
    metadata: "元数据",
    scope: "作用域",
    fields: "字段",
    relations: "关系",
    outgoing: "指向",
    incoming: "来源",
    noRelations: "没有关系。",
    parser: "解析器",
    from: "来自",
    kindPlural: {
      table: "表",
      struct: "结构",
      union: "联合",
      enum: "枚举"
    },
    kindSingular: {
      table: "表",
      struct: "结构",
      union: "联合",
      enum: "枚举"
    },
    edgeKind: {
      type: "类型",
      ref: "引用",
      derived: "聚合"
    }
  }
} as const;

export type Translation = (typeof translations)[keyof typeof translations];
