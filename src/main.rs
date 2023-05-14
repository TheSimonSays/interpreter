mod scanner;
mod ast;
mod parser;
mod interpreter;
mod stmt;
mod environment;

use crate::scanner::*;
use crate::parser::*;
use crate::interpreter::*;

use std::{env, process::exit, fs, io};
use std::io::{BufRead, Write};


pub fn run_file(path: &str) -> Result<(), String>{
    let mut interpreter = Interpreter::new();
    match fs::read_to_string(path) {
        Err(msg) => return Err(msg.to_string()),
        Ok(contents) => return run(&mut interpreter, &contents),
    } 

}


pub fn run(interpreter: &mut Interpreter, contents: &str) -> Result<(), String> {
    let mut scanner = Scanner::new(contents);
    scanner.scan_tokens()?;
    let tokens = scanner.tokens;

    let mut parser = Parser::new(tokens);
    let stmts = parser.parse()?;
    interpreter.interpret(stmts.iter().collect())?;

    return Ok(());
}


fn run_prompt() -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    let mut buffer = String::new();
    loop {
        print!("> ");
        let stdin = io::stdin();
        match io::stdout().flush() {
            Ok(_) => (),
            Err(_) => return Err("Couldnt flush stdout".to_string()),
        } 
        let mut handle = stdin.lock();
        let current_length = buffer.len();
        match handle.read_line(&mut buffer) {
            Ok(n) => {
                if n <= 1 {
                    return Ok(());
                }
            },
            Err(_) => return Err("Couldnt read stdin".to_string()),
        }
        println!("ECHO: {}", &buffer[current_length..]);
        match run(&mut interpreter,&buffer[current_length..]) {
            Ok(_) => (),
            Err(msg) => println!("{}", msg)
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rprt [script]");
        exit(64);
    } else if args.len() == 2 {
        match run_file(&args[1]) {
            Ok(_) => exit(0),
            Err(msg) => {
                println!("ERROR: {}", msg)
            }
        }
    } else {
        match run_prompt() {
            Ok(_) => exit(0),
            Err(msg) => {
                println!("ERROR: {}", msg)
            }
        }
    }
}
