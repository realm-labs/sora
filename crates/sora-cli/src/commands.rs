use anyhow::{Context, Result, bail};
use sora_excel::sync::ExcelSyncReport;
use sora_execution::{ExecutionContext, ExecutionOptions};
use sora_export::exporter::{ExportCompression, ExportOptions, ExportOutput, OutputKind};
use sora_input_schema::input::SchemaFileInput;

use crate::args::{
    CheckArgs, Cli, Command, DiffArgs, ExcelSyncArgs, ExcelTemplateArgs, ExportArgs,
    ExportCompressionArg, GenArgs, SchemaLockArgs, SourceFormatArg,
};
use crate::source::MixedProjectInput;

pub fn run(cli: Cli) -> Result<()> {
    if cli.jobs == Some(0) {
        bail!("--jobs must be greater than 0");
    }

    let execution = ExecutionContext::new(ExecutionOptions {
        parallel: !cli.serial,
        jobs: cli.jobs,
    })?;

    match cli.command {
        Command::Build(args) => crate::build::run(args, &execution),
        Command::Check(args) => check(args),
        Command::Init(args) => crate::init::run(args),
        Command::Gen { target, args } => generate(args, &target),
        Command::Export(args) => export(args, &execution),
        Command::Diff(args) => diff(args, &execution),
        Command::ExcelTemplate(args) => excel_template(args),
        Command::ExcelSync(args) => excel_sync(args),
        Command::SchemaLock(args) => schema_lock(args),
        Command::Studio(args) => crate::studio::run(args),
    }
}

fn check(args: CheckArgs) -> Result<()> {
    let input = SchemaFileInput::new(&args.project);
    match &args.lock {
        Some(lock) => {
            sora_core::pipeline::check_schema_with_lock(&input, lock).with_context(|| {
                format!(
                    "failed to check project `{}` against lock `{}`",
                    args.project.display(),
                    lock.display()
                )
            })
        }
        None => sora_core::pipeline::check_schema(&input)
            .with_context(|| format!("failed to check project `{}`", args.project.display())),
    }
}

fn generate(args: GenArgs, target: &str) -> Result<()> {
    let input = SchemaFileInput::new(&args.project);
    sora_core::pipeline::generate_code_with_scope_and_format(
        &input,
        target,
        &args.out,
        args.format_code.into(),
        args.scope.as_deref(),
    )
    .with_context(|| {
        format!(
            "failed to generate code from `{}` into `{}`",
            args.project.display(),
            args.out.display()
        )
    })
}

fn excel_template(args: ExcelTemplateArgs) -> Result<()> {
    let input = SchemaFileInput::new(&args.project);
    sora_core::pipeline::generate_excel_template_with_scope(
        &input,
        &args.out,
        args.scope.as_deref(),
    )
    .with_context(|| {
        format!(
            "failed to generate Excel templates from `{}`",
            args.project.display()
        )
    })
}

fn excel_sync(args: ExcelSyncArgs) -> Result<()> {
    let input = SchemaFileInput::new(&args.project);
    let report = if args.write {
        sora_core::pipeline::write_excel_sync(&input, &args.data_root, args.scope.as_deref())
    } else {
        sora_core::pipeline::preview_excel_sync(&input, &args.data_root, args.scope.as_deref())
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
        } else if write {
            "update"
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
            } else {
                "sync sheet"
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

fn schema_lock(args: SchemaLockArgs) -> Result<()> {
    let input = SchemaFileInput::new(&args.project);
    sora_core::pipeline::generate_schema_lock_with_scope(&input, &args.out, args.scope.as_deref())
        .with_context(|| {
            format!(
                "failed to generate schema lock from `{}` into `{}`",
                args.project.display(),
                args.out.display()
            )
        })
}

fn export(args: ExportArgs, execution: &ExecutionContext) -> Result<()> {
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
    let input = MixedProjectInput::new(
        schema_input,
        &args.data_root,
        args.default_source_format.map(SourceFormatArg::as_str),
    );
    let (ir, data) = sora_core::pipeline::load_project_data_with_context(&input, execution)?;
    sora_core::pipeline::export_loaded_data_with_scope_context_and_options(
        &ir,
        &data,
        format,
        output,
        args.scope.as_deref(),
        execution,
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
    Ok(ExportOptions { compression })
}

fn diff(args: DiffArgs, execution: &ExecutionContext) -> Result<()> {
    let default_source_format = args.default_source_format.map(SourceFormatArg::as_str);
    let left_schema = SchemaFileInput::new(&args.project);
    let right_schema = SchemaFileInput::new(&args.project);
    let left = MixedProjectInput::new(left_schema, &args.left_root, default_source_format);
    let right = MixedProjectInput::new(right_schema, &args.right_root, default_source_format);
    sora_core::pipeline::diff_data_with_scope_and_context(
        &left,
        &right,
        &args.out,
        args.scope.as_deref(),
        execution,
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

        excel_sync(ExcelSyncArgs {
            project: project.clone(),
            data_root: data_root.clone(),
            scope: None,
            write: false,
        })
        .unwrap();
        assert!(!data_root.join("Item.xlsx").exists());

        excel_sync(ExcelSyncArgs {
            project: project.clone(),
            data_root: data_root.clone(),
            scope: None,
            write: true,
        })
        .unwrap();
        assert!(data_root.join("Item.xlsx").exists());

        write_xlsx_project(&base, true);
        excel_sync(ExcelSyncArgs {
            project: project.clone(),
            data_root: data_root.clone(),
            scope: None,
            write: true,
        })
        .unwrap();
        assert!(data_root.join(".sora-backup").exists());

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

    fn temp_dir() -> PathBuf {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "sora-cli-commands-test-{}-{id}",
            std::process::id()
        ))
    }
}
