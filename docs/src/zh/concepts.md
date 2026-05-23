# 核心概念

## Project

项目清单声明 package 名称、schema 模块、构建输出、代码生成目标和导出目标。`sora check`、`sora build`、`sora gen` 和 `sora export` 都以它为入口。

## Schema

Schema 文件描述配置数据的形状。它定义枚举、结构体、联合、表、索引、引用和字段规则。Sora 会先把 schema 归一化成 IR，再进入校验、导出或代码生成。

## Table

表是一组命名行。表可以是 list、按某个字段做 key 的 map，或者 singleton。source 元数据说明可编辑数据来自哪里。

表 schema 也会用于生成 Excel 表头这类编辑器投影。电子表格不是契约本身，它只是编辑符合契约的行数据的一种方式。

## Value

Sora 会先把源数据单元格校验并转换成公共 value tree，再交给导出器。生成运行时从不同 runtime format 读取同一个形状，因此目标语言可以在 `sora`、`json`、`cbor`、`sora-protobuf` 之间切换，而不需要改 schema。

## Runtime Format

Runtime format 是生成代码在运行时加载的数据格式。它通过 `runtime_format` 按语言目标选择。

## Generator

Generator 是注册到 codegen registry 的语言后端。内置语言也是普通 registry entry，因此下游扩展可以复用同一条管线。

## Exporter

Exporter 负责把校验后的数据写成运行时数据包。导出器 registry 和代码生成分离，因此数据格式和语言目标可以独立演进。

## Scope

Schema、字段和表可以声明 `scope`。构建时可以选择 scope，只生成或导出某个运行环境需要的部分。
