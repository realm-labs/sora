# Localization

Sora treats translated text as a separate locale catalog, not as a normal config table.

Business config stores text keys with the `text` type. Locale source sheets provide translations for those keys. Runtime code loads the normal config bundle and mounts one or more locale packs separately.

```text
business tables -> config bundle
localization sources -> LocaleCatalog -> i18n locale packs
```

## Text Keys

Use `text` for fields that point to localized copy:

```toml
[[tables.fields]]
name = "title_key"
type = "text"

[[tables.fields]]
name = "body_keys"
type = "list<text>"
```

`text` is a key, not the translated text itself. Source data should contain values such as `quest.1001.title` or `ui.confirm`. Generated Rust code exposes this as `TextKey`; dynamic targets may represent it as a string key.

The catalog validator checks every `text` value in business data. A missing key is a build error.

## Catalog Sources

Declare localization at the project schema root:

```toml
[localization]
locales = ["zh_cn", "en_us"]
default_locale = "zh_cn"
fallback_locale = "en_us"
strict = true

[[localization.sources]]
name = "ui"
file = "Core.xlsx"
sheet = "UILocalization"

[[localization.sources]]
name = "quest"
file = "Quest.xlsx"
sheet = "QuestLocalization"
```

Each source is a wide table. The default key column is `key`:

| key | zh_cn | en_us | note |
| --- | --- | --- | --- |
| `ui.confirm` | 确认 | Confirm | button label |
| `quest.1001.title` | 第一章 | Chapter One | quest title |

Locale columns named in `locales` are exported into locale packs. Other columns, such as `note`, are editor-only metadata and are ignored by runtime packs.

Rules:

| Rule | Behavior |
| --- | --- |
| `source.name` | Must be an ASCII identifier. It is used for diagnostics and organization, not as a key prefix. |
| `key` values | May use dotted names such as `quest.1001.title`. |
| Multiple sources | All sources merge into one logical catalog. |
| Duplicate keys | Build error. Keys are globally unique across all sources. |
| Missing locale column | Build error. |
| Empty translations with `strict = true` | Build error. |

Use `key = "id"` on a source if the key column is not named `key`:

```toml
[[localization.sources]]
name = "ui"
file = "Core.xlsx"
sheet = "UILocalization"
key = "id"
```

## Export Locale Packs

Normal exports (`binary`, `json`, `cbor`, `sora-protobuf`, `proto`) contain business data and text keys only. They do not include translation text.

Add i18n exports in the build manifest:

```toml
[[build.exports]]
format = "binary"
out = "generated/config.sora"

[[build.exports]]
format = "i18n-binary"
out = "generated/i18n/zh_cn.sora-i18n"
locale = "zh_cn"

[[build.exports]]
format = "i18n-json"
out = "generated/i18n/en_us.json"
locale = "en_us"
```

Use `i18n-binary` for production locale packs. Use `i18n-json` for inspection, external translation handoff, or tests.

## Runtime Mounting

Generated runtimes load config and locale packs separately. In Rust:

```rust
let config = SoraConfig::from_bytes(config_bytes)?;
let pack = generated::runtime::LocalePack::from_bytes(locale_bytes)?;

let mut i18n = generated::SoraI18n::new();
i18n.mount(pack)?;
i18n.set_locale("zh_cn")?;

let quest = config.quest().get(&1001).unwrap();
let title = i18n.text(&quest.title_key)?;
```

Mounting validates:

| Check | Purpose |
| --- | --- |
| `schema_fingerprint` | Prevents loading a locale pack generated for a different schema. |
| locale declaration | Rejects packs for locales not declared in `[localization].locales`. |
| mounted locale | `set_locale` fails until a pack for that locale has been mounted. |

Business code does not know which source sheet a key came from. It only passes `TextKey` values to the i18n runtime.

## Migration From A Localization Table

If an existing project has a normal `Localization` table:

1. Move that table declaration out of `[[tables]]` and into `[localization].sources`.
2. Change business fields such as `title_key` from `string` to `text`.
3. Add `i18n-binary` or `i18n-json` exports with a `locale`.
4. Replace runtime calls such as `config.localization()` with `SoraI18n::mount` and `SoraI18n::text`.

This is intentionally incompatible. Localization catalogs are runtime assets, not business tables.
