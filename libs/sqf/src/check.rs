//! Checking one preprocessed SQF file.
//!
//! Shared so the CLI and the language server cannot disagree about what a
//! file's diagnostics are. They repeatedly have - see #1308 and #1309, both
//! of which were a rule applied in one copy of this pipeline and not the
//! other.

use std::sync::Arc;

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{
    addons::Addon,
    reporting::{Codes, Processed},
};

use crate::{
    Statements,
    analyze::{SqfReport, analyze},
    parser::{ParserError, database::Database},
};

/// The result of checking one preprocessed SQF file.
pub struct Checked {
    /// Every diagnostic for the file: preprocessor warnings, then whatever
    /// parsing or analysis produced.
    pub codes: Codes,
    /// The parsed statements, `None` if the file did not parse.
    pub statements: Option<Statements>,
    /// The analysis report, `None` unless analysis ran.
    pub report: Option<SqfReport>,
}

#[must_use]
/// Check one preprocessed SQF file.
///
/// Callers are left with what genuinely differs between them - the CLI
/// compiles the statements and pushes the report to the addon, the language
/// server turns the codes into LSP diagnostics.
pub fn check(
    processed: &Processed,
    project: Option<&ProjectConfig>,
    addon: &Arc<Addon>,
    database: Arc<Database>,
) -> Checked {
    // Preprocessor warnings belong to the file as much as lint codes do
    let mut codes: Codes = processed.warnings().to_vec();
    match crate::parser::run(&database, processed) {
        Ok(statements) => {
            let (lints, report) = analyze(&statements, project, processed, addon.clone(), database);
            codes.extend(lints);
            Checked {
                codes,
                statements: Some(statements),
                report,
            }
        }
        Err(ParserError::ParsingError(errors)) => {
            // CBA settings files use `force` as a statement prefix and never
            // parse; they are not meant to
            if !crate::is_cba_settings(processed.as_str()) {
                codes.extend(errors);
            }
            Checked {
                codes,
                statements: None,
                report: None,
            }
        }
        Err(ParserError::LexingError(errors)) => {
            codes.extend(errors);
            Checked {
                codes,
                statements: None,
                report: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use hemtt_workspace::{
        Workspace,
        addons::Addon,
        reporting::{Codes, Processed},
    };

    use super::check;
    use crate::parser::database::Database;

    fn processed(contents: &str) -> Processed {
        use std::io::Write;
        let workspace = Workspace::builder()
            .memory()
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .expect("workspace");
        let path = workspace.join("test.sqf").expect("join");
        let mut handle = path.create_file().expect("create");
        handle.write_all(contents.as_bytes()).expect("write");
        drop(handle);
        hemtt_preprocessor::Processor::run(
            &path,
            &hemtt_common::config::PreprocessorOptions::default(),
        )
        .expect("preprocesses")
    }

    fn checked(contents: &str) -> super::Checked {
        check(
            &processed(contents),
            None,
            &Arc::new(Addon::test_addon()),
            Arc::new(Database::a3(false)),
        )
    }

    fn idents(codes: &Codes) -> Vec<&'static str> {
        codes.iter().map(|code| code.ident()).collect()
    }

    /// The CLI reported these and the editor did not, because each side
    /// collected the codes it wanted separately.
    #[test]
    fn preprocessor_warnings_are_included() {
        let checked = checked("#define A 1\n#define A 2\nprivate _x = 1;\n");
        let codes = idents(&checked.codes);
        assert!(codes.contains(&"PW1"), "{codes:?}");
        assert!(checked.statements.is_some());
    }

    /// CBA settings files use `force` as a statement prefix and never parse.
    /// They are not meant to, so nothing is reported.
    #[test]
    fn cba_settings_are_skipped() {
        let checked = checked("force ace_medical_level = 2;\n");
        assert!(checked.codes.is_empty(), "{:?}", idents(&checked.codes));
        assert!(checked.statements.is_none());
    }

    #[test]
    fn a_file_that_does_not_parse_has_no_statements() {
        let checked = checked("private _x = ;\n");
        assert!(checked.statements.is_none());
        assert!(!checked.codes.is_empty());
    }
}
