use crate::saturday_class::SaturdayClass;
use std::rc::Rc;

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
