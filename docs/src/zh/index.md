# Sora

Sora 用来让游戏配置表保持易懂，同时让运行时代码获得强类型访问。

你先用 schema 描述表结构，再用 Excel、CSV、TOML、JSON 或 YAML 填行数据。Sora 会校验这些数据，校验通过后导出运行时数据包，并生成知道如何加载这个数据包的代码。

核心思路是：schema 是契约。Excel、CSV、TOML、生成代码和运行时数据包，都是这个契约的不同投影。策划可以在工作簿里编辑行数据，游戏代码则通过生成出来的强类型 API 读取配置。

一个小项目里的文件流通常是：

```text
project.toml
  -> schema/items.toml
  -> data/Item.xlsx
  -> generated/config.sora
  -> generated/rust
```

通常手写 `project.toml` 和 schema 文件。策划或工具编辑 `data/` 下的数据文件。`generated/` 下的文件由 Sora 生成。

## Sora 做什么

```text
schema modules -> Excel/CSV/TOML/JSON/YAML data -> validation
                                      |-> runtime bundle
                                      |-> generated code
```

Sora 当前聚焦这些阶段：

- 用 schema 描述表、记录、枚举、联合、引用、索引和校验规则；
- 在内置的 Sora Studio UI 中查看和编辑 schema module；
- 根据 schema 生成 Excel 模板，避免表头和字段定义漂移；
- 从 TOML、JSON、YAML、CSV 或 Excel `.xlsx` 加载表格数据；
- 按归一化 schema 校验数据和跨表引用；
- 导出 Sora binary、debug JSON、JSON bundle、CBOR bundle 或 Sora Protobuf bundle；
- 生成可以加载这些数据包的语言运行时代码。

## 常见术语

Sora 里 `format` 会出现在几个不同位置：

| 术语 | 含义 | 例子 |
| --- | --- | --- |
| Schema format | schema/project 文件本身的格式。 | TOML、YAML、JSON、Lua |
| Source format | 可编辑表格数据的格式。 | Excel `.xlsx`、CSV、TOML、JSON、YAML |
| Export format | 校验后写出的数据包格式。 | `binary`、`json`、`cbor` |
| Runtime format | 生成代码期望加载的数据包格式。 | `sora`、`json`、`cbor` |

例如 Rust 代码生成使用 `runtime_format = "sora"` 时，需要匹配一个 `binary` export。源数据仍然可以来自 Excel。

## 适用场景

Sora 适合游戏配置和类似的数据密集型项目：

- 策划或工具需要编辑表格数据；
- 运行时代码希望读取强类型配置，而不是散乱字典；
- schema 变更需要进入源码审查；
- 项目方可能需要扩展自己的语言生成器或导出格式。

项目仍处于早期阶段，公共 API 可能继续调整。设计目标是让核心 schema 和 IR 独立于具体语言后端，方便下游用户增加生成器或导出器，而不用 patch 核心管线。

需要稳定输出的项目应该固定 `sora` CLI 版本。只有真实的生成 runtime 不兼容时，Sora 才会升级 runtime/export format version；当前不会用 edition flag 保留旧 schema 语义。见[版本与兼容性](versioning.md)。

## 推荐阅读顺序

先读[快速开始](quick-start.md)，再读 [Sora Studio](studio.md)、[第一份配置](tutorial/first-config.md)和[Excel 工作流](tutorial/excel-workflow.md)。之后最常用的参考页是[类型](schema/types.md)、[表](schema/tables.md)、[单元格 Parser](schema/parsers.md)、[引用和派生字段](schema/references.md)和[版本与兼容性](versioning.md)。

设计说明和扩展页面适合已经理解基本构建流程之后再读。
