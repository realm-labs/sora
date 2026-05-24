# Sora Studio

Sora Studio is the browser-based schema editor embedded in the `sora` CLI. It is meant for inspecting and editing project schemas without running a separate frontend server.

Start it with a project file:

```bash
sora studio --project project.toml
```

By default Studio binds to `127.0.0.1:5174` and prints the local URL. Use `--host` or `--port` when that address is not suitable:

```bash
sora studio --project project.toml --port 5180
```

## What It Edits

Studio loads the project file and every schema module listed in `includes`. Project files and schema modules can be TOML, YAML, JSON, or Lua, and a project can mix those formats.

The editor can update:

- project package name and schema include list;
- schema module files, including creating and removing included files;
- tables, structs, enums, and unions;
- table fields, struct fields, enum values, and union variants;
- table mode, primary key, source settings, parser settings, defaults, comments, range and length constraints;
- reference fields and derived child-table fields.

Studio is a schema editor, not a row-data editor. Excel, CSV, TOML, JSON, and YAML table rows are still edited in their source files and validated by `sora check`, `sora export`, or `sora build`.

## Visualization

The main canvas shows schema nodes and their relationships:

- type edges for fields that use enums, structs, or unions;
- reference edges for `ref<Table>` fields;
- derived edges for child-table fields assembled from another table.

The sidebar can filter schemas by name, shows project summary counts, and groups nodes by kind. Diagnostics are shown in the UI so an invalid schema can be identified from Studio instead of making the whole editor unusable.

## Preview and Save

Use preview before saving to review the files Studio will write. Studio renders each changed project or schema file in its own format:

- `.toml` files are written as TOML;
- `.yaml` and `.yml` files are written as YAML;
- `.json` files are written as pretty JSON;
- `.lua` files are written as data-returning Lua tables.

Saving normalizes the touched files through Studio's renderer. This is intentional: Studio keeps the schema data model stable, but it does not preserve comments, exact whitespace, or hand-written ordering inside the edited files. Review the preview before committing.

## Delivery

Release builds embed the Studio frontend assets into the `sora` binary. End users only need the CLI from GitHub Releases or crates.io; they do not need Node.js or a local Vite server.

For release maintainers, build the frontend before building the CLI binary:

```bash
cd apps/studio
npm run build
cd ../..
cargo build -p sora-cli --release
```

If the embedded assets are missing, `sora studio` reports that `apps/studio` needs to be built before the CLI.
