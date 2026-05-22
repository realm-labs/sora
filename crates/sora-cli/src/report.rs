use std::fmt;

use sora_diagnostics::SoraError;

pub struct ErrorReport<'a> {
    error: &'a anyhow::Error,
}

impl<'a> ErrorReport<'a> {
    pub fn new(error: &'a anyhow::Error) -> Self {
        Self { error }
    }
}

impl fmt::Display for ErrorReport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sora_error = self
            .error
            .chain()
            .find_map(|cause| cause.downcast_ref::<SoraError>());

        match sora_error {
            Some(error) => {
                writeln!(f, "error[{}]: {}", error.code(), self.error)?;
                if let Some(path) = error.path() {
                    writeln!(f, "  --> {}", path.display())?;
                }
                if let Some(errors) = error.errors() {
                    writeln!(f, "validation errors:")?;
                    for (index, error) in errors.iter().enumerate() {
                        writeln!(f, "  {}. [{}] {}", index + 1, error.code(), error)?;
                    }
                }
            }
            None => {
                writeln!(f, "error: {}", self.error)?;
            }
        }

        let causes = self
            .error
            .chain()
            .skip(1)
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if !causes.is_empty() {
            writeln!(f, "caused by:")?;
            for (index, cause) in causes.iter().enumerate() {
                writeln!(f, "  {}: {}", index + 1, cause)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;

    #[test]
    fn prints_sora_error_code_and_path() {
        let error = Err::<(), _>(SoraError::ParseSchema {
            path: "schema/game.toml".into(),
            message: "bad type".to_owned(),
        })
        .context("failed to check project")
        .unwrap_err();

        let report = ErrorReport::new(&error).to_string();

        assert!(report.contains("error[SORA0004]: failed to check project"));
        assert!(report.contains("--> schema/game.toml"));
        assert!(report.contains("failed to parse schema `schema/game.toml`: bad type"));
    }

    #[test]
    fn prints_plain_anyhow_error() {
        let error = anyhow::anyhow!("bad argument");

        assert_eq!(
            ErrorReport::new(&error).to_string(),
            "error: bad argument\n"
        );
    }

    #[test]
    fn prints_aggregated_validation_errors() {
        let error = SoraError::validation_errors(vec![
            SoraError::MissingRequiredField {
                table: "Item".to_owned(),
                field: "name".to_owned(),
            },
            SoraError::DuplicateKey {
                table: "Item".to_owned(),
                key: "1001".to_owned(),
            },
        ]);
        let error = anyhow::Error::new(error).context("failed to validate data");

        let report = ErrorReport::new(&error).to_string();

        assert!(report.contains("error[SORA0035]: failed to validate data"));
        assert!(report.contains("1. [SORA0023] missing required field `name`"));
        assert!(report.contains("2. [SORA0027] duplicate key `1001`"));
    }
}
