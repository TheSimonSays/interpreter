use std::collections::HashMap;

fn is_digit(ch: char) -> bool {
    (ch as u8) >= '0' as u8 && ch as u8 <= '9' as u8
}

fn is_alpha(ch: char) -> bool {
    (ch as u8 >= 'a' as u8 && ch as u8 <= 'z' as u8) || (ch as u8 >= 'A' as u8 && ch as u8 <= 'Z' as u8) || ch == '_'
}

fn is_alpha_numeric(ch: char) -> bool {
    is_alpha(ch) || is_digit(ch)
}

fn get_keywords_hashmap() -> HashMap<&'static str, TokenType> {
    HashMap::from([
        ("and", TokenType::And),
        ("class", TokenType::Class),
        ("else", TokenType::Else),
        ("false", TokenType::False),
        ("for", TokenType::For),
        ("fun", TokenType::Fun),
        ("if", TokenType::If),
        ("nil", TokenType::Nil),
        ("or", TokenType::Or),
        ("print", TokenType::Print),
        ("return", TokenType::Return),
        ("super", TokenType::Super),
        ("this", TokenType::This),
        ("true", TokenType::True),
        ("var", TokenType::Var),
        ("while", TokenType::While),
    ])
}

pub struct Scanner {
    source: String,
    pub tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,

