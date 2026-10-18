use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Result as IoResult,
    path::{Component, Path, PathBuf},
    result::Result as StdResult,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use anyhow::{Context, Error as AnyhowError, Result, bail};
use mlua::{
    Error as LuaError, Function, HookTriggers, Lua, LuaOptions, Result as LuaResult, StdLib, Table,
    Value as LuaValue, VmState,
};
use serde_json::{Value as JsonValue, from_str};
use sha2::{Digest, Sha256};
use sora_data::model::{RowData, TableData, Value};
use sora_diagnostics::{Result as DiagnosticResult, SoraError};
use sora_execution::ExecutionContext;
use sora_input::source::{
    DataSourceDependency, DataSourceLoader, DataSourceRegistry, DataSourceRequest,
};

use crate::scripts::ScriptSource;

const MAX_ROWS: usize = 100_000;
const MAX_VALUE_DEPTH: usize = 64;
const MAX_FILES: usize = 4_096;
const MAX_READ_BYTES: usize = 64 * 1024 * 1024;
const MAX_LIST_ENTRIES: usize = 100_000;
const MAX_INSTRUCTIONS: usize = 10_000_000;
const HOOK_INTERVAL: u32 = 10_000;
const MAX_MEMORY_BYTES: usize = 128 * 1024 * 1024;

type DependencyScope = (String, PathBuf);
type DependencyTracker = BTreeMap<DependencyScope, BTreeMap<PathBuf, String>>;

pub(crate) fn register_lua_source_loaders_from_sources(
    registry: &mut DataSourceRegistry,
    scripts: &[ScriptSource],
) -> Result<()> {
    let mut declared_formats = BTreeSet::new();
    for source in scripts {
        let declared_path = source.path();
        let digest = sha256(source.source().as_bytes());
        let definitions = discover_definitions(declared_path, source.source())?;
        if definitions.is_empty() {
            bail!(
                "Lua source loader script `{}` did not register any source loaders",
                declared_path.display()
            );
        }
        for definition in definitions {
            if !declared_formats.insert(definition.format.clone()) {
                bail!("duplicate Lua source loader format `{}`", definition.format);
            }
            registry
                .try_register(LuaSourceLoader {
                    format: definition.format,
                    extensions: definition.extensions,
                    script_label: declared_path.to_path_buf(),
                    script_source: source.source().to_owned(),
                    script_digest: digest.clone(),
                    dependencies: Arc::new(Mutex::new(BTreeMap::new())),
                })
                .map_err(AnyhowError::from)?;
        }
    }
    Ok(())
}

#[derive(Debug)]
struct LoaderDefinition {
    format: String,
    extensions: Vec<String>,
}

