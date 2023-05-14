use crate::ast::Expr;
use crate::scanner::Token;

#[derive(Debug)]
pub enum Stmt {
    Expression { expression: Expr },
    Print { expression: Expr },
    Var { name: Token, initializer: Expr },
    Block { statements: Vec<Box<Stmt>> },
    IfStmt { predicate: Expr, then: Box<Stmt>, els: Option<Box<Stmt>> },
    WhileStmt {
        condition: Expr,
        body: Box<Stmt>,

    },
    ForStmt {
        var_decl: Option<Box<Stmt>>,
        expr_stmt: Option<Box<Stmt>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Box<Stmt>,
    },
}

impl Stmt {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        match self {
            Stmt::Expression { expression } => expression.to_string(),
            Stmt::Print { expression } => format!("(print {})", expression.to_string()),
            Stmt::Var { name, initializer: _ } => format!("(var {})", name.lexeme),
            Stmt::Block { statements } => format!(
                "(block {})",
                statements.into_iter().map(|stmt| stmt.to_string())
                .collect::<String>()
            ),
            Stmt::IfStmt { predicate: _, then: _, els: _ } => todo!(),
            Stmt::WhileStmt { condition: _, body: _ } => todo!(),
            Stmt::ForStmt {
                var_decl:_,
                expr_stmt: _,
                condition: _,
                increment: _,
                body: _ 
            } => todo!()
        }
    }
}