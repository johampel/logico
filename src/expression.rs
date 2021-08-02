use std::collections::{BTreeMap, BTreeSet};

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
                Err(format!("variable '{}' preset twice - ignoring second time", name))
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
            self.values.insert(var.to_string(), (values & (1 << i)) != 0);
        }
    }
}

/*
 * Expression
 */

pub trait Expression {
    fn eval(&self, ctxt: &EvaluationContext) -> bool;

    fn precedence(&self) -> usize;

    fn traverse(&self, callback: &dyn Fn(&dyn Expression)) -> ();

    fn as_variable(&self) -> Option<&Variable> {
        None
    }

    fn to_string(&self) -> String;

    fn to_dump_string(&self) -> String {
        self.to_string()
    }
}

fn to_string(expr: &Box<dyn Expression>, parent_precedence: usize) -> String {
    if expr.precedence() > parent_precedence {
        expr.to_string()
    } else {
        format!("({})", expr.to_string())
    }
}


/*
 * Value
 */

pub struct Value {
    value: bool
}


impl Value {
    pub fn new(value: bool) -> Value {
        Value { value }
    }
}

impl Expression for Value {
    fn eval(&self, _ctxt: &EvaluationContext) -> bool {
        self.value
    }

    fn precedence(&self) -> usize { 4 }

    fn traverse(&self, callback: &dyn Fn(&dyn Expression)) -> () {
        callback(self);
    }

    fn to_string(&self) -> String {
        if self.value { String::from("1") } else { String::from("0") }
    }

    fn to_dump_string(&self) -> String {
        if self.value { String::from("Value(1)") } else { String::from("Value(0)") }
    }
}


/*
 * Variable
 */

pub struct Variable {
    pub name: String
}

impl Variable {
    pub fn new(name: &str) -> Variable {
        Variable { name: name.to_string() }
    }
}

impl Expression for Variable {
    fn eval(&self, ctxt: &EvaluationContext) -> bool {
        ctxt.get(&self.name)
    }

    fn precedence(&self) -> usize { 4 }

    fn traverse(&self, callback: &dyn Fn(&dyn Expression)) -> () {
        callback(self);
    }

    fn as_variable(&self) -> Option<&Variable> {
        Some(&self)
    }

    fn to_string(&self) -> String {
        String::from(&self.name)
    }

    fn to_dump_string(&self) -> String {
        String::from(format!("Variable({})", &self.name))
    }
}


/*
 * UnaryOperator/Expression
 */

pub enum UnaryOperator {
    NEG
}

pub struct UnaryExpression {
    op: UnaryOperator,
    arg: Box<dyn Expression>,
}

impl UnaryExpression {
    pub fn new(op: UnaryOperator, arg: Box<dyn Expression>) -> UnaryExpression {
        UnaryExpression { op, arg }
    }
}

impl Expression for UnaryExpression {
    fn eval(&self, ctxt: &EvaluationContext) -> bool {
        match self.op {
            UnaryOperator::NEG => !self.arg.eval(ctxt)
        }
    }

    fn traverse(&self, callback: &dyn Fn(&dyn Expression)) -> () {
        callback(self);
        self.arg.traverse(callback);
    }

    fn precedence(&self) -> usize {
        match self.op {
            UnaryOperator::NEG => 3
        }
    }

    fn to_string(&self) -> String {
        match self.op {
            UnaryOperator::NEG => format!("!{}", to_string(&self.arg, self.precedence()))
        }
    }

    fn to_dump_string(&self) -> String {
        match self.op {
            UnaryOperator::NEG => format!("Neg({})", &self.arg.to_dump_string())
        }
    }
}


/*
 * BinaryOperator/Expression
 */

pub enum BinaryOperator {
    OR,
    AND,
    XOR,
    IMP,
    EQ,
}

pub struct BinaryExpression {
    op: BinaryOperator,
    left: Box<dyn Expression>,
    right: Box<dyn Expression>,
}

impl BinaryExpression {
    pub fn new(op: BinaryOperator, left: Box<dyn Expression>, right: Box<dyn Expression>) -> BinaryExpression {
        BinaryExpression { op, left, right }
    }
}

