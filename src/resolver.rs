use crate::error::SaturdayResult;
use crate::expr::{
  AssignExpr, BinaryExpr, CallExpr, Expr, ExprVisitor, GroupingExpr, LiteralExpr, LogicalExpr,
  UnaryExpr, VariableExpr,
};
use crate::interpreter::Interpreter;
use crate::stmt::{
  BlockStmt, BreakStmt, DefStmt, ExpressionStmt, FunctionStmt, IfStmt, PrintStmt, ReturnStmt, Stmt,
  StmtVisitor, WhileStmt,
};
use crate::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

struct Resolver {
  interpreter: Interpreter,
  scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
}

impl Resolver {
  /*
  pub fn new(interpreter: Interpreter) -> Self {
    Self {
      interpreter,
      scopes: RefCell::new(Vec::new()),
    }
  }
   */

  fn resolve(&self, statements: &Rc<Vec<Rc<Stmt>>>) -> Result<(), SaturdayResult> {
    for statement in statements.deref() {
      self.resolve_stmt(statement)?;
    }

    Ok(())
  }

  fn resolve_stmt(&self, stmt: &Rc<Stmt>) -> Result<(), SaturdayResult> {
    stmt.accept(stmt, self)
  }

  fn resolve_expr(&self, expr: &Rc<Expr>) -> Result<(), SaturdayResult> {
    expr.accept(expr, self)
  }

  fn begin_scope(&self) {
    self.scopes.borrow_mut().push(RefCell::new(HashMap::new()));
  }

  fn end_scope(&self) {
    self.scopes.borrow_mut().pop();
  }

  fn declare(&self, name: &Token) {
    if self.scopes.borrow().is_empty() {
      return;
    }

    self
      .scopes
      .borrow()
      .last()
      .unwrap()
      .borrow_mut()
      .insert(name.as_string(), false);
  }

  fn define(&self, name: &Token) {
    if self.scopes.borrow().is_empty() {
      return;
    }

    self
      .scopes
      .borrow()
      .last()
      .unwrap()
      .borrow_mut()
      .insert(name.as_string(), true);
  }

  fn resolve_local(&self, expr: &Rc<Expr>, name: &Token) {
    for (scope, map) in self.scopes.borrow().iter().rev().enumerate() {
      if map.borrow().contains_key(&name.as_string()) {
        self.interpreter.resolve(expr, scope);
        return;
      }
    }
  }
}

impl StmtVisitor<()> for Resolver {
  fn visit_block_stmt(&self, _: &Rc<Stmt>, stmt: &BlockStmt) -> Result<(), SaturdayResult> {
    self.begin_scope();
    self.resolve(&stmt.statements)?;
    self.end_scope();
    Ok(())
  }

  fn visit_break_stmt(&self, _: &Rc<Stmt>, _expr: &BreakStmt) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_expression_stmt(
    &self,
    _: &Rc<Stmt>,
    _expr: &ExpressionStmt,
  ) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_function_stmt(&self, _: &Rc<Stmt>, _expr: &FunctionStmt) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_if_stmt(&self, _: &Rc<Stmt>, _expr: &IfStmt) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_print_stmt(&self, _: &Rc<Stmt>, _expr: &PrintStmt) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_return_stmt(&self, _: &Rc<Stmt>, _expr: &ReturnStmt) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_def_stmt(&self, _: &Rc<Stmt>, stmt: &DefStmt) -> Result<(), SaturdayResult> {
    self.declare(&stmt.name);
    if let Some(init) = &stmt.initializer {
      self.resolve_expr(&init)?;
    }

    self.define(&stmt.name);
    Ok(())
  }

  fn visit_while_stmt(&self, _: &Rc<Stmt>, _expr: &WhileStmt) -> Result<(), SaturdayResult> {
    todo!()
  }
}

impl ExprVisitor<()> for Resolver {
  fn visit_assign_expr(&self, _: &Rc<Expr>, _expr: &AssignExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_binary_expr(&self, _: &Rc<Expr>, _expr: &BinaryExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_call_expr(&self, _: &Rc<Expr>, _expr: &CallExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_grouping_expr(&self, _: &Rc<Expr>, _expr: &GroupingExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_literal_expr(&self, _: &Rc<Expr>, _expr: &LiteralExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_logical_expr(&self, _: &Rc<Expr>, _expr: &LogicalExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_unary_expr(&self, _: &Rc<Expr>, _expr: &UnaryExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_variable_expr(
    &self,
    wrapper: &Rc<Expr>,
    expr: &VariableExpr,
  ) -> Result<(), SaturdayResult> {
    if !self.scopes.borrow().is_empty()
      && !self
        .scopes
        .borrow()
        .last()
        .unwrap()
        .borrow()
        .get(&expr.name.as_string())
        .copied()
        .unwrap()
    {
      Err(SaturdayResult::runtime_error(
        &expr.name,
        "Can't read local variable in its own initializer.",
      ))
    } else {
      self.resolve_local(wrapper, &expr.name);
      Ok(())
    }
  }
}
