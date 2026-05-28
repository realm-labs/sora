use std::{collections::BTreeMap, path::Path};

use heck::{ToShoutySnakeCase, ToSnakeCase};
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TypeIr};

use crate::{
    generator::{CodeGenerator, CodegenContext, ensure_sora_runtime_format},
    model::{
        BaseField, BaseImport, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion,
        BaseUnionVariant, build_base_model,
    },
    options::{CCodegenOptions, CStandard},
    render::{ensure_dir, render_template, write_file},
    type_mapping::{TypeMapping, TypeMappingContext, TypeMappingRegistry},
};

pub struct CCodeGenerator;
crate::impl_test_codegen_generate!(CCodeGenerator, "c");

impl CodeGenerator for CCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let codegen_options = context.options::<CCodegenOptions>()?;
        ensure_sora_runtime_format("c", codegen_options.runtime_format)?;
        ensure_dir(out_dir)?;

        let options = COptionsView::new(ir, &codegen_options)?;
        let mut helpers = CHelperRegistry::new(options.prefix.clone());
        let mapper = CTypeMapper::new(context.target, ir, context.type_mappings, &options);
        let model =
            CModel::from_base_model(ir, build_base_model(ir)?, &options, &mapper, &mut helpers);
        let helpers = helpers.into_helpers();

        for item in &model.enums {
            let rendered = render_template(
                "c",
                "enum.h.j2",
                context! { enum => item, options => &options },
            )?;
            write_file(&out_dir.join(format!("{}.h", item.snake_name)), rendered)?;
            let rendered = render_template(
                "c",
                "enum.c.j2",
                context! { enum => item, options => &options },
            )?;
            write_file(&out_dir.join(format!("{}.c", item.snake_name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "c",
                "record.h.j2",
                context! { record => record, options => &options },
            )?;
            write_file(&out_dir.join(format!("{}.h", record.snake_name)), rendered)?;
            let rendered = render_template(
                "c",
                "record.c.j2",
                context! { record => record, options => &options },
            )?;
            write_file(&out_dir.join(format!("{}.c", record.snake_name)), rendered)?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "c",
                "union.h.j2",
                context! { union => union, options => &options },
            )?;
            write_file(&out_dir.join(format!("{}.h", union.snake_name)), rendered)?;
            let rendered = render_template(
                "c",
                "union.c.j2",
                context! { union => union, options => &options },
            )?;
            write_file(&out_dir.join(format!("{}.c", union.snake_name)), rendered)?;
        }

        let rendered = render_template("c", "runtime.h.j2", context! { options => &options })?;
        write_file(&out_dir.join("sora_runtime.h"), rendered)?;
        let rendered = render_template("c", "runtime.c.j2", context! { options => &options })?;
        write_file(&out_dir.join("sora_runtime.c"), rendered)?;

        let rendered = render_template(
            "c",
            "types.h.j2",
            context! { model => &model, helpers => &helpers, options => &options },
        )?;
        write_file(&out_dir.join("sora_types.h"), rendered)?;
        let rendered = render_template(
            "c",
            "types.c.j2",
            context! { model => &model, helpers => &helpers, options => &options },
        )?;
        write_file(&out_dir.join("sora_types.c"), rendered)?;

        let rendered = render_template(
            "c",
            "config.h.j2",
            context! { model => &model, options => &options },
        )?;
        write_file(&out_dir.join("sora_config.h"), rendered)?;
        let rendered = render_template(
            "c",
            "config.c.j2",
            context! { model => &model, options => &options },
        )?;
        write_file(&out_dir.join("sora_config.c"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct COptionsView {
    prefix: String,
    prefix_upper: String,
    standard_name: &'static str,
}

impl COptionsView {
    fn new(ir: &ConfigIr, codegen_options: &CCodegenOptions) -> Result<Self> {
        let prefix = codegen_options
            .prefix
            .clone()
            .unwrap_or_else(|| ir.package.replace('.', "_").to_snake_case());
        if !is_c_identifier(&prefix) {
            return Err(SoraError::InvalidSchema(format!(
                "c prefix `{prefix}` must be a valid C identifier"
            )));
        }
        Ok(Self {
            prefix_upper: prefix.to_shouty_snake_case(),
            prefix,
            standard_name: c_standard_name(codegen_options.c_standard),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
struct CModel {
    schema_fingerprint: String,
    enums: Vec<CEnum>,
    unions: Vec<CUnion>,
    records: Vec<CRecord>,
    tables: Vec<CTable>,
    modules: Vec<String>,
    custom_imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CEnum {
    name: String,
    snake_name: String,
    type_name: String,
    decode_fn: String,
    values: Vec<CEnumValue>,
}

#[derive(Debug, Clone, Serialize)]
struct CEnumValue {
    name: String,
    ordinal: usize,
}

#[derive(Debug, Clone, Serialize)]
struct CRecord {
    pascal_name: String,
    snake_name: String,
    type_name: String,
    decode_fn: String,
    free_fn: String,
    imports: Vec<CImport>,
    custom_imports: Vec<String>,
    fields: Vec<CField>,
    table: Option<CTable>,
}

#[derive(Debug, Clone, Serialize)]
struct CUnion {
    pascal_name: String,
    snake_name: String,
    type_name: String,
    tag_type_name: String,
    decode_fn: String,
    free_fn: String,
    imports: Vec<CImport>,
    custom_imports: Vec<String>,
    variants: Vec<CUnionVariant>,
}

#[derive(Debug, Clone, Serialize)]
struct CUnionVariant {
    name: String,
    tag_name: String,
    field_name: String,
    fields: Vec<CField>,
}

#[derive(Debug, Clone, Serialize)]
struct CImport {
    module: String,
}

#[derive(Debug, Clone, Serialize)]
struct CField {
    raw_name: String,
    name: String,
    type_name: String,
    decode: String,
    free: Option<String>,
    imports: Vec<String>,
    comment: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CTable {
    name: String,
    snake_name: String,
    table_type: String,
    table_load_fn: String,
    table_free_fn: String,
    row_type: String,
    mode: String,
    rows_field: String,
    len_field: String,
    key_field_name: Option<String>,
    key_type: Option<String>,
    key_param_decl: Option<String>,
    key_match_expr: Option<String>,
    unique_indexes: Vec<CIndex>,
    non_unique_indexes: Vec<CIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct CIndex {
    name: String,
    method_name: String,
    field_name: String,
    key_type: String,
    param_decl: String,
    param_name: String,
    match_expr: String,
}

#[derive(Debug, Clone, Serialize)]
struct CTypeHelper {
    name: String,
    declaration: String,
    decode_fn: String,
    free_fn: String,
    implementation: String,
}

impl CModel {
    fn from_base_model(
        ir: &ConfigIr,
        model: BaseModel,
        options: &COptionsView,
        mapper: &CTypeMapper<'_>,
        helpers: &mut CHelperRegistry,
    ) -> Self {
        let enums = model
            .enums
            .into_iter()
            .map(|item| CEnum {
                type_name: c_named_type(options, &item.snake_name),
                decode_fn: c_decode_fn(options, &item.snake_name),
                name: item.pascal_name,
                snake_name: item.snake_name.clone(),
                values: item
                    .values
                    .into_iter()
                    .enumerate()
                    .map(|(ordinal, value)| CEnumValue {
                        name: format!(
                            "{}_{}_{}",
                            options.prefix_upper,
                            item.snake_name.to_shouty_snake_case(),
                            value.to_shouty_snake_case()
                        ),
                        ordinal,
                    })
                    .collect(),
            })
            .collect();
        let tables = model
            .tables
            .into_iter()
            .map(|item| c_table(ir, item, mapper, helpers))
            .collect::<Vec<_>>();
        let records = model
            .records
            .into_iter()
            .map(|item| {
                let table = tables
                    .iter()
                    .find(|table| table.snake_name == item.snake_name)
                    .cloned();
                c_record(ir, item, mapper, helpers, table)
            })
            .collect::<Vec<_>>();
        let unions = model
            .unions
            .into_iter()
            .map(|item| c_union(ir, item, mapper, helpers))
            .collect::<Vec<_>>();
        let custom_imports = collect_c_model_imports(&records, &unions);

        Self {
            schema_fingerprint: model.schema_fingerprint,
            enums,
            unions,
            records,
            tables,
            modules: model.modules,
            custom_imports,
        }
    }
}

fn c_record(
    ir: &ConfigIr,
    record: BaseRecord,
    mapper: &CTypeMapper<'_>,
    helpers: &mut CHelperRegistry,
    table: Option<CTable>,
) -> CRecord {
    let fields = record
        .fields
        .into_iter()
        .map(|field| c_field(ir, field, mapper, helpers))
        .collect::<Vec<_>>();
    let custom_imports = collect_c_imports(fields.iter());
    CRecord {
        pascal_name: record.pascal_name,
        type_name: c_named_type(mapper.options, &record.snake_name),
        decode_fn: c_decode_fn(mapper.options, &record.snake_name),
        free_fn: c_free_fn(mapper.options, &record.snake_name),
        snake_name: record.snake_name,
        imports: record.imports.into_iter().map(c_import).collect(),
        custom_imports,
        fields,
        table,
    }
}

fn c_union(
    ir: &ConfigIr,
    union: BaseUnion,
    mapper: &CTypeMapper<'_>,
    helpers: &mut CHelperRegistry,
) -> CUnion {
    let snake_name = union.snake_name;
    let variants = union
        .variants
        .into_iter()
        .map(|variant| c_variant(ir, &snake_name, variant, mapper, helpers))
        .collect::<Vec<_>>();
    let custom_imports = collect_c_imports(variants.iter().flat_map(|variant| &variant.fields));
    CUnion {
        pascal_name: union.pascal_name,
        type_name: c_named_type(mapper.options, &snake_name),
        tag_type_name: format!("{}_tag", c_named_type(mapper.options, &snake_name)),
        decode_fn: c_decode_fn(mapper.options, &snake_name),
        free_fn: c_free_fn(mapper.options, &snake_name),
        snake_name: snake_name.clone(),
        imports: union.imports.into_iter().map(c_import).collect(),
        custom_imports,
        variants,
    }
}

fn c_variant(
    ir: &ConfigIr,
    union_name: &str,
    variant: BaseUnionVariant,
    mapper: &CTypeMapper<'_>,
    helpers: &mut CHelperRegistry,
) -> CUnionVariant {
    CUnionVariant {
        tag_name: format!(
            "{}_{}_{}",
            mapper.options.prefix_upper,
            union_name.to_shouty_snake_case(),
            variant.snake_name.to_shouty_snake_case()
        ),
        field_name: variant.snake_name.clone(),
        name: variant.pascal_name,
        fields: variant
            .fields
            .into_iter()
            .map(|field| {
                let mut field = c_field(ir, field, mapper, helpers);
                field.decode = field
                    .decode
                    .replace("&out->", &format!("&out->value.{}.", variant.snake_name));
                field.free = field.free.map(|free| {
                    free.replace("value->", &format!("value->value.{}.", variant.snake_name))
                });
                field
            })
            .collect(),
    }
}

fn c_field(
    ir: &ConfigIr,
    field: BaseField,
    mapper: &CTypeMapper<'_>,
    helpers: &mut CHelperRegistry,
) -> CField {
    let ty = c_type_name(ir, &field.ty, mapper, helpers);
    CField {
        raw_name: field.raw_name,
        name: field.snake_name.clone(),
        type_name: ty,
        decode: c_decode_into(
            ir,
            &field.ty,
            &format!("&out->{}", field.snake_name),
            mapper,
            helpers,
        ),
        free: c_free_into(
            ir,
            &field.ty,
            &format!("&value->{}", field.snake_name),
            mapper,
            helpers,
        ),
        imports: mapper.imports(&field.ty),
        comment: field.comment,
    }
}

fn collect_c_imports<'a>(fields: impl Iterator<Item = &'a CField>) -> Vec<String> {
    let mut imports = fields
        .flat_map(|field| field.imports.iter().cloned())
        .collect::<Vec<_>>();
    imports.sort();
    imports.dedup();
    imports
}

fn collect_c_model_imports(records: &[CRecord], unions: &[CUnion]) -> Vec<String> {
    let mut imports = records
        .iter()
        .flat_map(|record| record.custom_imports.iter().cloned())
        .chain(
            unions
                .iter()
                .flat_map(|union| union.custom_imports.iter().cloned()),
        )
        .collect::<Vec<_>>();
    imports.sort();
    imports.dedup();
    imports
}

fn c_import(import: BaseImport) -> CImport {
    CImport {
        module: import.module,
    }
}

fn c_table(
    ir: &ConfigIr,
    table: BaseTable,
    mapper: &CTypeMapper<'_>,
    helpers: &mut CHelperRegistry,
) -> CTable {
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| c_type_name(ir, &field.ty, mapper, helpers));
    let key_param_decl = table
        .key_field
        .as_ref()
        .map(|field| c_param_decl(ir, &field.ty, "key", mapper, helpers));
    let key_match_expr = table
        .key_field
        .as_ref()
        .map(|field| c_key_match_expr(ir, &field.ty, &format!("row->{}", field.snake_name), "key"));

    CTable {
        name: table.name,
        table_type: format!("{}_{}_table", mapper.options.prefix, table.snake_name),
        table_load_fn: format!("{}_{}_table_load", mapper.options.prefix, table.snake_name),
        table_free_fn: format!("{}_{}_table_free", mapper.options.prefix, table.snake_name),
        row_type: c_named_type(mapper.options, &table.snake_name),
        mode: table.mode_name,
        rows_field: format!("{}_rows", table.snake_name),
        len_field: format!("{}_len", table.snake_name),
        key_field_name: table
            .key_field
            .as_ref()
            .map(|field| field.snake_name.clone()),
        key_type,
        key_param_decl,
        key_match_expr,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| c_index(ir, index, mapper, helpers))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| c_index(ir, index, mapper, helpers))
            .collect(),
        snake_name: table.snake_name,
    }
}

fn c_index(
    ir: &ConfigIr,
    index: BaseIndex,
    mapper: &CTypeMapper<'_>,
    helpers: &mut CHelperRegistry,
) -> CIndex {
    let param_name = index.field.snake_name.clone();
    CIndex {
        name: index.snake_name,
        method_name: index.method_name,
        field_name: index.field.snake_name.clone(),
        key_type: c_type_name(ir, &index.field.ty, mapper, helpers),
        param_decl: c_param_decl(ir, &index.field.ty, &param_name, mapper, helpers),
        match_expr: c_key_match_expr(
            ir,
            &index.field.ty,
            &format!("row->{}", index.field.snake_name),
            &param_name,
        ),
        param_name,
    }
}

#[derive(Debug)]
struct CHelperRegistry {
    prefix: String,
    helpers: BTreeMap<String, CTypeHelper>,
}

impl CHelperRegistry {
    fn new(prefix: String) -> Self {
        Self {
            prefix,
            helpers: BTreeMap::new(),
        }
    }

    fn into_helpers(self) -> Vec<CTypeHelper> {
        self.helpers.into_values().collect()
    }

    fn ensure_collection(
        &mut self,
        ir: &ConfigIr,
        ty: &TypeIr,
        fixed_len: Option<usize>,
        mapper: &CTypeMapper<'_>,
    ) -> String {
        let element_type = c_type_name(ir, ty, mapper, self);
        let suffix = c_type_suffix(ir, ty);
        let name = match fixed_len {
            Some(len) => format!("{}_{}_array_{len}", self.prefix, suffix),
            None => format!("{}_{}_array", self.prefix, suffix),
        };
        if self.helpers.contains_key(&name) {
            return name;
        }
        let decode_fn = format!("{name}_decode");
        let free_fn = format!("{name}_free");
        let element_decode = c_decode_into(ir, ty, "&out->data[index]", mapper, self);
        let element_free_out =
            c_free_into(ir, ty, "&out->data[index]", mapper, self).unwrap_or_default();
        let element_free_value =
            c_free_into(ir, ty, "&value->data[index]", mapper, self).unwrap_or_default();
        let len_check = fixed_len
            .map(|len| {
                format!(
                    r#"    if (length != {len}) {{
        return sora_error(SORA_ERROR_DECODE, "array length does not match schema");
    }}
"#
                )
            })
            .unwrap_or_default();
        let implementation = format!(
            r#"sora_result {decode_fn}(sora_reader* reader, {name}* out) {{
    uint32_t length = 0;
    SORA_TRY(sora_reader_read_u32(reader, &length));
{len_check}    out->data = NULL;
    out->len = length;
    if (length == 0) {{
        return sora_ok();
    }}
    out->data = ({element_type}*)calloc(length, sizeof({element_type}));
    if (out->data == NULL) {{
        return sora_error(SORA_ERROR_OUT_OF_MEMORY, "failed to allocate array");
    }}
    for (size_t index = 0; index < length; ++index) {{
        sora_result result = {element_decode};
        if (result.code != SORA_OK) {{
            for (size_t cleanup = 0; cleanup < index; ++cleanup) {{
                size_t previous = index;
                index = cleanup;
{element_free_out_block}                index = previous;
            }}
            free(out->data);
            out->data = NULL;
            out->len = 0;
            return result;
        }}
    }}
    return sora_ok();
}}

void {free_fn}({name}* value) {{
    if (value == NULL || value->data == NULL) {{
        return;
    }}
    for (size_t index = 0; index < value->len; ++index) {{
{element_free_value_block}    }}
    free(value->data);
    value->data = NULL;
    value->len = 0;
}}
"#,
            element_free_out_block = indent_optional_statement(&element_free_out, 8),
            element_free_value_block = indent_optional_statement(&element_free_value, 8),
        );
        self.helpers.insert(
            name.clone(),
            CTypeHelper {
                declaration: format!(
                    "typedef struct {name} {{\n    {element_type}* data;\n    size_t len;\n}} {name};"
                ),
                name: name.clone(),
                decode_fn,
                free_fn,
                implementation,
            },
        );
        name
    }

    fn ensure_map(
        &mut self,
        ir: &ConfigIr,
        key: &TypeIr,
        value: &TypeIr,
        mapper: &CTypeMapper<'_>,
    ) -> String {
        let key_type = c_type_name(ir, key, mapper, self);
        let value_type = c_type_name(ir, value, mapper, self);
        let key_suffix = c_type_suffix(ir, key);
        let value_suffix = c_type_suffix(ir, value);
        let name = format!("{}_{}_{}_map", self.prefix, key_suffix, value_suffix);
        if self.helpers.contains_key(&name) {
            return name;
        }
        let entry_name = format!("{name}_entry");
        let decode_fn = format!("{name}_decode");
        let free_fn = format!("{name}_free");
        let key_decode = c_decode_into(ir, key, "&out->data[index].key", mapper, self);
        let value_decode = c_decode_into(ir, value, "&out->data[index].value", mapper, self);
        let key_free_out =
            c_free_into(ir, key, "&out->data[index].key", mapper, self).unwrap_or_default();
        let value_free_out =
            c_free_into(ir, value, "&out->data[index].value", mapper, self).unwrap_or_default();
        let key_free_cleanup =
            c_free_into(ir, key, "&out->data[cleanup].key", mapper, self).unwrap_or_default();
        let value_free_cleanup =
            c_free_into(ir, value, "&out->data[cleanup].value", mapper, self).unwrap_or_default();
        let key_free_value =
            c_free_into(ir, key, "&value->data[index].key", mapper, self).unwrap_or_default();
        let value_free_value =
            c_free_into(ir, value, "&value->data[index].value", mapper, self).unwrap_or_default();
        let implementation = format!(
            r#"sora_result {decode_fn}(sora_reader* reader, {name}* out) {{
    uint32_t length = 0;
    SORA_TRY(sora_reader_read_u32(reader, &length));
    out->data = NULL;
    out->len = length;
    if (length == 0) {{
        return sora_ok();
    }}
    out->data = ({entry_name}*)calloc(length, sizeof({entry_name}));
    if (out->data == NULL) {{
        return sora_error(SORA_ERROR_OUT_OF_MEMORY, "failed to allocate map");
    }}
    for (size_t index = 0; index < length; ++index) {{
        sora_result result = {key_decode};
        if (result.code == SORA_OK) {{
            result = {value_decode};
        }}
        if (result.code != SORA_OK) {{
{key_free_out_block}{value_free_out_block}            for (size_t cleanup = 0; cleanup < index; ++cleanup) {{
{key_free_cleanup_block}{value_free_cleanup_block}            }}
            free(out->data);
            out->data = NULL;
            out->len = 0;
            return result;
        }}
    }}
    return sora_ok();
}}

void {free_fn}({name}* value) {{
    if (value == NULL || value->data == NULL) {{
        return;
    }}
    for (size_t index = 0; index < value->len; ++index) {{
{key_free_value_block}{value_free_value_block}    }}
    free(value->data);
    value->data = NULL;
    value->len = 0;
}}
"#,
            key_free_out_block = indent_optional_statement(&key_free_out, 12),
            value_free_out_block = indent_optional_statement(&value_free_out, 12),
            key_free_cleanup_block = indent_optional_statement(&key_free_cleanup, 16),
            value_free_cleanup_block = indent_optional_statement(&value_free_cleanup, 16),
            key_free_value_block = indent_optional_statement(&key_free_value, 8),
            value_free_value_block = indent_optional_statement(&value_free_value, 8),
        );
        self.helpers.insert(
            name.clone(),
            CTypeHelper {
                declaration: format!(
                    "typedef struct {entry_name} {{\n    {key_type} key;\n    {value_type} value;\n}} {entry_name};\n\ntypedef struct {name} {{\n    {entry_name}* data;\n    size_t len;\n}} {name};"
                ),
                name: name.clone(),
                decode_fn,
                free_fn,
                implementation,
            },
        );
        name
    }

    fn ensure_optional(&mut self, ir: &ConfigIr, ty: &TypeIr, mapper: &CTypeMapper<'_>) -> String {
        let value_type = c_type_name(ir, ty, mapper, self);
        let suffix = c_type_suffix(ir, ty);
        let name = format!("{}_optional_{}", self.prefix, suffix);
        if self.helpers.contains_key(&name) {
            return name;
        }
        let decode_fn = format!("{name}_decode");
        let free_fn = format!("{name}_free");
        let value_decode = c_decode_into(ir, ty, "out->value", mapper, self);
        let value_free_out = c_free_into(ir, ty, "out->value", mapper, self).unwrap_or_default();
        let value_free_value =
            c_free_into(ir, ty, "value->value", mapper, self).unwrap_or_default();
        let implementation = format!(
            r#"sora_result {decode_fn}(sora_reader* reader, {name}* out) {{
    uint8_t presence = 0;
    SORA_TRY(sora_reader_read_byte(reader, &presence));
    out->has_value = false;
    out->value = NULL;
    if (presence == 0) {{
        return sora_ok();
    }}
    if (presence != 1) {{
        return sora_error(SORA_ERROR_DECODE, "invalid optional presence");
    }}
    out->value = ({value_type}*)calloc(1, sizeof({value_type}));
    if (out->value == NULL) {{
        return sora_error(SORA_ERROR_OUT_OF_MEMORY, "failed to allocate optional");
    }}
    sora_result result = {value_decode};
    if (result.code != SORA_OK) {{
{value_free_out_block}        free(out->value);
        out->value = NULL;
        return result;
    }}
    out->has_value = true;
    return sora_ok();
}}

void {free_fn}({name}* value) {{
    if (value == NULL || value->value == NULL) {{
        return;
    }}
{value_free_value_block}    free(value->value);
    value->value = NULL;
    value->has_value = false;
}}
"#,
            value_free_out_block = indent_optional_statement(&value_free_out, 4),
            value_free_value_block = indent_optional_statement(&value_free_value, 4),
        );
        self.helpers.insert(
            name.clone(),
            CTypeHelper {
                declaration: format!(
                    "typedef struct {name} {{\n    bool has_value;\n    {value_type}* value;\n}} {name};"
                ),
                name: name.clone(),
                decode_fn,
                free_fn,
                implementation,
            },
        );
        name
    }
}

