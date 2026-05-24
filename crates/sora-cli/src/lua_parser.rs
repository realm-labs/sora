use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use mlua::{Function, Lua, LuaOptions, StdLib, Table, Value as LuaValue};
use serde::Deserialize;
use sora_data::model::Value;
use sora_diagnostics::SoraError;
use sora_input::{
    cell::{CellContext, CellLocation, CellValue},
    parser::{CellParser, ParserRegistry as CellParserRegistry},
};
use sora_ir::{
    model::TypeIr,
    parser::{ParserRegistry as SchemaParserRegistry, ParserValidator},
};
use sora_schema::model::ParserSchema;

pub struct ParserRegistries {
    pub schema: SchemaParserRegistry,
    pub cell: CellParserRegistry,
}

pub fn load_parser_registries(
    project: Option<&Path>,
    cli_paths: &[PathBuf],
) -> Result<ParserRegistries> {
    let mut schema = SchemaParserRegistry::builtin();
    let mut cell = CellParserRegistry::builtin();
    let paths = parser_script_paths(project, cli_paths)?;
    if paths.is_empty() {
        return Ok(ParserRegistries { schema, cell });
    }

    let parsers = Arc::new(LuaParserSet::load(&paths)?);
    for kind in parsers.kinds() {
        if schema.contains(kind) || cell.contains(kind) {
            bail!("Lua parser `{kind}` cannot override a built-in parser");
        }
    }
    for kind in parsers.kinds() {
        schema.register(LuaParserValidator {
            parsers: Arc::clone(&parsers),
            kind: kind.to_owned(),
        });
        cell.register(LuaCellParser {
            parsers: Arc::clone(&parsers),
            kind: kind.to_owned(),
        });
    }

    Ok(ParserRegistries { schema, cell })
}

fn parser_script_paths(project: Option<&Path>, cli_paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if let Some(project) = project {
        let config = ProjectParserDocument::load(project)?;
        let project_dir = project.parent().unwrap_or_else(|| Path::new("."));
        paths.extend(config.parsers.scripts.into_iter().map(|path| {
            if path.is_absolute() {
                path
            } else {
                project_dir.join(path)
            }
        }));
    }
    paths.extend(cli_paths.iter().cloned());
    Ok(paths)
}

#[derive(Debug, Default, Deserialize)]
struct ProjectParserDocument {
    #[serde(default)]
    parsers: ProjectParserConfig,
}

impl ProjectParserDocument {
    fn load(path: &Path) -> Result<Self> {
        sora_config_format::load_document(path).with_context(|| {
            format!(
                "failed to load parser config from project `{}`",
                path.display()
            )
        })
    }
}

#[derive(Debug, Default, Deserialize)]
struct ProjectParserConfig {
    #[serde(default)]
    scripts: Vec<PathBuf>,
}

#[derive(Debug)]
struct LuaParserSet {
    scripts: Vec<LuaParserScript>,
    definitions: BTreeMap<String, LuaParserDefinition>,
}

impl LuaParserSet {
    fn load(paths: &[PathBuf]) -> Result<Self> {
        let mut scripts = Vec::new();
        let mut definitions = BTreeMap::new();
        for path in paths {
            let source = fs::read_to_string(path).with_context(|| {
                format!("failed to read Lua parser script `{}`", path.display())
            })?;
            let script_index = scripts.len();
            let discovered = discover_script_parsers(path, &source)?;
            scripts.push(LuaParserScript {
                path: path.clone(),
                source,
            });
            for mut definition in discovered {
                if definitions.contains_key(&definition.kind) {
                    bail!("duplicate Lua parser `{}`", definition.kind);
                }
                definition.script_index = script_index;
                definitions.insert(definition.kind.clone(), definition);
            }
        }

        if definitions.is_empty() {
            bail!("Lua parser scripts did not register any parsers");
        }

        Ok(Self {
            scripts,
            definitions,
        })
    }

    fn kinds(&self) -> impl Iterator<Item = &str> {
        self.definitions.keys().map(String::as_str)
    }

    fn definition(&self, kind: &str) -> &LuaParserDefinition {
        self.definitions
            .get(kind)
            .expect("registered Lua parser should have definition")
    }

