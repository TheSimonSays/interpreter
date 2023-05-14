use crate::scanner::{Token, TokenType};
use crate::scanner;
use crate::environment::Environment;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Number(f32),
    StringValue(String),
    True,
    False,
    Nil,
}


fn unwrap_as_f32(literal: Option<scanner::LiteralValue>) -> f32 {
    match literal {
        Some(scanner::LiteralValue::IntValue(x)) => x as f32,
        Some(scanner::LiteralValue::FloatValue(x)) => x as f32,
        _ => panic!("Could not unwrap as f32"),
    }
}

fn unwrap_as_string(literal: Option<scanner::LiteralValue>) -> String {
    match literal {
        Some(scanner::LiteralValue::StringValue(s)) => s.clone(),
        Some(scanner::LiteralValue::IdentifierValue(s)) => s.clone(),
        _ => panic!("Could not unwrap as string"),
    }
}

impl LiteralValue {
    pub fn to_string(&self) -> String {
        match self {
            LiteralValue::Number(x) => x.to_string(),
            LiteralValue::StringValue(x) => format!("\"{}\"", x),
            LiteralValue::True => "true".to_string(),
            LiteralValue::False => "false".to_string(),
            LiteralValue::Nil => "nill".to_string(),
        }
    }

    pub fn to_type(&self) -> &str {
        match self {
            LiteralValue::Number(_) => "Number",
            LiteralValue::StringValue(_) => "String",
            LiteralValue::True => "True",
            LiteralValue::False => "False",
            LiteralValue::Nil => "Nil",
        }
    }

    pub fn from_token(token: Token) -> Self {
        match token.token_type {
            TokenType::Number => Self::Number(unwrap_as_f32(token.literal)),
            TokenType::String => Self::StringValue(unwrap_as_string(token.literal)),
            TokenType::False => Self::False,
            TokenType::True => Self::True,
            TokenType::Nil => Self::Nil,
            _ => panic!("Could not create LiteralValue from {:?}", token)
        }
    }

    pub fn is_falsy(&self) -> LiteralValue {
        match self {
            LiteralValue::Number(x) => if *x == 0.0 {LiteralValue::True} else {LiteralValue::False},
            LiteralValue::StringValue(s) => if s.len() == 0 {LiteralValue::True} else {LiteralValue::False},
            LiteralValue::True => LiteralValue::False,
            LiteralValue::False => LiteralValue::True,
            LiteralValue::Nil => LiteralValue::True,
        }
    }

    pub fn is_truthy(&self) -> LiteralValue {
        match self {
            LiteralValue::Number(x) => if *x == 0.0 {LiteralValue::False} else {LiteralValue::True},
            LiteralValue::StringValue(s) => if s.len() == 0 {LiteralValue::False} else {LiteralValue::True},
            LiteralValue::True => LiteralValue::True,
            LiteralValue::False => LiteralValue::False,
            LiteralValue::Nil => LiteralValue::False,
        }
    }