fn indent_optional_statement(statement: &str, spaces: usize) -> String {
    if statement.trim().is_empty() {
        return String::new();
    }
    let indent = " ".repeat(spaces);
    statement
        .lines()
        .map(|line| format!("{indent}{line}\n"))
        .collect()
}

struct CTypeMapper<'a> {
    target: &'a str,
    ir: &'a ConfigIr,
    mappings: &'a TypeMappingRegistry,
    options: &'a COptionsView,
}

impl<'a> CTypeMapper<'a> {
    fn new(
        target: &'a str,
        ir: &'a ConfigIr,
        mappings: &'a TypeMappingRegistry,
        options: &'a COptionsView,
    ) -> Self {
        Self {
            target,
            ir,
            mappings,
            options,
        }
    }

    fn imports(&self, ty: &TypeIr) -> Vec<String> {
        self.mappings.imports_for(self.target, self.ir, ty)
    }

    fn mapping(&self, ty: &TypeIr) -> Option<TypeMapping> {
        self.mappings.map_type(TypeMappingContext {
            target: self.target,
            ir: self.ir,
            ty,
        })
    }
}

fn c_type_name(
    ir: &ConfigIr,
    ty: &TypeIr,
    mapper: &CTypeMapper<'_>,
    helpers: &mut CHelperRegistry,
) -> String {
    if let Some(mapping) = mapper.mapping(ty) {
        return mapping.type_name;
    }

    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I8 => "int8_t".to_owned(),
        TypeIr::U8 => "uint8_t".to_owned(),
        TypeIr::I16 => "int16_t".to_owned(),
        TypeIr::U16 => "uint16_t".to_owned(),
        TypeIr::I32 => "int32_t".to_owned(),
        TypeIr::U32 => "uint32_t".to_owned(),
        TypeIr::I64 | TypeIr::Duration | TypeIr::DateTime => "int64_t".to_owned(),
        TypeIr::F32 => "float".to_owned(),
        TypeIr::F64 => "double".to_owned(),
        TypeIr::String => "sora_string".to_owned(),
        TypeIr::Text => "sora_text_key".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            c_named_type(mapper.options, &name.to_snake_case())
        }
        TypeIr::List(element) | TypeIr::Set(element) => {
            helpers.ensure_collection(ir, element, None, mapper)
        }
        TypeIr::Map { key, value } => helpers.ensure_map(ir, key, value, mapper),
        TypeIr::Array { element, len } => {
            helpers.ensure_collection(ir, element, Some(*len), mapper)
        }
        TypeIr::Ref { table, field } => ref_field_type(ir, table, field)
            .map(|field| c_type_name(ir, &field.ty, mapper, helpers))
            .unwrap_or_else(|| "int32_t".to_owned()),
        TypeIr::Optional(element) => helpers.ensure_optional(ir, element, mapper),
    }
}

