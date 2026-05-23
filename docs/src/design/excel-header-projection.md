# Excel Header Projection

Excel templates are generated from the normalized schema. The header is a projection, not an independent format definition.

## Why Generate Headers

Manually maintained spreadsheet headers tend to drift from code:

- a field is renamed in code but not in Excel;
- a type changes but old rows still look valid;
- a designer adds a column that no runtime reads;
- validation rules are documented in comments instead of enforced.

Sora avoids this by generating the workbook structure from schema.

## What the Header Contains

Generated rows include:

- table metadata: table name, mode, key, scope, and schema hash;
- stable field names;
- type hints;
- scope hints;
- validation and parser rules;
- comments for editors.

Only row data should be treated as authored content. Header rows can be regenerated whenever the schema changes.

## Practical Workflow

1. Change the schema.
2. Regenerate Excel templates.
3. Move or paste existing data rows into the updated template.
4. Run `sora build` or `sora export` to validate values and references.
5. Run `sora build` to produce exports and generated code.

This keeps Excel useful for editing while keeping the schema authoritative.
