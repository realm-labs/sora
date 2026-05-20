use std::path::Path;

use heck::{ToShoutySnakeCase, ToSnakeCase};
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, FieldIr, TypeIr};

use crate::{
    generator::CodeGenerator,
    render::{ensure_dir, write_file},
};

pub struct ProtoCodeGenerator;

impl CodeGenerator for ProtoCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        write_file(&out_dir.join("sora_config.proto"), render_proto(ir))
    }
}

fn render_proto(ir: &ConfigIr) -> String {
    let mut output = String::new();
    output.push_str("syntax = \"proto3\";\n\n");
    output.push_str(&format!("package {};\n\n", ir.package));

    output.push_str("message SoraConfigData {\n");
    for (index, table) in ir.tables.iter().enumerate() {
        let field_name = table.name.to_snake_case();
        let message_name = &table.name;
        if matches!(table.mode, sora_ir::model::TableModeIr::Singleton) {
            output.push_str(&format!("  {message_name} {field_name} = {};\n", index + 1));
        } else {
            output.push_str(&format!(
                "  repeated {message_name} {field_name} = {};\n",
                index + 1
            ));
        }
    }
    output.push_str("}\n\n");

    for item in &ir.enums {
        output.push_str(&format!("enum {} {{\n", item.name));
        let prefix = item.name.to_shouty_snake_case();
        for (index, value) in item.values.iter().enumerate() {
            output.push_str(&format!(
                "  {prefix}_{} = {index};\n",
                value.to_shouty_snake_case()
            ));
        }
        output.push_str("}\n\n");
    }

    for item in &ir.structs {
        output.push_str(&render_message(ir, &item.name, &item.fields));
    }

    for item in &ir.unions {
        output.push_str(&format!("message {} {{\n", item.name));
        output.push_str("  oneof kind {\n");
        for (index, variant) in item.variants.iter().enumerate() {
            output.push_str(&format!(
                "    {}{} {} = {};\n",
                item.name,
                variant.name,
                variant.name.to_snake_case(),
                index + 1
            ));
        }
        output.push_str("  }\n");
        output.push_str("}\n\n");

        for variant in &item.variants {
            output.push_str(&render_message(
                ir,
                &format!("{}{}", item.name, variant.name),
                &variant.fields,
            ));
        }
    }

    for table in &ir.tables {
        output.push_str(&render_message(ir, &table.name, &table.fields));
    }

    output
}

fn render_message(ir: &ConfigIr, name: &str, fields: &[FieldIr]) -> String {
    let mut output = String::new();
    output.push_str(&format!("message {name} {{\n"));
    for (index, field) in fields.iter().enumerate() {
        output.push_str(&format!(
            "  {}{} {} = {};\n",
            proto_label(ir, &field.ty),
            proto_type(ir, &field.ty),
            field.name.to_snake_case(),
            index + 1
        ));
    }
    output.push_str("}\n\n");
    output
}

fn proto_label(ir: &ConfigIr, ty: &TypeIr) -> &'static str {
    match ty {
        TypeIr::List(_) | TypeIr::Array { .. } => "repeated ",
        TypeIr::Optional(element) if supports_proto_optional(ir, element) => "optional ",
        _ => "",
    }
}

fn supports_proto_optional(ir: &ConfigIr, ty: &TypeIr) -> bool {
    match ty {
        TypeIr::Bool
        | TypeIr::I32
        | TypeIr::I64
        | TypeIr::F32
        | TypeIr::F64
        | TypeIr::String
        | TypeIr::Enum(_) => true,
        TypeIr::Ref { table, field } => supports_proto_optional(ir, ref_type(ir, table, field)),
        _ => false,
    }
}

fn proto_type(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I32 => "int32".to_owned(),
        TypeIr::I64 => "int64".to_owned(),
        TypeIr::F32 => "float".to_owned(),
        TypeIr::F64 => "double".to_owned(),
        TypeIr::String => "string".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Array { element, .. } | TypeIr::Optional(element) => {
            proto_type(ir, element)
        }
        TypeIr::Ref { table, field } => proto_type(ir, ref_type(ir, table, field)),
    }
}

fn ref_type<'a>(ir: &'a ConfigIr, table_name: &str, field_name: &str) -> &'a TypeIr {
    ir.tables
        .iter()
        .find(|table| table.name == table_name)
        .and_then(|table| table.fields.iter().find(|field| field.name == field_name))
        .map(|field| &field.ty)
        .unwrap_or(&TypeIr::I32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;

    #[test]
    fn generates_business_proto_schema() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "com.sora.game"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

[[structs]]
name = "Cost"

[[structs.fields]]
name = "item_id"
type = "i32"
required = true

[[unions]]
name = "Action"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"
required = true

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
required = true

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true

[[tables.fields]]
name = "tags"
type = "list<string>"
separator = ","

[[tables.fields]]
name = "action"
type = "union<Action>"
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();

        let proto = render_proto(&ir);

        assert!(proto.contains("syntax = \"proto3\";"));
        assert!(proto.contains("package com.sora.game;"));
        assert!(proto.contains("message SoraConfigData"));
        assert!(proto.contains("repeated Item item = 1;"));
        assert!(proto.contains("enum ItemType"));
        assert!(proto.contains("ITEM_TYPE_WEAPON = 0;"));
        assert!(proto.contains("message Action"));
        assert!(proto.contains("oneof kind"));
        assert!(proto.contains("ActionAddItem add_item = 1;"));
        assert!(proto.contains("repeated string tags = 3;"));
    }
}
