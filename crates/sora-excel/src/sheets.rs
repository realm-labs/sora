use std::collections::BTreeSet;

use sora_diagnostics::{Result, SoraError};
use sora_ir::model::TableIr;

pub fn resolve_table_sheet_names(table: &TableIr, available: &[String]) -> Result<Vec<String>> {
    let source = table
        .source
        .as_ref()
        .ok_or_else(|| SoraError::MissingTableSource {
            table: table.name.clone(),
        })?;
    if let Some(sheet) = &source.sheet {
        return Ok(vec![sheet.clone()]);
    }
    if source.sheets.is_empty() {
        return Ok(vec![table.name.clone()]);
    }

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
                if seen.insert(sheet.clone()) {
                    resolved.push(sheet);
                }
            }
        } else if seen.insert(selector.clone()) {
            resolved.push(selector.clone());
        }
    }
    Ok(resolved)
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
    use super::wildcard_matches;

    #[test]
    fn wildcard_matching_is_anchored() {
        assert!(wildcard_matches("2026-*", "2026-01"));
        assert!(wildcard_matches("event-??", "event-cn"));
        assert!(!wildcard_matches("2026-*", "archive-2026-01"));
        assert!(!wildcard_matches("event-??", "event-global"));
    }
}
