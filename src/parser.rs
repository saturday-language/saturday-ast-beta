use crate::expr::{
  AssignExpr, BinaryExpr, CallExpr, Expr, GetExpr, GroupingExpr, LiteralExpr, LogicalExpr, SetExpr,
  UnaryExpr, VariableExpr,
};
use crate::object::Object;
use crate::stmt::{
  BlockStmt, BreakStmt, ClassStmt, DefStmt, ExpressionStmt, FunctionStmt, IfStmt, PrintStmt,
  ReturnStmt, Stmt, WhileStmt,
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
  pub fn parse(&mut self) -> Result<Vec<Rc<Stmt>>, SaturdayResult> {
    let mut statements = Vec::new();
    while !self.is_at_end() {
      statements.push(self.declaration()?);
    }

    Ok(statements)
  }

  fn declaration(&mut self) -> Result<Rc<Stmt>, SaturdayResult> {
    let result = if self.is_match(&[TokenType::Class]) {
      self.class_declaration()
    } else if self.is_match(&[TokenType::Fun]) {
      self.function("function")
    } else if self.is_match(&[TokenType::Def]) {
      self.def_declaration()
    } else {
      self.statement()
    };

    if result.is_err() {
      self.synchronize();
    }

    result
  }

  fn def_declaration(&mut self) -> Result<Rc<Stmt>, SaturdayResult> {
    let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
    let initializer = if self.is_match(&[TokenType::Assign]) {
      Some(Rc::new(self.expression()?))
    } else {
      None
    };

    self.consume(
      TokenType::SemiColon,
      "Expect ';' after variable declaration",
    )?;
    Ok(Rc::new(Stmt::Def(Rc::new(DefStmt { name, initializer }))))
  }

  fn while_statement(&mut self) -> Result<Stmt, SaturdayResult> {
    let condition = Rc::new(self.expression()?);
    if !self.peek().is(TokenType::LeftBrace) {
      return Err(SaturdayResult::parse_error(
        self.peek(),
        "while must wrap by '{}'.",
      ));
    }

    let body = self.statement()?;
    Ok(Stmt::While(Rc::new(WhileStmt { condition, body })))
  }

  fn expression(&mut self) -> Result<Expr, SaturdayResult> {
    self.assignment()
  }

  fn class_declaration(&mut self) -> Result<Rc<Stmt>, SaturdayResult> {
    let name = self.consume(TokenType::Identifier, "Expect class name.")?;
    self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;

    let mut methods = Vec::new();
    while !self.check(TokenType::RightBrace) && !self.is_at_end() {
      methods.push(self.function("method")?);
    }
    self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;

    Ok(Rc::new(Stmt::Class(Rc::new(ClassStmt {
      name,
      methods: Rc::new(methods),
    }))))
  }

  fn statement(&mut self) -> Result<Rc<Stmt>, SaturdayResult> {
    if self.is_match(&[TokenType::Break]) {
      let token = self.previous().dup();
      self.consume(TokenType::SemiColon, "expect ';' after break statement.")?;
      return Ok(Rc::new(Stmt::Break(Rc::new(BreakStmt { token }))));
    }

    if self.is_match(&[TokenType::For]) {
      return self.for_statement();
    }

    if self.is_match(&[TokenType::If]) {
      return Ok(Rc::new(self.if_statement()?));
    }

    if self.is_match(&[TokenType::Print]) {
      return Ok(Rc::new(self.print_statement()?));
    }

    if self.is_match(&[TokenType::Return]) {
      return Ok(Rc::new(self.return_statement()?));
    }

    if self.is_match(&[TokenType::While]) {
      return Ok(Rc::new(self.while_statement()?));
    }

    if self.is_match(&[TokenType::LeftBrace]) {
      return Ok(Rc::new(Stmt::Block(Rc::new(BlockStmt {
        statements: Rc::new(self.block()?),
      }))));
    }

    self.expression_statement()
  }

  fn for_statement(&mut self) -> Result<Rc<Stmt>, SaturdayResult> {
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
      body = Rc::new(Stmt::Block(Rc::new(BlockStmt {
        statements: Rc::new(vec![
          body,
          Rc::new(Stmt::Expression(Rc::new(ExpressionStmt {
            expression: Rc::new(incr),
          }))),
        ]),
      })));
    }

    // 将for循环转换成while
    body = Rc::new(Stmt::While(Rc::new(WhileStmt {
      condition: if let Some(cond) = condition {
        Rc::new(cond)
      } else {
        Rc::new(Expr::Literal(Rc::new(LiteralExpr {
          value: Some(Object::Bool(true)),
        })))
      },
      body,
    })));

    // 在准备一个block将初始化表达式包裹进去
    if let Some(init) = initializer {
      body = Rc::new(Stmt::Block(Rc::new(BlockStmt {
        statements: Rc::new(vec![init, body]),
      })));
    }

    Ok(body)
  }

  fn if_statement(&mut self) -> Result<Stmt, SaturdayResult> {
    // 实现condition不带括号且必须有{的条件语句
    let condition = Rc::new(self.expression()?);
    if !self.peek().is(TokenType::LeftBrace) {
      return Err(SaturdayResult::parse_error(
        self.peek(),
        "then branch must wrap by '{}'.",
      ));
    }

    let then_branch = self.statement()?;
    let else_branch = if self.is_match(&[TokenType::Else]) {
      if !self.peek().is(TokenType::LeftBrace) {
        return Err(SaturdayResult::parse_error(
          self.peek(),
          "else branch must wrap by '{}'.",
        ));
      }

      Some(self.statement()?)
    } else {
      None
    };

    Ok(Stmt::If(Rc::new(IfStmt {
      condition,
      then_branch,
      else_branch,
    })))
  }

  fn print_statement(&mut self) -> Result<Stmt, SaturdayResult> {
    let value = Rc::new(self.expression()?);
    self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
    Ok(Stmt::Print(Rc::new(PrintStmt { expression: value })))
  }

  fn return_statement(&mut self) -> Result<Stmt, SaturdayResult> {
    let keyword = self.previous().dup();
    let value = if self.check(TokenType::SemiColon) {
      None
    } else {
      Some(Rc::new(self.expression()?))
    };

    self.consume(TokenType::SemiColon, "Expect ';' after return value.")?;
    Ok(Stmt::Return(Rc::new(ReturnStmt { keyword, value })))
  }

  fn expression_statement(&mut self) -> Result<Rc<Stmt>, SaturdayResult> {
    let expr = Rc::new(self.expression()?);
    self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
    Ok(Rc::new(Stmt::Expression(Rc::new(ExpressionStmt {
      expression: expr,
    }))))
  }

  fn function(&mut self, kind: &str) -> Result<Rc<Stmt>, SaturdayResult> {
    let name = self.consume(TokenType::Identifier, &format!("Expect {kind} name"))?;
    self.consume(
      TokenType::LeftParen,
      &format!("Expect '(' after {kind} name."),
    )?;

    let mut params: Vec<Token> = Vec::new();
    if !self.check(TokenType::RightParen) {
      params.push(self.consume(TokenType::Identifier, "Expect parameter name")?);
      while self.is_match(&[TokenType::Comma]) {
        if params.len() >= 255 && !self.had_error {
          let peek = self.peek().dup();
          self.error(&peek, "Can't have more than 255 parameters.");
        }

        params.push(self.consume(TokenType::Identifier, "Expect parameter name")?);
      }
    }

    self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

    self.consume(
      TokenType::LeftBrace,
      &format!("Expect '{{' before {kind} body"),
    )?;
    let body = Rc::new(self.block()?);
    Ok(Rc::new(Stmt::Function(Rc::new(FunctionStmt {
      name,
      params: Rc::new(params),
      body,
    }))))
  }

  fn block(&mut self) -> Result<Vec<Rc<Stmt>>, SaturdayResult> {
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
        return Ok(Expr::Assign(Rc::new(AssignExpr {
          name: expr.name.dup(),
          value: Rc::new(value),
        })));
      } else if let Expr::Get(get) = expr {
        return Ok(Expr::Set(Rc::new(SetExpr {
          object: Rc::clone(&get.object),
          name: get.name.clone(),
          value: Rc::new(value),
        })));
      }

      self.error(&equals, "Invalid assignment target.");
    }

    Ok(expr)
  }

  fn or(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.and()?;

    while self.is_match(&[TokenType::Or]) {
      let operator = self.previous().dup();
      let right = Rc::new(self.and()?);
      expr = Expr::Logical(Rc::new(LogicalExpr {
        left: Rc::new(expr),
        operator,
        right,
      }));
    }

    Ok(expr)
  }

  fn and(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.equality()?;

    while self.is_match(&[TokenType::And]) {
      let operator = self.previous().dup();
      let right = Rc::new(self.equality()?);
      expr = Expr::Logical(Rc::new(LogicalExpr {
        left: Rc::new(expr),
        operator,
        right,
      }));
    }

    Ok(expr)
  }

  fn equality(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.comparison()?;

    while self.is_match(&[TokenType::BangEqual, TokenType::Equal]) {
      let operator = self.previous().dup();
      let right = self.comparison()?;
      expr = Expr::Binary(Rc::new(BinaryExpr {
        left: Rc::new(expr),
        operator,
        right: Rc::new(right),
      }));
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
      expr = Expr::Binary(Rc::new(BinaryExpr {
        left: Rc::new(expr),
        operator,
        right: Rc::new(right),
      }));
    }

    Ok(expr)
  }

  fn term(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.factor()?;
    while self.is_match(&[TokenType::Minus, TokenType::Plus]) {
      let operator = self.previous().dup();
      let right = self.factor()?;
      expr = Expr::Binary(Rc::new(BinaryExpr {
        left: Rc::new(expr),
        operator,
        right: Rc::new(right),
      }));
    }

    Ok(expr)
  }

  fn factor(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.unary()?;
    while self.is_match(&[TokenType::Slash, TokenType::Star]) {
      let operator = self.previous().dup();
      let right = self.unary()?;
      expr = Expr::Binary(Rc::new(BinaryExpr {
        left: Rc::new(expr),
        operator,
        right: Rc::new(right),
      }));
    }

    Ok(expr)
  }

  fn unary(&mut self) -> Result<Expr, SaturdayResult> {
    if self.is_match(&[TokenType::Bang, TokenType::Minus]) {
      let operator = self.previous().dup();
      let right = self.unary()?;
      return Ok(Expr::Unary(Rc::new(UnaryExpr {
        operator,
        right: Rc::new(right),
      })));
    }

    self.call()
  }

  fn call(&mut self) -> Result<Expr, SaturdayResult> {
    let mut expr = self.primary()?;
    loop {
      if self.is_match(&[TokenType::LeftParen]) {
        expr = self.finish_call(&Rc::new(expr))?;
      } else if self.is_match(&[TokenType::Dot]) {
        let name = self.consume(TokenType::Identifier, "Expect property name after '.'.")?;
        expr = Expr::Get(Rc::new(GetExpr {
          object: Rc::new(expr),
          name,
        }))
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
      arguments.push(Rc::new(self.expression()?));
      while self.is_match(&[TokenType::Comma]) {
        if arguments.len() >= 255 && !self.had_error {
          let peek = self.peek().dup();
          self.error(&peek, "Can't have more than 255 arguments.");
          self.had_error = true;
        } else {
          arguments.push(Rc::new(self.expression()?));
        }
      }
    }

    let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
    Ok(Expr::Call(Rc::new(CallExpr {
      callee: Rc::clone(callee),
      paren,
      arguments,
    })))
  }

  fn primary(&mut self) -> Result<Expr, SaturdayResult> {
    if self.is_match(&[TokenType::False]) {
      return Ok(Expr::Literal(Rc::new(LiteralExpr {
        value: Some(Object::Bool(false)),
      })));
    }
    if self.is_match(&[TokenType::True]) {
      return Ok(Expr::Literal(Rc::new(LiteralExpr {
        value: Some(Object::Bool(true)),
      })));
    }
    if self.is_match(&[TokenType::Nil]) {
      return Ok(Expr::Literal(Rc::new(LiteralExpr {
        value: Some(Object::Nil),
      })));
    }

    if self.is_match(&[TokenType::Number, TokenType::String]) {
      return Ok(Expr::Literal(Rc::new(LiteralExpr {
        value: self.previous().literal.clone(),
      })));
    }

    if self.is_match(&[TokenType::Identifier]) {
      return Ok(Expr::Variable(Rc::new(VariableExpr {
        name: self.previous().dup(),
      })));
    }

    if self.is_match(&[TokenType::LeftParen]) {
      let expr = self.expression()?;
      self.consume(TokenType::RightParen, "Expect ')' after expression")?;
      return Ok(Expr::Grouping(Rc::new(GroupingExpr {
        expression: Rc::new(expr),
      })));
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
