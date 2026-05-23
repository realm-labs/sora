# Schema 格式

Sora schema 文件可以写成 TOML 或 YAML。两种格式会加载到同一个 schema model，后续生成的 IR、代码、Excel 模板、导出文件和 schema lock 都一致。

文件扩展名决定 parser：

| 扩展名 | 格式 |
| --- | --- |
| `.toml` | TOML |
| `.yaml`、`.yml` | YAML |

include 文件按自己的扩展名解析，所以 YAML 项目可以 include TOML module，TOML 项目也可以 include YAML module。

## TOML

```toml
package = "game_config"
includes = ["schema/items.toml"]

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true
```

## YAML

```yaml
package: game_config
includes:
  - schema/items.yaml

enums:
  - name: ItemType
    values: [Weapon, Armor]

tables:
  - name: Item
    mode: map
    key: id
    fields:
      - name: id
        type: i32
        key: true
        required: true
```

## 项目 Build 配置

项目文件里的 `build` 也可以使用 YAML：

```yaml
package: game_config
includes:
  - schema/items.yaml

build:
  default_source_format: xlsx
  data_root: data
  schema_lock: generated/schema.lock
  excel_templates: generated/excel
  codegen:
    - target: rust
      out: generated/rust
      format: auto
  exports:
    - format: binary
      out: generated/config.sora
```

