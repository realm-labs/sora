use anyhow::{Context, Result, bail};
use sora_excel::sync::ExcelSyncReport;
use sora_execution::ExecutionOptions;
use sora_export::exporter::{ExportCompression, ExportOptions, ExportOutput, OutputKind};
use sora_input_schema::input::SchemaFileInput;
use sora_workspace::{ProjectRuntime, RuntimeOptions, source::MixedProjectInput};
use std::sync::Arc;

use crate::args::{
    CheckArgs, Cli, Command, DiffArgs, ExcelSyncArgs, ExcelTemplateArgs, ExportArgs,
    ExportCompressionArg, GenArgs, SchemaLockArgs, SourceFormatArg,
};
pub fn run(cli: Cli) -> Result<()> {
    if cli.jobs == Some(0) {
        bail!("--jobs must be greater than 0");
    }

    let runtime_options = RuntimeOptions {
        execution: ExecutionOptions {
            parallel: !cli.serial,
            jobs: cli.jobs,
        },
        parser_scripts: cli.parser_script,
        type_mapping_scripts: cli.type_mapping_script,
    };

    let command = match cli.command {
        Command::Init(args) => return crate::init::run(args),
        Command::Mcp => return crate::mcp::run(),
        Command::Studio(args) => return crate::studio::run(args, runtime_options),
        command => command,
    };
    let project = command_project_path(&command)
        .expect("project commands must declare a manifest path")
        .to_path_buf();
    let context = ProjectRuntime::load(Some(&project), runtime_options)?;

    match command {
        Command::Build(args) => crate::build::run(args, &context),
        Command::Check(args) => check(args, &context),
        Command::Gen { target, args } => generate(args, &target, &context),
        Command::Export(args) => export(args, &context),
        Command::Diff(args) => diff(args, &context),
        Command::ExcelTemplate(args) => excel_template(args, &context),
        Command::ExcelSync(args) => excel_sync(args, &context),
        Command::SchemaLock(args) => schema_lock(args, &context),
        Command::Init(_) | Command::Studio(_) | Command::Mcp => {
            unreachable!("commands without project runtimes were dispatched earlier")
        }
    }
}

fn command_project_path(command: &Command) -> Option<&std::path::Path> {
    match command {
        Command::Build(args) => Some(&args.project),
        Command::Check(args) => Some(&args.project),
        Command::Init(_) => None,
        Command::Gen { args, .. } => Some(&args.project),
        Command::Export(args) => Some(&args.project),
        Command::Diff(args) => Some(&args.project),
        Command::ExcelTemplate(args) => Some(&args.project),
        Command::ExcelSync(args) => Some(&args.project),
        Command::SchemaLock(args) => Some(&args.project),
        Command::Studio(args) => Some(&args.project),
        Command::Mcp => None,
    }
}

fn check(args: CheckArgs, context: &ProjectRuntime) -> Result<()> {
    let input = SchemaFileInput::new(&args.project);
    match &args.lock {
        Some(lock) => sora_core::pipeline::check_schema_with_lock_and_parsers(
            &input,
            lock,
            context.schema_parsers(),
        )
        .with_context(|| {
            format!(
                "failed to check project `{}` against lock `{}`",
                args.project.display(),
                lock.display()
            )
        }),
        None => sora_core::pipeline::check_schema_with_parsers(&input, context.schema_parsers())
            .with_context(|| format!("failed to check project `{}`", args.project.display())),
    }
}

fn generate(args: GenArgs, target: &str, context: &ProjectRuntime) -> Result<()> {
    let input = SchemaFileInput::new(&args.project);
    sora_core::pipeline::generate_code_with_scope_format_and_parsers(
        &input,
        target,
        &args.out,
        args.format_code.into(),
        args.scope.as_deref(),
        context.schema_parsers(),
        context.type_mappings(),
    )
    .with_context(|| {
        format!(
            "failed to generate code from `{}` into `{}`",
            args.project.display(),
            args.out.display()
        )
    })
}

