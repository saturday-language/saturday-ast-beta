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

struct Resolver {
  interpreter: Interpreter,
  scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
}

impl Resolver {
  pub fn new(interpreter: Interpreter) -> Self {
    Self {
      interpreter,
      scopes: RefCell::new(Vec::new()),
    }
  }

  fn resolve(&self, statements: &[Stmt]) -> Result<(), SaturdayResult> {
    for statement in statements {
      self.resolve_stmt(statement)?;
    }

    Ok(())
  }

  fn resolve_stmt(&self, stmt: &Stmt) -> Result<(), SaturdayResult> {
    stmt.accept(self)
  }

  fn resolve_expr(&self, expr: &Expr) -> Result<(), SaturdayResult> {
    expr.accept(self)
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
}

impl StmtVisitor<()> for Resolver {
  fn visit_block_stmt(&self, stmt: &BlockStmt) -> Result<(), SaturdayResult> {
    self.begin_scope();
    self.resolve(&stmt.statements)?;
    self.end_scope();
    Ok(())
  }

  fn visit_break_stmt(&self, expr: &BreakStmt) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_expression_stmt(&self, expr: &ExpressionStmt) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_function_stmt(&self, expr: &FunctionStmt) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_if_stmt(&self, expr: &IfStmt) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_print_stmt(&self, expr: &PrintStmt) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_return_stmt(&self, expr: &ReturnStmt) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_def_stmt(&self, stmt: &DefStmt) -> Result<(), SaturdayResult> {
    self.declare(&stmt.name);
    if let Some(init) = &stmt.initializer {
      self.resolve_expr(&init)?;
    }

    self.define(&stmt.name);
    Ok(())
  }

  fn visit_while_stmt(&self, expr: &WhileStmt) -> Result<(), SaturdayResult> {
    todo!()
  }
}

impl ExprVisitor<()> for Resolver {
  fn visit_assign_expr(&self, expr: &AssignExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_call_expr(&self, expr: &CallExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_logical_expr(&self, expr: &LogicalExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<(), SaturdayResult> {
    todo!()
  }

  fn visit_variable_expr(&self, expr: &VariableExpr) -> Result<(), SaturdayResult> {
    todo!()
  }
}
