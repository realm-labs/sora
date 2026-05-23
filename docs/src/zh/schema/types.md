# 类型

Sora 的类型表达式在 schema 字段中以字符串形式书写。

## Primitive Types

| Type | Meaning |
| --- | --- |
| `bool` | 布尔值。 |
| `i32` | 32-bit signed integer。 |
| `i64` | 64-bit signed integer。 |
| `f32` | 32-bit floating point。 |
| `f64` | 64-bit floating point。 |
| `string` | UTF-8 字符串。 |

```toml
[[tables.fields]]
name = "level"
type = "i32"
range = [1, 100]
```

## Named Types

| Type | Example |
| --- | --- |
| Enum | `enum<ItemType>` |
| Struct | `struct<ResourceCost>` |
| Union | `union<RewardAction>` |
| Reference | `ref<Item.id>` |

引用必须指向 `mode = "map"` 表的主键。容器可以包住引用，例如 `list<ref<Item.id>>`。

```toml
[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

[[tables.fields]]
name = "price"
type = "struct<ResourceCost>"
parser = { kind = "tuple" }
```

## Collections

| Type | Meaning |
| --- | --- |
| `list<T>` | 有序重复值。 |
| `set<T>` | 唯一重复值。 |
| `array<T,N>` | 固定长度重复值。 |
| `map<K,V>` | 键值对。 |
| `optional<T>` | 可空或可缺省值。 |

```toml
[[tables.fields]]
name = "tags"
type = "set<string>"
parser = { kind = "json" }
default = "[\"misc\"]"

[[tables.fields]]
name = "attributes"
type = "map<string,i32>"
parser = { kind = "map" }
```

## Field Rules

`[[tables.fields]]`、`[[structs.fields]]` 和 `[[unions.variants.fields]]` 共享通用字段属性。表字段额外拥有 key 和派生值相关属性；这些表专用属性不能写在 struct field 或 union variant field 上。

字段是否可缺省由类型表达：`optional<T>` 表示值可以缺失或为空；其它类型都要求有值，除非 `default` 填充了缺失值。

| Property | 适用范围 | 作用 |
| --- | --- | --- |
| `name` | 所有字段 | 字段名。用于源数据、校验错误、生成代码和导出的运行时数据。 |
| `type` | 所有字段 | 类型表达式，例如 `i32`、`struct<ResourceCost>` 或 `list<union<RewardAction>>`。 |
| `default` | 除派生字段外的所有字段 | 源单元格或 object 字段缺失时使用的字符串值。 |
| `key` | 仅表字段 | 标记表 key 字段。通常和 table-level `key` 一致。 |
| `comment` | 所有字段 | 用于生成 Excel 表头说明。 |
| `range` | 数值字段和数值集合元素 | 数值闭区间，写作 `[min, max]`。 |
| `length` | `string`、`list`、`set`、`array`、`map` | 长度闭区间，写作 `[min, max]`。 |
| `parser` | 单元格输入和 default | 单元格 parser 提示。见[单元格 Parser](parsers.md)。 |
| `scope` | 所有字段 | 仅在选定 generation/export scope 下包含该字段。默认是 `all`。 |
| `from` | 仅表字段 | 可选的子表来源，用来声明派生字段。 |

default 写成字符串，因为它会走和源数据相同的类型感知转换路径。

`from` 用来描述从另一张表匹配行得到的派生字段，详见[引用和派生字段](references.md)。派生字段可以是 `list<T>`、`T` 或 `optional<T>`，且不能声明 `default`。
