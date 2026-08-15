use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use sora_diagnostics::{Result, SoraError};
use sora_ir::model::TableIr;

const EXCEL_SHEET_NAME_LIMIT: usize = 31;

/// Reject schemas that bind more than one table definition to one XLSX file.
///
/// A workbook is the ownership boundary for a table. Multiple worksheets in
/// that workbook may split the rows of that one table, but they cannot belong
/// to different table definitions.
pub fn ensure_single_table_definition(tables: &[&TableIr], file: &str) -> Result<()> {
    if tables.len() <= 1 {
        return Ok(());
    }

    let names = tables
        .iter()
        .map(|table| canonical_table_name(table))
        .collect::<Vec<_>>();
    Err(SoraError::InvalidSchema(format!(
        "XLSX workbook `{file}` is referenced by multiple table definitions: {}; one workbook may contain multiple data worksheets, but must belong to exactly one table definition",
        names.join(", ")
    )))
}

pub fn resolve_table_sheet_names(table: &TableIr, available: &[String]) -> Result<Vec<String>> {
    resolve_table_sheet_names_with_metadata(table, available, &BTreeMap::new())
}

pub fn resolve_table_sheet_names_with_metadata(
    table: &TableIr,
    available: &[String],
    table_metadata: &BTreeMap<String, String>,
) -> Result<Vec<String>> {
    let source = table
        .source
        .as_ref()
        .ok_or_else(|| SoraError::MissingTableSource {
            table: table.name.clone(),
        })?;
    if let Some(sheet) = &source.sheet {
        return Ok(vec![sheet.clone()]);
    }
    if !source.sheets.is_empty() {
        return resolve_explicit_selectors(table, available);
    }

    // Existing generated/synchronized workbooks declare the owning canonical
    // table in every data worksheet. All matching sheets are row partitions of
    // the same table, so multiple matches are intentional rather than ambiguous.
    let canonical_name = canonical_table_name(table);
    let metadata_matches = available
        .iter()
        .filter(|sheet| {
            table_metadata.get(*sheet).is_some_and(|declared_table| {
                declared_table == canonical_name || declared_table == &table.name
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    if !metadata_matches.is_empty() {
        return Ok(metadata_matches);
    }

    Ok(vec![default_sheet_name(&source.file)])
}

fn resolve_explicit_selectors(table: &TableIr, available: &[String]) -> Result<Vec<String>> {
    let source = table
        .source
        .as_ref()
        .expect("table source was checked by the caller");
    let mut resolved = Vec::new();
    let mut seen = BTreeSet::new();
    for selector in &source.sheets {
        if has_wildcard(selector) {
            let mut matches = available
                .iter()
                .filter(|sheet| wildcard_matches(selector, sheet))
                .cloned()
                .collect::<Vec<_>>();
            matches.sort();
            if matches.is_empty() {
                return Err(SoraError::InvalidSchema(format!(
                    "table `{}` sheet selector `{selector}` matched no worksheets in `{}`",
                    table.name, source.file
                )));
            }
            for sheet in matches {
                if seen.insert(sheet.to_lowercase()) {
                    resolved.push(sheet);
                }
            }
        } else if seen.insert(selector.to_lowercase()) {
            resolved.push(selector.clone());
        }
    }
    Ok(resolved)
}

fn default_sheet_name(file: &str) -> String {
    let stem = Path::new(file)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("Data");
    let mut name = stem
        .chars()
        .map(|character| match character {
            '[' | ']' | ':' | '*' | '?' | '/' | '\\' => '_',
            other => other,
        })
        .collect::<String>();
    name = name.trim_matches('\'').to_owned();
    if name.is_empty() || name.eq_ignore_ascii_case("History") {
        name = "Data".to_owned();
    }
    name.chars().take(EXCEL_SHEET_NAME_LIMIT).collect()
}

fn canonical_table_name(table: &TableIr) -> &str {
    if table.canonical_name.is_empty() {
        &table.name
    } else {
        &table.canonical_name
    }
}

pub fn has_wildcard(selector: &str) -> bool {
    selector.contains('*') || selector.contains('?')
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.chars().collect::<Vec<_>>();
    let value = value.chars().collect::<Vec<_>>();
    let mut previous = vec![false; value.len() + 1];
    previous[0] = true;
    for token in pattern {
        let mut current = vec![false; value.len() + 1];
        if token == '*' {
            current[0] = previous[0];
        }
        for index in 1..=value.len() {
            current[index] = match token {
                '*' => previous[index] || current[index - 1],
                '?' => previous[index - 1],
                literal => previous[index - 1] && literal == value[index - 1],
            };
        }
        previous = current;
    }
    previous[value.len()]
}

#[cfg(test)]
mod tests {
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::ProjectSchema;

    use super::*;

    #[test]
    fn wildcard_matching_is_anchored() {
        assert!(wildcard_matches("2026-*", "2026-01"));
        assert!(wildcard_matches("event-??", "event-cn"));
        assert!(!wildcard_matches("2026-*", "archive-2026-01"));
        assert!(!wildcard_matches("event-??", "event-global"));
    }

    #[test]
    fn default_sheet_comes_from_workbook_file_not_table_name() {
        let ir = schema_with_tables(&[(
            "visual",
            "presentation.SurfaceResourceVisualProfile",
            "VisualProfile.xlsx",
        )]);

        let names = resolve_table_sheet_names(&ir.tables[0], &[]).unwrap();

        assert_eq!(names, ["VisualProfile"]);
    }

    #[test]
    fn default_sheet_is_safely_truncated() {
        let ir = schema_with_tables(&[(
            "visual",
            "Visual",
            "AWorkbookNameThatIsFarLongerThanExcelAllows.xlsx",
        )]);

        let names = resolve_table_sheet_names(&ir.tables[0], &[]).unwrap();

        assert_eq!(names[0].chars().count(), EXCEL_SHEET_NAME_LIMIT);
        assert_eq!(names[0], "AWorkbookNameThatIsFarLongerTha");
    }

    #[test]
    fn metadata_selects_all_partitions_of_the_same_table() {
        let ir = schema_with_tables(&[("activity", "events.Activity", "Activity.xlsx")]);
        let available = vec![
            "2026-01".to_owned(),
            "Notes".to_owned(),
            "2026-02".to_owned(),
        ];
        let metadata = BTreeMap::from([
            ("2026-01".to_owned(), "events.Activity".to_owned()),
            ("2026-02".to_owned(), "events.Activity".to_owned()),
        ]);

        let names =
            resolve_table_sheet_names_with_metadata(&ir.tables[0], &available, &metadata).unwrap();

        assert_eq!(names, ["2026-01", "2026-02"]);
    }

    #[test]
    fn rejects_multiple_table_definitions_for_one_workbook() {
        let ir = schema_with_tables(&[
            ("item", "items.Item", "Shared.xlsx"),
            ("reward", "rewards.Reward", "Shared.xlsx"),
        ]);

        let error = ensure_single_table_definition(&[&ir.tables[0], &ir.tables[1]], "Shared.xlsx")
            .unwrap_err();

        assert!(error.to_string().contains("items.Item, rewards.Reward"));
        assert!(error.to_string().contains("exactly one table definition"));
    }

    fn schema_with_tables(tables: &[(&str, &str, &str)]) -> sora_ir::model::ConfigIr {
        let tables = tables
            .iter()
            .map(|(id, name, file)| {
                format!(
                    r#"[[tables]]
id = "{id}"
name = "{name}"
mode = "list"
source = {{ format = "xlsx", file = "{file}" }}
"#
                )
            })
            .collect::<String>();
        let schema: ProjectSchema = toml::from_str(&format!(
            r#"project = {{ id = "test" }}
groups = {{ common = {{ default = true }} }}
views = {{ default = {{ contract = "test/default", groups = ["common"] }} }}

{tables}
"#
        ))
        .unwrap();
        normalize_schema(schema).unwrap()
    }
}
