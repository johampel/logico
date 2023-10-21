use crate::expression;
use crate::expression::{BinaryExpression, BinaryOperator, UnaryExpression, UnaryOperator};
use crate::tokens::{ParseError, Token, token_len, token_name, token_pos};

pub fn parse(tokens: &[Token]) -> Result<Box<dyn expression::Expression>, ParseError> {
    match tokens.len() {
        0 => Err(ParseError {
            pos: 0,
            len: 0,
            message: String::from("Missing input"),
        }),

        1 => parse_single_token_expression(&tokens[0]),

        _ => match find_top_level_operator(tokens) {
            Some(pos) => parse_operator_expression(tokens, pos),
            _ => parse_paranthesis_expression(tokens),
        }
    }
}

fn parse_paranthesis_expression(tokens: &[Token]) -> Result<Box<dyn expression::Expression>, ParseError> {
    match tokens[0] {
        Token::OpenParanthesis(_) => (),
        _ => return Err(ParseError {
            pos: token_pos(&tokens[1]),
            len: token_len(&tokens[1]),
            message: String::from("operator expected"),
        })
    }

    let mut plevel = 0;

    for i in 0..tokens.len() {
        match &tokens[i] {
            Token::OpenParanthesis(_) => plevel += 1,
            Token::CloseParanthesis(_) => {
                plevel -= 1;
                if plevel < 0 && i + 1 < tokens.len() {
                    return Err(ParseError {
                        pos: token_pos(&tokens[i + 1]),
                        len: token_len(&tokens[i + 1]),
                        message: String::from("operator expected"),
                    });
                }
            }
            _ => {}
        }
    }

    if plevel > 0 {
        return Err(ParseError {
            pos: token_pos(&tokens[tokens.len() - 1]) + token_len(&tokens[tokens.len() - 1]),
            len: 0,
            message: String::from("\")\" expected"),
        });
    }

    parse(&tokens[1..(tokens.len() - 1)])
}

fn parse_operator_expression(tokens: &[Token], op_pos: usize) -> Result<Box<dyn expression::Expression>, ParseError> {
    let token = &tokens[op_pos];
    let left = if op_pos > 0 {
        match parse(&tokens[0..op_pos]) {
            Ok(expr) => Some(expr),
            Err(err) => return Err(err)
        }
    } else {
        None
    };
    let right = if op_pos < tokens.len() - 1 {
        match parse(&tokens[(op_pos + 1)..]) {
            Ok(expr) => Some(expr),
            Err(err) => return Err(err)
        }
    } else {
        None
    };

    if right.is_none() {
        return Err(ParseError {
            pos: token_pos(token) + token_len(token),
            len: 0,
            message: String::from("missing right hand side operand"),
        });
    }
    let right = right.unwrap();

    if left.is_some() {
        let left = left.unwrap();
        match token_name(token) {
            "!" => Err(ParseError {
                pos: token_pos(&tokens[0]),
                len: token_pos(&tokens[op_pos - 1]) + token_len(&tokens[op_pos - 1]),
                message: String::from("unexpected left hand side operand"),
            }),
            "|" => Ok(Box::new(BinaryExpression::new(BinaryOperator::OR, left, right))),
            "&" => Ok(Box::new(BinaryExpression::new(BinaryOperator::AND, left, right))),
            "^" => Ok(Box::new(BinaryExpression::new(BinaryOperator::XOR, left, right))),
            "=" => Ok(Box::new(BinaryExpression::new(BinaryOperator::EQ, left, right))),
            "=>" => Ok(Box::new(BinaryExpression::new(BinaryOperator::IMP, left, right))),
            _ => Err(ParseError {
                pos: token_pos(token),
                len: token_len(token),
                message: String::from(format!("unknown operator '{}'", token_name(token))),
            })
        }
    } else {
        match token_name(token) {
            "!" => Ok(Box::new(UnaryExpression::new(UnaryOperator::NEG, right))),
            "|" | "&" | "^" | "=" | "=>" => Err(ParseError {
                pos: token_pos(token),
                len: 0,
                message: String::from("missing left hand side operand"),
            }),
            _ => Err(ParseError {
                pos: token_pos(token),
                len: token_len(token),
                message: String::from(format!("unknown operator '{}'", token_name(token))),
            })
        }
    }
}

fn parse_single_token_expression(token: &Token) -> Result<Box<dyn expression::Expression>, ParseError> {
    match token {
        Token::Value(_, value) => Ok(Box::new(expression::Value::new(*value))),
        Token::Variable(_, name) => Ok(Box::new(expression::Variable::new(name))),
        Token::Operator(pos, name) => Err(ParseError { pos: *pos, len: name.len(), message: String::from("value or variable expected") }),
        Token::OpenParanthesis(pos) => Err(ParseError { pos: *pos, len: 1, message: String::from("value or variable expected") }),
        Token::CloseParanthesis(pos) => Err(ParseError { pos: *pos, len: 1, message: String::from("value or variable expected") })
    }
}

fn find_top_level_operator(tokens: &[Token]) -> Option<usize> {
    let mut plevel = 0;
    let mut result: Option<(usize, &Token)> = None;

    for i in 0..tokens.len() {
        let current = &tokens[i];
        match current {
            Token::Operator(_, _) => if plevel == 0 {
                if has_higher_precedence(result, current) {
                    result = Some((i, current));
                }
            },
            Token::OpenParanthesis(_) => plevel += 1,
            Token::CloseParanthesis(_) => plevel -= 1,
            _ => {}
        }
    }

    result.map(|(pos, _token)| pos)
}

fn has_higher_precedence(current: Option<(usize, &Token)>, new: &Token) -> bool {
    match current {
        Some((_, Token::Operator(_, cname))) => {
            match new {
                Token::Operator(_, nname) =>
                    get_precedence(cname) > get_precedence(nname),
                _ =>
                    false
            }
        }
        _ => true
    }
}

