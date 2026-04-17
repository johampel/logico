use crate::expression::Expr;
use crate::tokens::{ParseError, Token, TokenKind};

pub fn parse<'a>(tokens: &[Token<'a>]) -> Result<Expr<'a>, ParseError> {
    match tokens.len() {
        0 => Err(ParseError {
            pos: 0,
            len: 0,
            message: String::from("Missing input"),
        }),

        1 => parse_single_token_expression(&tokens[0]),

        _ => match find_top_level_operator(tokens) {
            Some(pos) => parse_operator_expression(tokens, pos),
            _ => parse_parenthesis_expression(tokens),
        },
    }
}

fn parse_parenthesis_expression<'a>(tokens: &[Token<'a>]) -> Result<Expr<'a>, ParseError> {
    if !matches!(tokens[0].kind, TokenKind::OpenParenthesis) {
        // We have at least two tokens...
        return Err(ParseError {
            pos: tokens[1].pos,
            len: tokens[1].len(),
            message: "operator expected".into(),
        });
    }

    let mut plevel = 0;

    for i in 0..tokens.len() {
        match tokens[i].kind {
            TokenKind::OpenParenthesis => plevel += 1,
            TokenKind::CloseParenthesis => {
                plevel -= 1;
                if plevel < 0 && i + 1 < tokens.len() {
                    return Err(ParseError {
                        pos: tokens[i + 1].pos,
                        len: tokens[i + 1].len(),
                        message: "operator expected".into(),
                    });
                }
            }
            _ => {}
        }
    }

    if plevel > 0 {
        return Err(ParseError {
            pos: tokens[tokens.len() - 1].pos + tokens[tokens.len() - 1].len(),
            len: 0,
            message: "\")\" expected".into(),
        });
    }

    parse(&tokens[1..(tokens.len() - 1)])
}

fn parse_operator_expression<'a>(
    tokens: &[Token<'a>],
    op_pos: usize,
) -> Result<Expr<'a>, ParseError> {
    let token = &tokens[op_pos];

    let right = if op_pos < tokens.len() - 1 {
        parse(&tokens[(op_pos + 1)..])?
    } else {
        return Err(ParseError {
            pos: token.pos + token.len(),
            len: 0,
            message: "missing right hand side operand".into(),
        });
    };

    let n = token.name();
    if op_pos > 0 {
        let left = parse(&tokens[0..op_pos])?;
        match n {
            "|" => Ok(Expr::Or(Box::new(left), Box::new(right))),
            "&" => Ok(Expr::And(Box::new(left), Box::new(right))),
            "^" => Ok(Expr::Xor(Box::new(left), Box::new(right))),
            "=" => Ok(Expr::Eq(Box::new(left), Box::new(right))),
            "=>" => Ok(Expr::Imp(Box::new(left), Box::new(right))),
            "!" => Err(ParseError {
                pos: tokens[0].pos,
                len: tokens[op_pos - 1].pos + tokens[op_pos - 1].len(),
                message: "unexpected left hand side operand".into(),
            }),
            _ => Err(ParseError {
                pos: token.pos,
                len: token.len(),
                message: format!("unexpected operator: \"{}\"", n),
            }),
        }
    } else {
        // Unary expression
        if n == "!" {
            Ok(Expr::Not(Box::new(right)))
        } else if n == "|" || n == "&" || n == "^" || n == "=" || n == "=>" {
            Err(ParseError {
                pos: token.pos,
                len: token.len(),
                message: "missing left hand side operand".into(),
            })
        } else {
            Err(ParseError {
                pos: token.pos,
                len: token.len(),
                message: format!("unknown operator \"{n}\""),
            })
        }
    }
}

fn parse_single_token_expression<'a>(token: &Token<'a>) -> Result<Expr<'a>, ParseError> {
    match token.kind {
        TokenKind::Value(value) => Ok(Expr::Value(value)),
        TokenKind::Variable(name) => Ok(Expr::Variable(name)),
        _ => Err(ParseError {
            pos: token.pos,
            len: token.len(),
            message: "value or variable expected".into(),
        }),
    }
}

fn find_top_level_operator(tokens: &[Token]) -> Option<usize> {
    let mut plevel = 0;
    let mut result: Option<(usize, &Token)> = None;

    for i in 0..tokens.len() {
        let current = &tokens[i];
        match current.kind {
            TokenKind::Operator(_) => {
                if plevel == 0 {
                    if has_higher_precedence(result, current) {
                        result = Some((i, current));
                    }
                }
            }
            TokenKind::OpenParenthesis => plevel += 1,
            TokenKind::CloseParenthesis => plevel -= 1,
            _ => {}
        }
    }

    result.map(|(pos, _token)| pos)
}

