use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::studio::{StudioField, StudioIndex, StudioNode, StudioNodeKind, StudioSchema};

/// Table modes accepted by schema mutation operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SchemaTableMode {
    List,
    Map,
    Singleton,
}

impl SchemaTableMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Map => "map",
            Self::Singleton => "singleton",
        }
    }
}

/// Schema entity kinds that can own fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FieldOwnerKind {
    Table,
    Struct,
    Union,
}

/// Identifies the owner of a field, including a union variant when applicable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FieldOwner {
    pub kind: FieldOwnerKind,
    pub name: String,
    pub variant: Option<String>,
}

/// Field properties used by add-field and add-variant operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FieldDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(default)]
    pub groups: Vec<String>,
    pub parser: Option<String>,
    pub comment: Option<String>,
    pub default: Option<String>,
    pub range: Option<[i64; 2]>,
    pub length: Option<[usize; 2]>,
}

/// A table's source declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TableSourceDefinition {
    pub file: String,
    pub format: Option<String>,
    pub sheet: Option<String>,
    #[serde(default)]
    pub sheets: Vec<String>,
}

/// A table index declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IndexDefinition {
    pub name: String,
    pub fields: Vec<String>,
    #[serde(default)]
    pub unique: bool,
}

/// A union variant and its initial fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UnionVariantDefinition {
    pub name: String,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub fields: Vec<FieldDefinition>,
}

/// A derived table-field relationship.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DerivedFieldSource {
    pub table: String,
    pub parent_key: String,
    pub child_key: String,
    #[serde(default)]
    pub map_key: Option<String>,
    pub value_field: Option<String>,
    pub order_by: Option<String>,
}

/// Closed set of schema edits supported by Sora.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum SchemaOperation {
    AddTable {
        id: String,
        name: String,
        source: String,
        mode: SchemaTableMode,
        key: Option<String>,
        table_source: Option<TableSourceDefinition>,
        #[serde(default)]
        groups: Vec<String>,
        #[serde(default)]
        fields: Vec<FieldDefinition>,
        #[serde(default)]
        indexes: Vec<IndexDefinition>,
    },
    RenameTable {
        from: String,
        to: String,
    },
    RemoveTable {
        name: String,
    },
    SetTableMode {
        table: String,
        mode: SchemaTableMode,
    },
    SetTableKey {
        table: String,
        key: Option<String>,
    },
    SetTableSource {
        table: String,
        source: Option<TableSourceDefinition>,
    },
    SetTableGroups {
        table: String,
        groups: Vec<String>,
    },
    AddField {
        owner: FieldOwner,
        field: FieldDefinition,
    },
    RenameField {
        owner: FieldOwner,
        from: String,
        to: String,
    },
    RemoveField {
        owner: FieldOwner,
        field: String,
    },
    ChangeFieldType {
        owner: FieldOwner,
        field: String,
        #[serde(rename = "type")]
        ty: String,
    },
    SetFieldDefault {
        owner: FieldOwner,
        field: String,
        default: Option<String>,
    },
    SetFieldRange {
        owner: FieldOwner,
        field: String,
        range: Option<[i64; 2]>,
    },
    SetFieldLength {
        owner: FieldOwner,
        field: String,
        length: Option<[usize; 2]>,
    },
    SetFieldParser {
        owner: FieldOwner,
        field: String,
        parser: Option<String>,
    },
    SetFieldGroups {
        owner: FieldOwner,
        field: String,
        groups: Vec<String>,
    },
    SetFieldReference {
        owner: FieldOwner,
        field: String,
        table: String,
        target_field: String,
        #[serde(default)]
        optional: bool,
    },
    SetFieldDerivedFrom {
        table: String,
        field: String,
        source: Option<DerivedFieldSource>,
    },
    AddIndex {
        table: String,
        index: IndexDefinition,
    },
    RemoveIndex {
        table: String,
        index: String,
    },
    AddEnum {
        name: String,
        source: String,
        #[serde(default)]
        comment: Option<String>,
        #[serde(default)]
        groups: Vec<String>,
        #[serde(default)]
        values: Vec<EnumValueDefinition>,
    },
    RenameEnum {
        from: String,
        to: String,
    },
    RemoveEnum {
        name: String,
    },
    AddEnumValue {
        enum_name: String,
        id: u32,
        name: String,
        #[serde(default)]
        comment: Option<String>,
    },
    RenameEnumValue {
        enum_name: String,
        from: String,
        to: String,
    },
    RemoveEnumValue {
        enum_name: String,
        name: String,
    },
    AddStruct {
        name: String,
        source: String,
        #[serde(default)]
        groups: Vec<String>,
        #[serde(default)]
        fields: Vec<FieldDefinition>,
    },
    RenameStruct {
        from: String,
        to: String,
    },
    RemoveStruct {
        name: String,
    },
    AddUnion {
        name: String,
        source: String,
        #[serde(default)]
        groups: Vec<String>,
        #[serde(default = "default_union_tag")]
        tag: String,
        #[serde(default)]
        variants: Vec<UnionVariantDefinition>,
    },
    RenameUnion {
        from: String,
        to: String,
    },
    RemoveUnion {
        name: String,
    },
    AddUnionVariant {
        union_name: String,
        variant: UnionVariantDefinition,
    },
    RenameUnionVariant {
        union_name: String,
        from: String,
        to: String,
    },
    RemoveUnionVariant {
        union_name: String,
        variant: String,
    },
    MoveEntitySource {
        kind: SchemaEntityKind,
        name: String,
        source: String,
    },
}

/// An enum value with a stable numeric identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnumValueDefinition {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub comment: Option<String>,
}

/// Schema entity kinds accepted by source-move operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SchemaEntityKind {
    Enum,
    Struct,
    Union,
    Table,
}

/// Result of applying operations to an in-memory Studio schema graph.
#[derive(Debug, Clone)]
pub struct SchemaExecution {
    pub schema: StudioSchema,
    pub affected_entities: BTreeSet<String>,
    pub affected_tables: BTreeSet<String>,
    pub requires_data_migration: bool,
}

