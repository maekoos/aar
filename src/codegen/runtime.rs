use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use log::warn;

use super::instructions::LiteralValue;
use super::{CallStack, RuntimeError, RuntimeErrorStack};

pub enum InvokeResult {
  /// Ok with the return value inside.
  Ok(Value),
  /// Exception with the `Throwable` object inside.
  Exception(Instance, Rc<CallStack>),
  /// Runtime exception for non-catchable errors.
  RuntimeError(RuntimeErrorStack),
}

impl InvokeResult {
  pub fn runtime(error: RuntimeError, stack: Rc<CallStack>) -> Self {
    Self::RuntimeError(RuntimeErrorStack::new(error, stack))
  }
}

impl std::fmt::Debug for InvokeResult {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    match self {
      InvokeResult::Ok(v) => writeln!(f, "Ok({})", format!("{:?}", v))?,
      InvokeResult::RuntimeError(e) => writeln!(f, "RuntimeError({})", format!("{:?}", e))?,
      InvokeResult::Exception(_e, _cs) => unimplemented!(),
    };
    Ok(())
  }
}

#[derive(Clone, Debug)]
pub enum Value {
  Void,
  Char(char),
  U32(i32),
  U64(i64),
  Instance(Arc<Mutex<Instance>>),
  Array(Vec<Value>),
  // String(String),
  // Native(std::rc::Rc<dyn NativeValue>),
  // Native(Box<dyn NativeValue>),
}

impl Value {
  pub fn new_instance() -> Self {
    Self::Instance(Arc::new(Mutex::new(Instance::default())))
  }

  pub fn from_instance(instance: Instance) -> Self {
    Self::Instance(Arc::new(Mutex::new(instance)))
  }

  pub fn is_void(&self) -> bool {
    match self {
      Value::Void => true,
      _ => false,
    }
  }

  pub fn is_char(&self) -> bool {
    match self {
      Value::Char(_) => true,
      _ => false,
    }
  }

  pub fn is_u32(&self) -> bool {
    match self {
      Value::U32(_) => true,
      _ => false,
    }
  }

  pub fn is_u64(&self) -> bool {
    match self {
      Value::U64(_) => true,
      _ => false,
    }
  }

  pub fn is_instance(&self) -> bool {
    match self {
      Value::Instance(_) => true,
      _ => false,
    }
  }

  pub fn is_array(&self) -> bool {
    match self {
      Value::Array(_) => true,
      _ => false,
    }
  }

  // pub fn is_string(&self) -> bool {
  //   match self {
  //     Value::String(_) => true,
  //     _ => false,
  //   }
  // }

  pub fn instance(&self) -> Result<&Arc<Mutex<Instance>>, RuntimeError> {
    match self {
      Value::Instance(i) => Ok(i),
      _ => Err(RuntimeError::CastError(format!("{:?} as instance", self))),
    }
  }

  pub fn to_single(&self) -> i32 {
    match self {
      Value::Void => 0,
      Value::Char(a) => *a as i32,
      Value::U32(a) => *a,
      Value::U64(a) => *a as i32,
      Value::Instance(_i) => {
        warn!("Trying to convert Instance to single value.");
        1
      }
      // Value::String(_s) => {
      //   warn!("Trying to convert String to single value.");
      //   1
      // }
      Value::Array(_) => {
        warn!("Trying to convert Array to single value.");
        1
      }
    }
  }
}

impl From<&LiteralValue> for Value {
  fn from(value: &LiteralValue) -> Self {
    match value {
      LiteralValue::Lit(i) => Value::U32(*i),
      LiteralValue::Wide(wide) => Value::U64(*wide),
      LiteralValue::String(s) => {
        //TODO Find a more effective way to store and handle strings?
        let out = Value::new_instance();
        {
          let mut inst = out.instance().unwrap().lock().unwrap();
          inst.set_class_type("java_lang_string".to_owned());
          let v = s.chars().map(|ch| Value::Char(ch)).collect();
          inst.set_field("data".to_owned(), Value::Array(v));
        }

        out
      }
      // LiteralValue::Class
      _ => unimplemented!(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Instance {
  fields: HashMap<String, Value>,
  type_: String,
}

impl Default for Instance {
  fn default() -> Self {
    Self {
      fields: HashMap::new(),
      type_: "unknown".to_owned(),
    }
  }
}

impl Instance {
  pub fn set_class_type(&mut self, type_: String) {
    self.type_ = type_;
  }

  pub fn get_class_type(&self) -> &String {
    &self.type_
  }

  pub fn set_field(&mut self, field: String, value: Value) {
    self.fields.insert(field, value);
  }

  pub fn get_field(&self, field: &str) -> Option<&Value> {
    self.fields.get(field)
  }
}
