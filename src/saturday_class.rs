use crate::callable::SaturdayCallable;
use crate::error::SaturdayResult;
use crate::interpreter::Interpreter;
use crate::object::Object;
use crate::saturday_instance::SaturdayInstance;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct SaturdayClass {
  name: String,
}

impl SaturdayClass {
  pub fn new(name: String) -> Self {
    Self { name }
  }

  pub fn instantiate(
    &self,
    _interpreter: &Interpreter,
    _arguments: Vec<Object>,
    class: Rc<SaturdayClass>,
  ) -> Result<Object, SaturdayResult> {
    Ok(Object::Instance(SaturdayInstance::new(class)))
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
    Err(SaturdayResult::system_error("tried to call a class"))
  }

  fn arity(&self) -> usize {
    0
  }

  fn to_string(&self) -> String {
    self.name.clone()
  }
}
