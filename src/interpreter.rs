use crate::ast::LiteralValue;
use crate::stmt::Stmt;
use crate::environment::Environment;
use std::rc::Rc;
use std::cell::RefCell;


pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<(), String> {
        for stmt in stmts {
            match stmt {
                Stmt::Expression { expression } => {
                    expression.evaluate(self.environment.clone())?;
                },
                Stmt::Print { expression } => {
                    let value = expression.evaluate(self.environment.clone())?;
                    println!("{}", value.to_string())
                },
                Stmt::Var { name, initializer } => {
                    let value = initializer.evaluate(self.environment.clone())?;

                    self.environment.borrow_mut().define(name.lexeme.clone(), value);
                },
                Stmt::Block { statements } => {
                    let mut new_environment = Environment::new();
                    new_environment.enclosing = Some(self.environment.clone());
                    let old_environment = self.environment.clone();
                    self.environment = Rc::new(RefCell::new(new_environment));
                    let block_result = self.interpret((*statements)
                        .iter()
                        .map(|b| b.as_ref())
                        .collect());
                    self.environment = old_environment;
                    block_result?;
                },
                Stmt::IfStmt { predicate, then, els } => {
                    let truth_value = predicate.evaluate(self.environment.clone())?;
                    if truth_value.is_truthy() == LiteralValue::True {
                        let statements = vec![then.as_ref()];
                        self.interpret(statements)?;
                    } else if let Some(els_stmt) = els {
                        let statements = vec![els_stmt.as_ref()];
                        self.interpret(statements)?;
                    }
                },
                Stmt::WhileStmt { condition, body } => {
                    let mut flag = condition.evaluate(self.environment.clone())?;
                    while flag.is_truthy() == LiteralValue::True {
                        let statements = vec![body.as_ref()];
                        self.interpret(statements)?;
                        flag = condition.evaluate(self.environment.clone())?;
                    }
                },
                Stmt::ForStmt {
                    var_decl: _,
                    expr_stmt: _,
                    condition: _,
                    increment:_ ,
                    body: _
                } => {
                    todo!()
                }
            };
        }
        Ok(())
    }
}