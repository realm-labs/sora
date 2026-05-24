use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use sora_data::model::{ConfigData, TableData};
use sora_diagnostics::{Result, SoraError};
use sora_execution::ExecutionContext;
use sora_input::{
    parser::ParserRegistry,
    source::{
        DataSourceLoader, DataSourceRegistry, DataSourceRequest,
        resolve_table_source_format_with_registry,
    },
    traits::{DataInput, SchemaInput},
};
use sora_input_csv::reader::load_csv_table_data_with_parsers;
use sora_input_structured::reader::{load_json_table_data, load_yaml_table_data};
use sora_input_toml::data::load_table_data_file;
use sora_input_xlsx::reader::load_xlsx_table_data_with_ir_and_parsers;
use sora_ir::model::{ConfigIr, TableIr};
use sora_schema::model::SchemaFile;

#[derive(Clone)]
pub struct MixedProjectInput<S> {
    schema_input: S,
    data_root: PathBuf,
    default_source_format: Option<Arc<str>>,
    source_registry: Arc<DataSourceRegistry>,
    parser_registry: Arc<ParserRegistry>,
}

impl<S> MixedProjectInput<S> {
    pub fn with_source_registry(
        schema_input: S,
        data_root: impl Into<PathBuf>,
        default_source_format: Option<&str>,
        source_registry: Arc<DataSourceRegistry>,
        parser_registry: Arc<ParserRegistry>,
    ) -> Self {
        Self {
            schema_input,
            data_root: data_root.into(),
            default_source_format: default_source_format.map(Arc::from),
            source_registry,
            parser_registry,
        }
    }

    pub fn with_parser_registry(
        schema_input: S,
        data_root: impl Into<PathBuf>,
        default_source_format: Option<&str>,
        parser_registry: Arc<ParserRegistry>,
    ) -> Self {
        Self::with_source_registry(
            schema_input,
            data_root,
            default_source_format,
            Arc::new(builtin_source_registry()),
            parser_registry,
        )
    }
}

impl<S: SchemaInput> SchemaInput for MixedProjectInput<S> {
    fn load_schema(&self) -> Result<SchemaFile> {
        self.schema_input.load_schema()
    }
}

impl<S: SchemaInput> DataInput for MixedProjectInput<S> {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData> {
        load_mixed_config_data_with_registry(
            ir,
            &self.data_root,
            self.default_source_format.as_deref(),
            &self.source_registry,
            &self.parser_registry,
        )
    }

    fn load_data_with_context(
        &self,
        ir: &ConfigIr,
        execution: &ExecutionContext,
    ) -> Result<ConfigData> {
        load_mixed_config_data_with_context(
            ir,
            &self.data_root,
            self.default_source_format.as_deref(),
            &self.source_registry,
            &self.parser_registry,
            execution,
        )
    }
}

pub fn load_mixed_config_data_with_registry(
    ir: &ConfigIr,
    data_root: &Path,
    default_source_format: Option<&str>,
    source_registry: &DataSourceRegistry,
    parser_registry: &ParserRegistry,
) -> Result<ConfigData> {
    load_mixed_config_data_with_context(
        ir,
        data_root,
        default_source_format,
        source_registry,
        parser_registry,
        &ExecutionContext::default(),
    )
}

pub fn load_mixed_config_data_with_context(
    ir: &ConfigIr,
    data_root: &Path,
    default_source_format: Option<&str>,
    source_registry: &DataSourceRegistry,
    parser_registry: &ParserRegistry,
    execution: &ExecutionContext,
) -> Result<ConfigData> {
    let mut tables = Vec::with_capacity(ir.tables.len());
    for table in &ir.tables {
        tables.push(load_mixed_table_data(
            ir,
            table,
            data_root,
            default_source_format,
            source_registry,
            parser_registry,
            execution,
        )?);
    }
    Ok(ConfigData { tables })
}

