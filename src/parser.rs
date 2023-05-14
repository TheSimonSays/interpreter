use crate::scanner::{Token, TokenType};
use crate::ast::{Expr, LiteralValue};
use crate::stmt::Stmt;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = vec![];
        let mut errors = vec![];

        while !self.is_at_end() {
            let stmt = self.declaration();
            match stmt {
                Ok(s) => stmts.push(s),
                Err(msg) => {
                    errors.push(msg);
                    self.syncronize();
                }
            }
        }
        if errors.len() == 0 {
            Ok(stmts)
        } else {
            Err(errors.join("\n"))
        }
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_token(TokenType::Var) {
            match self.var_declaration() {
                Ok(stmt) => Ok(stmt),
                Err(msg) => {
                    // self.syncronize();
                    Err(msg)
                }
            }
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let token = self.consume(TokenType::Identifier, "Expected variable name")?;
        let initializer;
        if self.match_token(TokenType::Equal) {
            initializer = self.expression()?;
        } else {
            initializer = Expr::Literal { value: LiteralValue::Nil };
        }
        self.consume(TokenType::Semicolon, "Expected ';' after variable declaration")?;
        Ok(Stmt::Var { name: token, initializer: initializer })
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(TokenType::Print) {
            self.print_statement()
        } else if self.match_token(TokenType::LeftBrace) {
            self.block_statement()
        } else if self.match_token(TokenType::If) {
            self.if_statement()
        } else if self.match_token(TokenType::While) {
            self.while_statement()
        } else if self.match_token(TokenType::For) {
            self.for_statement()  
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'.")?;

        let initializer;
        if self.match_token(TokenType::Semicolon) {
            initializer = None;
        } else if self.match_token(TokenType::Var) {
            let var_decl = self.var_declaration()?;
            initializer = Some(var_decl);
        } else {
            let expr = self.expression_statement()?;
            initializer = Some(expr);
        }

        let condition;
        if !self.check(TokenType::Semicolon) {
            let expr = self.expression()?;
            condition = Some(expr);
        } else {
            condition = None;
        }

        self.consume(TokenType::Semicolon, "Expected ';' after loop condition.")?;

        let increment;
        if !self.check(TokenType::RightParen) {
            let expr = self.expression()?;
            increment = Some(expr);
        } else {
            increment = None;
        }
        self.consume(TokenType::RightParen, "Expected ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(incr) = increment {
            body = Stmt::Block { statements: vec![Box::new(body), Box::new(Stmt::Expression { expression: incr })] };
        }

        let cond;
        match condition {
            None => cond = Expr::Literal { value: LiteralValue::True },
            Some(c) => cond = c,
        }
        body = Stmt::WhileStmt { condition: cond, body: Box::new(body) };

        if let Some(init) = initializer {
            body = Stmt::Block { statements: vec![Box::new(init), Box::new(body)] };
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition")?;
        let body = self.statement()?;
        Ok(Stmt::WhileStmt {
            condition: condition,
            body: Box::new(body) 
        })
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'")?;
        let predicate = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after if-predicate")?;

        let then = Box::new(self.statement()?);
        let els = if self.match_token(TokenType::Else) {
            let stmt = self.statement()?;
            Some(Box::new(stmt))
        } else {
            None
        };
        Ok(Stmt::IfStmt { predicate: predicate, then: then, els: els })
    }

    fn block_statement(&mut self) -> Result<Stmt, String> {
        let mut statements = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let decl = self.declaration()?;
            statements.push(Box::new(decl));
        }
        self.consume(TokenType::RightBrace, "Expected '}' after block.")?;
        Ok(Stmt::Block { statements })
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Expression { expression: expr })
    }

    pub fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;

        if self.match_token(TokenType::Equal) {
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name } => Ok(Expr::Assign {
                    name: name,
                    value: Box::from(value),
                }),
                _ => Err("Invalid assignment target.".to_string())
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;

        while self.match_token(TokenType::Or) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical { left: Box::new(expr), operator: operator, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.match_token(TokenType::And) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right)
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let rhs = self.comparison()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator: operator,
                right: Box::from(rhs),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous();
            let rhs = self.term()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_tokens(&[
            TokenType::Minus,
            TokenType::Plus,
            TokenType::Percent,
        ]) {
            let op = self.previous();
            let rhs = self.factor()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs)
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;
        while self.match_tokens(&[
            TokenType::Slash,
            TokenType::Star,
        ]) {
            let op = self.previous();
            let rhs = self.unary()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[
            TokenType::Bang,
            TokenType::Minus,
        ]) {
            let op = self.previous();
            let rhs = self.unary()?;
            Ok(Expr::Unary {
                operator: op,
                right: Box::from(rhs),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peek();
        let result;
        match token.token_type {
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expected ')'")?;
                result = Expr::Grouping { expression: Box::from(expr) };
            },
            TokenType::False |
            TokenType::True |
            TokenType::Nil |
            TokenType::Number |
            TokenType::Percent |
            TokenType::String => {
                self.advance();
                result = Expr::Literal { value: LiteralValue::from_token(token) };
            },
            TokenType::Identifier => {
                self.advance();
                result = Expr::Variable { name: self.previous() };
            },
            _ => return Err("Expected expression".to_string()),
        }
        Ok(result)
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<Token, String> {
        let token = self.peek();
        if token.token_type == token_type {
            self.advance();
            let token = self.previous();
            Ok(token)
        } else {
            Err(msg.to_string())
        }
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        self.peek().token_type == token_type
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        } else {
            if self.peek().token_type == token_type {
                self.advance();
                true
            } else {
                false
            }
        }
    }

    fn match_tokens(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.match_token(*token_type) {
                return true;
            }
        }
        false
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&mut self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&mut self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn syncronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            match self.peek().token_type {
                TokenType::Class |
                TokenType::Fun |
                TokenType::Var |
                TokenType::If |
                TokenType::While |
                TokenType::Print |
                TokenType::Return => return,
                _ => (),
            }
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{Scanner, LiteralValue};

    #[test]
    fn test_addition() {
        let one = Token {
            token_type: TokenType::Number,
            lexeme: "1".to_string(),
            literal: Some(LiteralValue::IntValue(1)),
            line_number: 0
        };
        let plus = Token {
            token_type: TokenType::Plus,
            lexeme: "+".to_string(),
            literal: None,
            line_number: 0
        };
        let two = Token {
            token_type: TokenType::Number,
            lexeme: "2".to_string(),
            literal: Some(LiteralValue::IntValue(2)),
            line_number: 0
        };
        let semicolon = Token {
            token_type: TokenType::Semicolon,
            lexeme: ";".to_string(),
            literal: None,
            line_number: 0
        };
        let tokens = vec![
            one, plus, two, semicolon
        ];
        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse().unwrap();
        println!("{:?}", parsed_expr);
        // let string_expression = parsed_expr.to_string();
        
        // assert_eq!(string_expression, "(+ 1 2)");
    }

    #[test]
    fn test_comparison() {
        let source = "1 + 2 == 5 + 7;";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();
        let tokens = scanner.tokens;
        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse().unwrap();
        let string_expr = parsed_expr[0].to_string();
        assert_eq!(string_expr, "(== (+ 1 2) (+ 5 7))")
    }

    #[test]
    fn test_eq_with_paren() {
        let source = "1 == (2 + 2);";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();
        let tokens = scanner.tokens;
        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse().unwrap();
        assert_eq!(parsed_expr.len(), 1);
        let string_expression = parsed_expr[0].to_string();
        assert_eq!(string_expression, "(== 1 (group (+ 2 2)))");
    }
}