    fn with_parser<R>(
        &self,
        kind: &str,
        callback: impl FnOnce(&Lua, Table) -> Result<R>,
    ) -> Result<R> {
        let definition = self.definition(kind);
        let script = &self.scripts[definition.script_index];
        let lua = parser_lua()?;
        let root: Table = lua
            .load(&script.source)
            .set_name(script.path.display().to_string())
            .eval()
            .with_context(|| {
                format!(
                    "failed to evaluate Lua parser script `{}`",
                    script.path.display()
                )
            })?;
        let parsers: Table = root.get("parsers").with_context(|| {
            format!(
                "Lua parser script `{}` must return a table with `parsers`",
                script.path.display()
            )
        })?;
        let parser: Table = parsers.get(kind).with_context(|| {
            format!(
                "Lua parser `{kind}` disappeared while loading `{}`",
                script.path.display()
            )
        })?;
        callback(&lua, parser)
    }
}

#[derive(Debug)]
struct LuaParserScript {
    path: PathBuf,
    source: String,
}

#[derive(Debug, Clone)]
struct LuaParserDefinition {
    kind: String,
    script_index: usize,
    options: BTreeSet<String>,
    has_validate: bool,
}

fn discover_script_parsers(path: &Path, source: &str) -> Result<Vec<LuaParserDefinition>> {
    let lua = parser_lua()?;
    let root: Table = lua
        .load(source)
        .set_name(path.display().to_string())
        .eval()
        .with_context(|| format!("failed to evaluate Lua parser script `{}`", path.display()))?;
    let parsers: Table = root.get("parsers").with_context(|| {
        format!(
            "Lua parser script `{}` must return a table with `parsers`",
            path.display()
        )
    })?;

    let mut definitions = Vec::new();
    for pair in parsers.pairs::<String, Table>() {
        let (kind, parser) = pair.with_context(|| {
            format!(
                "Lua parser script `{}` has an invalid parser entry",
                path.display()
            )
        })?;
        if kind.trim().is_empty() {
            bail!(
                "Lua parser script `{}` declares an empty parser kind",
                path.display()
            );
        }
        let _: Function = parser.get("parse").with_context(|| {
            format!(
                "Lua parser `{kind}` in `{}` must define `parse(cell, ctx)`",
                path.display()
            )
        })?;
        let has_validate = parser.get::<Option<Function>>("validate")?.is_some();
        definitions.push(LuaParserDefinition {
            kind,
            script_index: 0,
            options: parser_options(&parser)?,
            has_validate,
        });
    }

    Ok(definitions)
}

fn parser_options(parser: &Table) -> Result<BTreeSet<String>> {
    let Some(options) = parser.get::<Option<Table>>("options")? else {
        return Ok(BTreeSet::new());
    };
    let mut allowed = BTreeSet::new();
    for item in options.sequence_values::<String>() {
        let option = item?;
        if option.trim().is_empty() {
            bail!("Lua parser declares an empty option name");
        }
        allowed.insert(option);
    }
    Ok(allowed)
}

fn parser_lua() -> Result<Lua> {
    Lua::new_with(
        StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
        LuaOptions::default(),
    )
    .context("failed to create Lua parser runtime")
}

struct LuaParserValidator {
    parsers: Arc<LuaParserSet>,
    kind: String,
}

impl ParserValidator for LuaParserValidator {
    fn kind(&self) -> &str {
        &self.kind
    }

    fn validate(
        &self,
        field_name: &str,
        ty: &TypeIr,
        parser: &ParserSchema,
    ) -> sora_diagnostics::Result<()> {
        let definition = self.parsers.definition(&self.kind);
        for (key, value) in &parser.options {
            if !definition.options.contains(key) {
                return Err(SoraError::InvalidSchema(format!(
                    "field `{field_name}` declares unsupported option `{key}` for parser `{}`",
                    self.kind
                )));
            }
            if value.trim().is_empty() {
                return Err(SoraError::InvalidSchema(format!(
                    "field `{field_name}` declares empty `parser.{key}`"
                )));
            }
        }
        if !definition.has_validate {
            return Ok(());
        }

        self.parsers
            .with_parser(&self.kind, |lua, parser_table| {
                let validate: Function = parser_table.get("validate")?;
                let field = lua.create_table()?;
                field.set("name", field_name)?;
                field.set("type", ty.to_string())?;
                field.set("options", lua_options_table(lua, &parser.options)?)?;
                validate.call::<()>(field)?;
                Ok(())
            })
            .map_err(|error| {
                SoraError::InvalidSchema(format!(
                    "field `{field_name}` failed Lua parser `{}` validation: {error:#}",
                    self.kind
                ))
            })
    }
}

