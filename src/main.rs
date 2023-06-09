extern crate core;

use std::env::args;
use std::io;
use std::io::{stdout, BufRead, Write};
use std::rc::Rc;

use error::*;
use scanner::*;
// use crate::ast_printer::AstPrinter;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;

mod error;
mod expr;
mod parser;
mod scanner;
mod token;
mod token_type;
// mod ast_printer;
mod callable;
mod environment;
mod interpreter;
mod native_functions;
mod object;
mod resolver;
mod saturday_class;
mod saturday_function;
mod saturday_instance;
mod stmt;

fn main() {
  let args: Vec<String> = args().collect();
  let saturday = Saturday::new();

  match args.len() {
    1 => saturday.run_prompt(),
    2 => saturday.run_file(&args[1]).expect("Could not run file"),
    _ => {
      println!("Usage: saturday-ast [script]");
      std::process::exit(64);
    }
  }
}

struct Saturday {
  interpreter: Interpreter,
}

impl Saturday {
  pub fn new() -> Self {
    Self {
      interpreter: Interpreter::new(),
    }
  }

  fn run_file(&self, path: &str) -> io::Result<()> {
    let buf = std::fs::read_to_string(path)?;
    if self.run(buf).is_err() {
      // Ignore: error was already reported
      std::process::exit(65);
    }

    Ok(())
  }

  fn run_prompt(&self) {
    let stdin = io::stdin();
    print!("> ");
    stdout().flush().expect("flush error");
    for line in stdin.lock().lines() {
      if let Ok(line) = line {
        if line.is_empty() {
          break;
        }

        let _ = self.run(line);
      } else {
        break;
      }

      print!("> ");
      stdout().flush().expect("flush error");
    }
  }

  fn run(&self, source: String) -> Result<(), SaturdayResult> {
    if source == "@" {
      self.interpreter.print_environment();
      return Ok(());
    }

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;

    if parser.success() {
      let resolver = Resolver::new(&self.interpreter);
      let s = Rc::new(statements);
      resolver.resolve(&Rc::clone(&s))?;

      if resolver.success() {
        self.interpreter.interpreter(&Rc::clone(&s));
      }
    }

    Ok(())
  }
}