fn c_type_suffix(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I8 => "i8".to_owned(),
        TypeIr::U8 => "u8".to_owned(),
        TypeIr::I16 => "i16".to_owned(),
        TypeIr::U16 => "u16".to_owned(),
        TypeIr::I32 => "i32".to_owned(),
        TypeIr::U32 => "u32".to_owned(),
        TypeIr::I64 | TypeIr::Duration | TypeIr::DateTime => "i64".to_owned(),
        TypeIr::F32 => "f32".to_owned(),
        TypeIr::F64 => "f64".to_owned(),
        TypeIr::String => "string".to_owned(),
        TypeIr::Text => "text".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.to_snake_case(),
        TypeIr::List(element) | TypeIr::Set(element) => {
            format!("{}_array", c_type_suffix(ir, element))
        }
        TypeIr::Map { key, value } => {
            format!(
                "{}_{}_map",
                c_type_suffix(ir, key),
                c_type_suffix(ir, value)
            )
        }
        TypeIr::Array { element, len } => format!("{}_array_{len}", c_type_suffix(ir, element)),
        TypeIr::Ref { table, field } => ref_field_type(ir, table, field)
            .map(|field| c_type_suffix(ir, &field.ty))
            .unwrap_or_else(|| "i32".to_owned()),
        TypeIr::Optional(element) => format!("optional_{}", c_type_suffix(ir, element)),
    }
}