impl Expression for BinaryExpression {
    fn eval(&self, ctxt: &EvaluationContext) -> bool {
        match self.op {
            BinaryOperator::OR => self.left.eval(ctxt) || self.right.eval(ctxt),
            BinaryOperator::XOR => self.left.eval(ctxt) != self.right.eval(ctxt),
            BinaryOperator::AND => self.left.eval(ctxt) && self.right.eval(ctxt),
            BinaryOperator::EQ => self.left.eval(ctxt) == self.right.eval(ctxt),
            BinaryOperator::IMP => !self.left.eval(ctxt) || self.right.eval(ctxt)
        }
    }

    fn precedence(&self) -> usize {
        match self.op {
            BinaryOperator::OR => 1,
            BinaryOperator::XOR => 1,
            BinaryOperator::AND => 2,
            BinaryOperator::EQ => 0,
            BinaryOperator::IMP => 0
        }
    }

    fn traverse(&self, callback: &dyn Fn(&dyn Expression)) -> () {
        callback(self);
        self.left.traverse(callback);
        self.right.traverse(callback);
    }

    fn to_string(&self) -> String {
        let left = to_string(&self.left, self.precedence());
        let right = to_string(&self.right, self.precedence());
        match self.op {
            BinaryOperator::OR => format!("{} | {}", left, right),
            BinaryOperator::XOR => format!("{} ^ {}", left, right),
            BinaryOperator::AND => format!("{} & {}", left, right),
            BinaryOperator::EQ => format!("{} = {}", left, right),
            BinaryOperator::IMP => format!("{} => {}", left, right)
        }
    }

    fn to_dump_string(&self) -> String {
        match self.op {
            BinaryOperator::OR => format!("Or({},{})", self.left.to_dump_string(), self.right.to_dump_string()),
            BinaryOperator::XOR => format!("Xor({},{})", self.left.to_dump_string(), self.right.to_dump_string()),
            BinaryOperator::AND => format!("And({},{})", self.left.to_dump_string(), self.right.to_dump_string()),
            BinaryOperator::EQ => format!("Eq({},{})", self.left.to_dump_string(), self.right.to_dump_string()),
            BinaryOperator::IMP => format!("Imp({},{})", self.left.to_dump_string(), self.right.to_dump_string())
        }
    }
}


/*
 * Tests
 */

#[cfg(test)]
mod tests {
    use super::*;

    impl EvaluationContext {
        pub fn set(&mut self, name: &str, value: bool) {
            self.values.insert(name.to_string(), value);
        }
    }
    
    #[test]
    fn value_tests() {
        let vars = BTreeSet::new();
        let ctxt = EvaluationContext::new(vars);

        let expr = Value::new(true);
        assert_eq!(expr.eval(&ctxt), true);
        assert_eq!(expr.to_string(), "1");

        let expr = Value::new(false);
        assert_eq!(expr.eval(&ctxt), false);
        assert_eq!(expr.to_string(), "0");
    }

    #[test]
    fn variable_tests() {
        let mut vars = BTreeSet::new();
        vars.insert(String::from("a"));
        let mut ctxt = EvaluationContext::new(vars);
        let expr = Variable::new("a");

        ctxt.set("a", true);
        assert_eq!(expr.eval(&ctxt), true);

        ctxt.set("a", false);
        assert_eq!(expr.eval(&ctxt), false);

        assert_eq!(expr.to_string(), "a");
    }

    #[test]
    fn neg_tests() {
        let mut vars = BTreeSet::new();
        vars.insert(String::from("a"));
        let mut ctxt = EvaluationContext::new(vars);
        let a = Variable::new("a");
        let expr = UnaryExpression::new(UnaryOperator::NEG, Box::new(a));

        ctxt.set("a", true);
        assert_eq!(expr.eval(&ctxt), false);

        ctxt.set("a", false);
        assert_eq!(expr.eval(&ctxt), true);

        assert_eq!(expr.to_string(), "!a");
    }

