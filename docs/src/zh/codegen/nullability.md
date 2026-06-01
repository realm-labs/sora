# 空值表达

Schema 使用 `optional<T>` 表达可空性。代码生成器会把这个 schema 类型映射成目标语言中能表达的最强可空形式。

运行时数据包会显式编码 optional presence。生成代码的公开 API 应该保留这个区别，而不是依赖没有文档说明的 `null` 约定。

## 内置表示

| Target | `optional<T>` 表示 |
| --- | --- |
| Rust | `Option<T>` |
| C# | 启用 nullable reference types 的 `T?` |
| Kotlin | `T?` |
| Dart | `T?` |
| Scala | `Option[T]` |
| TypeScript | `T | undefined` |
| JavaScript d.ts | `T | undefined` |
| Python | `T | None` |
| C++ | C++17 及更新版本使用 `std::optional<T>`；旧标准使用 `SoraOptional<T>` |
| C | 带 presence state 的生成 optional wrapper type |
| Go | `*T` |
| Erlang | `T | undefined` typespec |
| Lua | `T?` EmmyLua annotation |
| Godot | 支持 `null` 的 `Variant` |
| Java | 可空 value type 加 annotation |

JavaScript、Lua、Godot 这类动态目标主要只能为工具链标注可空性。静态类型目标会尽量在生成类型中表达。

## Java Annotation

Java 没有标准的可空类型语法。Sora 会用 annotation 标记可空 Java 字段、构造参数和可能返回 `null` 的 lookup 结果。

默认情况下，Java 生成代码使用自包含的 package-local `SoraNullable` annotation：

```java
@SoraNullable
public final String nickname;
```

如果项目使用特定 annotation 包，可以配置：

```toml
[codegen.java]
nullable_annotation = "org.jetbrains.annotations.Nullable"
```

设置 `nullable_annotation = ""` 可以保留可空 Java 值，但不生成 annotation。

## 自定义类型映射

当目标语言中 `optional<YourType>` 需要不同类型表达式时，类型映射脚本可以提供 `nullable_type_name`：

```lua
{
  target = "java",
  schema_type = "UserId",
  type_name = "int",
  nullable_type_name = "Integer",
}
```

这个字段只改变生成的类型表达式。optional presence 的 decode 方式仍然由对应语言后端控制。