fn has_higher_precedence(current: Option<(usize, &Token)>, token: &Token) -> bool {
    if let Some(current) = current
        && let TokenKind::Operator(current_op) = &current.1.kind
        && let TokenKind::Operator(new_op) = &token.kind
        && get_precedence(current_op) <= get_precedence(new_op)
    {
        false
    } else {
        true
    }
}

fn get_precedence(operator: &str) -> usize {
    match operator {
        "=" | "=>" => 0,
        "|" | "^" => 1,
        "&" => 2,
        "!" => 3,
        _ => panic!("unsupported operator '{}'", operator),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{
        find_top_level_operator, parse, parse_operator_expression, parse_parenthesis_expression,
        parse_single_token_expression,
    };
    use crate::tokens::tokenize;

    #[test]
    fn find_top_level_operator_returns_none_if_no_operator() {
        let tokens = tokenize("a b").unwrap_or_else(|_| vec![]);
        assert_eq!(find_top_level_operator(&tokens), None)
    }

    #[test]
    fn find_top_level_operator_returns_none_if_operators_in_paranthesis() {
        let tokens = tokenize("(a &  b)").unwrap_or_else(|_| vec![]);
        assert_eq!(find_top_level_operator(&tokens), None)
    }

    #[test]
    fn find_top_level_operator_returns_first_matching_operator() {
        let tokens = tokenize("a & b & c").unwrap_or_else(|_| vec![]);
        assert_eq!(find_top_level_operator(&tokens), Some(1))
    }

    #[test]
    fn find_top_level_operator_returns_operator_with_highest_precedence() {
        let tokens = tokenize("!a & b ^ c | d = e => f").unwrap_or_else(|_| vec![]);
        assert_eq!(find_top_level_operator(&tokens), Some(8));

        let tokens = tokenize("!a & b ^ c | e => f").unwrap_or_else(|_| vec![]);
        assert_eq!(find_top_level_operator(&tokens), Some(8));

        let tokens = tokenize("!a & b ^ c | e").unwrap_or_else(|_| vec![]);
        assert_eq!(find_top_level_operator(&tokens), Some(4));

        let tokens = tokenize("!a & b | e").unwrap_or_else(|_| vec![]);
        assert_eq!(find_top_level_operator(&tokens), Some(4));

        let tokens = tokenize("!a & b").unwrap_or_else(|_| vec![]);
        assert_eq!(find_top_level_operator(&tokens), Some(2));

        let tokens = tokenize("!a").unwrap_or_else(|_| vec![]);
        assert_eq!(find_top_level_operator(&tokens), Some(0));
    }

    #[test]
    fn parse_single_token_expression_return_ok_for_value_or_variable_token() {
        let tokens = tokenize("0 1 a bc").unwrap_or_else(|_| vec![]);

        let result = parse_single_token_expression(tokens.get(0).unwrap());
        assert_eq!(format!("{:?}", result), "Ok(Value(false))");

        let result = parse_single_token_expression(tokens.get(1).unwrap());
        assert_eq!(format!("{:?}", result), "Ok(Value(true))");

        let result = parse_single_token_expression(tokens.get(2).unwrap());
        assert_eq!(format!("{:?}", result), "Ok(Variable(\"a\"))");

        let result = parse_single_token_expression(tokens.get(3).unwrap());
        assert_eq!(format!("{:?}", result), "Ok(Variable(\"bc\"))");
    }

    #[test]
    fn parse_single_token_expression_return_err_for_operators() {
        let tokens = tokenize("& ! ( )").unwrap_or_else(|_| vec![]);

        let result = parse_single_token_expression(tokens.get(0).unwrap());
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"value or variable expected\", pos: 0, len: 1 })"
        );

        let result = parse_single_token_expression(tokens.get(1).unwrap());
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"value or variable expected\", pos: 2, len: 1 })"
        );

        let result = parse_single_token_expression(tokens.get(2).unwrap());
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"value or variable expected\", pos: 4, len: 1 })"
        );

        let result = parse_single_token_expression(tokens.get(3).unwrap());
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"value or variable expected\", pos: 6, len: 1 })"
        );
    }

    #[test]
    fn parse_operator_expression_return_err_if_rhs_not_found() {
        let tokens = tokenize("!").unwrap_or_else(|_| vec![]);
        let result = parse_operator_expression(&tokens, 0);
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"missing right hand side operand\", pos: 1, len: 0 })"
        );

        let tokens = tokenize("A&").unwrap_or_else(|_| vec![]);
        let result = parse_operator_expression(&tokens, 1);
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"missing right hand side operand\", pos: 2, len: 0 })"
        );
    }

    #[test]
    fn parse_operator_expression_return_err_if_rhs_invalid() {
        let tokens = tokenize("!|").unwrap_or_else(|_| vec![]);
        let result = parse_operator_expression(&tokens, 0);
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"value or variable expected\", pos: 1, len: 1 })"
        );

        let tokens = tokenize("A&|").unwrap_or_else(|_| vec![]);
        let result = parse_operator_expression(&tokens, 1);
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"value or variable expected\", pos: 2, len: 1 })"
        );
    }

    #[test]
    fn parse_operator_expression_return_err_if_lhs_not_found() {
        let tokens = tokenize("&a").unwrap_or_else(|_| vec![]);
        let result = parse_operator_expression(&tokens, 0);
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"missing left hand side operand\", pos: 0, len: 1 })"
        );
    }

    #[test]
    fn parse_operator_expression_return_err_if_lhs_not_expected() {
        let tokens = tokenize("a!b").unwrap_or_else(|_| vec![]);
        let result = parse_operator_expression(&tokens, 1);
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"unexpected left hand side operand\", pos: 0, len: 1 })"
        );
    }

    #[test]
    fn parse_operator_expression_return_ok_for_correct_expressions() {
        let tokens = tokenize("!a a|b a&b a^b a=>b a=b").unwrap_or_else(|_| vec![]);

        let result = parse_operator_expression(&tokens[..2], 0);
        assert_eq!(format!("{:?}", result), "Ok(Not(Variable(\"a\")))");

        let result = parse_operator_expression(&tokens[2..5], 1);
        assert_eq!(
            format!("{:?}", result),
            "Ok(Or(Variable(\"a\"), Variable(\"b\")))"
        );

        let result = parse_operator_expression(&tokens[5..8], 1);
        assert_eq!(
            format!("{:?}", result),
            "Ok(And(Variable(\"a\"), Variable(\"b\")))"
        );

        let result = parse_operator_expression(&tokens[8..11], 1);
        assert_eq!(
            format!("{:?}", result),
            "Ok(Xor(Variable(\"a\"), Variable(\"b\")))"
        );

        let result = parse_operator_expression(&tokens[11..14], 1);
        assert_eq!(
            format!("{:?}", result),
            "Ok(Imp(Variable(\"a\"), Variable(\"b\")))"
        );

        let result = parse_operator_expression(&tokens[14..], 1);
        assert_eq!(
            format!("{:?}", result),
            "Ok(Eq(Variable(\"a\"), Variable(\"b\")))"
        );
    }

    #[test]
    fn parse_paranthesis_expression_return_err_if_not_starting_with_paranthesis_open() {
        let tokens = tokenize("a&b").unwrap_or_else(|_| vec![]);
        let result = parse_parenthesis_expression(&tokens);
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"operator expected\", pos: 1, len: 1 })"
        );
    }

    #[test]
    fn parse_paranthesis_expression_return_err_if_missing_paranthesis_close() {
        let tokens = tokenize("(a").unwrap_or_else(|_| vec![]);
        let result = parse_parenthesis_expression(&tokens);
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"\\\")\\\" expected\", pos: 2, len: 0 })"
        );
    }

    #[test]
    fn parse_paranthesis_expression_return_err_if_unblanced_paranthesis() {
        let tokens = tokenize("(a))").unwrap_or_else(|_| vec![]);
        let result = parse_parenthesis_expression(&tokens);
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"operator expected\", pos: 2, len: 1 })"
        );
    }

    #[test]
    fn parse_paranthesis_expression_return_err_if_more_than_one_paranthesis() {
        let tokens = tokenize("(a)(b)").unwrap_or_else(|_| vec![]);
        let result = parse_parenthesis_expression(&tokens);
        assert_eq!(
            format!("{:?}", result),
            "Err(ParseError { message: \"operator expected\", pos: 2, len: 1 })"
        );
    }

    #[test]
    fn parse_paranthesis_expression_return_ok_for_complex_expressions_with_paranthesis() {
        let tokens = tokenize("((a|b)&c)").unwrap_or_else(|_| vec![]);
        let result = parse_parenthesis_expression(&tokens);
        assert_eq!(
            format!("{:?}", result),
            "Ok(And(Or(Variable(\"a\"), Variable(\"b\")), Variable(\"c\")))"
        );
    }

    #[test]
    fn parse_parses_complex_expressions() {
        let tokens = tokenize("(a|b&c) = ((a|b)&c)").unwrap_or_else(|_| vec![]);
        let result = parse(&tokens);
        assert_eq!(
            format!("{:?}", result),
            "Ok(Eq(Or(Variable(\"a\"), And(Variable(\"b\"), Variable(\"c\"))), And(Or(Variable(\"a\"), Variable(\"b\")), Variable(\"c\"))))"
        );
    }
}
