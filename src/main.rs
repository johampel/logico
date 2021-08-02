use std::cell::RefCell;
use std::collections::BTreeSet;
use std::env;
use std::process::exit;

use crate::expression::{EvaluationContext, Expression};
use crate::parser::parse;
use crate::tokens::{ParseError, tokenize};

mod tokens;
mod expression;
mod parser;

fn print_usage(app_name: &str) {
    println!("Evaluates logical expressions");
    println!("usage: {} <expr> [<preset>...]", app_name);
    println!("<expr>:   Is the logical expression to evaluate. An expression consists of values,");
    println!("          variables and operators");
    println!("          `0` represents a `false` value and `1` a `true` value,");
    println!("          any sequence of alphabetic characters is interpreted as a variable name.");
    println!("          The following operators are known:");
    println!("          `&` - logical and            `!`  - Logical negation");
    println!("          `|` - logical or             `=>` - Logical implication");
    println!("          `^` - logical exclusive or   `=`  - Equality");
    println!("          The precedence rules are as follows (objects/operators appear first are");
    println!("          evaluated first): value, variable, `!`, `&`, `^`, `|`, `=>`, `=` ");
    println!("          it is possible to influence the precedence using paranthesis. Examples:");
    println!("          `a&b`,  `(abc | !def) ^ (!abc & def)` `(a=0) & (b=1)`");
    println!("<preset>: A preset predefines the value of a variable when evaluation the");
    println!("          expression. The syntax of a preset is `[+-]<var>`, whereas `-var` means");
    println!("          to preset the variable with `false` (or `0`) and `+var` means to preset");
    println!("          the variable with `true` (or `1`).");
}

fn print_err(app_name: &str, message: &str) {
    eprintln!("*** error {}: {}", app_name, message);
}

fn print_parse_err(app_name: &str, expr: &str, err: ParseError) {
    print_err(app_name, format!("parse error in '{}'", expr).as_str());
    if err.len > 0 {
        eprint_chars(28 + app_name.len() + err.pos, ' ');
        eprint_chars(err.len, '~');
        println!();
    }
    eprint_chars(28 + app_name.len() + err.pos + (err.len / 2), ' ');
    eprintln!("|");
    eprint_chars(28 + app_name.len() + err.pos - err.message.len() / 2, ' ');
    eprintln!("{}", err.message);
}

fn eprint_chars(count: usize, ch: char) -> () {
    for _ in 0..count {
        eprint!("{}", ch);
    }
}

fn print_table_header(ctxt: &EvaluationContext) -> () {
    print!("|");
    for var in &ctxt.variables {
        print!(" {} |", var);
    }
    println!("|   |");

    print!("+");
    for var in &ctxt.variables {
        print_chars(var.len() + 2, '-');
        print!("+");
    }
    println!("+---+");
}

fn print_table_result(ctxt: &EvaluationContext, result: bool) -> () {
    print!("|");
    for var in &ctxt.variables {
        print_chars(var.len(), ' ');
        print!("{} |", if ctxt.get(var.as_str()) { '1' } else { '0' });
    }
    println!("| {} |", if result { '1' } else { '0' });
}

fn print_chars(count: usize, ch: char) -> () {
    for _ in 0..count {
        print!("{}", ch);
    }
}


fn parse_expr(str: &str) -> Result<Box<dyn Expression>, ParseError> {
    match tokenize(str) {
        Ok(tokens) => {
            match parse(&tokens) {
                Err(err) => Err(err),
                Ok(expr) => Ok(expr)
            }
        }
        Err(err) => Err(err)
    }
}

fn collect_variables(expr: &dyn Expression) -> BTreeSet<String> {
    let mut variables: BTreeSet<String> = BTreeSet::new();
    let names = RefCell::new(&mut variables);
    expr.traverse(&|e| {
        if let Some(var) = e.as_variable() {
            names.borrow_mut().insert(var.name.clone());
        }
    });
    variables
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let app_name = if let Some(index) = args[0].rfind("/") { &args[0][(index + 1)..] } else { &args[0] };
    if args.len() < 2 {
        print_usage(app_name);
        exit(1);
    }

    // Parse expression
    let expr = match parse_expr(&args[1]) {
        Ok(expr) => expr,
        Err(err) => {
            print_parse_err(app_name, &args[1], err);
            exit(1);
        }
    };
    let mut ctx = EvaluationContext::new(collect_variables(expr.as_ref()));

    // parse presets
    for arg in &args[2..] {
        if arg.starts_with("-") || arg.starts_with("+") {
            let var = &arg[1..];
            let val = arg.starts_with("+");
            if let Err(message) = ctx.preset(var, val) {
                eprintln!("warning {}: {}", app_name, message);
            }
        } else {
            print_err(app_name, format!("invalid preset '{}'", arg).as_str());
            exit(1);
        }
    }

    let count: u128 = 1 << ctx.not_preset.len();
    print_table_header(&ctx);
    for i in 0..count {
        ctx.set_not_presets(i);
        print_table_result(&ctx, expr.eval(&ctx));
    }
}

