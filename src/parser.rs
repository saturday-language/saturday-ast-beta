use crate::expr::{
  AssignExpr, BinaryExpr, CallExpr, Expr, GroupingExpr, LiteralExpr, LogicalExpr, UnaryExpr,
  VariableExpr,
};
use crate::object::Object;
use crate::stmt::{
  BlockStmt, BreakStmt, DefStmt, ExpressionStmt, IfStmt, PrintStmt, Stmt, WhileStmt,
};
use crate::token::Token;
use crate::token_type::*;
use crate::SaturdayResult;
use std::rc::Rc;

pub struct Parser<'a> {
  tokens: &'a [Token],
  current: usize,
  had_error: bool,
}

impl<'a> Parser<'a> {
  pub fn new(tokens: &'a [Token]) -> Self {
    Self {
      tokens,
      current: 0,
      had_error: false,
    }
  }

  pub fn success(&self) -> bool {
    !self.had_error
  }

  /// # 解析方法，调用expression解析tokens生成表达式
  pub fn parse(&mut self) -> Result<Vec<Stmt>, SaturdayResult> {
    let mut statements = Vec::new();
    while !self.is_at_end() {
      statements.push(self.declaration()?);
    }

    Ok(statements)
  }

  fn declaration(&mut self) -> Result<Stmt, SaturdayResult> {
    let result = if self.is_match(&[TokenType::Def]) {
      self.def_declaration()
    } else {
      self.statement()
    };

    if result.is_err() {
      self.synchronize();
    }

    result
  }

  fn def_declaration(&mut self) -> Result<Stmt, SaturdayResult> {
    let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
    let initializer = if self.is_match(&[TokenType::Assign]) {
      Some(self.expression()?)
    } else {
      None
    };

    self.consume(
      TokenType::SemiColon,
      "Expect ';' after variable declaration",
    )?;
    Ok(Stmt::Def(DefStmt { name, initializer }))
  }

  fn while_statement(&mut self) -> Result<Stmt, SaturdayResult> {
    let condition = self.expression()?;
    if !self.peek().is(TokenType::LeftBrace) {
      return Err(SaturdayResult::parse_error(
        self.peek(),
        "while must wrap by '{}'.",
      ));
    }

    let body = Box::new(self.statement()?);
    Ok(Stmt::While(WhileStmt { condition, body }))
  }

  fn expression(&mut self) -> Result<Expr, SaturdayResult> {
    self.assignment()
  }

  fn statement(&mut self) -> Result<Stmt, SaturdayResult> {
    if self.is_match(&[TokenType::Break]) {
      let token = self.peek().dup();
      self.consume(TokenType::SemiColon, "expect ';' after break statement.")?;
      return Ok(Stmt::Break(BreakStmt { token }));
    }

    if self.is_match(&[TokenType::For]) {
      return self.for_statement();
    }

    if self.is_match(&[TokenType::If]) {
      return self.if_statement();
    }

    if self.is_match(&[TokenType::Print]) {
      return self.print_statement();
    }

    if self.is_match(&[TokenType::While]) {
      return self.while_statement();
    }

    if self.is_match(&[TokenType::LeftBrace]) {
      return Ok(Stmt::Block(BlockStmt {
        statements: self.block()?,
      }));
    }

    self.expression_statement()
  }

  fn for_statement(&mut self) -> Result<Stmt, SaturdayResult> {
    let initializer = if self.is_match(&[TokenType::SemiColon]) {
      None
    } else if self.is_match(&[TokenType::Def]) {
      Some(self.def_declaration()?)
    } else {
      Some(self.expression_statement()?)
    };

    let condition = if self.check(TokenType::SemiColon) {
      None
    } else {
      Some(self.expression()?)
    };
    self.consume(TokenType::SemiColon, "Expect ';' after loop condition.")?;

    let increment = if self.check(TokenType::LeftBrace) {
      None
    } else {
      Some(self.expression()?)
    };

    let mut body = self.statement()?;
    // 执行完逻辑后将条件值增加
    if let Some(incr) = increment {
      body = Stmt::Block(BlockStmt {
        statements: vec![body, Stmt::Expression(ExpressionStmt { expression: incr })],
      });
    }

    // 将for循环转换成while
    body = Stmt::While(WhileStmt {
      condition: if let Some(cond) = condition {
        cond
      } else {
        Expr::Literal(LiteralExpr {
          value: Some(Object::Bool(true)),
        })
      },
      body: Box::new(body),
    });

    // 在准备一个block将初始化表达式包裹进去
    if let Some(init) = initializer {
      body = Stmt::Block(BlockStmt {
        statements: vec![init, body],
      });
    }

    Ok(body)
  }

