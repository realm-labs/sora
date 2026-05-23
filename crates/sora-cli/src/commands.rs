use anyhow::{Context, Result, bail};
use sora_execution::{ExecutionContext, ExecutionOptions};
use sora_export::exporter::{ExportCompression, ExportOptions, ExportOutput, OutputKind};
use sora_input_schema::input::SchemaFileInput;

use crate::args::{
    CheckArgs, Cli, Command, DiffArgs, ExcelTemplateArgs, ExportArgs, ExportCompressionArg,
    GenArgs, SchemaLockArgs, SourceFormatArg,
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
        Command::SchemaLock(args) => schema_lock(args),
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
