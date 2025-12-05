use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression};

// Pattern: minOut + ((value - minIn) * (maxOut - minOut)) / (maxIn - minIn)
// This also matches linear interpolation: x1 + (t - y1) * (x2 - x1) / (y2 - y1)
// Both can be replaced with linearConversion for better readability
//
// Add(
//     left = minOut (or x1),
//     right = Divide(
//         left = Multiply(
//             left = Subtract(value, minIn),
//             right = Subtract(maxOut, minOut)
//         ),
//         right = Subtract(maxIn, minIn)
//     )
// )

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    // Check for Add: minOut + (...)
    let Expression::BinaryCommand(BinaryCommand::Add, add_lhs, add_rhs, _) = target else {
        return codes;
    };

    // Check for Divide on the right side of Add
    let Expression::BinaryCommand(BinaryCommand::Div, div_lhs, div_rhs, _) = &**add_rhs else {
        return codes;
    };

    // Check for Multiply on the left side of Divide
    let Expression::BinaryCommand(BinaryCommand::Mul, mul_lhs, mul_rhs, _) = &**div_lhs else {
        return codes;
    };

    // Check for Subtract on the left side of Multiply: (value - minIn)
    let Expression::BinaryCommand(BinaryCommand::Sub, value, min_in, _) = &**mul_lhs else {
        return codes;
    };

    // Check for Subtract on the right side of Multiply: (maxOut - minOut)
    let Expression::BinaryCommand(BinaryCommand::Sub, max_out, _min_out, _) = &**mul_rhs else {
        return codes;
    };

    // Check for Subtract on the right side of Divide: (maxIn - minIn)
    let Expression::BinaryCommand(BinaryCommand::Sub, max_in, _min_in2, _) = &**div_rhs else {
        return codes;
    };

    // Extract source text for all parameters
    // linearConversion [minIn, maxIn, value, minOut, maxOut]
    let min_in_text = min_in.source(false);
    let max_in_text = max_in.source(false);
    let value_text = value.source(false);
    let min_out_text = add_lhs.source(false); // minOut is the left side of the top-level Add
    let max_out_text = max_out.source(false);

    codes.push(Arc::new(
        CodeS33ReimplementingCommandLinearConversion::new(
            target.full_span(),
            min_in_text,
            max_in_text,
            value_text,
            min_out_text,
            max_out_text,
            processed,
            config.severity(),
        ),
    ));

    codes
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandLinearConversion {
    span: Range<usize>,
    min_in: String,
    max_in: String,
    value: String,
    min_out: String,
    max_out: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandLinearConversion {
    fn ident(&self) -> &'static str {
        "L-S33-LINEAR-CONVERSION"
    }
    
    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `linearConversion`")
    }

    fn label_message(&self) -> String {
        String::from("use `linearConversion`")
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }

    fn suggestion(&self) -> Option<String> {
        // Calculate approximate length of single-line suggestion
        let single_line = format!(
            "linearConversion [{}, {}, {}, {}, {}]",
            self.min_in, self.max_in, self.value, self.min_out, self.max_out
        );
        
        // If the line would be too long (> 80 chars), format on multiple lines
        if single_line.len() > 80 {
            Some(format!(
                "linearConversion [\n    {},\n    {},\n    {},\n    {},\n    {}\n]",
                self.min_in, self.max_in, self.value, self.min_out, self.max_out
            ))
        } else {
            Some(single_line)
        }
    }
}

impl CodeS33ReimplementingCommandLinearConversion {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        span: Range<usize>,
        min_in: String,
        max_in: String,
        value: String,
        min_out: String,
        max_out: String,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            min_in,
            max_in,
            value,
            min_out,
            max_out,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic =
            Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
