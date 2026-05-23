# Core Concepts

## Project

A project manifest declares the package name, schema modules, build outputs, codegen targets, and export targets. It is the entry point used by `sora check`, `sora build`, `sora gen`, and `sora export`.

## Schema

Schema files describe the shape of configuration data. They define enums, structs, unions, tables, indexes, references, and field rules. Sora normalizes schema files into an IR before validation, export, or code generation.

## Table

A table is a named collection of rows. Tables can be list-like, keyed by one field, or singleton. Source metadata tells Sora where the editable data comes from.

The table schema is also used to generate editor projections such as Excel headers. The spreadsheet is not the contract; it is one way to edit rows that conform to the contract.

## Value

Sora validates source cells into a common value tree before export. Generated runtimes read that same shape from different runtime formats, so a target language can switch between `sora`, `json`, `cbor`, or `sora-protobuf` without changing the schema.

## Runtime Format

A runtime format is the wire format that generated code loads. It is selected per language target with `runtime_format`.

## Generator

A generator is a language backend registered in the codegen registry. Built-in generators are ordinary registry entries, which keeps the pipeline open to downstream extensions.

## Exporter

An exporter writes validated data into a runtime bundle. The exporter registry is separate from code generation so data formats and language targets can evolve independently.

## Scope

Schemas, fields, and tables can declare a `scope`. A build can select a scope to generate or export only the pieces needed by one runtime environment.