fn excel_template(args: ExcelTemplateArgs, context: &ProjectRuntime) -> Result<()> {
    let input = SchemaFileInput::new(&args.project);
    sora_core::pipeline::generate_excel_template_with_scope_and_parsers(
        &input,
        &args.out,
        args.scope.as_deref(),
        context.schema_parsers(),
    )
    .with_context(|| {
        format!(
            "failed to generate Excel templates from `{}`",
            args.project.display()
        )
    })
}

fn excel_sync(args: ExcelSyncArgs, context: &ProjectRuntime) -> Result<()> {
    let input = SchemaFileInput::new(&args.project);
    let report = if args.write {
        sora_core::pipeline::write_excel_sync_with_parsers(
            &input,
            &args.data_root,
            args.scope.as_deref(),
            context.schema_parsers(),
        )
    } else {
        sora_core::pipeline::preview_excel_sync_with_parsers(
            &input,
            &args.data_root,
            args.scope.as_deref(),
            context.schema_parsers(),
        )
    }
    .with_context(|| {
        format!(
            "failed to sync Excel headers from `{}` into `{}`",
            args.project.display(),
            args.data_root.display()
        )
    })?;
    print_excel_sync_report(&report, args.write);
    if !args.write {
        println!("Preview only. Re-run with --write to update workbooks.");
    }
    Ok(())
}

fn print_excel_sync_report(report: &ExcelSyncReport, write: bool) {
    if report.is_empty() {
        println!("No xlsx tables to sync.");
        return;
    }

    for workbook in &report.workbooks {
        let action = if workbook.created {
            "create"
        } else if workbook.written {
            "update"
        } else if write {
            "up-to-date"
        } else {
            "preview"
        };
        println!("{action}: {}", workbook.path.display());
        if let Some(backup_path) = &workbook.backup_path {
            println!("  backup: {}", backup_path.display());
        }
        for sheet in &workbook.sheets {
            let sheet_action = if sheet.created {
                "create sheet"
            } else if sheet.changed {
                "sync sheet"
            } else {
                "unchanged sheet"
            };
            println!(
                "  {sheet_action}: {} ({} data rows)",
                sheet.sheet, sheet.rows
            );
            if !sheet.added_columns.is_empty() {
                println!("    add columns: {}", sheet.added_columns.join(", "));
            }
            if !sheet.legacy_columns.is_empty() {
                println!(
                    "    keep legacy columns ignored by schema: {}",
                    sheet.legacy_columns.join(", ")
                );
            }
        }
        if !workbook.preserved_sheets.is_empty() {
            println!(
                "  preserve non-schema sheets: {}",
                workbook.preserved_sheets.join(", ")
            );
        }
    }
}

fn schema_lock(args: SchemaLockArgs, context: &ProjectRuntime) -> Result<()> {
    let input = SchemaFileInput::new(&args.project);
    sora_core::pipeline::generate_schema_lock_with_scope_and_parsers(
        &input,
        &args.out,
        args.scope.as_deref(),
        context.schema_parsers(),
    )
    .with_context(|| {
        format!(
            "failed to generate schema lock from `{}` into `{}`",
            args.project.display(),
            args.out.display()
        )
    })
}

fn export(args: ExportArgs, context: &ProjectRuntime) -> Result<()> {
    let options = export_options(args.compression, args.compression_level)?;
    let format = args.format.as_str();
    if matches!(options.compression, ExportCompression::Zstd { .. }) && format != "binary" {
        bail!("export compression `zstd` is only supported by `binary` exports, got `{format}`");
    }

    let output = match sora_core::pipeline::export_output_kind(format) {
        Some(OutputKind::File) => ExportOutput::File(args.out.clone()),
        Some(OutputKind::Directory) => ExportOutput::Directory(args.out.clone()),
        None => {
            bail!(
                "unknown export format `{}`; supported formats: {}",
                format,
                sora_core::pipeline::supported_export_formats().join(", ")
            );
        }
    };

    let schema_input = SchemaFileInput::new(&args.project);
    let input = MixedProjectInput::with_parser_registry(
        schema_input,
        &args.data_root,
        args.default_source_format.map(SourceFormatArg::as_str),
        Arc::clone(context.cell_parsers()),
    );
    let (ir, data) = sora_core::pipeline::load_project_data_with_context_and_parsers(
        &input,
        context.execution(),
        context.schema_parsers(),
        context.cell_parsers(),
    )?;
    sora_core::pipeline::export_loaded_data_with_scope_context_and_options(
        &ir,
        &data,
        format,
        output,
        args.scope.as_deref(),
        context.execution(),
        options,
    )
    .with_context(|| {
        format!(
            "failed to export `{}` data from `{}`",
            args.format,
            args.data_root.display()
        )
    })
}

