# 多语言

Sora 将翻译文本作为独立的 locale catalog 处理，而不是普通业务表。

业务配置只保存 `text` 类型的文案 key。localization source 表提供这些 key 的各语言翻译。运行时先加载普通配置包，再单独挂载一个或多个语言包。

```text
业务表 -> config bundle
localization sources -> LocaleCatalog -> i18n locale packs
```

## Text Key

需要本地化的字段使用 `text`：

```toml
[[tables.fields]]
name = "title_key"
type = "text"

[[tables.fields]]
name = "body_keys"
type = "list<text>"
```

`text` 保存的是 key，不是实际翻译文本。源数据里应填写 `quest.1001.title`、`ui.confirm` 这类值。Rust 生成代码会暴露为 `TextKey`；动态语言目标可以仍表现为字符串 key。

catalog 校验会扫描业务数据里的所有 `text` 值。key 不存在会直接构建失败。

## Catalog Sources

在 project schema root 声明 localization：

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

每个 source 是宽表。默认 key 列名是 `key`：

| key | zh_cn | en_us | note |
| --- | --- | --- | --- |
| `ui.confirm` | 确认 | Confirm | button label |
| `quest.1001.title` | 第一章 | Chapter One | quest title |

`locales` 里声明的语言列会进入语言包。其它列，例如 `note`，只作为编辑和诊断元数据，不进入运行时包。

规则：

| 规则 | 行为 |
| --- | --- |
| `source.name` | 必须是 ASCII 标识符风格。它用于诊断和组织，不作为 key 前缀。 |
| `key` 值 | 可以使用 `quest.1001.title` 这类 dotted key。 |
| 多个 source | 合并成一个逻辑 catalog。 |
| 重复 key | 构建失败。key 在所有 source 里全局唯一。 |
| 缺少 locale 列 | 构建失败。 |
| `strict = true` 时翻译为空 | 构建失败。 |

如果 key 列不叫 `key`，可以在 source 上指定：

```toml
[[localization.sources]]
name = "ui"
file = "Core.xlsx"
sheet = "UILocalization"
key = "id"
```

## 导出语言包

普通导出格式（`binary`、`json`、`cbor`、`sora-protobuf`、`proto`）只包含业务数据和 text key，不包含实际翻译文本。

在 build manifest 里添加 i18n 导出：

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

`i18n-binary` 面向生产语言包。`i18n-json` 面向检查、外包翻译交付和测试。

## 运行时挂载

生成运行时会分开加载配置包和语言包。Rust 示例：

```rust
let config = SoraConfig::from_bytes(config_bytes)?;
let pack = generated::runtime::LocalePack::from_bytes(locale_bytes)?;

let mut i18n = generated::SoraI18n::new();
i18n.mount(pack)?;
i18n.set_locale("zh_cn")?;

let quest = config.quest().get(&1001).unwrap();
let title = i18n.text(&quest.title_key)?;
```

挂载时会校验：

| 校验 | 作用 |
| --- | --- |
| `schema_fingerprint` | 防止加载另一个 schema 生成的语言包。 |
| locale 声明 | 拒绝 `[localization].locales` 未声明的语言包。 |
| 已挂载语言 | `set_locale` 只能切到已经 mount 的语言。 |

业务代码不感知 key 来自哪张 source 表，只把 `TextKey` 交给 i18n runtime。