fn c_decode_into(
    ir: &ConfigIr,
    ty: &TypeIr,
    target: &str,
    mapper: &CTypeMapper<'_>,
    helpers: &mut CHelperRegistry,
) -> String {
    if let Some(mapping) = mapper.mapping(ty) {
        let base_expr = match ty {
            TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
                format!(
                    "{}(reader, {target})",
                    c_decode_fn(mapper.options, &name.to_snake_case())
                )
            }
            _ => String::new(),
        };
        return mapping.wrap_decode_into(&base_expr, target);
    }

    match ty {
        TypeIr::Bool => format!("sora_reader_read_bool(reader, {target})"),
        TypeIr::I8 => format!("sora_reader_read_i8(reader, {target})"),
        TypeIr::U8 => format!("sora_reader_read_u8(reader, {target})"),
        TypeIr::I16 => format!("sora_reader_read_i16(reader, {target})"),
        TypeIr::U16 => format!("sora_reader_read_u16(reader, {target})"),
        TypeIr::I32 => format!("sora_reader_read_i32(reader, {target})"),
        TypeIr::U32 => format!("sora_reader_read_u32(reader, {target})"),
        TypeIr::I64 | TypeIr::Duration | TypeIr::DateTime => {
            format!("sora_reader_read_i64(reader, {target})")
        }
        TypeIr::F32 => format!("sora_reader_read_f32(reader, {target})"),
        TypeIr::F64 => format!("sora_reader_read_f64(reader, {target})"),
        TypeIr::String => format!("sora_reader_read_string(reader, {target})"),
        TypeIr::Text => format!("sora_reader_read_text_key(reader, {target})"),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!(
                "{}(reader, {target})",
                c_decode_fn(mapper.options, &name.to_snake_case())
            )
        }
        TypeIr::List(_)
        | TypeIr::Set(_)
        | TypeIr::Map { .. }
        | TypeIr::Array { .. }
        | TypeIr::Optional(_) => {
            let helper = c_type_name(ir, ty, mapper, helpers);
            format!("{helper}_decode(reader, {target})")
        }
        TypeIr::Ref { table, field } => ref_field_type(ir, table, field)
            .map(|field| c_decode_into(ir, &field.ty, target, mapper, helpers))
            .unwrap_or_else(|| format!("sora_reader_read_i32(reader, {target})")),
    }
}