fn export_options(
    compression: ExportCompressionArg,
    compression_level: Option<i32>,
) -> Result<ExportOptions> {
    let compression = match compression {
        ExportCompressionArg::None => ExportCompression::None,
        ExportCompressionArg::Zstd => ExportCompression::Zstd {
            level: compression_level.unwrap_or(3),
        },
    };
    Ok(ExportOptions {
        compression,
        locale: None,
    })
}

fn diff(args: DiffArgs, context: &ProjectRuntime) -> Result<()> {
    let default_source_format = args.default_source_format.map(SourceFormatArg::as_str);
    let left_schema = SchemaFileInput::new(&args.project);
    let right_schema = SchemaFileInput::new(&args.project);
    let parser_registry = Arc::clone(context.cell_parsers());
    let left = MixedProjectInput::with_parser_registry(
        left_schema,
        &args.left_root,
        default_source_format,
        Arc::clone(&parser_registry),
    );
    let right = MixedProjectInput::with_parser_registry(
        right_schema,
        &args.right_root,
        default_source_format,
        parser_registry,
    );
    sora_core::pipeline::diff_data_with_scope_context_and_parsers(
        &left,
        &right,
        &args.out,
        args.scope.as_deref(),
        context.execution(),
        context.schema_parsers(),
        context.cell_parsers(),
    )
    .with_context(|| {
        format!(
            "failed to diff `{}` against `{}` into `{}`",
            args.left_root.display(),
            args.right_root.display(),
            args.out.display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn excel_sync_command_preview_is_readonly_and_write_updates_workbook() {
        let base = temp_dir();
        let project = write_xlsx_project(&base, false);
        let data_root = base.join("data");
        let context = test_context();

        excel_sync(
            ExcelSyncArgs {
                project: project.clone(),
                data_root: data_root.clone(),
                scope: None,
                write: false,
            },
            &context,
        )
        .unwrap();
        assert!(!data_root.join("Item.xlsx").exists());

        excel_sync(
            ExcelSyncArgs {
                project: project.clone(),
                data_root: data_root.clone(),
                scope: None,
                write: true,
            },
            &context,
        )
        .unwrap();
        assert!(data_root.join("Item.xlsx").exists());

        write_xlsx_project(&base, true);
        excel_sync(
            ExcelSyncArgs {
                project: project.clone(),
                data_root: data_root.clone(),
                scope: None,
                write: true,
            },
            &context,
        )
        .unwrap();
        assert!(data_root.join(".sora-backup").exists());

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn lua_parser_script_extends_check_and_export() {
        let base = temp_dir();
        let project = write_csv_project_with_lua_parser(&base, false);
        let parser_script = write_scale_parser(&base);
        let default_context = test_context();

        let error = check(
            CheckArgs {
                project: project.clone(),
                lock: None,
            },
            &default_context,
        )
        .unwrap_err();
        assert!(format!("{error:#}").contains("scale"), "{error:#}");

        let context = context_with_parser_script(&parser_script);
        check(
            CheckArgs {
                project: project.clone(),
                lock: None,
            },
            &context,
        )
        .unwrap();
        let out = base.join("config.json");
        export(
            ExportArgs {
                format: "json".to_owned(),
                default_source_format: None,
                project: project.clone(),
                data_root: base.join("data"),
                out: out.clone(),
                scope: None,
                compression: ExportCompressionArg::None,
                compression_level: None,
            },
            &context,
        )
        .unwrap();

        let json = fs::read_to_string(out).unwrap();
        assert!(json.contains(r#""cooldown": 3000"#));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn project_parser_script_extends_check_and_export() {
        let base = temp_dir();
        let project = write_csv_project_with_lua_parser(&base, true);
        write_scale_parser(&base);
        let context = context_for_project(&project);

        check(
            CheckArgs {
                project: project.clone(),
                lock: None,
            },
            &context,
        )
        .unwrap();
        let out = base.join("config.json");
        export(
            ExportArgs {
                format: "json".to_owned(),
                default_source_format: None,
                project: project.clone(),
                data_root: base.join("data"),
                out: out.clone(),
                scope: None,
                compression: ExportCompressionArg::None,
                compression_level: None,
            },
            &context,
        )
        .unwrap();

        let json = fs::read_to_string(out).unwrap();
        assert!(json.contains(r#""cooldown": 3000"#));

        let _ = fs::remove_dir_all(base);
    }

    fn write_xlsx_project(base: &Path, include_name: bool) -> PathBuf {
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project = base.join("project.toml");
        fs::write(
            &project,
            r#"
package = "game_config"
includes = ["schema/items.toml"]
"#,
        )
        .unwrap();
        let name_field = if include_name {
            r#"
[[tables.fields]]
name = "name"
type = "string"
"#
        } else {
            ""
        };
        fs::write(
            schema_dir.join("items.toml"),
            format!(
                r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"

[tables.source]
file = "Item.xlsx"

[[tables.fields]]
name = "id"
type = "i32"
{name_field}
"#
            ),
        )
        .unwrap();
        project
    }

    fn write_csv_project_with_lua_parser(base: &Path, project_script: bool) -> PathBuf {
        let schema_dir = base.join("schema");
        let data_dir = base.join("data");
        fs::create_dir_all(&schema_dir).unwrap();
        fs::create_dir_all(&data_dir).unwrap();
        let project = base.join("project.toml");
        let parser_config = if project_script {
            r#"
[parsers]
scripts = ["tools/parsers.lua"]
"#
        } else {
            ""
        };
        fs::write(
            &project,
            format!(
                r#"
package = "game_config"
includes = ["schema/items.toml"]
{parser_config}
"#,
            ),
        )
        .unwrap();
        fs::write(
            schema_dir.join("items.toml"),
            r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"

[tables.source]
file = "Item.csv"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "cooldown"
type = "i32"
parser = { kind = "scale", factor = "1000" }
"#,
        )
        .unwrap();
        fs::write(data_dir.join("Item.csv"), "id,cooldown\n1001,3\n").unwrap();
        project
    }

    fn write_scale_parser(base: &Path) -> PathBuf {
        let tools = base.join("tools");
        fs::create_dir_all(&tools).unwrap();
        let path = tools.join("parsers.lua");
        fs::write(
            &path,
            r#"
return {
  parsers = {
    scale = {
      options = { "factor" },
      validate = function(field)
        if field.type ~= "i32" then
          error("scale parser requires i32")
        end
      end,
      parse = function(cell, ctx)
        local value = tonumber(cell.text)
        if value == nil then
          error("expected number")
        end
        return value * tonumber(ctx.options.factor or "1")
      end,
    },
  },
}
"#,
        )
        .unwrap();
        path
    }

    fn temp_dir() -> PathBuf {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "sora-cli-commands-test-{}-{id}",
            std::process::id()
        ))
    }

    fn test_context() -> ProjectRuntime {
        ProjectRuntime::load(None, RuntimeOptions::default()).unwrap()
    }

    fn context_with_parser_script(path: &Path) -> ProjectRuntime {
        ProjectRuntime::load(
            None,
            RuntimeOptions {
                parser_scripts: vec![path.to_path_buf()],
                ..RuntimeOptions::default()
            },
        )
        .unwrap()
    }

    fn context_for_project(project: &Path) -> ProjectRuntime {
        ProjectRuntime::load(Some(project), RuntimeOptions::default()).unwrap()
    }
}
