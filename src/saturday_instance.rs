use std::rc::Rc;
use crate::saturday_class::SaturdayClass;

#[derive(Debug, Clone, PartialEq)]
pub struct SaturdayInstance {
  pub class: Rc<SaturdayClass>,
}

impl SaturdayInstance {
  pub fn new(class: Rc<SaturdayClass>) -> Self {
    Self {
      class: Rc::clone(&class),
    }
  }
}