fn c_free_into(
    ir: &ConfigIr,
    ty: &TypeIr,
    target: &str,
    mapper: &CTypeMapper<'_>,
    helpers: &mut CHelperRegistry,
) -> Option<String> {
    if let Some(mapping) = mapper.mapping(ty) {
        return mapping.wrap_free(target);
    }

    match ty {
        TypeIr::String => Some(format!("sora_string_free({target});")),
        TypeIr::Text => Some(format!("sora_text_key_free({target});")),
        TypeIr::Struct(name) | TypeIr::Union(name) => Some(format!(
            "{}({target});",
            c_free_fn(mapper.options, &name.to_snake_case())
        )),
        TypeIr::List(_)
        | TypeIr::Set(_)
        | TypeIr::Map { .. }
        | TypeIr::Array { .. }
        | TypeIr::Optional(_) => {
            let helper = c_type_name(ir, ty, mapper, helpers);
            Some(format!("{helper}_free({target});"))
        }
        TypeIr::Ref { table, field } => ref_field_type(ir, table, field)
            .and_then(|field| c_free_into(ir, &field.ty, target, mapper, helpers)),
        TypeIr::Bool
        | TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Duration
        | TypeIr::DateTime
        | TypeIr::F32
        | TypeIr::F64
        | TypeIr::Enum(_) => None,
    }
}

