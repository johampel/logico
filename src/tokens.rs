#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub pos: usize,
    pub kind: TokenKind<'a>,
}

impl<'a> Token<'a> {
    pub fn open_parenthesis(pos: usize) -> Token<'a> {
        Token {
            pos,
            kind: TokenKind::OpenParenthesis,
        }
    }

    pub fn close_parenthesis(pos: usize) -> Token<'a> {
        Token {
            pos,
            kind: TokenKind::CloseParenthesis,
        }
    }

    pub fn value(pos: usize, value: bool) -> Token<'a> {
        Token {
            pos,
            kind: TokenKind::Value(value),
        }
    }

    pub fn variable(pos: usize, variable: &'a str) -> Token<'a> {
        Token {
            pos,
            kind: TokenKind::Variable(variable),
        }
    }

    pub fn operator(pos: usize, operator: &'a str) -> Token<'a> {
        Token {
            pos,
            kind: TokenKind::Operator(operator),
        }
    }

    pub fn len(&self) -> usize {
        self.kind.len()
    }

    pub fn name(&self) -> &str {
        self.kind.name()
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind<'a> {
    Value(bool),
    Variable(&'a str),
    Operator(&'a str),
    OpenParenthesis,
    CloseParenthesis,
}

impl<'a> TokenKind<'a> {
    fn len(&self) -> usize {
        match self {
            TokenKind::Variable(x) => x.len(),
            TokenKind::Operator(x) => x.len(),
            _ => 1,
        }
    }
    fn name(&self) -> &str {
        match self {
            TokenKind::Value(cval) => {
                if *cval {
                    "1"
                } else {
                    "0"
                }
            }
            TokenKind::Variable(x) => x,
            TokenKind::Operator(x) => x,
            TokenKind::OpenParenthesis => "(",
            TokenKind::CloseParenthesis => ")",
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub pos: usize,
    pub len: usize,
}

pub fn tokenize<'a>(str: &'a str) -> Result<Vec<Token<'a>>, ParseError> {
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
            tokens.push(Token::variable(pos, &rest[..i]));
            rest = &rest[i..];
            pos += i;
            continue;
        }

        // Check other literals
        if rest.starts_with("=>") {
            tokens.push(Token::operator(pos, "=>"));
            rest = &rest[2..];
            pos += 2;
            continue;
        }

        // Any other character
        let ch = char_at(rest, 0);
        match ch {
            '0' | '1' => tokens.push(Token::value(pos, ch == '1')),
            '&' | '|' | '^' | '=' | '!' => tokens.push(Token::operator(pos, &rest[0..1])),
            '(' => tokens.push(Token::open_parenthesis(pos)),
            ')' => tokens.push(Token::close_parenthesis(pos)),
            _ => {
                return Err(ParseError {
                    pos,
                    len: 1,
                    message: String::from(format!("Invalid character '{}'", char_at(rest, 0))),
                });
            }
        }
        rest = &rest[1..];
        pos += 1;
    }

    if tokens.is_empty() {
        return Err(ParseError {
            message: String::from("no input."),
            pos: str.len(),
            len: 0,
        });
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
        assert_eq!(tokens, vec![Token::variable(0, "abc")])
    }

    #[test]
    fn tokenize_parses_two_letter_tokens() {
        let res = tokenize("=>");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::operator(0, "=>")]);
    }

    #[test]
    fn tokenize_parses_one_letter_tokens() {
        let res = tokenize("0");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::value(0, false)]);

        let res = tokenize("1");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::value(0, true)]);

        let res = tokenize("&");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::operator(0, "&")]);

        let res = tokenize("|");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::operator(0, "|")]);

        let res = tokenize("=");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::operator(0, "=")]);

        let res = tokenize("^");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::operator(0, "^")]);

        let res = tokenize("(");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::open_parenthesis(0)]);

        let res = tokenize(")");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(tokens, vec![Token::close_parenthesis(0)]);
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
        assert_eq!(
            tokens,
            vec![
                Token::variable(0, "a"),
                Token::operator(1, "&"),
                Token::variable(2, "b"),
                Token::operator(3, "^"),
                Token::variable(4, "def"),
            ]
        );
    }

    #[test]
    fn tokenize_ignores_whitespace() {
        let res = tokenize(" a | b = a & b ");
        assert_eq!(res.is_err(), false);
        let tokens = res.unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::variable(1, "a"),
                Token::operator(3, "|"),
                Token::variable(5, "b"),
                Token::operator(7, "="),
                Token::variable(9, "a"),
                Token::operator(11, "&"),
                Token::variable(13, "b")
            ]
        );
    }
}
