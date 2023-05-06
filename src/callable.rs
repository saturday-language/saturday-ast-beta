use crate::object::Object;
use crate::Interpreter;
use crate::SaturdayResult;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use std::vec::Vec;

#[derive(Clone)]
pub struct Callable {
  pub func: Rc<dyn SaturdayCallable>,
}

impl Debug for Callable {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "<Callable>")
  }
}

impl Display for Callable {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "<Callable>")
  }
}

impl PartialEq for Callable {
  fn eq(&self, other: &Self) -> bool {
    std::ptr::eq(
      Rc::as_ptr(&self.func) as *const (),
      Rc::as_ptr(&other.func) as *const (),
    )
  }
}

pub trait SaturdayCallable {
  fn call(
    &self,
    interpreter: &Interpreter,
    arguments: Vec<Object>,
  ) -> Result<Object, SaturdayResult>;
  fn arity(&self) -> usize;
}
