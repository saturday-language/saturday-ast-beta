use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
  Num(f64),
  Str(String),
  Bool(bool),
  Func(Callable),
  Nil,
  ArithmeticError,
}

impl fmt::Display for Object {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Object::Num(x) => write!(f, "{x}"),
      Object::Str(x) => write!(f, "{x}"),
      Object::Bool(x) => {
        if *x {
          write!(f, "true")
        } else {
          write!(f, "false")
        }
      }
      Object::Func(_) => write!(f, "<func>"),
      Object::Nil => write!(f, "nil"),
      Object::ArithmeticError => panic!("Should not be trying to print this"),
    }
  }
}

use crate::Interpreter;
use crate::SaturdayResult;
use std::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub struct Callable;

impl Callable {
  pub fn call(
    &self,
    _interpreter: &Interpreter,
    _arguments: Vec<Object>,
  ) -> Result<Object, SaturdayResult> {
    Ok(Object::Nil)
  }
}

// TODO 准备看第11天 已经完成了方法调用的解析 但是卡在了无法解析方法名上面
