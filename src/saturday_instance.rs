use crate::error::SaturdayResult;
use crate::object::Object;
use crate::saturday_class::SaturdayClass;
use crate::token::Token;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct SaturdayInstance {
  pub class: Rc<SaturdayClass>,
  fields: RefCell<HashMap<String, Object>>,
}

impl SaturdayInstance {
  pub fn new(class: Rc<SaturdayClass>) -> Self {
    Self {
      class: Rc::clone(&class),
      fields: RefCell::new(HashMap::new()),
    }
  }

  pub fn get(&self, name: &Token) -> Result<Object, SaturdayResult> {
    if let Entry::Occupied(o) = self.fields.borrow_mut().entry(name.as_string()) {
      Ok(o.get().clone())
    } else if let Some(method) = self.class.find_method(&name.as_string()) {
      Ok(method)
    } else {
      Err(SaturdayResult::runtime_error(
        name,
        &format!("Undefined property '{}'.", name.as_string()),
      ))
    }
  }

  pub fn set(&self, name: &Token, value: Object) {
    self.fields.borrow_mut().insert(name.as_string(), value);
  }
}

impl ToString for SaturdayInstance {
  fn to_string(&self) -> String {
    format!("<Instance of {}>", self.class.to_string())
  }
}