fn c_param_decl(
    ir: &ConfigIr,
    ty: &TypeIr,
    name: &str,
    mapper: &CTypeMapper<'_>,
    helpers: &mut CHelperRegistry,
) -> String {
    let type_name = c_type_name(ir, ty, mapper, helpers);
    if c_type_is_pointer_param(ir, ty) {
        format!("const {type_name}* {name}")
    } else {
        format!("{type_name} {name}")
    }
}

fn c_type_is_pointer_param(ir: &ConfigIr, ty: &TypeIr) -> bool {
    match ty {
        TypeIr::String
        | TypeIr::Text
        | TypeIr::List(_)
        | TypeIr::Set(_)
        | TypeIr::Map { .. }
        | TypeIr::Array { .. }
        | TypeIr::Optional(_) => true,
        TypeIr::Struct(_) | TypeIr::Union(_) => true,
        TypeIr::Ref { table, field } => ref_field_type(ir, table, field)
            .is_some_and(|field| c_type_is_pointer_param(ir, &field.ty)),
        TypeIr::Bool
        | TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Duration
        | TypeIr::DateTime
        | TypeIr::F32
        | TypeIr::F64
        | TypeIr::Enum(_) => false,
    }
}

fn c_key_match_expr(ir: &ConfigIr, ty: &TypeIr, row_value: &str, param_name: &str) -> String {
    match ty {
        TypeIr::String => format!("sora_string_equal(&{row_value}, {param_name})"),
        TypeIr::Text => format!("sora_text_key_equal(&{row_value}, {param_name})"),
        TypeIr::Ref { table, field } => ref_field_type(ir, table, field)
            .map(|field| c_key_match_expr(ir, &field.ty, row_value, param_name))
            .unwrap_or_else(|| format!("{row_value} == {param_name}")),
        TypeIr::List(_)
        | TypeIr::Set(_)
        | TypeIr::Map { .. }
        | TypeIr::Array { .. }
        | TypeIr::Optional(_)
        | TypeIr::Struct(_)
        | TypeIr::Union(_) => {
            format!("memcmp(&{row_value}, {param_name}, sizeof({row_value})) == 0")
        }
        _ => format!("{row_value} == {param_name}"),
    }
}

