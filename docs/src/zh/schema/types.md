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
required = true
range = [1, 100]
```

## Named Types

| Type | Example |
| --- | --- |
| Enum | `enum<ItemType>` |
| Struct | `struct<ResourceCost>` |
| Union | `union<RewardAction>` |
| Reference | `ref<Item.id>` |

```toml
[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true

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

| Property | Purpose |
| --- | --- |
| `required` | 要求有值，除非存在 default。 |
| `default` | 源单元格为空时使用的值。 |
| `key` | 标记表 key 字段。 |
| `comment` | 用于生成 Excel 表头说明。 |
| `range` | 数值闭区间。 |
| `length` | 字符串或集合长度闭区间。 |
| `parser` | 单元格 parser 提示，例如 `json`、`tuple`、`tuple_list` 或 `map`。 |
| `scope` | 仅在选定 generation/export scope 下包含该字段。 |

default 写成字符串，因为它会走和源数据相同的类型感知转换路径。
