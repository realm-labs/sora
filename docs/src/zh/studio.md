# Sora Studio

Sora Studio 是内置在 `sora` CLI 里的浏览器 schema 编辑器。它用于查看和编辑项目 schema，不需要用户单独启动前端开发服务器。

用项目文件启动：

```bash
sora studio --project project.toml
```

默认监听 `127.0.0.1:5174`，并在终端打印本地地址。如果需要换地址，可以使用 `--host` 或 `--port`：

```bash
sora studio --project project.toml --port 5180
```

## 可以编辑什么

Studio 会加载项目文件，以及 `includes` 中列出的每个 schema module。项目文件和 schema module 可以使用 TOML、YAML、JSON 或 Lua，同一个项目里也可以混用这些格式。

编辑器可以修改：

- 项目 package 名称和 schema include 列表；
- schema module 文件，包括新增和删除 include 文件；
- 表、结构体、枚举和联合；
- 表字段、结构体字段、枚举值和联合分支；
- 表模式、主键、数据源设置、parser 设置、默认值、备注、范围和长度约束；
- 引用字段和从子表派生出来的字段。

Studio 是 schema 编辑器，不是行数据编辑器。Excel、CSV 和 TOML 表数据仍然在各自的源文件中编辑，并通过 `sora check`、`sora export` 或 `sora build` 校验。

## 可视化能力

主画布会展示 schema 节点和它们之间的关系：

- 字段使用枚举、结构体或联合时产生 type edge；
- `ref<Table>` 字段产生 reference edge；
- 从其他表组装出的子表字段产生 derived edge。

侧边栏可以按名称过滤 schema，展示项目统计，并按类型组织节点。诊断信息会显示在 UI 中，所以某个 schema 出错时，可以在 Studio 中定位错误，而不是让整个编辑器不可用。

## 预览和保存

保存前可以先预览 Studio 将要写入的文件。Studio 会按每个项目文件或 schema 文件自己的格式输出：

- `.toml` 文件写成 TOML；
- `.yaml` 和 `.yml` 文件写成 YAML；
- `.json` 文件写成格式化 JSON；
- `.lua` 文件写成返回数据表的 Lua。

保存会用 Studio 的 renderer 归一化被修改的文件。这是有意的：Studio 保持 schema 数据模型稳定，但不会保留注释、精确空白或编辑文件中的手写排序。提交前应该先看预览。

## 交付方式

发布版会把 Studio 前端资源嵌入 `sora` 二进制。最终用户只需要从 GitHub Releases 或 crates.io 安装 CLI，不需要 Node.js，也不需要本地 Vite server。

发布维护者在构建 CLI 前需要先构建前端：

```bash
cd apps/studio
npm run build
cd ../..
cargo build -p sora-cli --release
```

如果嵌入资源缺失，`sora studio` 会提示需要先构建 `apps/studio` 再构建 CLI。
