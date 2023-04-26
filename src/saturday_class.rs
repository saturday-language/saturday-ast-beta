use crate::callable::SaturdayCallable;
use crate::error::SaturdayResult;
use crate::interpreter::Interpreter;
use crate::object::Object;

#[derive(Debug, Clone, PartialEq)]
pub struct SaturdayClass {
  name: String,
}

impl SaturdayClass {
  pub fn new(name: String) -> Self {
    Self { name }
  }
}

/*
impl ToString for SaturdayClass {
  fn to_string(&self) -> String {
    self.name.clone()
  }
}
 */

impl SaturdayCallable for SaturdayClass {
  fn call(
    &self,
    _interpreter: &Interpreter,
    _arguments: Vec<Object>,
  ) -> Result<Object, SaturdayResult> {
    Ok(Object::Nil)
  }

  fn arity(&self) -> usize {
    0
  }

  fn to_string(&self) -> String {
    self.name.clone()
  }
}
