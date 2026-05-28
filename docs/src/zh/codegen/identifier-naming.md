# 标识符命名

Schema 名称是事实来源。生成代码会把这些名称转换成目标语言习惯的标识符，但运行时数据查找仍然使用原始 schema 名称。

例如，schema 字段 `max_stack` 在 TypeScript 中可能生成 `maxStack`，在 C# 中生成 `MaxStack`，在 Rust 中仍然是 `max_stack`。生成的 decoder 读取运行时数据包时，字段名仍然是 `max_stack`。

## 命名流程

Sora 会先从每个 schema 名称派生通用命名形式，再交给语言生成器：

| 形式 | `max_stack` 的例子 | 常见用途 |
| --- | --- | --- |
| Raw | `max_stack` | 运行时 table 名、field 名、enum 文本值、union tag。 |
| Pascal | `MaxStack` | 类型、类、enum variant、导出符号。 |
| Camel | `maxStack` | camel-case 语言中的字段、属性、参数、方法。 |
| Snake | `max_stack` | snake-case 语言中的文件、模块、字段、函数。 |

语言生成器会选择合适的形式，并可以继续做语言相关的合法化处理，例如处理非法字符或保留字。

## 语言约定

内置生成器遵循目标语言常见的公开 API 风格：

| 目标 | 类型 | 字段和访问器 | 文件/模块 |
| --- | --- | --- | --- |
| Rust | `PascalCase` | `snake_case` | `snake_case.rs` |
| C | 带前缀的 `snake_case` | `snake_case` | `snake_case.h`, `snake_case.c` |
| C++ | `PascalCase` | `snake_case` | `snake_case.hpp` |
| C# | `PascalCase` | `PascalCase` property | `PascalCase.cs` |
| Go | `PascalCase` 导出名称 | `PascalCase` 导出字段 | `snake_case.go` |
| Java | `PascalCase` | `lowerCamelCase` | `PascalCase.java` |
| Kotlin | `PascalCase` | `lowerCamelCase` | target layout |
| Scala | `PascalCase` | `lowerCamelCase` | `PascalCase.scala` |
| TypeScript | `PascalCase` | `lowerCamelCase` | `snake_case.ts` |
| JavaScript | `PascalCase` | `lowerCamelCase` | `snake_case.js`, `snake_case.d.ts` |
| Python | `PascalCase` | `snake_case` | `snake_case.py` |
| Dart | `PascalCase` | `lowerCamelCase` | `snake_case.dart` |
| Lua | `PascalCase` table-like 类型 | `lowerCamelCase` | `snake_case.lua` |
| Erlang | `snake_case` 模块 | `snake_case` map key/function | `snake_case.erl` |
| Godot | `PascalCase` class | `snake_case` | `snake_case.gd` |

这个表描述的是生成代码里的标识符，不是运行时数据名。

## 运行时名称保持 Raw

下面这些值保留原始 schema 拼写：

- 数据包和 table metadata 中的 table 名；
- 从运行时 row 中读取的 field 名；
- enum string value；
- union variant tag value；
- schema lock 和 fingerprint 的输入。

修改 schema 名称会修改数据契约。仅修改某个目标语言的生成标识符风格，不应该修改数据契约。

## 自定义类型映射

自定义类型映射不会重命名生成的 schema 标识符。它只控制目标语言类型表达式、import/include，以及可选的转换 hook。

映射函数名是用户为目标语言编写的原生代码，所以应该遵循目标语言自己的命名习惯。映射 key 仍然是命名 schema 类型，例如 `Vec3`。