/// Stable failures produced by the pure schema operation executor.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SchemaMutationError {
    #[error("schema operation batch must contain at least one operation")]
    EmptyBatch,
    #[error("schema source `{0}` is not declared by the project")]
    UnknownSource(String),
    #[error("{kind} `{name}` already exists")]
    DuplicateEntity { kind: &'static str, name: String },
    #[error("{kind} `{name}` does not exist")]
    UnknownEntity { kind: &'static str, name: String },
    #[error("field `{field}` already exists on {owner}")]
    DuplicateField { owner: String, field: String },
    #[error("field `{field}` does not exist on {owner}")]
    UnknownField { owner: String, field: String },
    #[error("index `{index}` already exists on table `{table}`")]
    DuplicateIndex { table: String, index: String },
    #[error("index `{index}` does not exist on table `{table}`")]
    UnknownIndex { table: String, index: String },
    #[error("enum value `{value}` already exists on enum `{enum_name}`")]
    DuplicateEnumValue { enum_name: String, value: String },
    #[error("enum value id `{id}` already exists on enum `{enum_name}`")]
    DuplicateEnumValueId { enum_name: String, id: u32 },
    #[error("enum value `{value}` does not exist on enum `{enum_name}`")]
    UnknownEnumValue { enum_name: String, value: String },
    #[error("union variant `{variant}` already exists on union `{union_name}`")]
    DuplicateUnionVariant { union_name: String, variant: String },
    #[error("union variant `{variant}` does not exist on union `{union_name}`")]
    UnknownUnionVariant { union_name: String, variant: String },
    #[error("union fields require a variant in their owner selector")]
    MissingUnionVariant,
    #[error("only union field selectors may include a variant")]
    UnexpectedVariant,
    #[error("schema name `{0}` is empty or not an ASCII identifier")]
    InvalidName(String),
    #[error("group `{0}` is not declared by the project")]
    UnknownGroup(String),
    #[error("table `{0}` must declare a key when mode is `map`")]
    MissingMapKey(String),
    #[error("field range lower bound exceeds its upper bound")]
    InvalidRange,
    #[error("field length lower bound exceeds its upper bound")]
    InvalidLength,
}

/// Applies an ordered schema operation batch without touching the filesystem.
pub fn execute_schema_operations(
    base: &StudioSchema,
    operations: &[SchemaOperation],
) -> Result<SchemaExecution, SchemaMutationError> {
    if operations.is_empty() {
        return Err(SchemaMutationError::EmptyBatch);
    }
    let mut execution = SchemaExecution {
        schema: base.clone(),
        affected_entities: BTreeSet::new(),
        affected_tables: BTreeSet::new(),
        requires_data_migration: false,
    };
    for operation in operations {
        apply_operation(&mut execution, operation)?;
    }
    validate_declared_groups(&execution.schema)?;
    crate::studio::graph::refresh_schema_graph(&mut execution.schema);
    let related = execution
        .schema
        .edges
        .iter()
        .filter(|edge| {
            execution.affected_entities.contains(&edge.source)
                || execution.affected_entities.contains(&edge.target)
        })
        .flat_map(|edge| [edge.source.clone(), edge.target.clone()])
        .collect::<Vec<_>>();
    execution.affected_entities.extend(related);
    Ok(execution)
}

fn validate_declared_groups(schema: &StudioSchema) -> Result<(), SchemaMutationError> {
    for group in schema.nodes.iter().flat_map(|node| {
        node.groups
            .iter()
            .chain(node.fields.iter().flat_map(|field| field.groups.iter()))
    }) {
        if !schema.groups.contains_key(group) {
            return Err(SchemaMutationError::UnknownGroup(group.clone()));
        }
    }
    Ok(())
}

fn apply_operation(
    execution: &mut SchemaExecution,
    operation: &SchemaOperation,
) -> Result<(), SchemaMutationError> {
    match operation {
        SchemaOperation::AddTable {
            id,
            name,
            source,
            mode,
            key,
            table_source,
            groups,
            fields,
            indexes,
        } => {
            validate_new_entity(&execution.schema, StudioNodeKind::Table, name, source)?;
            if *mode == SchemaTableMode::Map && key.is_none() {
                return Err(SchemaMutationError::MissingMapKey(name.clone()));
            }
            let mut metadata = BTreeMap::from([
                ("id".to_owned(), id.clone()),
                ("mode".to_owned(), mode.as_str().to_owned()),
                (
                    "key".to_owned(),
                    key.clone().unwrap_or_else(|| "<none>".to_owned()),
                ),
            ]);
            set_table_source_metadata(&mut metadata, table_source.as_ref());
            execution.schema.nodes.push(StudioNode {
                id: crate::studio::graph::node_id(StudioNodeKind::Table, name),
                name: name.clone(),
                kind: StudioNodeKind::Table,
                source: source.clone(),
                groups: normalize_groups(groups),
                subtitle: String::new(),
                fields: fields.iter().map(studio_field).collect::<Result<_, _>>()?,
                aliases: Vec::new(),
                indexes: indexes.iter().map(studio_index).collect(),
                metadata,
            });
            affect_table(execution, name, true);
        }
        SchemaOperation::RenameTable { from, to } => {
            rename_entity(execution, StudioNodeKind::Table, from, to)?;
            replace_table_references(&mut execution.schema, from, to);
            affect_table(execution, from, true);
            affect_table(execution, to, true);
        }
        SchemaOperation::RemoveTable { name } => {
            remove_entity(execution, StudioNodeKind::Table, name)?;
            affect_table(execution, name, true);
        }
        SchemaOperation::SetTableMode { table, mode } => {
            let node = node_mut(&mut execution.schema, StudioNodeKind::Table, table)?;
            node.metadata
                .insert("mode".to_owned(), mode.as_str().to_owned());
            if *mode != SchemaTableMode::Map {
                node.metadata.insert("key".to_owned(), "<none>".to_owned());
            }
            affect_table(execution, table, true);
        }
        SchemaOperation::SetTableKey { table, key } => {
            let node = node_mut(&mut execution.schema, StudioNodeKind::Table, table)?;
            node.metadata.insert(
                "key".to_owned(),
                key.clone().unwrap_or_else(|| "<none>".to_owned()),
            );
            affect_table(execution, table, true);
        }
        SchemaOperation::SetTableSource { table, source } => {
            let node = node_mut(&mut execution.schema, StudioNodeKind::Table, table)?;
            set_table_source_metadata(&mut node.metadata, source.as_ref());
            affect_table(execution, table, true);
        }
        SchemaOperation::SetTableGroups { table, groups } => {
            node_mut(&mut execution.schema, StudioNodeKind::Table, table)?.groups =
                normalize_groups(groups);
            affect_table(execution, table, false);
        }
        SchemaOperation::AddField { owner, field } => {
            let node = owner_node_mut(&mut execution.schema, owner)?;
            validate_field_owner(owner)?;
            let stored_name = stored_field_name(owner, &field.name)?;
            if node.fields.iter().any(|item| item.name == stored_name) {
                return Err(SchemaMutationError::DuplicateField {
                    owner: owner_label(owner),
                    field: field.name.clone(),
                });
            }
            let mut field = studio_field(field)?;
            field.name = stored_name;
            node.fields.push(field);
            affect_owner(execution, owner, true);
        }
        SchemaOperation::RenameField { owner, from, to } => {
            rename_field(execution, owner, from, to)?;
        }
        SchemaOperation::RemoveField { owner, field } => {
            let stored = stored_field_name(owner, field)?;
            let node = owner_node_mut(&mut execution.schema, owner)?;
            let position = node
                .fields
                .iter()
                .position(|item| item.name == stored)
                .ok_or_else(|| SchemaMutationError::UnknownField {
                    owner: owner_label(owner),
                    field: field.clone(),
                })?;
            node.fields.remove(position);
            affect_owner(execution, owner, true);
        }
        SchemaOperation::ChangeFieldType { owner, field, ty } => {
            field_mut(&mut execution.schema, owner, field)?.ty = ty.clone();
            affect_owner(execution, owner, true);
        }
        SchemaOperation::SetFieldDefault {
            owner,
            field,
            default,
        } => {
            field_mut(&mut execution.schema, owner, field)?.default = default.clone();
            affect_owner(execution, owner, true);
        }
        SchemaOperation::SetFieldRange {
            owner,
            field,
            range,
        } => {
            if range.is_some_and(|range| range[0] > range[1]) {
                return Err(SchemaMutationError::InvalidRange);
            }
            field_mut(&mut execution.schema, owner, field)?.range = *range;
            affect_owner(execution, owner, true);
        }
        SchemaOperation::SetFieldLength {
            owner,
            field,
            length,
        } => {
            if length.is_some_and(|length| length[0] > length[1]) {
                return Err(SchemaMutationError::InvalidLength);
            }
            field_mut(&mut execution.schema, owner, field)?.length = *length;
            affect_owner(execution, owner, true);
        }
        SchemaOperation::SetFieldParser {
            owner,
            field,
            parser,
        } => {
            field_mut(&mut execution.schema, owner, field)?.parser = parser.clone();
            affect_owner(execution, owner, true);
        }
        SchemaOperation::SetFieldGroups {
            owner,
            field,
            groups,
        } => {
            field_mut(&mut execution.schema, owner, field)?.groups = normalize_groups(groups);
            affect_owner(execution, owner, false);
        }
        SchemaOperation::SetFieldReference {
            owner,
            field,
            table,
            target_field,
            optional,
        } => {
            validate_name(table)?;
            validate_name(target_field)?;
            let ty = format!("ref<{table}.{target_field}>");
            field_mut(&mut execution.schema, owner, field)?.ty = if *optional {
                format!("optional<{ty}>")
            } else {
                ty
            };
            affect_owner(execution, owner, true);
            execution.affected_tables.insert(table.clone());
        }
        SchemaOperation::SetFieldDerivedFrom {
            table,
            field,
            source,
        } => {
            let owner = FieldOwner {
                kind: FieldOwnerKind::Table,
                name: table.clone(),
                variant: None,
            };
            field_mut(&mut execution.schema, &owner, field)?.source =
                source.as_ref().map(format_derived_source);
            affect_table(execution, table, true);
            if let Some(source) = source {
                execution.affected_tables.insert(source.table.clone());
            }
        }
        SchemaOperation::AddIndex { table, index } => {
            validate_name(&index.name)?;
            let node = node_mut(&mut execution.schema, StudioNodeKind::Table, table)?;
            if node.indexes.iter().any(|item| item.name == index.name) {
                return Err(SchemaMutationError::DuplicateIndex {
                    table: table.clone(),
                    index: index.name.clone(),
                });
            }
            node.indexes.push(studio_index(index));
            affect_table(execution, table, false);
        }
        SchemaOperation::RemoveIndex { table, index } => {
            let node = node_mut(&mut execution.schema, StudioNodeKind::Table, table)?;
            let position = node
                .indexes
                .iter()
                .position(|item| item.name == *index)
                .ok_or_else(|| SchemaMutationError::UnknownIndex {
                    table: table.clone(),
                    index: index.clone(),
                })?;
            node.indexes.remove(position);
            affect_table(execution, table, false);
        }
        SchemaOperation::AddEnum {
            name,
            source,
            comment,
            groups,
            values,
        } => {
            validate_new_entity(&execution.schema, StudioNodeKind::Enum, name, source)?;
            validate_enum_values(name, values)?;
            execution.schema.nodes.push(StudioNode {
                id: crate::studio::graph::node_id(StudioNodeKind::Enum, name),
                name: name.clone(),
                kind: StudioNodeKind::Enum,
                source: source.clone(),
                groups: normalize_groups(groups),
                subtitle: String::new(),
                fields: values
                    .iter()
                    .map(|value| StudioField {
                        name: value.name.clone(),
                        ty: "enum value".to_owned(),
                        enum_value_id: Some(value.id),
                        groups: normalize_groups(groups),
                        parser: None,
                        comment: value.comment.clone(),
                        default: None,
                        range: None,
                        length: None,
                        source: None,
                    })
                    .collect(),
                aliases: Vec::new(),
                indexes: Vec::new(),
                metadata: comment
                    .as_ref()
                    .map(|comment| BTreeMap::from([("comment".to_owned(), comment.clone())]))
                    .unwrap_or_default(),
            });
            affect_entity(execution, "enum", name);
        }
        SchemaOperation::RenameEnum { from, to } => {
            rename_entity(execution, StudioNodeKind::Enum, from, to)?;
            replace_type_symbol(&mut execution.schema, from, to);
        }
        SchemaOperation::RemoveEnum { name } => {
            remove_entity(execution, StudioNodeKind::Enum, name)?;
        }
        SchemaOperation::AddEnumValue {
            enum_name,
            id,
            name,
            comment,
        } => {
            validate_name(name)?;
            let node = node_mut(&mut execution.schema, StudioNodeKind::Enum, enum_name)?;
            if node.fields.iter().any(|field| field.name == *name) {
                return Err(SchemaMutationError::DuplicateEnumValue {
                    enum_name: enum_name.clone(),
                    value: name.clone(),
                });
            }
            if node
                .fields
                .iter()
                .any(|field| field.enum_value_id == Some(*id))
            {
                return Err(SchemaMutationError::DuplicateEnumValueId {
                    enum_name: enum_name.clone(),
                    id: *id,
                });
            }
            node.fields.push(StudioField {
                name: name.clone(),
                ty: "enum value".to_owned(),
                enum_value_id: Some(*id),
                groups: node.groups.clone(),
                parser: None,
                comment: comment.clone(),
                default: None,
                range: None,
                length: None,
                source: None,
            });
            affect_entity(execution, "enum", enum_name);
            execution.requires_data_migration = true;
        }
        SchemaOperation::RenameEnumValue {
            enum_name,
            from,
            to,
        } => {
            validate_name(to)?;
            let node = node_mut(&mut execution.schema, StudioNodeKind::Enum, enum_name)?;
            if node.fields.iter().any(|field| field.name == *to) {
                return Err(SchemaMutationError::DuplicateEnumValue {
                    enum_name: enum_name.clone(),
                    value: to.clone(),
                });
            }
            let field = node
                .fields
                .iter_mut()
                .find(|field| field.name == *from)
                .ok_or_else(|| SchemaMutationError::UnknownEnumValue {
                    enum_name: enum_name.clone(),
                    value: from.clone(),
                })?;
            field.name = to.clone();
            for alias in &mut node.aliases {
                if alias.name == *from {
                    alias.name = to.clone();
                }
            }
            for owner in &mut execution.schema.nodes {
                for field in &mut owner.fields {
                    if type_mentions_symbol(&field.ty, enum_name)
                        && field.default.as_deref() == Some(from)
                    {
                        field.default = Some(to.clone());
                    }
                }
            }
            affect_entity(execution, "enum", enum_name);
            execution.requires_data_migration = true;
        }
        SchemaOperation::RemoveEnumValue { enum_name, name } => {
            let node = node_mut(&mut execution.schema, StudioNodeKind::Enum, enum_name)?;
            let position = node
                .fields
                .iter()
                .position(|field| field.name == *name)
                .ok_or_else(|| SchemaMutationError::UnknownEnumValue {
                    enum_name: enum_name.clone(),
                    value: name.clone(),
                })?;
            node.fields.remove(position);
            node.aliases.retain(|alias| alias.name != *name);
            affect_entity(execution, "enum", enum_name);
            execution.requires_data_migration = true;
        }
        SchemaOperation::AddStruct {
            name,
            source,
            groups,
            fields,
        } => {
            add_field_entity(
                execution,
                StudioNodeKind::Struct,
                name,
                source,
                groups,
                fields,
                None,
            )?;
        }
        SchemaOperation::RenameStruct { from, to } => {
            rename_entity(execution, StudioNodeKind::Struct, from, to)?;
            replace_type_symbol(&mut execution.schema, from, to);
        }
        SchemaOperation::RemoveStruct { name } => {
            remove_entity(execution, StudioNodeKind::Struct, name)?;
        }
        SchemaOperation::AddUnion {
            name,
            source,
            groups,
            tag,
            variants,
        } => {
            validate_new_entity(&execution.schema, StudioNodeKind::Union, name, source)?;
            let mut fields = Vec::new();
            for variant in variants {
                append_union_variant(&mut fields, name, variant)?;
            }
            execution.schema.nodes.push(StudioNode {
                id: crate::studio::graph::node_id(StudioNodeKind::Union, name),
                name: name.clone(),
                kind: StudioNodeKind::Union,
                source: source.clone(),
                groups: normalize_groups(groups),
                subtitle: String::new(),
                fields,
                aliases: Vec::new(),
                indexes: Vec::new(),
                metadata: BTreeMap::from([("tag".to_owned(), tag.clone())]),
            });
            affect_entity(execution, "union", name);
        }
        SchemaOperation::RenameUnion { from, to } => {
            rename_entity(execution, StudioNodeKind::Union, from, to)?;
            replace_type_symbol(&mut execution.schema, from, to);
        }
        SchemaOperation::RemoveUnion { name } => {
            remove_entity(execution, StudioNodeKind::Union, name)?;
        }
        SchemaOperation::AddUnionVariant {
            union_name,
            variant,
        } => {
            let node = node_mut(&mut execution.schema, StudioNodeKind::Union, union_name)?;
            if union_variant_exists(node, &variant.name) {
                return Err(SchemaMutationError::DuplicateUnionVariant {
                    union_name: union_name.clone(),
                    variant: variant.name.clone(),
                });
            }
            append_union_variant(&mut node.fields, union_name, variant)?;
            affect_entity(execution, "union", union_name);
        }
        SchemaOperation::RenameUnionVariant {
            union_name,
            from,
            to,
        } => {
            validate_name(to)?;
            let node = node_mut(&mut execution.schema, StudioNodeKind::Union, union_name)?;
            if union_variant_exists(node, to) {
                return Err(SchemaMutationError::DuplicateUnionVariant {
                    union_name: union_name.clone(),
                    variant: to.clone(),
                });
            }
            if !union_variant_exists(node, from) {
                return Err(SchemaMutationError::UnknownUnionVariant {
                    union_name: union_name.clone(),
                    variant: from.clone(),
                });
            }
            for field in &mut node.fields {
                if field.ty == "variant" && field.name == *from {
                    field.name = to.clone();
                } else if let Some(name) = field.name.strip_prefix(&format!("{from}.")) {
                    field.name = format!("{to}.{name}");
                }
            }
            affect_entity(execution, "union", union_name);
            execution.requires_data_migration = true;
        }
        SchemaOperation::RemoveUnionVariant {
            union_name,
            variant,
        } => {
            let node = node_mut(&mut execution.schema, StudioNodeKind::Union, union_name)?;
            if !union_variant_exists(node, variant) {
                return Err(SchemaMutationError::UnknownUnionVariant {
                    union_name: union_name.clone(),
                    variant: variant.clone(),
                });
            }
            let prefix = format!("{variant}.");
            node.fields
                .retain(|field| field.name != *variant && !field.name.starts_with(&prefix));
            affect_entity(execution, "union", union_name);
            execution.requires_data_migration = true;
        }
        SchemaOperation::MoveEntitySource { kind, name, source } => {
            ensure_source(&execution.schema, source)?;
            let node_kind = studio_kind(*kind);
            node_mut(&mut execution.schema, node_kind, name)?.source = source.clone();
            affect_entity(execution, entity_kind_name(node_kind), name);
        }
    }
    Ok(())
}

fn add_field_entity(
    execution: &mut SchemaExecution,
    kind: StudioNodeKind,
    name: &str,
    source: &str,
    groups: &[String],
    fields: &[FieldDefinition],
    metadata: Option<BTreeMap<String, String>>,
) -> Result<(), SchemaMutationError> {
    validate_new_entity(&execution.schema, kind, name, source)?;
    execution.schema.nodes.push(StudioNode {
        id: crate::studio::graph::node_id(kind, name),
        name: name.to_owned(),
        kind,
        source: source.to_owned(),
        groups: normalize_groups(groups),
        subtitle: String::new(),
        fields: fields.iter().map(studio_field).collect::<Result<_, _>>()?,
        aliases: Vec::new(),
        indexes: Vec::new(),
        metadata: metadata.unwrap_or_default(),
    });
    affect_entity(execution, entity_kind_name(kind), name);
    Ok(())
}

fn validate_new_entity(
    schema: &StudioSchema,
    kind: StudioNodeKind,
    name: &str,
    source: &str,
) -> Result<(), SchemaMutationError> {
    validate_name(name)?;
    ensure_source(schema, source)?;
    if schema
        .nodes
        .iter()
        .any(|node| node.kind == kind && node.name == name)
    {
        return Err(SchemaMutationError::DuplicateEntity {
            kind: entity_kind_name(kind),
            name: name.to_owned(),
        });
    }
    Ok(())
}

fn ensure_source(schema: &StudioSchema, source: &str) -> Result<(), SchemaMutationError> {
    if schema.sources.iter().any(|candidate| candidate == source) {
        Ok(())
    } else {
        Err(SchemaMutationError::UnknownSource(source.to_owned()))
    }
}

fn rename_entity(
    execution: &mut SchemaExecution,
    kind: StudioNodeKind,
    from: &str,
    to: &str,
) -> Result<(), SchemaMutationError> {
    validate_name(to)?;
    if execution
        .schema
        .nodes
        .iter()
        .any(|node| node.kind == kind && node.name == to)
    {
        return Err(SchemaMutationError::DuplicateEntity {
            kind: entity_kind_name(kind),
            name: to.to_owned(),
        });
    }
    let node = node_mut(&mut execution.schema, kind, from)?;
    node.name = to.to_owned();
    node.id = crate::studio::graph::node_id(kind, to);
    affect_entity(execution, entity_kind_name(kind), from);
    affect_entity(execution, entity_kind_name(kind), to);
    if kind == StudioNodeKind::Table {
        execution.requires_data_migration = true;
    }
    Ok(())
}

fn remove_entity(
    execution: &mut SchemaExecution,
    kind: StudioNodeKind,
    name: &str,
) -> Result<(), SchemaMutationError> {
    let position = execution
        .schema
        .nodes
        .iter()
        .position(|node| node.kind == kind && node.name == name)
        .ok_or_else(|| SchemaMutationError::UnknownEntity {
            kind: entity_kind_name(kind),
            name: name.to_owned(),
        })?;
    execution.schema.nodes.remove(position);
    affect_entity(execution, entity_kind_name(kind), name);
    execution.requires_data_migration = true;
    Ok(())
}

fn rename_field(
    execution: &mut SchemaExecution,
    owner: &FieldOwner,
    from: &str,
    to: &str,
) -> Result<(), SchemaMutationError> {
    validate_name(to)?;
    let from_stored = stored_field_name(owner, from)?;
    let to_stored = stored_field_name(owner, to)?;
    {
        let node = owner_node_mut(&mut execution.schema, owner)?;
        if node.fields.iter().any(|field| field.name == to_stored) {
            return Err(SchemaMutationError::DuplicateField {
                owner: owner_label(owner),
                field: to.to_owned(),
            });
        }
        let field = node
            .fields
            .iter_mut()
            .find(|field| field.name == from_stored)
            .ok_or_else(|| SchemaMutationError::UnknownField {
                owner: owner_label(owner),
                field: from.to_owned(),
            })?;
        field.name = to_stored;
        if owner.kind == FieldOwnerKind::Table {
            if node.metadata.get("key").is_some_and(|key| key == from) {
                node.metadata.insert("key".to_owned(), to.to_owned());
            }
            for index in &mut node.indexes {
                for field in &mut index.fields {
                    if field == from {
                        *field = to.to_owned();
                    }
                }
            }
        }
    }
    if owner.kind == FieldOwnerKind::Table {
        let old_ref = format!("ref<{}.{from}>", owner.name);
        let new_ref = format!("ref<{}.{to}>", owner.name);
        for node in &mut execution.schema.nodes {
            for field in &mut node.fields {
                field.ty = field.ty.replace(&old_ref, &new_ref);
                if let Some(source) = field.source.as_mut() {
                    *source = rename_derived_field(source, &owner.name, from, to);
                }
            }
        }
    }
    affect_owner(execution, owner, true);
    Ok(())
}

fn replace_table_references(schema: &mut StudioSchema, from: &str, to: &str) {
    replace_type_symbol(schema, from, to);
    for node in &mut schema.nodes {
        for field in &mut node.fields {
            if let Some(source) = field.source.as_mut()
                && let Some(rest) = source.strip_prefix(&format!("{from}:"))
            {
                *source = format!("{to}:{rest}");
            }
        }
    }
}

fn replace_type_symbol(schema: &mut StudioSchema, from: &str, to: &str) {
    for node in &mut schema.nodes {
        for field in &mut node.fields {
            field.ty = replace_identifier(&field.ty, from, to);
        }
    }
}

fn replace_identifier(value: &str, from: &str, to: &str) -> String {
    if from.is_empty() {
        return value.to_owned();
    }
    let bytes = value.as_bytes();
    let mut out = String::with_capacity(value.len());
    let mut cursor = 0;
    while let Some(relative) = value[cursor..].find(from) {
        let start = cursor + relative;
        let end = start + from.len();
        let before = start.checked_sub(1).and_then(|index| bytes.get(index));
        let after = bytes.get(end);
        if before.is_none_or(|byte| !is_identifier_byte(*byte))
            && after.is_none_or(|byte| !is_identifier_byte(*byte))
        {
            out.push_str(&value[cursor..start]);
            out.push_str(to);
            cursor = end;
        } else {
            out.push_str(&value[cursor..end]);
            cursor = end;
        }
    }
    out.push_str(&value[cursor..]);
    out
}

fn rename_derived_field(source: &str, table: &str, from: &str, to: &str) -> String {
    let Some((source_table, rest)) = source.split_once(':') else {
        return source.to_owned();
    };
    let (keys, options) = rest.split_once(',').unwrap_or((rest, ""));
    let Some((child, parent)) = keys.trim().split_once(" -> ") else {
        return source.to_owned();
    };
    let child = if source_table.trim() == table && child.trim() == from {
        to
    } else {
        child.trim()
    };
    let parent = if parent.trim() == from {
        to
    } else {
        parent.trim()
    };
    let options = options
        .split(',')
        .filter(|option| !option.trim().is_empty())
        .map(|option| {
            let Some((key, value)) = option.trim().split_once('=') else {
                return option.trim().to_owned();
            };
            let value = if source_table.trim() == table
                && matches!(key, "key" | "field" | "order_by")
                && value == from
            {
                to
            } else {
                value
            };
            format!("{key}={value}")
        })
        .collect::<Vec<_>>();
    let suffix = if options.is_empty() {
        String::new()
    } else {
        format!(", {}", options.join(", "))
    };
    format!("{}: {child} -> {parent}{suffix}", source_table.trim())
}

fn node_mut<'a>(
    schema: &'a mut StudioSchema,
    kind: StudioNodeKind,
    name: &str,
) -> Result<&'a mut StudioNode, SchemaMutationError> {
    schema
        .nodes
        .iter_mut()
        .find(|node| node.kind == kind && node.name == name)
        .ok_or_else(|| SchemaMutationError::UnknownEntity {
            kind: entity_kind_name(kind),
            name: name.to_owned(),
        })
}