  fn if_statement(&mut self) -> Result<Stmt, SaturdayResult> {
    // 实现condition不带括号且必须有{的条件语句
    let condition = self.expression()?;
    if !self.peek().is(TokenType::LeftBrace) {
      return Err(SaturdayResult::parse_error(
        self.peek(),
        "then branch must wrap by '{}'.",
      ));
    }

    let then_branch = Box::new(self.statement()?);
    let else_branch = if self.is_match(&[TokenType::Else]) {
      if !self.peek().is(TokenType::LeftBrace) {
        return Err(SaturdayResult::parse_error(
          self.peek(),
          "else branch must wrap by '{}'.",
        ));
      }

      Some(Box::new(self.statement()?))
    } else {
      None
    };

    Ok(Stmt::If(IfStmt {
      condition,
      then_branch,
      else_branch,
    }))
  }

  fn print_statement(&mut self) -> Result<Stmt, SaturdayResult> {
    let value = self.expression()?;
    self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
    Ok(Stmt::Print(PrintStmt { expression: value }))
  }

  fn expression_statement(&mut self) -> Result<Stmt, SaturdayResult> {
    let expr = self.expression()?;
    self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
    Ok(Stmt::Expression(ExpressionStmt { expression: expr }))
  }

  fn block(&mut self) -> Result<Vec<Stmt>, SaturdayResult> {
    let mut statements = Vec::new();
    while !self.check(TokenType::RightBrace) && !self.is_at_end() {
      statements.push(self.declaration()?);
    }

    self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
    Ok(statements)
  }

  fn assignment(&mut self) -> Result<Expr, SaturdayResult> {
    let expr = self.or()?;

    if self.is_match(&[TokenType::Assign]) {
      let equals = self.previous().dup();
      let value = self.assignment()?;

      if let Expr::Variable(expr) = expr {
        return Ok(Expr::Assign(AssignExpr {
          name: expr.name.dup(),
          value: Box::new(value),
        }));
      }

      self.error(&equals, "Invalid assignment target.");
    }

    Ok(expr)
  }

  fn or(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.and()?;

    while self.is_match(&[TokenType::Or]) {
      let operator = self.previous().dup();
      let right = Box::new(self.and()?);
      expr = Expr::Logical(LogicalExpr {
        left: Box::new(expr),
        operator,
        right,
      });
    }

    Ok(expr)
  }

  fn and(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.equality()?;

    while self.is_match(&[TokenType::And]) {
      let operator = self.previous().dup();
      let right = Box::new(self.equality()?);
      expr = Expr::Logical(LogicalExpr {
        left: Box::new(expr),
        operator,
        right,
      });
    }

    Ok(expr)
  }

  fn equality(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.comparison()?;

    while self.is_match(&[TokenType::BangEqual, TokenType::Equal]) {
      let operator = self.previous().dup();
      let right = self.comparison()?;
      expr = Expr::Binary(BinaryExpr {
        left: Box::new(expr),
        operator,
        right: Box::new(right),
      });
    }

    Ok(expr)
  }

  fn comparison(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.term()?;
    while self.is_match(&[
      TokenType::Greater,
      TokenType::GreaterEqual,
      TokenType::Less,
      TokenType::LessEqual,
    ]) {
      let operator = self.previous().dup();
      let right = self.term()?;
      expr = Expr::Binary(BinaryExpr {
        left: Box::new(expr),
        operator,
        right: Box::new(right),
      });
    }

    Ok(expr)
  }

  fn term(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.factor()?;
    while self.is_match(&[TokenType::Minus, TokenType::Plus]) {
      let operator = self.previous().dup();
      let right = self.factor()?;
      expr = Expr::Binary(BinaryExpr {
        left: Box::new(expr),
        operator,
        right: Box::new(right),
      });
    }

    Ok(expr)
  }

  fn factor(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.unary()?;
    while self.is_match(&[TokenType::Slash, TokenType::Star]) {
      let operator = self.previous().dup();
      let right = self.unary()?;
      expr = Expr::Binary(BinaryExpr {
        left: Box::new(expr),
        operator,
        right: Box::new(right),
      });
    }

    Ok(expr)
  }

