# Nullability

Schema nullability is expressed with `optional<T>`. Code generators map that schema type to the strongest nullability representation available in each target language.

Runtime bundles encode optional values with explicit presence. Generated code should preserve that distinction in its public API instead of relying on undocumented `null` conventions.

## Built-In Representations

| Target | `optional<T>` representation |
| --- | --- |
| Rust | `Option<T>` |
| C# | `T?` with nullable reference types enabled |
| Kotlin | `T?` |
| Dart | `T?` |
| Scala | `Option[T]` |
| TypeScript | `T | undefined` |
| JavaScript d.ts | `T | undefined` |
| Python | `T | None` |
| C++ | `std::optional<T>` for C++17 and newer; `SoraOptional<T>` for older standards |
| C | generated optional wrapper type with presence state |
| Go | `*T` |
| Erlang | `T | undefined` typespec |
| Lua | `T?` EmmyLua annotation |
| Godot | `Variant` with `null` |
| Java | nullable value type plus annotation |

Dynamic targets such as JavaScript, Lua, and Godot can only document nullability for tooling. Statically typed targets expose it in the generated type whenever the language supports that.

## Java Annotations

Java has no standard nullable type syntax. Sora emits nullable Java fields, constructor parameters, and nullable lookup results with an annotation.

By default, Java generation uses a self-contained package-local `SoraNullable` annotation:

```java
@SoraNullable
public final String nickname;
```

Projects that use a specific annotation package can configure it:

```toml
[codegen.java]
nullable_annotation = "org.jetbrains.annotations.Nullable"
```

Set `nullable_annotation = ""` to emit nullable Java values without annotations.

## Custom Type Mappings

Type mapping scripts can provide `nullable_type_name` when the target language needs a different type expression for `optional<YourType>`:

```lua
{
  target = "java",
  schema_type = "UserId",
  type_name = "int",
  nullable_type_name = "Integer",
}
```

This only changes the generated type expression. The backend still controls how optional presence is decoded.
