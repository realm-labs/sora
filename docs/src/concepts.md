# Core Concepts

## Project

A project manifest declares the package name, schema modules, build outputs, codegen targets, and export targets.

## Schema

Schema files describe the shape of configuration data. They define enums, structs, unions, and tables. Sora normalizes schema files into an IR before validation or code generation.

## Table

A table is a named collection of rows. Tables can be list-like, keyed by one field, or singleton. Source metadata tells Sora where the editable data comes from.

## Runtime Format

A runtime format is the wire format that generated code loads. It is selected per language target with `runtime_format`.

## Generator

A generator is a language backend registered in the codegen registry. Built-in generators are ordinary registry entries, which keeps the pipeline open to downstream extensions.

## Exporter

An exporter writes validated data into a runtime bundle. The exporter registry is separate from code generation so data formats and language targets can evolve independently.
