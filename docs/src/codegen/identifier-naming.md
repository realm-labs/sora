# Identifier Naming

Schema names are the source of truth. Generated code adapts those names to each target language's naming style, while runtime data lookup keeps using the original schema names.

For example, a schema field named `max_stack` may become `maxStack` in TypeScript, `MaxStack` in C#, and `max_stack` in Rust. The generated decoder still reads the field named `max_stack` from the runtime bundle.

## Naming Pipeline

Sora derives common name forms from each schema name before language generation:

| Form | Example from `max_stack` | Common use |
| --- | --- | --- |
| Raw | `max_stack` | Runtime table names, field names, enum text values, union tags. |
| Pascal | `MaxStack` | Types, classes, enum variants, exported symbols. |
| Camel | `maxStack` | Fields, properties, parameters, methods in camel-case languages. |
| Snake | `max_stack` | Files, modules, fields, functions in snake-case languages. |

Language generators choose from these forms and may apply additional language-specific sanitization for invalid identifiers or reserved words.

## Language Conventions

The built-in generators follow the target language's normal public API style:

| Target | Types | Fields and accessors | Files/modules |
| --- | --- | --- | --- |
| Rust | `PascalCase` | `snake_case` | `snake_case.rs` |
| C | prefixed `snake_case` | `snake_case` | `snake_case.h`, `snake_case.c` |
| C++ | `PascalCase` | `snake_case` | `snake_case.hpp` |
| C# | `PascalCase` | `PascalCase` properties | `PascalCase.cs` |
| Go | `PascalCase` exported names | `PascalCase` exported fields | `snake_case.go` |
| Java | `PascalCase` | `lowerCamelCase` | `PascalCase.java` |
| Kotlin | `PascalCase` | `lowerCamelCase` | target layout |
| Scala | `PascalCase` | `lowerCamelCase` | `PascalCase.scala` |
| TypeScript | `PascalCase` | `lowerCamelCase` | `snake_case.ts` |
| JavaScript | `PascalCase` | `lowerCamelCase` | `snake_case.js`, `snake_case.d.ts` |
| Python | `PascalCase` | `snake_case` | `snake_case.py` |
| Dart | `PascalCase` | `lowerCamelCase` | `snake_case.dart` |
| Lua | `PascalCase` table-like types | `lowerCamelCase` | `snake_case.lua` |
| Erlang | `snake_case` modules | `snake_case` map keys/functions | `snake_case.erl` |
| Godot | `PascalCase` classes | `snake_case` | `snake_case.gd` |

This table describes generated code identifiers, not runtime data names.

## Runtime Names Stay Raw

The following values keep the original schema spelling:

- table names in bundles and table metadata;
- field names read from runtime rows;
- enum string values;
- union variant tag values;
- schema lock and fingerprint input.

Changing a schema name changes the data contract. Changing only a target language's generated identifier style should not.

## Custom Type Mappings

Custom type mappings do not rename generated schema identifiers. They only control target-language type expressions, imports/includes, and optional conversion hooks.

Mapping function names are native code written by the user for that target language, so they should follow that language's own naming convention. The mapping key remains the named schema type, such as `Vec3`.
