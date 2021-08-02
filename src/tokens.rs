#[derive(Debug)]
pub enum Token {
    Value(usize, bool),
    Variable(usize, String),
    Operator(usize, String),
    OpenParanthesis(usize),
    CloseParanthesis(usize),
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Token::Value(spos, svalue) => {
                match other {
                    Token::Value(opos, ovalue) => spos == opos && svalue == ovalue,
                    _ => false
                }
            }
            Token::Variable(spos, sname) => {
                match other {
                    Token::Variable(opos, oname) => spos == opos && sname == oname,
                    _ => false
                }
            }
            Token::Operator(spos, sname) => {
                match other {
                    Token::Operator(opos, oname) => spos == opos && sname == oname,
                    _ => false
                }
            }
            Token::OpenParanthesis(spos) => {
                match other {
                    Token::OpenParanthesis(opos) => spos == opos,
                    _ => false
                }
            }
            Token::CloseParanthesis(spos) => {
                match other {
                    Token::CloseParanthesis(opos) => spos == opos,
                    _ => false
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub pos: usize,
    pub len: usize,
}


pub fn token_pos(token: &Token) -> usize {
    match token {
        Token::Value(pos, _) => *pos,
        Token::Variable(pos, _) => *pos,
        Token::Operator(pos, _) => *pos,
        Token::OpenParanthesis(pos) => *pos,
        Token::CloseParanthesis(pos) => *pos,
    }
}

pub fn token_len(token: &Token) -> usize {
    match token {
        Token::Value(_, _) => 1,
        Token::Variable(_, name) => name.len(),
        Token::Operator(_, name) => name.len(),
        Token::OpenParanthesis(_) => 1,
        Token::CloseParanthesis(_) => 1,
    }
}

pub fn token_name(token: &Token) -> &str {
    match token {
        Token::Value(_, value) => if *value { "1" } else { "0" },
        Token::Operator(_, name) => name,
        Token::Variable(_, name) => name,
        Token::OpenParanthesis(_) => "(",
        Token::CloseParanthesis(_) => ")",
    }
}

pub fn tokenize(str: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut rest = str;
    let mut pos: usize = 0;

    while !rest.is_empty() {

        // Check whitespace
        if rest.starts_with(" ") || rest.starts_with("\t") {
            rest = &rest[1..];
            pos += 1;
            continue;
        }

        // Check identifier
        let mut i: usize = 0;
        while i < rest.len() && char_at(rest, i).is_alphabetic() {
            i += 1
        }
        if i > 0 {
            tokens.push(Token::Variable(pos, String::from(&rest[..i])));
            rest = &rest[i..];
            pos += i;
            continue;
        }

        // Check other literals
        if rest.starts_with("=>") {
            tokens.push(Token::Operator(pos, String::from("=>")));
            rest = &rest[2..];
            pos += 2;
            continue;
        }

        // Any other character
        let ch = char_at(rest, 0);
        match ch {
            '0' | '1' => tokens.push(Token::Value(pos, ch == '1')),
            '&' | '|' | '^' | '=' | '!' => tokens.push(Token::Operator(pos, String::from(ch))),
            '(' => tokens.push(Token::OpenParanthesis(pos)),
            ')' => tokens.push(Token::CloseParanthesis(pos)),
            _ => return Err(ParseError { pos, len: 1, message: String::from(format!("Invalid character '{}'", char_at(rest, 0))) })
        }
        rest = &rest[1..];
        pos += 1;
    }

    if tokens.is_empty() {
        return Err(ParseError { message: String::from("no input."), pos: str.len(), len: 0 });
    }
    Ok(tokens)
}


fn char_at(str: &str, pos: usize) -> char {
    str.chars().nth(pos).unwrap()
}

/*
 * Tests
 */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_returns_err_for_empty_str() {
        let res = tokenize("");
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn tokenize_parses_identifiers() {
        let res = tokenize("abc");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::Variable(0, String::from("abc"))])
    }

    #[test]
    fn tokenize_parses_two_letter_tokens() {
        let res = tokenize("=>");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::Operator(0, String::from("=>"))]);
    }

    #[test]
    fn tokenize_parses_one_letter_tokens() {
        let res = tokenize("0");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::Value(0, false)]);

        let res = tokenize("1");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::Value(0, true)]);

        let res = tokenize("&");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::Operator(0, String::from("&"))]);

        let res = tokenize("|");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::Operator(0, String::from("|"))]);

        let res = tokenize("=");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::Operator(0, String::from("="))]);

        let res = tokenize("^");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::Operator(0, String::from("^"))]);

        let res = tokenize("(");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::OpenParanthesis(0)]);

        let res = tokenize(")");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::CloseParanthesis(0)]);
    }

    #[test]
    fn tokenize_returns_err_for_unknown_characters() {
        let res = tokenize("#");
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn tokenize_parses_multiple_tokens() {
        let res = tokenize("a&b^def");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::Variable(0, String::from("a")),
                                Token::Operator(1, String::from("&")),
                                Token::Variable(2, String::from("b")),
                                Token::Operator(3, String::from("^")),
                                Token::Variable(4, String::from("def")), ]);
    }

    #[test]
    fn tokenize_ignores_whitespace() {
        let res = tokenize(" a | b = a & b ");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::Variable(1, String::from("a")),
                                Token::Operator(3, String::from("|")),
                                Token::Variable(5, String::from("b")),
                                Token::Operator(7, String::from("=")),
                                Token::Variable(9, String::from("a")),
                                Token::Operator(11, String::from("&")),
                                Token::Variable(13, String::from("b"))]);
    }
}
