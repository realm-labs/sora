use std::sync::Arc;

use sora_ir::model::{ConfigIr, TypeIr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeMapping {
    pub type_name: String,
    pub decode: Option<String>,
    pub value_decode: Option<String>,
    pub imports: Vec<String>,
}

impl TypeMapping {
    pub fn wrap_decode(&self, base_expr: &str) -> String {
        wrap_expr(self.decode.as_deref(), base_expr)
    }

    pub fn wrap_value_decode(&self, base_expr: &str) -> String {
        wrap_expr(self.value_decode.as_deref(), base_expr)
    }
}

fn wrap_expr(template: Option<&str>, base_expr: &str) -> String {
    template
        .map(|template| template.replace("{value}", base_expr))
        .unwrap_or_else(|| base_expr.to_owned())
}

#[derive(Debug, Clone, Copy)]
pub struct TypeMappingContext<'a> {
    pub target: &'a str,
    pub ir: &'a ConfigIr,
    pub ty: &'a TypeIr,
}

impl TypeMappingContext<'_> {
    pub fn named_type(&self) -> Option<&str> {
        match self.ty {
            TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => Some(name),
            _ => None,
        }
    }
}

pub trait TypeMappingProvider: Send + Sync {
    fn map_type(&self, context: TypeMappingContext<'_>) -> Option<TypeMapping>;
}

#[derive(Default)]
pub struct TypeMappingRegistry {
    providers: Vec<Arc<dyn TypeMappingProvider>>,
}

impl TypeMappingRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_provider(mut self, provider: impl TypeMappingProvider + 'static) -> Self {
        self.register(provider);
        self
    }

    pub fn register(&mut self, provider: impl TypeMappingProvider + 'static) {
        self.providers.push(Arc::new(provider));
    }

    pub fn register_arc(&mut self, provider: Arc<dyn TypeMappingProvider>) {
        self.providers.push(provider);
    }

    pub fn map_type(&self, context: TypeMappingContext<'_>) -> Option<TypeMapping> {
        self.providers
            .iter()
            .find_map(|provider| provider.map_type(context))
    }

    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    pub fn imports_for(&self, target: &str, ir: &ConfigIr, ty: &TypeIr) -> Vec<String> {
        let mut imports = Vec::new();
        self.collect_imports(target, ir, ty, &mut imports);
        imports.sort();
        imports.dedup();
        imports
    }

    fn collect_imports(&self, target: &str, ir: &ConfigIr, ty: &TypeIr, imports: &mut Vec<String>) {
        if let Some(mapping) = self.map_type(TypeMappingContext { target, ir, ty }) {
            imports.extend(mapping.imports);
            return;
        }

        match ty {
            TypeIr::List(element)
            | TypeIr::Set(element)
            | TypeIr::Array { element, .. }
            | TypeIr::Optional(element) => self.collect_imports(target, ir, element, imports),
            TypeIr::Map { key, value } => {
                self.collect_imports(target, ir, key, imports);
                self.collect_imports(target, ir, value, imports);
            }
            TypeIr::Ref { table, field } => {
                if let Some(ty) = ir
                    .tables
                    .iter()
                    .find(|candidate| candidate.name == *table)
                    .and_then(|table| {
                        table
                            .fields
                            .iter()
                            .find(|candidate| candidate.name == *field)
                    })
                    .map(|field| &field.ty)
                {
                    self.collect_imports(target, ir, ty, imports);
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticTypeMappingRule {
    pub target: String,
    pub schema_type: String,
    pub type_name: String,
    pub decode: Option<String>,
    pub value_decode: Option<String>,
    pub imports: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct StaticTypeMappingProvider {
    rules: Vec<StaticTypeMappingRule>,
}

impl StaticTypeMappingProvider {
    pub fn new(rules: Vec<StaticTypeMappingRule>) -> Self {
        Self { rules }
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

impl TypeMappingProvider for StaticTypeMappingProvider {
    fn map_type(&self, context: TypeMappingContext<'_>) -> Option<TypeMapping> {
        let schema_type = context.named_type()?;
        self.rules
            .iter()
            .find(|rule| rule.target == context.target && rule.schema_type == schema_type)
            .map(|rule| TypeMapping {
                type_name: rule.type_name.clone(),
                decode: rule.decode.clone(),
                value_decode: rule.value_decode.clone(),
                imports: rule.imports.clone(),
            })
    }
}
