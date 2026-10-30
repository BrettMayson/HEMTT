use crate::{Expression, Statement, Statements};
#[allow(unused_imports)]
use tracing::{trace, warn};

impl Statements {
    #[must_use]
    pub fn simplify_end_var(mut self) -> Self {
        let Some(Statement::Expression(Expression::Variable(last_var, _), _)) = self.content.get(self.content.len().saturating_sub(1)) else {
            return self;
        };
        let Some(Statement::AssignLocal(penultimate_var, _, _)) = self.content.get(self.content.len().saturating_sub(2)) else {
            return self;
        };
        if !last_var.eq_ignore_ascii_case(penultimate_var) {
            return self;
        }
        #[cfg(debug_assertions)]
        trace!("optimizing end-var [{last_var}]");
        self.content.pop();
        let Some(Statement::AssignLocal(_, penultimate_expression, penultimate_range)) = self.content.pop() else {
            unreachable!("Must be an AssignLocal statement here")
        };
        self.content.push(Statement::Expression(penultimate_expression, penultimate_range));
        self
    }
}