fn get_precedence(operator: &str) -> usize {
    match operator {
        "=" | "=>" => 0,
        "|" | "^" => 1,
        "&" => 2,
        "!" => 3,
        _ => panic!("unsupported operator '{}'", operator)
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::{find_top_level_operator, parse_operator_expression, parse_paranthesis_expression, parse_single_token_expression, parse};
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
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap().to_dump_string(), "Value(0)");

        let result = parse_single_token_expression(tokens.get(1).unwrap());
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap().to_dump_string(), "Value(1)");

        let result = parse_single_token_expression(tokens.get(2).unwrap());
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap().to_dump_string(), "Variable(a)");

        let result = parse_single_token_expression(tokens.get(3).unwrap());
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap().to_dump_string(), "Variable(bc)");
    }

    #[test]
    fn parse_single_token_expression_return_err_for_operators() {
        let tokens = tokenize("& ! ( )").unwrap_or_else(|_| vec![]);

        let result = parse_single_token_expression(tokens.get(0).unwrap());
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "value or variable expected");

        let result = parse_single_token_expression(tokens.get(1).unwrap());
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "value or variable expected");

        let result = parse_single_token_expression(tokens.get(2).unwrap());
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "value or variable expected");

        let result = parse_single_token_expression(tokens.get(3).unwrap());
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "value or variable expected");
    }

    #[test]
    fn parse_operator_expression_return_err_if_rhs_not_found() {
        let tokens = tokenize("!").unwrap_or_else(|_| vec![]);
        let result = parse_operator_expression(&tokens, 0);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "missing right hand side operand");

        let tokens = tokenize("A&").unwrap_or_else(|_| vec![]);
        let result = parse_operator_expression(&tokens, 1);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "missing right hand side operand");
    }

    #[test]
    fn parse_operator_expression_return_err_if_rhs_invalid() {
        let tokens = tokenize("!|").unwrap_or_else(|_| vec![]);
        let result = parse_operator_expression(&tokens, 0);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "value or variable expected");

        let tokens = tokenize("A&|").unwrap_or_else(|_| vec![]);
        let result = parse_operator_expression(&tokens, 1);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "value or variable expected");
    }

    #[test]
    fn parse_operator_expression_return_err_if_lhs_not_found() {
        let tokens = tokenize("&a").unwrap_or_else(|_| vec![]);
        let result = parse_operator_expression(&tokens, 0);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "missing left hand side operand");
    }

    #[test]
    fn parse_operator_expression_return_err_if_lhs_not_expected() {
        let tokens = tokenize("a!b").unwrap_or_else(|_| vec![]);
        let result = parse_operator_expression(&tokens, 1);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "unexpected left hand side operand");
    }

    #[test]
    fn parse_operator_expression_return_ok_for_correct_expressions() {
        let tokens = tokenize("!a a|b a&b a^b a=>b a=b").unwrap_or_else(|_| vec![]);

        let result = parse_operator_expression(&tokens[..2], 0);
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap().to_dump_string(), "Neg(Variable(a))");

        let result = parse_operator_expression(&tokens[2..5], 1);
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap().to_dump_string(), "Or(Variable(a),Variable(b))");

        let result = parse_operator_expression(&tokens[5..8], 1);
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap().to_dump_string(), "And(Variable(a),Variable(b))");

        let result = parse_operator_expression(&tokens[8..11], 1);
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap().to_dump_string(), "Xor(Variable(a),Variable(b))");

        let result = parse_operator_expression(&tokens[11..14], 1);
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap().to_dump_string(), "Imp(Variable(a),Variable(b))");

        let result = parse_operator_expression(&tokens[14..], 1);
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap().to_dump_string(), "Eq(Variable(a),Variable(b))");
    }

    #[test]
    fn parse_paranthesis_expression_return_err_if_not_starting_with_paranthesis_open() {
        let tokens = tokenize("a&b").unwrap_or_else(|_| vec![]);
        let result = parse_paranthesis_expression(&tokens);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "operator expected");
    }

    #[test]
    fn parse_paranthesis_expression_return_err_if_missing_paranthesis_close() {
        let tokens = tokenize("(a").unwrap_or_else(|_| vec![]);
        let result = parse_paranthesis_expression(&tokens);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "\")\" expected");
    }

    #[test]
    fn parse_paranthesis_expression_return_err_if_unblanced_paranthesis() {
        let tokens = tokenize("(a))").unwrap_or_else(|_| vec![]);
        let result = parse_paranthesis_expression(&tokens);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "operator expected");
    }

    #[test]
    fn parse_paranthesis_expression_return_err_if_more_than_one_paranthesis() {
        let tokens = tokenize("(a)(b)").unwrap_or_else(|_| vec![]);
        let result = parse_paranthesis_expression(&tokens);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap().message, "operator expected");
    }

    #[test]
    fn parse_paranthesis_expression_return_ok_for_complex_expressions_with_paranthesis() {
        let tokens = tokenize("((a|b)&c)").unwrap_or_else(|_| vec![]);
        let result = parse_paranthesis_expression(&tokens);
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap().to_dump_string(), "And(Or(Variable(a),Variable(b)),Variable(c))");
    }

    #[test]
    fn parse_parses_complex_expressions() {
        let tokens = tokenize("(a|b&c) = ((a|b)&c)").unwrap_or_else(|_| vec![]);
        let result = parse(&tokens);
        assert_eq!(result.is_err(), false);
        assert_eq!(result.unwrap().to_dump_string(), "Eq(Or(Variable(a),And(Variable(b),Variable(c))),And(Or(Variable(a),Variable(b)),Variable(c)))");
    }
}
