# Sora

Sora 是一个 schema-first 的游戏配置编译器。

它读取 schema 模块和表格数据，验证并归一化成统一模型，然后生成运行时可加载的数据包以及强类型代码。

核心思路是：schema 是契约。Excel、CSV、TOML、生成代码和运行时数据包，都是这个契约的不同投影。策划可以在工作簿里编辑行数据，游戏代码则通过生成出来的强类型 API 读取配置。

## Sora 做什么

```text
schema modules -> Excel/CSV/TOML data -> validation
                                      |-> runtime bundle
                                      |-> generated code
```

Sora 当前聚焦这些阶段：

- 用 TOML schema 描述表、记录、枚举、联合、引用、索引和校验规则；
- 根据 schema 生成 Excel 模板，避免表头和字段定义漂移；
- 从 TOML、CSV 或 Excel `.xlsx` 加载表格数据；
- 按归一化 schema 校验数据和跨表引用；
- 导出 Sora binary、debug JSON、JSON bundle、CBOR bundle 或 Sora Protobuf bundle；
- 生成可以加载这些数据包的语言运行时代码。

## 适用场景

Sora 适合游戏配置和类似的数据密集型项目：

- 策划或工具需要编辑表格数据；
- 运行时代码希望读取强类型配置，而不是散乱字典；
- schema 变更需要进入源码审查；
- 项目方可能需要扩展自己的语言生成器或导出格式。

项目仍处于早期阶段，公共 API 可能继续调整。设计目标是让核心 schema 和 IR 独立于具体语言后端，方便下游用户增加生成器或导出器，而不用 patch 核心管线。