fn owner_node_mut<'a>(
    schema: &'a mut StudioSchema,
    owner: &FieldOwner,
) -> Result<&'a mut StudioNode, SchemaMutationError> {
    validate_field_owner(owner)?;
    node_mut(schema, owner_kind(owner.kind), &owner.name)
}

fn field_mut<'a>(
    schema: &'a mut StudioSchema,
    owner: &FieldOwner,
    field: &str,
) -> Result<&'a mut StudioField, SchemaMutationError> {
    let stored = stored_field_name(owner, field)?;
    owner_node_mut(schema, owner)?
        .fields
        .iter_mut()
        .find(|candidate| candidate.name == stored)
        .ok_or_else(|| SchemaMutationError::UnknownField {
            owner: owner_label(owner),
            field: field.to_owned(),
        })
}

fn validate_field_owner(owner: &FieldOwner) -> Result<(), SchemaMutationError> {
    match (owner.kind, owner.variant.as_ref()) {
        (FieldOwnerKind::Union, None) => Err(SchemaMutationError::MissingUnionVariant),
        (FieldOwnerKind::Table | FieldOwnerKind::Struct, Some(_)) => {
            Err(SchemaMutationError::UnexpectedVariant)
        }
        _ => Ok(()),
    }
}

fn stored_field_name(owner: &FieldOwner, field: &str) -> Result<String, SchemaMutationError> {
    validate_name(field)?;
    validate_field_owner(owner)?;
    Ok(match owner.variant.as_deref() {
        Some(variant) => {
            validate_name(variant)?;
            format!("{variant}.{field}")
        }
        None => field.to_owned(),
    })
}

