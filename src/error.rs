use crate::token::Token;
use crate::token_type::TokenType;

pub enum SaturdayResult {
  ParseError { token: Token, message: String },
  RuntimeError { token: Token, message: String },
  Error { line: usize, message: String },
  Break,
}

impl SaturdayResult {
  pub fn error(line: usize, message: &str) -> Self {
    let err = Self::Error {
      line,
      message: message.to_string(),
    };
    err.report("");
    err
  }

  pub fn runtime_error(token: &Token, message: &str) -> Self {
    let err = Self::RuntimeError {
      token: token.dup(),
      message: message.to_string(),
    };
    err.report("");
    err
  }

  pub fn parse_error(token: &Token, message: &str) -> Self {
    let err = Self::ParseError {
      token: token.dup(),
      message: message.to_string(),
    };
    err.report("");
    err
  }

  fn report(&self, loc: &str) {
    match self {
      Self::ParseError { token, message } | Self::RuntimeError { token, message } => {
        if token.is(TokenType::Eof) {
          eprintln!("{} at end {}", token.line, message);
        } else {
          eprintln!("{} at '{}' {}", token.line, token.as_string(), message);
        }
      }
      Self::Error { line, message } => {
        eprintln!("[line {}] Error{}: {}", line, loc, message);
      }
      Self::Break => {}
    };
  }
}