  fn unary(&mut self) -> Result<Expr, SaturdayResult> {
    if self.is_match(&[TokenType::Bang, TokenType::Minus]) {
      let operator = self.previous().dup();
      let right = self.unary()?;
      return Ok(Expr::Unary(UnaryExpr {
        operator,
        right: Box::new(right),
      }));
    }

    self.call()
  }

  fn call(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.primary()?;
    loop {
      if self.is_match(&[TokenType::LeftParen]) {
        expr = self.finish_call(&Rc::new(expr))?;
      } else {
        break;
      }
    }

    Ok(expr)
  }

  /// 解析方法参数
  fn finish_call(&mut self, callee: &Rc<Expr>) -> Result<Expr, SaturdayResult> {
    let mut arguments = Vec::new();
    if !self.check(TokenType::RightParen) {
      arguments.push(self.expression()?);
      while self.is_match(&[TokenType::Comma]) {
        if arguments.len() >= 255 && !self.had_error {
          let peek = self.peek().dup();
          SaturdayResult::parse_error(&peek, "Can't have more than 255 arguments.");
          self.had_error = true;
        } else {
          arguments.push(self.expression()?);
        }
      }
    }

    let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
    Ok(Expr::Call(CallExpr {
      callee: Rc::clone(callee),
      paren,
      arguments,
    }))
  }

  fn primary(&mut self) -> Result<Expr, SaturdayResult> {
    if self.is_match(&[TokenType::False]) {
      return Ok(Expr::Literal(LiteralExpr {
        value: Some(Object::Bool(false)),
      }));
    }
    if self.is_match(&[TokenType::True]) {
      return Ok(Expr::Literal(LiteralExpr {
        value: Some(Object::Bool(true)),
      }));
    }
    if self.is_match(&[TokenType::Nil]) {
      return Ok(Expr::Literal(LiteralExpr {
        value: Some(Object::Nil),
      }));
    }

    if self.is_match(&[TokenType::Number, TokenType::String]) {
      return Ok(Expr::Literal(LiteralExpr {
        value: self.previous().literal.clone(),
      }));
    }

    if self.is_match(&[TokenType::Identifier]) {
      return Ok(Expr::Variable(VariableExpr {
        name: self.previous().dup(),
      }));
    }

    if self.is_match(&[TokenType::LeftParen]) {
      let expr = self.expression()?;
      self.consume(TokenType::RightParen, "Expect ')' after expression")?;
      return Ok(Expr::Grouping(GroupingExpr {
        expression: Box::new(expr),
      }));
    }

    let peek = self.peek().dup();
    Err(SaturdayResult::parse_error(&peek, "Expect expression."))
  }

  fn consume(&mut self, t_token: TokenType, message: &str) -> Result<Token, SaturdayResult> {
    if self.check(t_token) {
      Ok(self.advance().dup())
    } else {
      Err(self.error(&self.peek().dup(), message))
    }
  }

  fn error(&mut self, token: &Token, message: &str) -> SaturdayResult {
    self.had_error = true;
    SaturdayResult::parse_error(token, message)
  }

  fn synchronize(&mut self) {
    self.advance();

    while !self.is_at_end() {
      if self.previous().is(TokenType::SemiColon) {
        return;
      }

      if matches!(
        self.peek().token_type(),
        TokenType::Class
          | TokenType::Fun
          | TokenType::Var
          | TokenType::Def
          | TokenType::For
          | TokenType::If
          | TokenType::While
          | TokenType::Print
          | TokenType::Return
      ) {
        return;
      }

      self.advance();
    }
  }

  fn is_match(&mut self, types: &[TokenType]) -> bool {
    for &t in types {
      if self.check(t) {
        self.advance();
        return true;
      }
    }

    false
  }

  fn check(&self, t_type: TokenType) -> bool {
    if self.is_at_end() {
      false
    } else {
      self.peek().is(t_type)
    }
  }

  fn advance(&mut self) -> &Token {
    if !self.is_at_end() {
      self.current += 1;
    }

    self.previous()
  }

  fn is_at_end(&self) -> bool {
    self.peek().is(TokenType::Eof)
  }

  fn peek(&self) -> &Token {
    self.tokens.get(self.current).unwrap()
  }

  fn previous(&self) -> &Token {
    self.tokens.get(self.current - 1).unwrap()
  }
}