    pub fn from_bool(b: bool) -> Self {
        if b {
            LiteralValue::True
        } else {
            LiteralValue::False
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>
    },
    Grouping { expression: Box<Expr> },
    Literal { value: LiteralValue},
    Unary { operator: Token, right: Box<Expr> },
    Variable { name: Token },
    Logical { left: Box<Expr>, operator: Token, right: Box<Expr> },
    Call {
        calee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
}

impl Expr {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        match self {
            Expr::Assign {
                name,
                value 
            } => format!("({:?}={})", name, value.to_string()),
            Expr::Binary {
                left,
                operator,
                right
            } => format!(
                "({} {} {})",
                operator.lexeme,
                left.to_string(),
                right.to_string()
            ),
            Expr::Grouping { expression } => format!("(group {})", (*expression).to_string()),
            Expr::Literal { value } => format!("{}", value.to_string()),
            Expr::Unary { operator, right } => {
                let operator_str = operator.lexeme.clone();
                let right_str = (*right).to_string();
                format!("({} {})", operator_str, right_str)
            },
            Expr::Variable { name } => format!("(var {})", name.lexeme),
            Expr::Logical { left, operator, right } => format!(
                "({} {} {})", operator.to_string(), left.to_string(), right.to_string()
            ),
            Expr::Call { calee, paren, arguments } => format!("(call {} {} {:?})", calee.to_string(), paren.to_string(), arguments),
        }
    }

    pub fn evaluate(&self, environment: Rc<RefCell<Environment>>) -> Result<LiteralValue, String> {
        match self {
            Expr::Assign { name, value } => {
                let new_value = (*value).evaluate(environment.clone())?;
                let assign_success = environment.borrow_mut().assign(&name.lexeme, new_value.clone());
                if assign_success {
                    Ok(new_value)
                } else {
                    Err(format!("Variable {} has not been declared.", name.lexeme))
                }
            }
            Expr::Variable { name } => match environment.borrow().get(&name.lexeme) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Variable '{}' has not been declared", name.lexeme))
            },
            Expr::Literal { value } => Ok((*value).clone()),
            Expr::Grouping { expression } => expression.evaluate(environment),
            Expr::Unary { operator, right } => {
                let right = right.evaluate(environment)?;
                match (&right, operator.token_type) {
                    (LiteralValue::Number(x), TokenType::Minus) => Ok(LiteralValue::Number(-x)),
                    (_, TokenType::Minus) => return Err(format!("Minus not implemented for {}", right.to_type())),
                    (any, TokenType::Bang) => {
                        Ok(any.is_falsy())
                    }
                    (_, token_type) => Err(format!("{:?} is not a valid unary operator", token_type))
                }
            },
            Expr::Binary { left, operator, right } => {
                let left = left.evaluate(environment.clone())?;
                let right = right.evaluate(environment.clone())?;

                match (&left, operator.token_type, &right) {
                    (LiteralValue::Number(x), TokenType::Plus, LiteralValue::Number(y)) => Ok(LiteralValue::Number(x + y)),
                    (LiteralValue::Number(x), TokenType::Minus, LiteralValue::Number(y)) => Ok(LiteralValue::Number(x - y)),
                    (LiteralValue::Number(x), TokenType::Star, LiteralValue::Number(y)) => Ok(LiteralValue::Number(x * y)),
                    (LiteralValue::Number(x), TokenType::Slash, LiteralValue::Number(y)) => Ok(LiteralValue::Number(x / y)),
                    (LiteralValue::Number(x), TokenType::Percent, LiteralValue::Number(y)) => Ok(LiteralValue::Number(x % y)),
                    (LiteralValue::Number(x), TokenType::Less, LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x < y)),
                    (LiteralValue::Number(x), TokenType::LessEqual, LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x <= y)),
                    (LiteralValue::Number(x), TokenType::Greater, LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x > y)),
                    (LiteralValue::Number(x), TokenType::GreaterEqual, LiteralValue::Number(y)) => Ok(LiteralValue::from_bool(x >= y)),
                    (LiteralValue::StringValue(_), op, LiteralValue::Number(_)) => {
                        Err(format!("{:?} is not defined for string and numbers", op))
                    },
                    (LiteralValue::Number(_), op, LiteralValue::StringValue(_)) => {
                        Err(format!("{:?} is not defined for string and numbers", op))
                    },
                    (LiteralValue::StringValue(s1), TokenType::Plus, LiteralValue::StringValue(s2)) => {
                        Ok(LiteralValue::StringValue(format!("{}{}", s1, s2)))
                    },
                    (x, TokenType::BangEqual, y) => Ok(LiteralValue::from_bool(x != y)),
                    (x, TokenType::EqualEqual, y) => Ok(LiteralValue::from_bool(x == y)),
                    
                    (LiteralValue::StringValue(s1), TokenType::Greater, LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 > s2)),
                    (LiteralValue::StringValue(s1), TokenType::GreaterEqual, LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 >= s2)),
                    (LiteralValue::StringValue(s1), TokenType::Less, LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 < s2)),
                    (LiteralValue::StringValue(s1), TokenType::LessEqual, LiteralValue::StringValue(s2)) => Ok(LiteralValue::from_bool(s1 <= s2)),
                    (x, token_type, y) => {
                        Err(format!("{:?} is not implemented for operands {:?} and {:?}", token_type, x, y))
                    }
                }
            },
            Expr::Logical { left, operator, right } => {
                match operator.token_type {
                    TokenType::Or => {
                        let lhs_value = left.evaluate(environment.clone())?;
                        let lhs_true = lhs_value.is_truthy();
                        if lhs_true == LiteralValue::True {
                            Ok(lhs_value)
                        } else {
                            right.evaluate(environment.clone())
                        }
                    },
                    TokenType::And => {
                        let lhs_value = left.evaluate(environment.clone())?;
                        let lsh_true = lhs_value.is_truthy();
                        if lsh_true == LiteralValue::False {
                            Ok(lsh_true)
                        } else {
                            right.evaluate(environment.clone())
                        }
                    },
                    token_type => Err(format!("Invalid token in logical expression: {:?}", token_type)),
                }
            },
            Expr::Call { calee: _, paren: _, arguments: _ } => todo!(),
        }
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        println!("{}", self.to_string());
    }
}