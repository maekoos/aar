pub use dexparser::{
  AccessFlag, ClassDefinition, Code, DexFile, EncodedField, EncodedMethod, Field,
};
use log::{debug, warn};

pub mod code;
pub mod control_flow;
pub mod generated;

use super::codegen;
use code::generate_code;

mod instruction_queue;
pub(crate) use instruction_queue::InstructionQueue;

#[derive(Debug)]
pub enum ParserError {
  EOF,
}

#[derive(Debug)]
pub struct ParserOutput {
  pub definitions: String,
  pub source: String,
}

pub fn parse_class(
  c: &ClassDefinition,
  dex: &DexFile,
  module: &mut codegen::Module,
) -> Result<(), ParserError> {
  //TODO: Generate a destructor? (close open files, threads, etc)

  debug!("Class: {:?} ; Superclass: {:?}", c.class_type, c.superclass);

  let c_name = format_classname(&c.class_type);

  if let Some(cd) = &c.class_data {
    for f in &cd.static_fields {
      let f_name = format_name(&f.field.name);

      module.add_static(format!("{}__{}", c_name, f_name));
    }
  }

  if let Some(cd) = &c.class_data {
    for dm in &cd.direct_methods {
      debug!("DM: {}: {}", &dm.method.name, &dm.method.prototype.shorty);
      let (name, fn_) = parse_method(&c_name, dm, &c, dex);
      module.add_function(name, fn_);
    }

    for vm in &cd.virtual_methods {
      debug!("VM: {}: {}", &vm.method.name, &vm.method.prototype.shorty);
      let (name, fn_) = parse_method(&c_name, vm, &c, dex);
      module.add_function(name, fn_);
    }
  } else {
    warn!("Class without class_data: {}", c_name);
  }

  Ok(())
}

fn parse_method(
  c_name: &str,
  method: &EncodedMethod,
  class: &ClassDefinition,
  dex: &DexFile,
) -> (String, codegen::Function) {
  let m = &method.method;

  let m_name = format_name(&m.name);
  let m_full_name = format!("{}__{}", c_name, m_name);
  // let params = parse_params(&c_name, &method);

  let mut cg_fn = codegen::FunctionBuilder::new();

  if let Some(code) = &method.code {
    generate_code(code, &method, c_name, &class, dex, &mut cg_fn);
  } else {
    warn!("No code associated with method: {} ({})", m_name, c_name);
  }

  (m_full_name, cg_fn.build())
}

pub fn format_classname(classname: &str) -> String {
  format!(
    "CLASS_{}",
    classname[1..]
      .replace("__", "____")
      .replace("/", "__")
      .replace(";", "")
  )
}

pub fn format_name(name: &str) -> String {
  name
    .replace("__", "____")
    .replace("<", "__")
    .replace(">", "__")
}