fn ref_field_type<'a>(
    ir: &'a ConfigIr,
    table_name: &str,
    field_name: &str,
) -> Option<&'a sora_ir::model::FieldIr> {
    ir.tables
        .iter()
        .find(|table| table.name == table_name)
        .and_then(|table| table.fields.iter().find(|field| field.name == field_name))
}

fn c_named_type(options: &COptionsView, snake_name: &str) -> String {
    format!("{}_{}", options.prefix, snake_name)
}

fn c_decode_fn(options: &COptionsView, snake_name: &str) -> String {
    format!("{}_{}_decode", options.prefix, snake_name)
}

fn c_free_fn(options: &COptionsView, snake_name: &str) -> String {
    format!("{}_{}_free", options.prefix, snake_name)
}

fn c_standard_name(standard: CStandard) -> &'static str {
    match standard {
        CStandard::C99 => "c99",
        CStandard::C11 => "c11",
        CStandard::C17 => "c17",
        CStandard::C23 => "c23",
    }
}

fn is_c_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn generates_c_files() {
        let ir = example_ir();
        let base = temp_dir();

        CCodeGenerator.generate(&ir, &base).unwrap();

        let item = std::fs::read_to_string(base.join("item.h")).unwrap();
        let action = std::fs::read_to_string(base.join("action.h")).unwrap();
        let config = std::fs::read_to_string(base.join("sora_config.h")).unwrap();
        let types = std::fs::read_to_string(base.join("sora_types.h")).unwrap();

        assert!(item.contains("typedef struct game_config_item"));
        assert!(item.contains("game_config_item_type item_type;"));
        assert!(item.contains("game_config_action action;"));
        assert!(item.contains("game_config_string_i32_map weights;"));
        assert!(action.contains("typedef enum game_config_action_tag"));
        assert!(action.contains("union {"));
        assert!(config.contains("typedef struct game_config_config game_config_config;"));
        assert!(config.contains("game_config_config_load_from_bytes"));
        assert!(item.contains("typedef struct game_config_item_table game_config_item_table;"));
        assert!(item.contains("game_config_item_table_load"));
        assert!(item.contains("game_config_item_table_get"));
        assert!(config.contains("game_config_config_item"));
        assert!(!config.contains("game_config_config_get_item"));
        assert!(types.contains("typedef struct game_config_string_array"));
        assert!(types.contains("typedef struct game_config_string_i32_map_entry"));
        assert!(types.contains("sora_string key;"));
        assert!(types.contains("int32_t value;"));

        let _ = std::fs::remove_dir_all(base);
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[codegen.c]
prefix = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

[[unions]]
name = "Action"
tag = "kind"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

[[tables.fields]]
name = "action"
type = "union<Action>"

[[tables.fields]]
name = "tags"
type = "list<string>"
parser = { kind = "split", separator = "|" }

[[tables.fields]]
name = "weights"
type = "map<string,i32>"

[[tables.fields]]
name = "maybe_count"
type = "optional<i32>"

[[tables.indexes]]
name = "by_name"
fields = ["name"]
unique = true
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-c-codegen-test-{unique}"))
    }
}