fn load_mixed_table_data(
    ir: &ConfigIr,
    table: &TableIr,
    data_root: &Path,
    default_source_format: Option<&str>,
    source_registry: &DataSourceRegistry,
    parser_registry: &ParserRegistry,
    execution: &ExecutionContext,
) -> Result<TableData> {
    let source = table
        .source
        .as_ref()
        .ok_or_else(|| SoraError::MissingTableSource {
            table: table.name.clone(),
        })?;
    let path = data_root.join(&source.file);

    let format =
        resolve_table_source_format_with_registry(table, default_source_format, source_registry)?;
    let loader = source_registry
        .get(format)
        .ok_or_else(|| SoraError::InvalidSchema(format!("missing source loader `{format}`")))?;
    loader.load_table(DataSourceRequest {
        ir,
        table,
        source,
        path: &path,
        execution,
        parser_registry,
    })
}

pub fn builtin_source_registry() -> DataSourceRegistry {
    let mut registry = DataSourceRegistry::new();
    registry.register(CsvSourceLoader);
    registry.register(JsonSourceLoader);
    registry.register(TomlSourceLoader);
    registry.register(XlsxSourceLoader);
    registry.register(YamlSourceLoader);
    registry
}

struct CsvSourceLoader;

impl DataSourceLoader for CsvSourceLoader {
    fn format_name(&self) -> &'static str {
        "csv"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["csv"]
    }

    fn load_table(&self, request: DataSourceRequest<'_>) -> Result<TableData> {
        load_csv_table_data_with_parsers(
            request.ir,
            request.table,
            request.path,
            request.parser_registry,
        )
    }
}

struct TomlSourceLoader;

impl DataSourceLoader for TomlSourceLoader {
    fn format_name(&self) -> &'static str {
        "toml"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["toml"]
    }

    fn load_table(&self, request: DataSourceRequest<'_>) -> Result<TableData> {
        load_table_data_file(&request.table.name, request.path)
    }
}

struct JsonSourceLoader;

impl DataSourceLoader for JsonSourceLoader {
    fn format_name(&self) -> &'static str {
        "json"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["json"]
    }

    fn load_table(&self, request: DataSourceRequest<'_>) -> Result<TableData> {
        load_json_table_data(&request.table.name, request.path)
    }
}

struct YamlSourceLoader;

impl DataSourceLoader for YamlSourceLoader {
    fn format_name(&self) -> &'static str {
        "yaml"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["yaml", "yml"]
    }

    fn load_table(&self, request: DataSourceRequest<'_>) -> Result<TableData> {
        load_yaml_table_data(&request.table.name, request.path)
    }
}

struct XlsxSourceLoader;