    #[test]
    fn or_tests() {
        let mut vars = BTreeSet::new();
        vars.insert(String::from("a"));
        vars.insert(String::from("b"));
        let mut ctxt = EvaluationContext::new(vars);
        let a = Variable::new("a");
        let b = Variable::new("b");
        let expr = BinaryExpression::new(BinaryOperator::OR, Box::new(a), Box::new(b));

        ctxt.set("a", true);
        ctxt.set("b", true);
        assert_eq!(expr.eval(&ctxt), true);

        ctxt.set("a", true);
        ctxt.set("b", false);
        assert_eq!(expr.eval(&ctxt), true);

        ctxt.set("a", false);
        ctxt.set("b", true);
        assert_eq!(expr.eval(&ctxt), true);

        ctxt.set("a", false);
        ctxt.set("b", false);
        assert_eq!(expr.eval(&ctxt), false);

        assert_eq!(expr.to_string(), "a | b");
    }

    #[test]
    fn xor_tests() {
        let mut vars = BTreeSet::new();
        vars.insert(String::from("a"));
        vars.insert(String::from("b"));
        let mut ctxt = EvaluationContext::new(vars);
        let a = Variable::new("a");
        let b = Variable::new("b");
        let expr = BinaryExpression::new(BinaryOperator::XOR, Box::new(a), Box::new(b));

        ctxt.set("a", true);
        ctxt.set("b", true);
        assert_eq!(expr.eval(&ctxt), false);

        ctxt.set("a", true);
        ctxt.set("b", false);
        assert_eq!(expr.eval(&ctxt), true);

        ctxt.set("a", false);
        ctxt.set("b", true);
        assert_eq!(expr.eval(&ctxt), true);

        ctxt.set("a", false);
        ctxt.set("b", false);
        assert_eq!(expr.eval(&ctxt), false);

        assert_eq!(expr.to_string(), "a ^ b");
    }

    #[test]
    fn and_tests() {
        let mut vars = BTreeSet::new();
        vars.insert(String::from("a"));
        vars.insert(String::from("b"));
        let mut ctxt = EvaluationContext::new(vars);
        let a = Variable::new("a");
        let b = Variable::new("b");
        let expr = BinaryExpression::new(BinaryOperator::AND, Box::new(a), Box::new(b));

        ctxt.set("a", true);
        ctxt.set("b", true);
        assert_eq!(expr.eval(&ctxt), true);

        ctxt.set("a", true);
        ctxt.set("b", false);
        assert_eq!(expr.eval(&ctxt), false);

        ctxt.set("a", false);
        ctxt.set("b", true);
        assert_eq!(expr.eval(&ctxt), false);

        ctxt.set("a", false);
        ctxt.set("b", false);
        assert_eq!(expr.eval(&ctxt), false);

        assert_eq!(expr.to_string(), "a & b");
    }

    #[test]
    fn imp_tests() {
        let mut vars = BTreeSet::new();
        vars.insert(String::from("a"));
        vars.insert(String::from("b"));
        let mut ctxt = EvaluationContext::new(vars);
        let a = Variable::new("a");
        let b = Variable::new("b");
        let expr = BinaryExpression::new(BinaryOperator::IMP, Box::new(a), Box::new(b));

        ctxt.set("a", true);
        ctxt.set("b", true);
        assert_eq!(expr.eval(&ctxt), true);

        ctxt.set("a", true);
        ctxt.set("b", false);
        assert_eq!(expr.eval(&ctxt), false);

        ctxt.set("a", false);
        ctxt.set("b", true);
        assert_eq!(expr.eval(&ctxt), true);

        ctxt.set("a", false);
        ctxt.set("b", false);
        assert_eq!(expr.eval(&ctxt), true);

        assert_eq!(expr.to_string(), "a => b");
    }

    #[test]
    fn eq_tests() {
        let mut vars = BTreeSet::new();
        vars.insert(String::from("a"));
        vars.insert(String::from("b"));
        let mut ctxt = EvaluationContext::new(vars);
        let a = Variable::new("a");
        let b = Variable::new("b");
        let expr = BinaryExpression::new(BinaryOperator::EQ, Box::new(a), Box::new(b));

        ctxt.set("a", true);
        ctxt.set("b", true);
        assert_eq!(expr.eval(&ctxt), true);

        ctxt.set("a", true);
        ctxt.set("b", false);
        assert_eq!(expr.eval(&ctxt), false);

        ctxt.set("a", false);
        ctxt.set("b", true);
        assert_eq!(expr.eval(&ctxt), false);

        ctxt.set("a", false);
        ctxt.set("b", false);
        assert_eq!(expr.eval(&ctxt), true);

        assert_eq!(expr.to_string(), "a = b");
    }
}