fn studio_field(field: &FieldDefinition) -> Result<StudioField, SchemaMutationError> {
    validate_name(&field.name)?;
    if field.range.is_some_and(|range| range[0] > range[1]) {
        return Err(SchemaMutationError::InvalidRange);
    }
    if field.length.is_some_and(|length| length[0] > length[1]) {
        return Err(SchemaMutationError::InvalidLength);
    }
    Ok(StudioField {
        name: field.name.clone(),
        ty: field.ty.clone(),
        enum_value_id: None,
        groups: normalize_groups(&field.groups),
        parser: field.parser.clone(),
        comment: field.comment.clone(),
        default: field.default.clone(),
        range: field.range,
        length: field.length,
        source: None,
    })
}

fn studio_index(index: &IndexDefinition) -> StudioIndex {
    StudioIndex {
        name: index.name.clone(),
        fields: index.fields.clone(),
        unique: index.unique,
    }
}

fn set_table_source_metadata(
    metadata: &mut BTreeMap<String, String>,
    source: Option<&TableSourceDefinition>,
) {
    metadata.remove("source");
    metadata.remove("format");
    metadata.remove("sheet");
    metadata.remove("sheets");
    if let Some(source) = source {
        metadata.insert("source".to_owned(), source.file.clone());
        if let Some(format) = &source.format {
            metadata.insert("format".to_owned(), format.clone());
        }
        if let Some(sheet) = &source.sheet {
            metadata.insert("sheet".to_owned(), sheet.clone());
        }
        if !source.sheets.is_empty() {
            metadata.insert(
                "sheets".to_owned(),
                serde_json::to_string(&source.sheets).expect("string list serializes"),
            );
        }
    }
}