impl DataSourceLoader for XlsxSourceLoader {
    fn format_name(&self) -> &'static str {
        "xlsx"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["xlsx"]
    }

    fn load_table(&self, request: DataSourceRequest<'_>) -> Result<TableData> {
        let sheet = request
            .source
            .sheet
            .as_deref()
            .unwrap_or(&request.table.name);
        let _ = request.execution;
        load_xlsx_table_data_with_ir_and_parsers(
            request.ir,
            request.table,
            request.path,
            sheet,
            request.parser_registry,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use sora_data::model::{RowData, Value};
    use sora_ir::model::{FieldIr, ScopeIr, TableModeIr, TableSourceIr, TypeIr};

    use super::*;

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn mixed_source_loads_tables_through_registered_loader() {
        let mut registry = DataSourceRegistry::new();
        registry.register(FakeSourceLoader);
        let ir = ConfigIr {
            package: "game".to_owned(),
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: vec![TableIr {
                name: "Item".to_owned(),
                scope: ScopeIr::default(),
                mode: TableModeIr::Map,
                key: Some("id".to_owned()),
                source: Some(TableSourceIr {
                    format: None,
                    file: "items.fake".to_owned(),
                    sheet: None,
                }),
                fields: vec![FieldIr {
                    name: "id".to_owned(),
                    ty: TypeIr::I32,
                    scope: ScopeIr::default(),
                    key: true,
                    comment: None,
                    default: None,
                    range: None,
                    length: None,
                    parser: None,
                    derived_from: None,
                }],
                indexes: Vec::new(),
            }],
        };

        let parser_registry = ParserRegistry::builtin();
        let data = load_mixed_config_data_with_registry(
            &ir,
            Path::new("data"),
            None,
            &registry,
            &parser_registry,
        )
        .unwrap();

        assert_eq!(data.tables[0].name, "Item");
        assert_eq!(data.tables[0].rows[0].values["id"], Value::Integer(1001));
    }

    #[test]
    fn builtin_source_loads_json_table_file() {
        let base = temp_dir();
        let data_root = base.join("data");
        fs::create_dir_all(&data_root).unwrap();
        fs::write(
            data_root.join("items.json"),
            r#"[{"id": 1001, "name": "Sword"}]"#,
        )
        .unwrap();
        let ir = item_ir("items.json", None);
        let parser_registry = ParserRegistry::builtin();

        let data = load_mixed_config_data_with_registry(
            &ir,
            &data_root,
            None,
            &builtin_source_registry(),
            &parser_registry,
        )
        .unwrap();

        assert_eq!(data.tables[0].rows[0].values["id"], Value::Integer(1001));
        assert_eq!(
            data.tables[0].rows[0].values["name"],
            Value::String("Sword".to_owned())
        );

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn builtin_source_loads_yaml_directory() {
        let base = temp_dir();
        let data_root = base.join("data");
        let item_dir = data_root.join("items");
        fs::create_dir_all(&item_dir).unwrap();
        fs::write(item_dir.join("1001.yaml"), "id: 1001\nname: Sword\n").unwrap();
        fs::write(item_dir.join("1002.yml"), "id: 1002\nname: Potion\n").unwrap();
        let ir = item_ir("items", Some("yaml"));
        let parser_registry = ParserRegistry::builtin();

        let data = load_mixed_config_data_with_registry(
            &ir,
            &data_root,
            None,
            &builtin_source_registry(),
            &parser_registry,
        )
        .unwrap();

        assert_eq!(data.tables[0].rows.len(), 2);
        assert_eq!(data.tables[0].rows[0].values["id"], Value::Integer(1001));
        assert_eq!(data.tables[0].rows[1].values["id"], Value::Integer(1002));

        let _ = fs::remove_dir_all(base);
    }

    fn item_ir(file: &str, format: Option<&str>) -> ConfigIr {
        ConfigIr {
            package: "game".to_owned(),
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: vec![TableIr {
                name: "Item".to_owned(),
                scope: ScopeIr::default(),
                mode: TableModeIr::Map,
                key: Some("id".to_owned()),
                source: Some(TableSourceIr {
                    format: format.map(str::to_owned),
                    file: file.to_owned(),
                    sheet: None,
                }),
                fields: vec![
                    FieldIr {
                        name: "id".to_owned(),
                        ty: TypeIr::I32,
                        scope: ScopeIr::default(),
                        key: true,
                        comment: None,
                        default: None,
                        range: None,
                        length: None,
                        parser: None,
                        derived_from: None,
                    },
                    FieldIr {
                        name: "name".to_owned(),
                        ty: TypeIr::String,
                        scope: ScopeIr::default(),
                        key: false,
                        comment: None,
                        default: None,
                        range: None,
                        length: None,
                        parser: None,
                        derived_from: None,
                    },
                ],
                indexes: Vec::new(),
            }],
        }
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-cli-source-test-{unique}"))
    }

    struct FakeSourceLoader;

    impl DataSourceLoader for FakeSourceLoader {
        fn format_name(&self) -> &'static str {
            "fake"
        }

        fn file_extensions(&self) -> &'static [&'static str] {
            &["fake"]
        }

        fn load_table(&self, request: DataSourceRequest<'_>) -> Result<TableData> {
            assert_eq!(request.path, Path::new("data").join("items.fake"));
            Ok(TableData {
                name: request.table.name.clone(),
                rows: vec![RowData {
                    values: BTreeMap::from([("id".to_owned(), Value::Integer(1001))]),
                }],
            })
        }
    }
}
