use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

/*
 * EvaluationContext
 */

pub struct EvaluationContext {
    pub variables: BTreeSet<String>,
    pub not_preset: BTreeSet<String>,
    values: BTreeMap<String, bool>,
}

impl EvaluationContext {
    pub fn new(variables: BTreeSet<String>) -> EvaluationContext {
        let mut not_preset = BTreeSet::new();
        for var in &variables {
            not_preset.insert(var.clone());
        }
        EvaluationContext {
            variables,
            not_preset,
            values: BTreeMap::new(),
        }
    }

    pub fn preset(&mut self, name: &str, preset_value: bool) -> Result<bool, String> {
        if self.variables.contains(name) {
            if self.not_preset.contains(name) {
                self.not_preset.remove(name);
                self.values.insert(String::from(name), preset_value);
                Ok(preset_value)
            } else {
                Err(format!(
                    "variable '{}' preset twice - ignoring second time",
                    name
                ))
            }
        } else {
            Err(format!("no variable named '{}'", name))
        }
    }

    pub fn get(&self, name: &str) -> bool {
        *self.values.get(name).unwrap()
    }

    pub fn set_not_presets(&mut self, values: u128) {
        for (i, var) in self.not_preset.iter().enumerate() {
            self.values
                .insert(var.to_string(), (values & (1 << i)) != 0);
        }
    }
}

/*
 * Expression
 */

