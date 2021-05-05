use std::fmt;
use std::rc::Rc;

pub mod function_builder;
pub mod interpreted;

pub use interpreted::InterpretedFunction;

use super::{instructions, runtime, InvokeResult, Module, RuntimeError};

#[derive(Debug)]
pub enum Function {
  Native(NativeFunction),
  Interpreted(interpreted::InterpretedFunction),
}

impl Function {
  pub fn build_ir(&self, name: &str) -> String {
    match self {
      Function::Interpreted(ifn) => ifn.build_ir(name),
      Function::Native(_) => format!("native func {:?};", name),
    }
  }
}

pub struct NativeFunction(
  pub fn(Vec<runtime::Value>, Rc<CallStack>, &super::Module) -> InvokeResult,
);

impl fmt::Debug for NativeFunction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "NativeFunction")
  }
}

pub struct CallStack {
  pub prev: Option<Rc<CallStack>>,
  pub cur: String,
}

impl Default for CallStack {
  fn default() -> Self {
    Self {
      prev: None,
      cur: String::from("root"),
    }
  }
}

impl CallStack {
  pub fn extend(cur: String, prev: Rc<CallStack>) -> Self {
    Self {
      prev: Some(prev),
      cur,
    }
  }
  /// Turn the call stack into a finished version, to be printed
  pub fn finalize(&self) -> String {
    let mut out = vec![];

    out.push(format!("\tin {:?}", self.cur));

    match &self.prev {
      Some(prev) => {
        out.push(prev.finalize());
      }
      None => {}
    };

    out.join("\n")
  }
}
