pub type Value = f64;

pub struct ValueArray {
  values: Vec<Value>,
}

pub fn init_value_array() -> ValueArray {
  ValueArray {
    values: Vec::new(),
  }
}

impl ValueArray {
  pub fn write(&mut self, value: Value) {
    self.values.push(value);
  }

  pub fn len(&self) -> usize {
    self.values.len()
  }

  pub fn at(&self, idx: usize) -> Value {
    self.values[idx]
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn init_value_array_test() {
    let value_array = init_value_array();
    assert_eq!(value_array.values.len(), 0);
  }

  #[test]
  fn write_test() {
    let mut value_array = init_value_array();

    value_array.write(1.2);
    value_array.write(5.0);

    assert_eq!(value_array.values.len(), 2);
    assert_eq!(value_array.values[0], 1.2);
    assert_eq!(value_array.values[1], 5.0);
  }

  #[test]
  fn len_test() {
    let mut value_array = init_value_array();

    value_array.write(1.2);
    value_array.write(5.0);

    assert_eq!(value_array.len(), 2);
  }
}