    keywords: HashMap<&'static str, TokenType>,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            keywords: get_keywords_hashmap(),
        }
    }

    pub fn scan_tokens(self: &mut Self) -> Result<(), String> {
        let mut errors = vec![];
        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(_) => (),
                Err(msg) => errors.push(msg),
            }
            // self.scan_tokens()?;
        }
        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line_number: self.line
        });
        if errors.len() > 0 {
            let mut joined = "".to_string();
            errors.iter().for_each(|msg| {
                joined.push_str(&msg);
                joined.push_str("\n");
            });
            return Err(joined);
        }
        Ok(())
    }
    
    fn is_at_end(self: &Self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(self: &mut Self) -> Result<(), String> {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            '%' => self.add_token(TokenType::Percent),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let token = if self.char_match('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token)
            },
            '=' => {
                let token = if self.char_match('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token);
            },
            '<' => {
                let token = if self.char_match('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token);
            },
            '>' => {
                let token = if self.char_match('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token);
            },
            '/' => {
                if self.char_match('/') {
                    loop {
                        if self.peek() == '\n' || self.is_at_end() {
                            break;
                        }
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            },
            ' ' | '\r' | '\t' => {},
            '\n' => {
                self.line += 1
            },
            '"' => self.string()?,
            c => {
                if is_digit(c) {
                    self.number()?;
                } else if is_alpha(c) {
                    self.identifier();
                } else {
                    return Err(format!("Unrecognized char at line {}: {}", self.line, c));
                }
            },
        }
        Ok(())
    }

    fn identifier(self: &mut Self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let substring = &self.source[self.start..self.current];
        if let Some(&t_type) = self.keywords.get(substring) {
            self.add_token(t_type);
        } else {
            self.add_token(TokenType::Identifier);
        }
    }

    fn number(self: &mut Self) -> Result<(), String>{
        while is_digit(self.peek()) {
            self.advance();
        }
        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }
        let substring = &self.source[self.start..self.current];
        let value = substring.parse::<f64>();
        match value {
            Ok(value) => self.add_token_lit(TokenType::Number, Some(LiteralValue::FloatValue(value))),
            Err(_) => return Err(format!("Could not parse: {}", substring)),
        }
        Ok(())
    }

    fn peek_next(self: &mut Self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        return self.source.chars().nth(self.current + 1).unwrap();
    }

    fn string(self: &mut Self) -> Result<(), String>{
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err("Unterminated string".to_string());
        }
        self.advance();
        let value = &self.source[self.start + 1..self.current - 1];
            // .collect::<String>();

        self.add_token_lit(TokenType::String, Some(LiteralValue::StringValue(value.to_string())));
        Ok(())
    }

    fn peek(self: &mut Self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn char_match(self: &mut Self, ch: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != ch {
            return false;
        } else {
            self.current += 1;
            return true;
        }
    }

    fn advance(self: &mut Self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        c
    }

    fn add_token(self: &mut Self, token_type: TokenType) {
        self.add_token_lit(token_type, None);
    }

    fn add_token_lit(self: &mut Self, token_type: TokenType, literal: Option<LiteralValue>) {
        let text = &self.source[self.start..self.current];

        self.tokens.push(Token {
            token_type: token_type,
            lexeme: text.to_string(),
            literal: literal,
            line_number: self.line,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Percent,
    Semicolon,
    Slash,
    Start,
    Star,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier,
    String,
    Number,

    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum LiteralValue {
    IntValue(i64),
    FloatValue(f64),
    StringValue(String),
    IdentifierValue(String),
}


#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub line_number: usize,
}

impl Token {
    pub fn to_string(self: &Self) -> String {
        format!("{:?} {} {:?}", self.token_type, self.lexeme, self.literal)
    }
}


// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn handle_one_char_tokens() {
//         let source = "(( )) }{";
//         let mut scanner = Scanner::new(source);
//         scanner.scan_tokens();
//         assert_eq!(scanner.tokens.len(), 7);
//         assert_eq!(scanner.tokens[0].token_type, TokenType::LeftParen);
//         assert_eq!(scanner.tokens[1].token_type, TokenType::LeftParen);
//         assert_eq!(scanner.tokens[2].token_type, TokenType::RightParen);
//         assert_eq!(scanner.tokens[3].token_type, TokenType::RightParen);
//         assert_eq!(scanner.tokens[4].token_type, TokenType::RightBrace);
//         assert_eq!(scanner.tokens[5].token_type, TokenType::LeftBrace);
//         assert_eq!(scanner.tokens[6].token_type, TokenType::Eof);
//     }

//     #[test]
//     fn handle_two_char_tokens() {
//         let source = "! != == >=";
//         let mut scanner = Scanner::new(source);
//         scanner.scan_tokens().unwrap();
//         assert_eq!(scanner.tokens.len(), 5);
//         assert_eq!(scanner.tokens[0].token_type, TokenType::Bang);
//         assert_eq!(scanner.tokens[1].token_type, TokenType::BangEqual);
//         assert_eq!(scanner.tokens[2].token_type, TokenType::EqualEqual);
//         assert_eq!(scanner.tokens[3].token_type, TokenType::GreaterEqual);
//         assert_eq!(scanner.tokens[4].token_type, TokenType::Eof);
//     }

//     #[test]
//     fn handle_string_lit() {
//         let source = r#""ABC""#;
//         let mut scanner = Scanner::new(source);
//         scanner.scan_tokens().unwrap();
//         assert_eq!(scanner.tokens.len(), 2);
//         assert_eq!(scanner.tokens[0].token_type, TokenType::String);
//         match scanner.tokens[0].literal.as_ref().unwrap() {
//             LiteralValue::StringValue(value) => assert_eq!(value, &"ABC"),
//             _ => panic!("Incorrect literal value"),
//         }
//         assert_eq!(scanner.tokens[1].token_type, TokenType::Eof);
//     }

//     #[test]
//     fn handle_string_lit_unterminated() {
//         let source = r#""ABC"#;
//         let mut scanner = Scanner::new(source);
//         let result = scanner.scan_tokens();
//         match result {
//             Err(_) => (),
//             _ => panic!("Should have failed"),
//         }
//     }

//     #[test]
//     fn handle_string_lit_multiline() {
//         let source = "\"ABC\ndef\"";
//         let mut scanner = Scanner::new(source);
//         scanner.scan_tokens().unwrap();
//         assert_eq!(scanner.tokens.len(), 2);
//         assert_eq!(scanner.tokens[0].token_type, TokenType::String);
//         match scanner.tokens[0].literal.as_ref().unwrap() {
//             LiteralValue::StringValue(value) => assert_eq!(value, &"ABC\ndef"),
//             _ => panic!("Incorrect literal value"),
//         }
//         assert_eq!(scanner.tokens[1].token_type, TokenType::Eof);
//     }

//     #[test]
//     fn number_litterals() {
//         let source = "123.123\n321.0\n5";
//         let mut scanner = Scanner::new(source);
//         scanner.scan_tokens().unwrap();
//         assert_eq!(scanner.tokens.len(), 4);
        
//         for i in 0..3 {
//             assert_eq!(scanner.tokens[i].token_type, TokenType::Number)
//         }
//         match scanner.tokens[0].literal {
//             Some(LiteralValue::FloatValue(value)) => assert_eq!(value, 123.123),
//             _ => panic!("Incorrect literal value"),
//         }
//         match scanner.tokens[1].literal {
//             Some(LiteralValue::FloatValue(value)) => assert_eq!(value, 321.0),
//             _ => panic!("Incorrect literal value"),
//         }
//         match scanner.tokens[2].literal {
//             Some(LiteralValue::FloatValue(value)) => assert_eq!(value, 5.0),
//             _ => panic!("Incorrect literal value"),
//         }
//     }

//     #[test]
//     fn get_identifier() {
//         let source = "this_is_a_var = 12;";
//         let mut scanner = Scanner::new(source);
//         scanner.scan_tokens().unwrap();

//         assert_eq!(scanner.tokens.len(), 5);
//         assert_eq!(scanner.tokens[0].token_type, TokenType::Identifier);
//         assert_eq!(scanner.tokens[1].token_type, TokenType::Equal);
//         assert_eq!(scanner.tokens[2].token_type, TokenType::Number);
//         assert_eq!(scanner.tokens[3].token_type, TokenType::Semicolon);
//         assert_eq!(scanner.tokens[4].token_type, TokenType::Eof);
//     }

//     #[test]
//     fn get_keywords() {
//         let source = "var this_is_var = 12;\nwhile true { print 3 };";
//         let mut scanner = Scanner::new(source);
//         scanner.scan_tokens().unwrap();

//         assert_eq!(scanner.tokens.len(), 13);
//         assert_eq!(scanner.tokens[0].token_type, TokenType::Var);
//         assert_eq!(scanner.tokens[1].token_type, TokenType::Identifier);
//         assert_eq!(scanner.tokens[2].token_type, TokenType::Equal);
//         assert_eq!(scanner.tokens[3].token_type, TokenType::Number);
//         assert_eq!(scanner.tokens[4].token_type, TokenType::Semicolon);
//         assert_eq!(scanner.tokens[5].token_type, TokenType::While);
//         assert_eq!(scanner.tokens[6].token_type, TokenType::True);
//         assert_eq!(scanner.tokens[7].token_type, TokenType::LeftBrace);
//         assert_eq!(scanner.tokens[8].token_type, TokenType::Print);
//         assert_eq!(scanner.tokens[9].token_type, TokenType::Number);
//         assert_eq!(scanner.tokens[10].token_type, TokenType::RightBrace);
//         assert_eq!(scanner.tokens[11].token_type, TokenType::Semicolon);
//         assert_eq!(scanner.tokens[12].token_type, TokenType::Eof);
//     }
// }