fn validate_enum_values(
    enum_name: &str,
    values: &[EnumValueDefinition],
) -> Result<(), SchemaMutationError> {
    let mut names = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for value in values {
        validate_name(&value.name)?;
        if !names.insert(&value.name) {
            return Err(SchemaMutationError::DuplicateEnumValue {
                enum_name: enum_name.to_owned(),
                value: value.name.clone(),
            });
        }
        if !ids.insert(value.id) {
            return Err(SchemaMutationError::DuplicateEnumValueId {
                enum_name: enum_name.to_owned(),
                id: value.id,
            });
        }
    }
    Ok(())
}

fn append_union_variant(
    fields: &mut Vec<StudioField>,
    union_name: &str,
    variant: &UnionVariantDefinition,
) -> Result<(), SchemaMutationError> {
    validate_name(&variant.name)?;
    if fields
        .iter()
        .any(|field| field.ty == "variant" && field.name == variant.name)
    {
        return Err(SchemaMutationError::DuplicateUnionVariant {
            union_name: union_name.to_owned(),
            variant: variant.name.clone(),
        });
    }
    fields.push(StudioField {
        name: variant.name.clone(),
        ty: "variant".to_owned(),
        enum_value_id: None,
        groups: normalize_groups(&variant.groups),
        parser: None,
        comment: None,
        default: None,
        range: None,
        length: None,
        source: None,
    });
    for field in &variant.fields {
        let mut field = studio_field(field)?;
        field.name = format!("{}.{}", variant.name, field.name);
        fields.push(field);
    }
    Ok(())
}