struct LuaCellParser {
    parsers: Arc<LuaParserSet>,
    kind: String,
}

impl CellParser for LuaCellParser {
    fn kind(&self) -> &str {
        &self.kind
    }

    fn parse(
        &self,
        cell: &CellValue<'_>,
        ty: &TypeIr,
        context: &CellContext<'_>,
        _registry: &CellParserRegistry,
    ) -> sora_diagnostics::Result<Value> {
        self.parsers
            .with_parser(&self.kind, |lua, parser_table| {
                let parse: Function = parser_table.get("parse")?;
                let result: LuaValue = parse.call((
                    lua_cell_table(lua, cell)?,
                    lua_context_table(lua, ty, context)?,
                ))?;
                lua_value_to_data_value(result)
            })
            .map_err(|error| context.error(format!("Lua parser `{}` failed: {error:#}", self.kind)))
    }
}

fn lua_cell_table(lua: &Lua, cell: &CellValue<'_>) -> Result<Table> {
    let table = lua.create_table()?;
    match cell {
        CellValue::Empty => {
            table.set("kind", "empty")?;
        }
        CellValue::Text(value) => {
            table.set("kind", "text")?;
            table.set("value", value.as_ref())?;
        }
        CellValue::Integer(value) => {
            table.set("kind", "integer")?;
            table.set("value", *value)?;
        }
        CellValue::Float(value) => {
            table.set("kind", "float")?;
            table.set("value", *value)?;
        }
        CellValue::Bool(value) => {
            table.set("kind", "bool")?;
            table.set("value", *value)?;
        }
        CellValue::Error(value) => {
            table.set("kind", "error")?;
            table.set("value", value.as_ref())?;
        }
    }
    table.set("text", cell.display_text())?;
    Ok(table)
}

fn lua_context_table(lua: &Lua, ty: &TypeIr, context: &CellContext<'_>) -> Result<Table> {
    let table = lua.create_table()?;
    table.set("field", context.field)?;
    table.set("type", ty.to_string())?;
    table.set("path", context.path.display().to_string())?;
    if let Some(parser) = context.parser {
        table.set("options", lua_options_table(lua, &parser.options)?)?;
    } else {
        table.set("options", lua.create_table()?)?;
    }
    match context.location {
        CellLocation::Default => {
            table.set("location", "default")?;
        }
        CellLocation::Csv { row, column } => {
            table.set("location", "csv")?;
            table.set("row", row)?;
            table.set("column", column)?;
        }
        CellLocation::Worksheet { sheet, row, column } => {
            table.set("location", "worksheet")?;
            table.set("sheet", sheet)?;
            table.set("row", row)?;
            table.set("column", column)?;
        }
    }
    Ok(table)
}

fn lua_options_table(lua: &Lua, options: &BTreeMap<String, String>) -> Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in options {
        table.set(key.as_str(), value.as_str())?;
    }
    Ok(table)
}

fn lua_value_to_data_value(value: LuaValue) -> Result<Value> {
    Ok(match value {
        LuaValue::Nil => Value::Null,
        LuaValue::Boolean(value) => Value::Bool(value),
        LuaValue::Integer(value) => Value::Integer(value),
        LuaValue::Number(value) => Value::Float(value),
        LuaValue::String(value) => Value::String(value.to_str()?.to_owned()),
        LuaValue::Table(table) => lua_table_to_data_value(table)?,
        other => bail!(
            "unsupported Lua parser return value `{}`",
            other.type_name()
        ),
    })
}

fn lua_table_to_data_value(table: Table) -> Result<Value> {
    let len = table.raw_len();
    if len > 0 {
        let mut values = Vec::with_capacity(len);
        for index in 1..=len {
            values.push(lua_value_to_data_value(table.raw_get(index)?)?);
        }
        return Ok(Value::List(values));
    }

    let mut values = BTreeMap::new();
    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair?;
        let key = match key {
            LuaValue::String(value) => value.to_str()?.to_owned(),
            other => bail!(
                "Lua object return key must be string, got `{}`",
                other.type_name()
            ),
        };
        values.insert(key, lua_value_to_data_value(value)?);
    }
    Ok(Value::Object(values))
}
