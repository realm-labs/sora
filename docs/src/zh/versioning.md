# 版本与兼容性

Sora 仍处于早期阶段。项目不提供类似 Rust edition 的旧 schema 语义兼容模式。需要稳定输出的项目应该固定使用的 `sora` CLI 版本，并把 CLI 升级视为一次显式迁移。

## 需要固定什么

在项目工具链里固定 CLI 二进制或 crate 版本：

- 下载指定版本的 GitHub Release 资源，并在 CI 中持续使用这个版本；
- 用 `cargo install sora-cli --version X.Y.Z` 安装指定 crates.io 版本；
- 在项目搭建文档或构建脚本中记录期望的 `sora --version`。

同一次项目构建中的生成代码、生成 Excel 模板、schema lock 和导出的运行时数据包，都应该来自同一个固定的 CLI 版本。

## 运行时数据包版本

导出的运行时数据包会携带格式版本。Sora binary bundle 也有文件头版本，生成的 runtime 会拒绝读取不支持的版本。

只有当生成 runtime 无法安全读取旧布局写出的数据时，Sora 才会升级这些 runtime/export format version。例如：

- `.sora` 二进制 section 布局发生变化；
- 生成 runtime 依赖的 manifest 字段发生破坏性变化；
- JSON、CBOR 或 Protobuf bundle 结构变化，导致旧生成代码无法读取；
- 导出的运行时数据包中的值编码规则发生变化。

在早期开发阶段，普通实现变化不会自动升级 `format_version`。版本升级是手动动作，只保留给真实的 runtime/export 不兼容。

## Schema 和 Codegen 语义

项目还年轻时，schema 语法、parser 行为、校验规则、Studio 渲染和生成语言 API 都可能继续调整。Sora 不会用 `edition` flag 或其他兼容模式保留旧行为。

如果新版 CLI 改变了 schema 或 codegen 语义，用户应该：

1. 有意识地升级 CLI；
2. 重新生成 schema lock、模板、导出数据和代码；
3. 审查 diff；
4. 按需要更新 schema/data/project 文件。

Schema fingerprint 和 schema lock 可以帮助发现生成代码、schema 和数据之间的不匹配，但它们不是迁移工具。它们负责避免静默不兼容，不负责保留旧语义。
