# Schema Formats

Sora schema files can be written as TOML or YAML. Both formats load into the same schema model and produce the same IR, generated code, Excel templates, exports, and schema locks.

The file extension selects the parser:

| Extension | Format |
| --- | --- |
| `.toml` | TOML |
| `.yaml`, `.yml` | YAML |

Includes are parsed by their own file extension, so a YAML project can include TOML modules and a TOML project can include YAML modules.

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

## Project Build Config

The project file can also use YAML for `build`:

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

