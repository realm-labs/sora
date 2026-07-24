use std::collections::BTreeSet;

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::Serialize;
use sora_input::traits::SchemaInput;
use sora_input_schema::input::SchemaFileInput;
use sora_ir::model::ConfigIr;

use crate::{Diagnostic, ProjectRevision, ProjectSession, diagnostics_from_anyhow};

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProjectInspection {
    pub project_id: crate::ProjectId,
    pub package: String,
    pub schema_sources: Vec<String>,
    pub data_sources: Vec<ProjectDataSource>,
    pub scopes: Vec<String>,
    pub build_outputs: Vec<ProjectBuildOutput>,
    pub codegen_targets: Vec<String>,
    pub export_formats: Vec<String>,
    pub localization: Option<ProjectLocalization>,
    pub revision: ProjectRevision,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProjectDataSource {
    pub table: String,
    pub file: String,
    pub format: Option<String>,
    pub sheet: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProjectBuildOutput {
    pub kind: String,
    pub name: String,
    pub path: String,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProjectLocalization {
    pub locales: Vec<String>,
    pub default_locale: String,
    pub fallback_locale: Option<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ValidationReport {
    pub ok: bool,
    pub revision: ProjectRevision,
    pub diagnostics: Vec<Diagnostic>,
}

impl ProjectSession {
    pub fn normalized_schema(&self) -> Result<ConfigIr> {
        let input = SchemaFileInput::new(self.manifest_path());
        let schema = input.load_schema()?;
        let ir = sora_ir::normalize::normalize_schema_with_parsers(
            schema,
            self.runtime().schema_parsers(),
        )
        .with_context(|| {
            format!(
                "failed to normalize project `{}`",
                self.manifest_path().display()
            )
        })?;
        sora_ir::validate::validate_config_ir(&ir).with_context(|| {
            format!(
                "failed to validate project `{}`",
                self.manifest_path().display()
            )
        })?;
        Ok(ir)
    }

    pub fn validate_schema(&self) -> ValidationReport {
        match self.normalized_schema() {
            Ok(_) => ValidationReport {
                ok: true,
                revision: self.revision(),
                diagnostics: Vec::new(),
            },
            Err(error) => ValidationReport {
                ok: false,
                revision: self.revision(),
                diagnostics: diagnostics_from_anyhow(&error),
            },
        }
    }

    pub fn inspect(&self) -> Result<ProjectInspection> {
        let ir = self.normalized_schema()?;
        let mut scopes = BTreeSet::new();
        for scope in ir
            .enums
            .iter()
            .map(|item| item.scope.display())
            .chain(ir.structs.iter().map(|item| item.scope.display()))
            .chain(ir.unions.iter().map(|item| item.scope.display()))
            .chain(ir.tables.iter().map(|item| item.scope.display()))
        {
            scopes.insert(scope);
        }
        let data_sources = ir
            .tables
            .iter()
            .filter_map(|table| {
                table.source.as_ref().map(|source| ProjectDataSource {
                    table: table.name.clone(),
                    file: source.file.clone(),
                    format: source.format.clone(),
                    sheet: source.sheet.clone(),
                })
            })
            .collect();
        let build = &self.manifest().build;
        let mut build_outputs = Vec::new();
        if let Some(path) = &build.schema_lock {
            build_outputs.push(ProjectBuildOutput {
                kind: "schema_lock".to_owned(),
                name: "schema_lock".to_owned(),
                path: path.to_string_lossy().into_owned(),
                scope: build.scope.clone(),
            });
        }
        if let Some(path) = &build.excel_templates {
            build_outputs.push(ProjectBuildOutput {
                kind: "excel_templates".to_owned(),
                name: "excel_templates".to_owned(),
                path: path.to_string_lossy().into_owned(),
                scope: build.scope.clone(),
            });
        }
        for item in &build.codegen {
            build_outputs.push(ProjectBuildOutput {
                kind: "codegen".to_owned(),
                name: item.target.clone(),
                path: item.out.to_string_lossy().into_owned(),
                scope: item.scope.clone().or_else(|| build.scope.clone()),
            });
        }
        for item in &build.exports {
            build_outputs.push(ProjectBuildOutput {
                kind: "export".to_owned(),
                name: item.format.clone(),
                path: item.out.to_string_lossy().into_owned(),
                scope: item.scope.clone().or_else(|| build.scope.clone()),
            });
        }
        let codegen_targets = build
            .codegen
            .iter()
            .map(|item| item.target.clone())
            .collect();
        let export_formats = build
            .exports
            .iter()
            .map(|item| item.format.clone())
            .collect();
        let localization = ir
            .localization
            .as_ref()
            .map(|localization| ProjectLocalization {
                locales: localization.locales.clone(),
                default_locale: localization.default_locale.clone(),
                fallback_locale: localization.fallback_locale.clone(),
                sources: localization
                    .sources
                    .iter()
                    .map(|source| source.name.clone())
                    .collect(),
            });
        Ok(ProjectInspection {
            project_id: self.id().clone(),
            package: ir.package,
            schema_sources: self.manifest().includes.clone(),
            data_sources,
            scopes: scopes.into_iter().collect(),
            build_outputs,
            codegen_targets,
            export_formats,
            localization,
            revision: self.revision(),
        })
    }
}