fn union_variant_exists(node: &StudioNode, variant: &str) -> bool {
    node.fields
        .iter()
        .any(|field| field.ty == "variant" && field.name == variant)
}

fn format_derived_source(source: &DerivedFieldSource) -> String {
    let mut value = format!(
        "{}: {} -> {}",
        source.table, source.child_key, source.parent_key
    );
    if let Some(map_key) = &source.map_key {
        value.push_str(&format!(", key={map_key}"));
    }
    if let Some(field) = &source.value_field {
        value.push_str(&format!(", field={field}"));
    }
    if let Some(order_by) = &source.order_by {
        value.push_str(&format!(", order_by={order_by}"));
    }
    value
}

fn affect_owner(execution: &mut SchemaExecution, owner: &FieldOwner, data: bool) {
    affect_entity(
        execution,
        match owner.kind {
            FieldOwnerKind::Table => "table",
            FieldOwnerKind::Struct => "struct",
            FieldOwnerKind::Union => "union",
        },
        &owner.name,
    );
    if owner.kind == FieldOwnerKind::Table {
        affect_table(execution, &owner.name, data);
    } else if data {
        execution.requires_data_migration = true;
    }
}

fn affect_table(execution: &mut SchemaExecution, table: &str, data: bool) {
    affect_entity(execution, "table", table);
    execution.affected_tables.insert(table.to_owned());
    execution.requires_data_migration |= data;
}

fn affect_entity(execution: &mut SchemaExecution, kind: &str, name: &str) {
    execution.affected_entities.insert(format!("{kind}:{name}"));
}

