use pest::Parser;

mod node;
pub use node::Node;

mod report;
pub use report::Report;

mod statement;
pub use statement::Statement;

use crate::ArmaConfigError;

#[derive(Parser)]
#[grammar = "parser/config.pest"]
pub struct ConfigParser;

#[derive(Debug, Clone)]
/// Abstract Syntax Tree
pub struct AST {
    pub config: Node,
    pub processed: bool,
    pub report: Option<Report>,
}

impl AST {
    pub fn valid(&self) -> bool {
        if let Some(report) = &self.report {
            report.errors.is_empty()
        } else {
            true
        }
    }
}

/// Converts a raw string into an AST
///
/// ```
/// let content = "value = 123;";
/// armalint::config::parse(content);
/// ```
pub fn parse(source: &str) -> Result<AST, String> {
    let clean = source.replace("\r", "");
    let pair = ConfigParser::parse(Rule::file, &clean)
        .unwrap()
        .next()
        .unwrap();
    let pair = pair.into_inner().next().unwrap();
    let config = Node::from_expr(std::env::current_dir().unwrap(), source, pair)?;
    Ok(AST {
        config,
        processed: false,
        report: None,
    })
}

pub fn get_ident(stmt: Statement) -> Result<String, ArmaConfigError> {
    Ok(match stmt {
        Statement::Ident(val) => val,
        Statement::IdentArray(val) => val,
        _ => panic!("get ident wasn't given ident: {:#?}", stmt),
    })
}