fn discover_definitions(path: &Path, source: &str) -> Result<Vec<LoaderDefinition>> {
    let lua = source_lua(None)?;
    let root: Table = lua
        .load(source)
        .set_name(normalized_path(path))
        .eval()
        .map_err(|error| AnyhowError::from(map_lua_error(path, error)))?;
    let loaders: Table = root.get("source_loaders").with_context(|| {
        format!(
            "Lua source loader script `{}` must return a table with `source_loaders`",
            path.display()
        )
    })?;
    let mut definitions = Vec::new();
    for pair in loaders.pairs::<LuaValue, Table>() {
        let (key, definition) = pair.map_err(|error| map_lua_error(path, error))?;
        let format = match key {
            LuaValue::String(value) => value.to_str()?.to_owned(),
            other => bail!(
                "Lua source loader format name must be a string, got `{}`",
                other.type_name()
            ),
        };
        validate_identifier("format", &format)?;
        let _: Function = definition.get("load").with_context(|| {
            format!("Lua source loader `{format}` must declare a `load` function")
        })?;
        let extensions = definition
            .get::<Option<Table>>("extensions")?
            .map(|values| {
                values
                    .sequence_values::<String>()
                    .map(|value| {
                        let value = value?;
                        let value = value.trim_start_matches('.').to_ascii_lowercase();
                        validate_identifier("extension", &value)?;
                        Ok(value)
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();
        if extensions.iter().collect::<BTreeSet<_>>().len() != extensions.len() {
            bail!("Lua source loader `{format}` declares duplicate extensions");
        }
        definitions.push(LoaderDefinition { format, extensions });
    }
    definitions.sort_by(|left, right| left.format.cmp(&right.format));
    Ok(definitions)
}

fn validate_identifier(kind: &str, value: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        bail!("Lua source loader {kind} `{value}` is not a portable identifier");
    }
    Ok(())
}

struct LuaSourceLoader {
    format: String,
    extensions: Vec<String>,
    script_label: PathBuf,
    script_source: String,
    script_digest: String,
    dependencies: Arc<Mutex<DependencyTracker>>,
}

impl DataSourceLoader for LuaSourceLoader {
    fn format_name(&self) -> &str {
        &self.format
    }

    fn extensions(&self) -> Vec<&str> {
        self.extensions.iter().map(String::as_str).collect()
    }

    fn dependencies(&self) -> Vec<DataSourceDependency> {
        let mut dependencies = vec![DataSourceDependency {
            path: self.script_label.clone(),
            digest: self.script_digest.clone(),
        }];
        let reads = self
            .dependencies
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        dependencies.extend(reads.values().flat_map(|scope| {
            scope.iter().map(|(path, digest)| DataSourceDependency {
                path: path.clone(),
                digest: digest.clone(),
            })
        }));
        dependencies
    }

    fn load_table(&self, request: DataSourceRequest<'_>) -> DiagnosticResult<TableData> {
        self.load(request)
    }
}

impl LuaSourceLoader {
    fn load(&self, request: DataSourceRequest<'_>) -> DiagnosticResult<TableData> {
        if request.execution.is_cancelled() {
            return Err(SoraError::OperationCancelled {
                operation: "Lua source load",
            });
        }
        validate_relative_path(&request.source.file)
            .map_err(|message| source_error(Path::new(&request.source.file), message))?;
        let dependency_scope = (
            request.table.name.clone(),
            PathBuf::from(&request.source.file),
        );
        self.dependencies
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(dependency_scope.clone(), BTreeMap::new());
        let root = request.path.canonicalize().map_err(|error| {
            source_error(
                Path::new(&request.source.file),
                format!("failed to resolve source root: {error}"),
            )
        })?;
        let root_is_file = root.is_file();
        if !root_is_file && !root.is_dir() {
            return Err(source_error(
                Path::new(&request.source.file),
                "source root must be a file or directory".to_owned(),
            ));
        }

        let lua = source_lua(Some(request.execution))
            .map_err(|error| source_error(&self.script_label, error.to_string()))?;
        let state = Arc::new(Mutex::new(FileCapabilityState {
            root,
            root_is_file,
            source_label: PathBuf::from(&request.source.file),
            files: 0,
            bytes: 0,
            listed_entries: 0,
            dependencies: Arc::clone(&self.dependencies),
            dependency_scope,
        }));
        let context = create_host_context(&lua, state)
            .map_err(|error| map_lua_error(&self.script_label, error))?;
        let source = lua
            .create_table()
            .map_err(|error| map_lua_error(&self.script_label, error))?;
        source
            .set("table", request.table.name.as_str())
            .and_then(|()| source.set("format", self.format.as_str()))
            .and_then(|()| source.set("path", request.source.file.as_str()))
            .map_err(|error| map_lua_error(&self.script_label, error))?;

        let result = (|| -> LuaResult<Table> {
            let root: Table = lua
                .load(&self.script_source)
                .set_name(normalized_path(&self.script_label))
                .eval()?;
            let loaders: Table = root.get("source_loaders")?;
            let definition: Table = loaders.get(self.format.as_str())?;
            let load: Function = definition.get("load")?;
            load.call((source, context))
        })()
        .map_err(|error| map_lua_error(&self.script_label, error))?;

        let rows: Table = result.get("rows").map_err(|error| {
            source_error(
                &self.script_label,
                format!(
                    "loader `{}` must return `{{ rows = {{ ... }} }}`: {error}",
                    self.format
                ),
            )
        })?;
        let row_count = validate_sequence_table(&rows).map_err(|message| {
            source_error(
                &self.script_label,
                format!("loader `{}` rows: {message}", self.format),
            )
        })?;
        if row_count > MAX_ROWS {
            return Err(source_error(
                &self.script_label,
                format!(
                    "loader `{}` returned {row_count} rows; limit is {MAX_ROWS}",
                    self.format
                ),
            ));
        }
        let mut out = Vec::with_capacity(row_count);
        for index in 1..=row_count {
            let row: Table = rows
                .raw_get(index)
                .map_err(|error| map_lua_error(&self.script_label, error))?;
            out.push(RowData {
                values: lua_object_to_row(row, 0).map_err(|message| {
                    source_error(
                        &self.script_label,
                        format!("loader `{}` row {index}: {message}", self.format),
                    )
                })?,
            });
        }
        Ok(TableData {
            name: request.table.name.clone(),
            rows: out,
        })
    }
}

fn source_lua(execution: Option<&ExecutionContext>) -> Result<Lua> {
    let lua = Lua::new_with(
        StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
        LuaOptions::default(),
    )
    .context("failed to create Lua source loader runtime")?;
    let globals = lua.globals();
    for name in [
        "dofile", "loadfile", "require", "io", "os", "package", "debug", "print", "warn",
    ] {
        globals.set(name, LuaValue::Nil)?;
    }
    if let Ok(math) = globals.get::<Table>("math") {
        math.set("random", LuaValue::Nil)?;
        math.set("randomseed", LuaValue::Nil)?;
    }
    lua.set_memory_limit(MAX_MEMORY_BYTES)
        .context("failed to set Lua source loader memory limit")?;
    let execution = execution.cloned();
    let instructions = AtomicUsize::new(0);
    lua.set_hook(
        HookTriggers::new().every_nth_instruction(HOOK_INTERVAL),
        move |_, _| {
            if execution.as_ref().is_some_and(|value| value.is_cancelled()) {
                return Err(LuaError::runtime("__sora_cancelled"));
            }
            let instructions = instructions.fetch_add(HOOK_INTERVAL as usize, Ordering::Relaxed)
                + HOOK_INTERVAL as usize;
            if instructions > MAX_INSTRUCTIONS {
                return Err(LuaError::runtime("__sora_instruction_budget"));
            }
            Ok(VmState::Continue)
        },
    )?;
    Ok(lua)
}

struct FileCapabilityState {
    root: PathBuf,
    root_is_file: bool,
    source_label: PathBuf,
    files: usize,
    bytes: usize,
    listed_entries: usize,
    dependencies: Arc<Mutex<DependencyTracker>>,
    dependency_scope: DependencyScope,
}

fn create_host_context(lua: &Lua, state: Arc<Mutex<FileCapabilityState>>) -> LuaResult<Table> {
    let context = lua.create_table()?;
    let null = create_null_sentinel(lua)?;
    context.set("null", null.clone())?;
    let text_state = Arc::clone(&state);
    context.set(
        "read_text",
        lua.create_function(move |_, relative: String| {
            let bytes = read_capability_file(&text_state, &relative)?;
            String::from_utf8(bytes).map_err(|_| {
                LuaError::external("source file is not valid UTF-8; use ctx.read_bytes")
            })
        })?,
    )?;
    let bytes_state = Arc::clone(&state);
    context.set(
        "read_bytes",
        lua.create_function(move |lua, relative: String| {
            let bytes = read_capability_file(&bytes_state, &relative)?;
            lua.create_string(bytes)
        })?,
    )?;
    let list_state = Arc::clone(&state);
    context.set(
        "list",
        lua.create_function(move |lua, relative: Option<String>| {
            list_capability_directory(lua, &list_state, relative.as_deref().unwrap_or("."))
        })?,
    )?;
    context.set(
        "json_decode",
        lua.create_function(move |lua, text: String| {
            let value: JsonValue = from_str(&text)
                .map_err(|error| LuaError::external(format!("invalid JSON: {error}")))?;
            json_to_lua(lua, &value, &null, 0)
        })?,
    )?;
    context.set(
        "array",
        lua.create_function(|lua, table: Table| {
            mark_array(lua, &table)?;
            Ok(table)
        })?,
    )?;
    context.set(
        "error",
        lua.create_function(|_, diagnostic: Table| -> LuaResult<()> {
            let path = diagnostic
                .get::<Option<String>>("path")?
                .unwrap_or_default();
            validate_relative_path(&path).map_err(LuaError::external)?;
            let message = diagnostic.get::<String>("message")?;
            Err(LuaError::external(HostDiagnostic {
                path: PathBuf::from(path),
                line: diagnostic.get("line")?,
                column: diagnostic.get("column")?,
                field: diagnostic.get("field")?,
                message,
            }))
        })?,
    )?;
    Ok(context)
}

fn read_capability_file(
    state: &Arc<Mutex<FileCapabilityState>>,
    relative: &str,
) -> LuaResult<Vec<u8>> {
    let mut state = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (path, label) = resolve_capability_path(&state, relative)?;
    if !path.is_file() {
        return Err(LuaError::external(format!(
            "source path `{}` is not a file",
            normalized_path(&label)
        )));
    }
    if state.files >= MAX_FILES {
        return Err(LuaError::external(format!(
            "source loader file limit exceeded ({MAX_FILES})"
        )));
    }
    let bytes = fs::read(&path).map_err(|error| {
        LuaError::external(format!(
            "failed to read source file `{}`: {error}",
            normalized_path(&label)
        ))
    })?;
    if state.bytes.saturating_add(bytes.len()) > MAX_READ_BYTES {
        return Err(LuaError::external(format!(
            "source loader read byte limit exceeded ({MAX_READ_BYTES})"
        )));
    }
    state.files += 1;
    state.bytes += bytes.len();
    state
        .dependencies
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .entry(state.dependency_scope.clone())
        .or_default()
        .insert(label, sha256(&bytes));
    Ok(bytes)
}

fn list_capability_directory(
    lua: &Lua,
    state: &Arc<Mutex<FileCapabilityState>>,
    relative: &str,
) -> LuaResult<Table> {
    let mut state = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (directory, _) = resolve_capability_path(&state, relative)?;
    if !directory.is_dir() {
        return Err(LuaError::external("source path is not a directory"));
    }
    let mut entries = fs::read_dir(&directory)
        .map_err(|error| {
            LuaError::external(format!("failed to enumerate source directory: {error}"))
        })?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<IoResult<Vec<_>>>()
        .map_err(LuaError::external)?;
    entries.sort_by(|left, right| {
        normalized_path(left.strip_prefix(&state.root).unwrap_or(left)).cmp(&normalized_path(
            right.strip_prefix(&state.root).unwrap_or(right),
        ))
    });
    if state.listed_entries.saturating_add(entries.len()) > MAX_LIST_ENTRIES {
        return Err(LuaError::external(format!(
            "source loader directory entry limit exceeded ({MAX_LIST_ENTRIES})"
        )));
    }
    state.listed_entries += entries.len();
    let output = lua.create_table_with_capacity(entries.len(), 0)?;
    for (index, entry) in entries.into_iter().enumerate() {
        let canonical = entry.canonicalize().map_err(|error| {
            LuaError::external(format!("failed to resolve source directory entry: {error}"))
        })?;
        if !canonical.starts_with(&state.root) {
            return Err(LuaError::external("source symlink escapes the source root"));
        }
        let relative = canonical
            .strip_prefix(&state.root)
            .map_err(LuaError::external)?;
        let item = lua.create_table()?;
        item.set("path", normalized_path(relative))?;
        item.set(
            "name",
            relative
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default(),
        )?;
        item.set(
            "kind",
            if canonical.is_dir() {
                "directory"
            } else {
                "file"
            },
        )?;
        output.raw_set(index + 1, item)?;
    }
    Ok(output)
}

fn resolve_capability_path(
    state: &FileCapabilityState,
    relative: &str,
) -> LuaResult<(PathBuf, PathBuf)> {
    validate_relative_path(relative).map_err(LuaError::external)?;
    let relative = if relative.is_empty() {
        Path::new(".")
    } else {
        Path::new(relative)
    };
    let joined = if state.root_is_file {
        if relative != Path::new(".") {
            return Err(LuaError::external(
                "a file source root only permits `.` as the capability path",
            ));
        }
        state.root.clone()
    } else {
        state.root.join(relative)
    };
    let canonical = joined.canonicalize().map_err(|error| {
        LuaError::external(format!(
            "failed to resolve source path `{relative:?}`: {error}"
        ))
    })?;
    let contained = if state.root_is_file {
        canonical == state.root
    } else {
        canonical.starts_with(&state.root)
    };
    if !contained {
        return Err(LuaError::external("source path escapes the source root"));
    }
    let suffix = if state.root_is_file {
        PathBuf::new()
    } else {
        canonical
            .strip_prefix(&state.root)
            .map(PathBuf::from)
            .map_err(LuaError::external)?
    };
    Ok((canonical, state.source_label.join(suffix)))
}

fn validate_relative_path(value: &str) -> StdResult<(), String> {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err("source capability paths must be relative and cannot contain `..`".to_owned());
    }
    Ok(())
}

fn lua_object_to_row(table: Table, depth: usize) -> StdResult<BTreeMap<String, Value>, String> {
    if depth > MAX_VALUE_DEPTH {
        return Err(format!("value nesting exceeds {MAX_VALUE_DEPTH}"));
    }
    let mut out = BTreeMap::new();
    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair.map_err(|error| error.to_string())?;
        let key = match key {
            LuaValue::String(value) => value
                .to_str()
                .map_err(|_| "object key must be UTF-8".to_owned())?
                .to_owned(),
            other => {
                return Err(format!(
                    "object key must be string, got `{}`",
                    other.type_name()
                ));
            }
        };
        out.insert(key, lua_value_to_data(value, depth + 1)?);
    }
    Ok(out)
}

fn lua_value_to_data(value: LuaValue, depth: usize) -> StdResult<Value, String> {
    if depth > MAX_VALUE_DEPTH {
        return Err(format!("value nesting exceeds {MAX_VALUE_DEPTH}"));
    }
    Ok(match value {
        LuaValue::Nil => Value::Null,
        LuaValue::Boolean(value) => Value::Bool(value),
        LuaValue::Integer(value) => Value::Integer(value),
        LuaValue::Number(value) if value.is_finite() => Value::Float(value),
        LuaValue::Number(_) => return Err("float must be finite".to_owned()),
        LuaValue::String(value) => Value::String(
            value
                .to_str()
                .map_err(|_| "string value must be UTF-8".to_owned())?
                .to_owned(),
        ),
        LuaValue::Table(table) => {
            if is_null_sentinel(&table) {
                return Ok(Value::Null);
            }
            let marked_array = table
                .metatable()
                .and_then(|meta| meta.raw_get::<bool>("__sora_array").ok())
                .unwrap_or(false);
            if table.raw_len() > 0 || marked_array {
                let len = validate_sequence_table(&table)?;
                let mut values = Vec::with_capacity(len);
                for index in 1..=len {
                    values.push(lua_value_to_data(
                        table.raw_get(index).map_err(|error| error.to_string())?,
                        depth + 1,
                    )?);
                }
                Value::List(values)
            } else {
                Value::Object(lua_object_to_row(table, depth + 1)?)
            }
        }
        other => return Err(format!("unsupported Lua value `{}`", other.type_name())),
    })
}

fn validate_sequence_table(table: &Table) -> StdResult<usize, String> {
    let len = table.raw_len();
    let mut entries = 0usize;
    for pair in table.clone().pairs::<LuaValue, LuaValue>() {
        let (key, _) = pair.map_err(|error| error.to_string())?;
        match key {
            LuaValue::Integer(index) if index >= 1 && index as usize <= len => entries += 1,
            other => {
                return Err(format!(
                    "array keys must be contiguous integers starting at 1, got `{}`",
                    other.type_name()
                ));
            }
        }
    }
    if entries != len {
        return Err("array keys must be contiguous integers starting at 1".to_owned());
    }
    Ok(len)
}

fn json_to_lua(lua: &Lua, value: &JsonValue, null: &Table, depth: usize) -> LuaResult<LuaValue> {
    if depth > MAX_VALUE_DEPTH {
        return Err(LuaError::external(format!(
            "JSON nesting exceeds {MAX_VALUE_DEPTH}"
        )));
    }
    Ok(match value {
        JsonValue::Null => LuaValue::Table(null.clone()),
        JsonValue::Bool(value) => LuaValue::Boolean(*value),
        JsonValue::Number(value) => {
            if let Some(value) = value.as_i64() {
                LuaValue::Integer(value)
            } else {
                LuaValue::Number(
                    value
                        .as_f64()
                        .ok_or_else(|| LuaError::external("invalid JSON number"))?,
                )
            }
        }
        JsonValue::String(value) => LuaValue::String(lua.create_string(value)?),
        JsonValue::Array(values) => {
            let table = lua.create_table_with_capacity(values.len(), 0)?;
            for (index, value) in values.iter().enumerate() {
                table.raw_set(index + 1, json_to_lua(lua, value, null, depth + 1)?)?;
            }
            mark_array(lua, &table)?;
            LuaValue::Table(table)
        }
        JsonValue::Object(values) => {
            let table = lua.create_table_with_capacity(0, values.len())?;
            for (key, value) in values {
                table.raw_set(key.as_str(), json_to_lua(lua, value, null, depth + 1)?)?;
            }
            LuaValue::Table(table)
        }
    })
}

fn create_null_sentinel(lua: &Lua) -> LuaResult<Table> {
    let null = lua.create_table()?;
    let meta = lua.create_table()?;
    meta.raw_set("__sora_null", true)?;
    meta.raw_set("__metatable", "sora.null")?;
    null.set_metatable(Some(meta))?;
    Ok(null)
}

fn is_null_sentinel(table: &Table) -> bool {
    table
        .metatable()
        .and_then(|meta| meta.raw_get::<bool>("__sora_null").ok())
        .unwrap_or(false)
}

fn mark_array(lua: &Lua, table: &Table) -> LuaResult<()> {
    let meta = lua.create_table()?;
    meta.raw_set("__sora_array", true)?;
    table.set_metatable(Some(meta))?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
struct HostDiagnostic {
    path: PathBuf,
    line: Option<usize>,
    column: Option<usize>,
    field: Option<String>,
    message: String,
}

fn map_lua_error(script: &Path, error: LuaError) -> SoraError {
    if lua_error_contains(&error, "__sora_cancelled") {
        return SoraError::OperationCancelled {
            operation: "Lua source load",
        };
    }
    if lua_error_contains(&error, "__sora_instruction_budget") {
        return source_error(
            script,
            format!("Lua source loader instruction budget exceeded ({MAX_INSTRUCTIONS})"),
        );
    }
    if let Some(diagnostic) = find_host_diagnostic(&error) {
        return SoraError::SourceLoaderDiagnostic {
            path: diagnostic.path.clone(),
            line: diagnostic.line,
            column: diagnostic.column,
            field: diagnostic.field.clone(),
            message: diagnostic.message.clone(),
        };
    }
    source_error(script, stable_lua_message(&error))
}

fn find_host_diagnostic(error: &LuaError) -> Option<&HostDiagnostic> {
    match error {
        LuaError::ExternalError(error) => error.downcast_ref(),
        LuaError::CallbackError { cause, .. }
        | LuaError::WithContext { cause, .. }
        | LuaError::BadArgument { cause, .. } => find_host_diagnostic(cause),
        _ => None,
    }
}

fn lua_error_contains(error: &LuaError, needle: &str) -> bool {
    match error {
        LuaError::RuntimeError(message) => message.contains(needle),
        LuaError::CallbackError { cause, .. }
        | LuaError::WithContext { cause, .. }
        | LuaError::BadArgument { cause, .. } => lua_error_contains(cause, needle),
        _ => false,
    }
}

fn stable_lua_message(error: &LuaError) -> String {
    match error {
        LuaError::CallbackError { cause, .. }
        | LuaError::WithContext { cause, .. }
        | LuaError::BadArgument { cause, .. } => stable_lua_message(cause),
        LuaError::MemoryError(_) => "Lua source loader memory limit exceeded".to_owned(),
        other => other
            .to_string()
            .lines()
            .next()
            .unwrap_or("Lua source loader failed")
            .to_owned(),
    }
}

fn source_error(path: &Path, message: String) -> SoraError {
    SoraError::SourceLoaderDiagnostic {
        path: path.to_path_buf(),
        line: None,
        column: None,
        field: None,
        message,
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!(
        "sha256:{}",
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn normalized_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir as system_temp_dir;

    use sora_input::parser::ParserRegistry as CellParserRegistry;
    use sora_ir::model::{ConfigIr, TableIr, TableModeIr, TableSourceIr};
    use uuid::Uuid;

    use super::*;

    const SNAPSHOT_SOURCE: &str = r#"
return { source_loaders = { snapshot = {
  extensions = { ".SNAPSHOT" },
  load = function(source, ctx)
    return { rows = { { value = "memory" } } }
  end,
} } }
"#;

    #[test]
    fn source_loader_sources_keep_memory_definitions_execution_and_digest() {
        let label = PathBuf::from("tools/source_loaders.lua");
        let ir = ConfigIr {
            project_id: "snapshot".to_owned(),
            contract_id: "snapshot".to_owned(),
            view: None,
            group_defaults: BTreeMap::new(),
            views: BTreeMap::new(),
            localization: None,
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: vec![TableIr {
                id: "snapshot".to_owned(),
                name: "Snapshot".to_owned(),
                canonical_name: "Snapshot".to_owned(),
                groups: Default::default(),
                mode: TableModeIr::List,
                key: None,
                source: Some(TableSourceIr {
                    format: Some("snapshot".to_owned()),
                    file: "input".to_owned(),
                    sheet: None,
                    sheets: Vec::new(),
                }),
                fields: Vec::new(),
                indexes: Vec::new(),
            }],
        };
        let execution = ExecutionContext::default();
        let parsers = CellParserRegistry::builtin();
        for missing in [false, true] {
            let root = system_temp_dir().join(format!("sora-loader-snapshot-{}", Uuid::new_v4()));
            let path = root.join(&label);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, SNAPSHOT_SOURCE).unwrap();
            let script = ScriptSource::new(label.clone(), SNAPSHOT_SOURCE.to_owned());
            let replacement = SNAPSHOT_SOURCE
                .replace("snapshot", "replaced")
                .replace("memory", "disk");
            if missing {
                fs::remove_file(&path).unwrap();
            } else {
                fs::write(&path, &replacement).unwrap();
            }

            let mut registry = DataSourceRegistry::new();
            register_lua_source_loaders_from_sources(&mut registry, &[script]).unwrap();
            assert_eq!(registry.supported_formats(), ["snapshot"]);
            let loader = registry.get("snapshot").unwrap();
            assert_eq!(loader.extensions(), ["snapshot"]);
            let expected = vec![DataSourceDependency {
                path: label.clone(),
                digest: sha256(SNAPSHOT_SOURCE.as_bytes()),
            }];
            assert_ne!(expected[0].digest, sha256(replacement.as_bytes()));
            assert_eq!(registry.dependencies(), expected);
            if !missing {
                fs::remove_file(&path).unwrap();
            }

            let table = &ir.tables[0];
            let data = loader
                .load_table(DataSourceRequest {
                    ir: &ir,
                    table,
                    source: table.source.as_ref().unwrap(),
                    path: &root,
                    execution: &execution,
                    parser_registry: &parsers,
                })
                .unwrap();
            assert_eq!(data.rows.len(), 1);
            assert_eq!(
                data.rows[0].values["value"],
                Value::String("memory".to_owned())
            );
            assert_eq!(registry.dependencies(), expected);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn sha256_preserves_prefixed_lowercase_hex() {
        assert_eq!(
            sha256(b""),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256(b"abc"),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