fn validate_name(name: &str) -> Result<(), SchemaMutationError> {
    let mut bytes = name.bytes();
    let valid_first = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');
    let valid_rest = bytes.all(is_identifier_byte);
    if valid_first && valid_rest {
        Ok(())
    } else {
        Err(SchemaMutationError::InvalidName(name.to_owned()))
    }
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn normalize_groups(groups: &[String]) -> Vec<String> {
    groups
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn default_union_tag() -> String {
    "type".to_owned()
}

fn owner_kind(kind: FieldOwnerKind) -> StudioNodeKind {
    match kind {
        FieldOwnerKind::Table => StudioNodeKind::Table,
        FieldOwnerKind::Struct => StudioNodeKind::Struct,
        FieldOwnerKind::Union => StudioNodeKind::Union,
    }
}

fn studio_kind(kind: SchemaEntityKind) -> StudioNodeKind {
    match kind {
        SchemaEntityKind::Enum => StudioNodeKind::Enum,
        SchemaEntityKind::Struct => StudioNodeKind::Struct,
        SchemaEntityKind::Union => StudioNodeKind::Union,
        SchemaEntityKind::Table => StudioNodeKind::Table,
    }
}

fn entity_kind_name(kind: StudioNodeKind) -> &'static str {
    match kind {
        StudioNodeKind::Enum => "enum",
        StudioNodeKind::Struct => "struct",
        StudioNodeKind::Union => "union",
        StudioNodeKind::Table => "table",
    }
}

fn owner_label(owner: &FieldOwner) -> String {
    match &owner.variant {
        Some(variant) => format!("union `{}::{variant}`", owner.name),
        None => format!(
            "{} `{}`",
            match owner.kind {
                FieldOwnerKind::Table => "table",
                FieldOwnerKind::Struct => "struct",
                FieldOwnerKind::Union => "union",
            },
            owner.name
        ),
    }
}

fn type_mentions_symbol(ty: &str, symbol: &str) -> bool {
    ty.match_indices(symbol).any(|(start, _)| {
        let end = start + symbol.len();
        let bytes = ty.as_bytes();
        start
            .checked_sub(1)
            .and_then(|index| bytes.get(index))
            .is_none_or(|byte| !is_identifier_byte(*byte))
            && bytes.get(end).is_none_or(|byte| !is_identifier_byte(*byte))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::studio::StudioSummary;

    fn schema() -> StudioSchema {
        StudioSchema {
            project_id: "demo".to_owned(),
            groups: std::collections::BTreeMap::from([("common".to_owned(), true)]),
            views: std::collections::BTreeMap::new(),
            sources: vec!["schema.toml".to_owned()],
            module_namespaces: std::collections::BTreeMap::new(),
            module_imports: std::collections::BTreeMap::new(),
            summary: StudioSummary {
                enums: 0,
                structs: 0,
                unions: 0,
                tables: 1,
                edges: 0,
            },
            nodes: vec![StudioNode {
                id: "table:Item".to_owned(),
                name: "Item".to_owned(),
                kind: StudioNodeKind::Table,
                source: "schema.toml".to_owned(),
                groups: Vec::new(),
                subtitle: String::new(),
                fields: vec![StudioField {
                    name: "id".to_owned(),
                    ty: "i32".to_owned(),
                    enum_value_id: None,
                    groups: Vec::new(),
                    parser: None,
                    comment: None,
                    default: None,
                    range: None,
                    length: None,
                    source: None,
                }],
                aliases: Vec::new(),
                indexes: vec![StudioIndex {
                    name: "by_id".to_owned(),
                    fields: vec!["id".to_owned()],
                    unique: true,
                }],
                metadata: BTreeMap::from([
                    ("mode".to_owned(), "map".to_owned()),
                    ("key".to_owned(), "id".to_owned()),
                ]),
            }],
            edges: Vec::new(),
        }
    }

    #[test]
    fn rename_field_updates_key_indexes_and_references() {
        let mut base = schema();
        base.nodes.push(StudioNode {
            id: "table:Drop".to_owned(),
            name: "Drop".to_owned(),
            kind: StudioNodeKind::Table,
            source: "schema.toml".to_owned(),
            groups: Vec::new(),
            subtitle: String::new(),
            fields: vec![StudioField {
                name: "item".to_owned(),
                ty: "ref<Item.id>".to_owned(),
                enum_value_id: None,
                groups: Vec::new(),
                parser: None,
                comment: None,
                default: None,
                range: None,
                length: None,
                source: None,
            }],
            aliases: Vec::new(),
            indexes: Vec::new(),
            metadata: BTreeMap::from([("mode".to_owned(), "list".to_owned())]),
        });
        let result = execute_schema_operations(
            &base,
            &[SchemaOperation::RenameField {
                owner: FieldOwner {
                    kind: FieldOwnerKind::Table,
                    name: "Item".to_owned(),
                    variant: None,
                },
                from: "id".to_owned(),
                to: "item_id".to_owned(),
            }],
        )
        .unwrap();
        let item = result
            .schema
            .nodes
            .iter()
            .find(|node| node.name == "Item")
            .unwrap();
        assert_eq!(item.metadata["key"], "item_id");
        assert_eq!(item.indexes[0].fields, ["item_id"]);
        let drop_table = result
            .schema
            .nodes
            .iter()
            .find(|node| node.name == "Drop")
            .unwrap();
        assert_eq!(drop_table.fields[0].ty, "ref<Item.item_id>");
    }

    #[test]
    fn enum_value_ids_remain_stable_across_rename() {
        let base = execute_schema_operations(
            &schema(),
            &[SchemaOperation::AddEnum {
                name: "Kind".to_owned(),
                source: "schema.toml".to_owned(),
                comment: Some("Kind documentation".to_owned()),
                groups: Vec::new(),
                values: vec![EnumValueDefinition {
                    id: 7,
                    name: "Old".to_owned(),
                    comment: Some("Value documentation".to_owned()),
                }],
            }],
        )
        .unwrap()
        .schema;
        let next = execute_schema_operations(
            &base,
            &[SchemaOperation::RenameEnumValue {
                enum_name: "Kind".to_owned(),
                from: "Old".to_owned(),
                to: "New".to_owned(),
            }],
        )
        .unwrap();
        let value = &next
            .schema
            .nodes
            .iter()
            .find(|node| node.name == "Kind")
            .unwrap()
            .fields[0];
        assert_eq!(value.name, "New");
        assert_eq!(value.enum_value_id, Some(7));
        assert_eq!(value.comment.as_deref(), Some("Value documentation"));
        assert_eq!(
            next.schema
                .nodes
                .iter()
                .find(|node| node.name == "Kind")
                .unwrap()
                .metadata
                .get("comment")
                .map(String::as_str),
            Some("Kind documentation")
        );
    }

    #[test]
    fn every_schema_operation_variant_executes_in_an_ordered_batch() {
        let mut base = schema();
        base.sources.push("other.toml".to_owned());
        let table_owner = |name: &str| FieldOwner {
            kind: FieldOwnerKind::Table,
            name: name.to_owned(),
            variant: None,
        };
        let field = |name: &str, ty: &str| FieldDefinition {
            name: name.to_owned(),
            ty: ty.to_owned(),
            groups: Vec::new(),
            parser: None,
            comment: None,
            default: None,
            range: None,
            length: None,
        };
        let operations = vec![
            SchemaOperation::AddTable {
                id: "temp".to_owned(),
                name: "Temp".to_owned(),
                source: "schema.toml".to_owned(),
                mode: SchemaTableMode::List,
                key: None,
                table_source: None,
                groups: Vec::new(),
                fields: vec![field("id", "i32")],
                indexes: Vec::new(),
            },
            SchemaOperation::SetTableMode {
                table: "Temp".to_owned(),
                mode: SchemaTableMode::Map,
            },
            SchemaOperation::SetTableKey {
                table: "Temp".to_owned(),
                key: Some("id".to_owned()),
            },
            SchemaOperation::SetTableSource {
                table: "Temp".to_owned(),
                source: Some(TableSourceDefinition {
                    file: "Temp.csv".to_owned(),
                    format: Some("csv".to_owned()),
                    sheet: None,
                    sheets: Vec::new(),
                }),
            },
            SchemaOperation::SetTableGroups {
                table: "Temp".to_owned(),
                groups: vec!["client".to_owned()],
            },
            SchemaOperation::AddField {
                owner: table_owner("Temp"),
                field: field("name", "string"),
            },
            SchemaOperation::RenameField {
                owner: table_owner("Temp"),
                from: "name".to_owned(),
                to: "title".to_owned(),
            },
            SchemaOperation::ChangeFieldType {
                owner: table_owner("Temp"),
                field: "title".to_owned(),
                ty: "optional<string>".to_owned(),
            },
            SchemaOperation::SetFieldDefault {
                owner: table_owner("Temp"),
                field: "title".to_owned(),
                default: Some("unknown".to_owned()),
            },
            SchemaOperation::SetFieldRange {
                owner: table_owner("Temp"),
                field: "id".to_owned(),
                range: Some([1, 100]),
            },
            SchemaOperation::SetFieldLength {
                owner: table_owner("Temp"),
                field: "title".to_owned(),
                length: Some([1, 64]),
            },
            SchemaOperation::SetFieldParser {
                owner: table_owner("Temp"),
                field: "title".to_owned(),
                parser: Some("split (separator=\",\")".to_owned()),
            },
            SchemaOperation::SetFieldGroups {
                owner: table_owner("Temp"),
                field: "title".to_owned(),
                groups: vec!["client".to_owned()],
            },
            SchemaOperation::SetFieldReference {
                owner: table_owner("Temp"),
                field: "title".to_owned(),
                table: "Item".to_owned(),
                target_field: "id".to_owned(),
                optional: true,
            },
            SchemaOperation::SetFieldDerivedFrom {
                table: "Temp".to_owned(),
                field: "title".to_owned(),
                source: Some(DerivedFieldSource {
                    table: "Item".to_owned(),
                    parent_key: "id".to_owned(),
                    child_key: "id".to_owned(),
                    map_key: None,
                    value_field: Some("id".to_owned()),
                    order_by: None,
                }),
            },
            SchemaOperation::AddIndex {
                table: "Temp".to_owned(),
                index: IndexDefinition {
                    name: "by_title".to_owned(),
                    fields: vec!["title".to_owned()],
                    unique: false,
                },
            },
            SchemaOperation::RemoveIndex {
                table: "Temp".to_owned(),
                index: "by_title".to_owned(),
            },
            SchemaOperation::RemoveField {
                owner: table_owner("Temp"),
                field: "title".to_owned(),
            },
            SchemaOperation::RenameTable {
                from: "Temp".to_owned(),
                to: "TempRenamed".to_owned(),
            },
            SchemaOperation::RemoveTable {
                name: "TempRenamed".to_owned(),
            },
            SchemaOperation::AddEnum {
                name: "Kind".to_owned(),
                source: "schema.toml".to_owned(),
                comment: None,
                groups: Vec::new(),
                values: vec![EnumValueDefinition {
                    id: 1,
                    name: "One".to_owned(),
                    comment: None,
                }],
            },
            SchemaOperation::AddEnumValue {
                enum_name: "Kind".to_owned(),
                id: 2,
                name: "Two".to_owned(),
                comment: None,
            },
            SchemaOperation::RenameEnumValue {
                enum_name: "Kind".to_owned(),
                from: "Two".to_owned(),
                to: "Second".to_owned(),
            },
            SchemaOperation::RemoveEnumValue {
                enum_name: "Kind".to_owned(),
                name: "Second".to_owned(),
            },
            SchemaOperation::RenameEnum {
                from: "Kind".to_owned(),
                to: "ItemKind".to_owned(),
            },
            SchemaOperation::RemoveEnum {
                name: "ItemKind".to_owned(),
            },
            SchemaOperation::AddStruct {
                name: "Cost".to_owned(),
                source: "schema.toml".to_owned(),
                groups: Vec::new(),
                fields: vec![field("amount", "i32")],
            },
            SchemaOperation::RenameStruct {
                from: "Cost".to_owned(),
                to: "Price".to_owned(),
            },
            SchemaOperation::RemoveStruct {
                name: "Price".to_owned(),
            },
            SchemaOperation::AddUnion {
                name: "Reward".to_owned(),
                source: "schema.toml".to_owned(),
                groups: Vec::new(),
                tag: "kind".to_owned(),
                variants: vec![UnionVariantDefinition {
                    name: "None".to_owned(),
                    groups: Vec::new(),
                    fields: Vec::new(),
                }],
            },
            SchemaOperation::AddUnionVariant {
                union_name: "Reward".to_owned(),
                variant: UnionVariantDefinition {
                    name: "Item".to_owned(),
                    groups: Vec::new(),
                    fields: vec![field("id", "i32")],
                },
            },
            SchemaOperation::RenameUnionVariant {
                union_name: "Reward".to_owned(),
                from: "Item".to_owned(),
                to: "Product".to_owned(),
            },
            SchemaOperation::RemoveUnionVariant {
                union_name: "Reward".to_owned(),
                variant: "Product".to_owned(),
            },
            SchemaOperation::RenameUnion {
                from: "Reward".to_owned(),
                to: "Grant".to_owned(),
            },
            SchemaOperation::RemoveUnion {
                name: "Grant".to_owned(),
            },
            SchemaOperation::MoveEntitySource {
                kind: SchemaEntityKind::Table,
                name: "Item".to_owned(),
                source: "other.toml".to_owned(),
            },
        ];

        let result = execute_schema_operations(&base, &operations).unwrap();

        assert_eq!(
            result
                .schema
                .nodes
                .iter()
                .find(|node| node.name == "Item")
                .unwrap()
                .source,
            "other.toml"
        );
        assert!(result.requires_data_migration);
    }
}
