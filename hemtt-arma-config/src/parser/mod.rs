use pest::Parser;

mod node;
pub use node::Node;

mod statement;
pub use statement::Statement;

#[derive(Parser)]
#[grammar = "parser/config.pest"]
pub struct ConfigParser;

#[derive(Debug, Clone)]
/// Abstract Syntax Tree
pub struct AST {
    pub config: Node,
}

// impl AST {
//     pub fn valid(&self) -> bool {
//         if let Some(report) = &self.report {
//             report.errors.is_empty()
//         } else {
//             true
//         }
//     }
// }

/// Converts a raw string into an AST
///
/// ```
/// let content = "value = 123;";
/// hemtt_arma_config::parse(content);
/// ```
pub fn parse(source: &str) -> Result<AST, String> {
    let clean = source.replace("\r", "");
    let pair = ConfigParser::parse(Rule::file, &clean)
        .unwrap()
        .next()
        .unwrap();
    let pair = pair.into_inner().next().unwrap();
    let config = Node::from_expr(std::env::current_dir().unwrap(), source, pair)?;
    Ok(AST { config })
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn property() {
        let ast = parse("value = 123;");
        println!("{:?}", ast);
    }
}