#[derive(Debug)]
pub enum Expr<'a> {
    Value(bool),
    Variable(&'a str),
    Not(Box<Expr<'a>>),
    Or(Box<Expr<'a>>, Box<Expr<'a>>),
    And(Box<Expr<'a>>, Box<Expr<'a>>),
    Xor(Box<Expr<'a>>, Box<Expr<'a>>),
    Imp(Box<Expr<'a>>, Box<Expr<'a>>),
    Eq(Box<Expr<'a>>, Box<Expr<'a>>),
}

impl<'a> Expr<'a> {
    pub(crate) fn eval(&self, ctxt: &EvaluationContext) -> bool {
        match self {
            Expr::Value(value) => *value,
            Expr::Variable(name) => ctxt.get(name),
            Expr::Not(expr) => !expr.eval(ctxt),
            Expr::Or(left, right) => left.eval(ctxt) || right.eval(ctxt),
            Expr::And(left, right) => left.eval(ctxt) && right.eval(ctxt),
            Expr::Xor(left, right) => left.eval(ctxt) ^ right.eval(ctxt),
            Expr::Imp(left, right) => !left.eval(ctxt) || right.eval(ctxt),
            Expr::Eq(left, right) => left.eval(ctxt) == right.eval(ctxt),
        }
    }

    fn precedence(&self) -> usize {
        match self {
            Expr::Value(_) => 4,
            Expr::Variable(_) => 4,
            Expr::Not(_) => 3,
            Expr::Or(_, _) => 1,
            Expr::And(_, _) => 2,
            Expr::Xor(_, _) => 1,
            Expr::Imp(_, _) => 0,
            Expr::Eq(_, _) => 0,
        }
    }

    pub(crate) fn traverse(&self, callback: &dyn Fn(&Expr)) -> () {
        callback(self);
        match self {
            Expr::Not(a) => a.traverse(callback),
            Expr::Or(a, b) => {
                a.traverse(callback);
                b.traverse(callback);
            }
            Expr::And(a, b) => {
                a.traverse(callback);
                b.traverse(callback);
            }
            Expr::Xor(a, b) => {
                a.traverse(callback);
                b.traverse(callback);
            }
            Expr::Imp(a, b) => {
                a.traverse(callback);
                b.traverse(callback);
            }
            Expr::Eq(a, b) => {
                a.traverse(callback);
                b.traverse(callback);
            }
            _ => {}
        }
    }

    pub(crate) fn as_variable(&self) -> Option<&str> {
        match self {
            Expr::Variable(name) => Some(*name),
            _ => None,
        }
    }

    fn to_string_internal(&self, parent_precedence: usize) -> String {
        if self.precedence() > parent_precedence {
            self.to_string()
        } else {
            format!("({})", self.to_string())
        }
    }
}

impl<'a> Display for Expr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Value(value) => {
                if *value {
                    write!(f, "1")
                } else {
                    write!(f, "0")
                }
            }
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::Not(expr) => write!(f, "!{}", expr.to_string_internal(self.precedence())),
            Expr::Or(left, right) => write!(
                f,
                "{} | {}",
                left.to_string_internal(self.precedence()),
                right.to_string_internal(self.precedence())
            ),
            Expr::And(left, right) => write!(
                f,
                "{} & {}",
                left.to_string_internal(self.precedence()),
                right.to_string_internal(self.precedence())
            ),
            Expr::Xor(left, right) => write!(
                f,
                "{} ^ {}",
                left.to_string_internal(self.precedence()),
                right.to_string_internal(self.precedence())
            ),
            Expr::Imp(left, right) => write!(
                f,
                "{} => {}",
                left.to_string_internal(self.precedence()),
                right.to_string_internal(self.precedence())
            ),
            Expr::Eq(left, right) => write!(
                f,
                "{} = {}",
                left.to_string_internal(self.precedence()),
                right.to_string_internal(self.precedence())
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn ctx_with(values: &[(&str, bool)]) -> EvaluationContext {
        let mut variables = BTreeSet::new();
        for (name, _) in values {
            variables.insert((*name).to_string());
        }

        let mut ctx = EvaluationContext::new(variables);
        for (name, value) in values {
            ctx.preset(name, *value).unwrap();
        }
        ctx
    }

    #[test]
    fn value_and_variable_display() {
        assert_eq!(Expr::Value(true).to_string(), "1");
        assert_eq!(Expr::Value(false).to_string(), "0");
        assert_eq!(Expr::Variable("a").to_string(), "a");
    }

    #[test]
    fn as_variable_returns_name_only_for_variable() {
        assert_eq!(Expr::Variable("foo").as_variable(), Some("foo"));
        assert_eq!(Expr::Value(true).as_variable(), None);
        assert_eq!(
            Expr::Not(Box::new(Expr::Variable("foo"))).as_variable(),
            None
        );
    }

    #[test]
    fn evals_basic_boolean_values() {
        let ctx = ctx_with(&[]);
        assert!(Expr::Value(true).eval(&ctx));
        assert!(!Expr::Value(false).eval(&ctx));
    }

    #[test]
    fn evals_variables() {
        let ctx = ctx_with(&[("a", true), ("b", false)]);
        assert!(Expr::Variable("a").eval(&ctx));
        assert!(!Expr::Variable("b").eval(&ctx));
    }

    #[test]
    fn evals_unary_and_binary_operators() {
        let ctx = ctx_with(&[("a", true), ("b", false)]);

        assert!(!Expr::Not(Box::new(Expr::Variable("a"))).eval(&ctx));
        assert!(Expr::Or(Box::new(Expr::Variable("a")), Box::new(Expr::Variable("b"))).eval(&ctx));
        assert!(
            !Expr::And(Box::new(Expr::Variable("a")), Box::new(Expr::Variable("b"))).eval(&ctx)
        );
        assert!(Expr::Xor(Box::new(Expr::Variable("a")), Box::new(Expr::Variable("b"))).eval(&ctx));
        assert!(
            !Expr::Imp(Box::new(Expr::Variable("a")), Box::new(Expr::Variable("b"))).eval(&ctx)
        );
        assert!(Expr::Eq(Box::new(Expr::Variable("a")), Box::new(Expr::Value(true))).eval(&ctx));
    }

    #[test]
    fn traverse_visits_nodes_preorder() {
        let expr = Expr::Or(
            Box::new(Expr::Not(Box::new(Expr::Variable("a")))),
            Box::new(Expr::And(
                Box::new(Expr::Variable("b")),
                Box::new(Expr::Value(true)),
            )),
        );

        let mut visited = Vec::new();
        let visited_ref = RefCell::new(&mut visited);
        expr.traverse(&|e| visited_ref.borrow_mut().push(e.to_string()));

        assert_eq!(visited, vec!["!a | b & 1", "!a", "a", "b & 1", "b", "1"]);
    }

    #[test]
    fn display_adds_parentheses_according_to_precedence() {
        assert_eq!(
            Expr::Or(
                Box::new(Expr::Variable("a")),
                Box::new(Expr::And(
                    Box::new(Expr::Variable("b")),
                    Box::new(Expr::Variable("c"))
                ))
            )
            .to_string(),
            "a | b & c"
        );

        assert_eq!(
            Expr::And(
                Box::new(Expr::Or(
                    Box::new(Expr::Variable("a")),
                    Box::new(Expr::Variable("b"))
                )),
                Box::new(Expr::Variable("c"))
            )
            .to_string(),
            "(a | b) & c"
        );

        assert_eq!(
            Expr::Not(Box::new(Expr::Or(
                Box::new(Expr::Variable("a")),
                Box::new(Expr::Variable("b"))
            )))
            .to_string(),
            "!(a | b)"
        );

        assert_eq!(
            Expr::Imp(
                Box::new(Expr::And(
                    Box::new(Expr::Variable("a")),
                    Box::new(Expr::Variable("b"))
                )),
                Box::new(Expr::Eq(
                    Box::new(Expr::Variable("c")),
                    Box::new(Expr::Variable("d"))
                ))
            )
            .to_string(),
            "a & b => (c = d)"
        );
    }
}
