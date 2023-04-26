#[derive(Debug, Clone, PartialEq)]
pub struct SaturdayClass {
  pub name: String,
}

impl SaturdayClass {
  pub fn new(name: String) -> Self {
    Self { name }
  }
}

impl ToString for SaturdayClass {
  fn to_string(&self) -> String {
    self.name.clone()
  }
}
