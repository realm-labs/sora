# Versioning and Compatibility

Sora is still early. The project does not provide Rust-style editions or compatibility modes for old schema semantics. A project that needs stable output should pin the `sora` CLI version it uses, and treat a CLI upgrade as an explicit migration step.

## What To Pin

Pin the CLI binary or crate version in the project tooling:

- download a specific GitHub Release asset and keep using that version in CI;
- install a specific crates.io version with `cargo install sora-cli --version X.Y.Z`;
- record the expected `sora --version` in project setup docs or build scripts.

Generated code, generated Excel templates, schema locks, and exported runtime bundles should be produced by the same pinned CLI version for a given project build.

## Runtime Bundle Versions

Exported runtime bundles carry a format version. The Sora binary bundle also has a file header version, and generated runtimes reject bundles with unsupported versions.

Sora only bumps these runtime/export format versions when the generated runtime can no longer safely read data written by an older layout. Examples include:

- changing the `.sora` binary section layout;
- changing the manifest fields required by generated runtimes;
- changing JSON, CBOR, or Protobuf bundle structure in a way that old generated code cannot read;
- changing value encoding rules in exported runtime bundles.

During the early development stage, ordinary implementation changes do not automatically bump `format_version`. Version bumps are manual and reserved for actual runtime/export incompatibility.

## Schema and Codegen Semantics

Schema syntax, parser behavior, validation rules, Studio rendering, and generated language APIs may still change while the project is young. Sora does not keep old behavior behind an `edition` flag or any other compatibility mode.

If a newer CLI changes schema or codegen semantics, users should:

1. upgrade the CLI intentionally;
2. regenerate schema locks, templates, exports, and code;
3. review diffs;
4. update schema/data/project files as needed.

Schema fingerprints and schema locks help detect mismatches between generated code, schema, and data, but they are not migration tools. They prevent silent incompatibility; they do not preserve old semantics.
